use std::sync::Arc;
use std::time::Instant;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use reqwest::Client;
use tokio::time::{sleep, Duration};

use crate::stats::SharedStats;
use crate::utils::{HTTP_METHODS, USER_AGENTS, random_pick};

/// A single worker loop: keeps firing requests until the deadline token is
/// cancelled. Each worker owns its own `SmallRng` — no shared RNG mutex.
pub(crate) async fn run_worker(
    client:   Client,
    targets:  Arc<Vec<String>>,
    stats:    SharedStats,
    deadline: Arc<tokio::sync::Notify>,
) {
    // Thread-local, non-cryptographic RNG — fastest option for random picks.
    let mut rng = SmallRng::from_os_rng();

    loop {
        // Non-blocking check: has the deadline fired?
        // We use try_recv pattern via a flag set by the timer task.
        // The Notify is used as a broadcast; we peek without consuming.
        if Arc::strong_count(&deadline) == 1 {
            // Only our reference remains — timer dropped its half. Stop.
            break;
        }

        let url    = random_pick(&targets, &mut rng).clone();
        let method = random_pick(HTTP_METHODS, &mut rng);
        let ua     = random_pick(USER_AGENTS,  &mut rng);

        let request = build_request(&client, method, &url, ua);

        let t0 = Instant::now();
        match request.send().await {
            Ok(_)  => stats.record_success(t0.elapsed()),
            Err(_) => stats.record_error(),
        }

        // Tiny yield so the tokio scheduler can interleave other tasks
        // without burning 100 % of a core between awaits.
        sleep(Duration::ZERO).await;
    }
}

fn build_request(
    client: &Client,
    method: &str,
    url:    &str,
    ua:     &str,
) -> reqwest::RequestBuilder {
    let rb = match method {
        "GET"     => client.get(url),
        "POST"    => client.post(url),
        "PUT"     => client.put(url),
        "DELETE"  => client.delete(url),
        "PATCH"   => client.patch(url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, url),
        _         => client.get(url),
    };
    rb.header("User-Agent", ua)
      .header("Accept", "*/*")
      .timeout(std::time::Duration::from_secs(10))
}

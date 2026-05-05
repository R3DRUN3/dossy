use std::sync::Arc;
use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use reqwest::Client;
use tokio_util::sync::CancellationToken;

use crate::stats::SharedStats;
use crate::utils::{HTTP_METHODS, USER_AGENTS, random_pick, random_path, roll_random_path};

const FLUSH_EVERY: u64 = 64;

pub(crate) async fn run_worker(
    client:  Client,
    targets: Arc<Vec<String>>,
    stats:   SharedStats,
    token:   CancellationToken,
) {
    let mut rng = SmallRng::from_os_rng();

    // Per-worker local accumulators: no shared atomic touched until flush.
    let mut local_sent       = 0u64;
    let mut local_success    = 0u64;
    let mut local_errors     = 0u64;
    let mut local_latency_us = 0u64; // microseconds, matching Stats field

    // Reusable URL buffer: one allocation per worker, not one per request.
    let mut url_buf = String::with_capacity(256);

    loop {
        // ── Build URL ────────────────────────────────────────────────────────
        url_buf.clear();
        let base = random_pick(&targets, &mut rng);
        url_buf.push_str(base);

        if roll_random_path(&mut rng) {
            let path = random_path(&mut rng);
            if !base.ends_with('/') {
                url_buf.push('/');
            }
            url_buf.push_str(&path);
        }

        let method = random_pick(HTTP_METHODS, &mut rng);
        let ua     = random_pick(USER_AGENTS, &mut rng);
        let request = build_request(&client, method, &url_buf, ua);

        // ── Fire request — cancellation always takes priority ────────────────
        tokio::select! {
            biased;

            _ = token.cancelled() => {
                // Flush remainder before exiting so final report is accurate.
                stats.flush(local_sent, local_success, local_errors, local_latency_us);
                break;
            }

            result = async {
                let t0  = Instant::now();
                let res = request.send().await;
                (res, t0.elapsed())
            } => {
                let (result, elapsed) = result;
                local_sent += 1;

                match result {
                    Ok(_) => {
                        local_success    += 1;
                        local_latency_us += elapsed.as_micros() as u64;
                    }
                    Err(_) => {
                        local_errors += 1;
                    }
                }

                // Batch-flush every FLUSH_EVERY requests.
                if local_sent % FLUSH_EVERY == 0 {
                    stats.flush(local_sent, local_success, local_errors, local_latency_us);
                    local_sent       = 0;
                    local_success    = 0;
                    local_errors     = 0;
                    local_latency_us = 0;
                }
            }
        }

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
    // Timeout is set once at the client level in main.rs, no per-request
    // override needed, avoids a small allocation on every single request.
    rb.header("User-Agent", ua)
      .header("Accept", "*/*")
}

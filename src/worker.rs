use std::sync::Arc;
use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use reqwest::Client;
use tokio_util::sync::CancellationToken;

use crate::stats::SharedStats;
use crate::utils::{HTTP_METHODS, USER_AGENTS, random_pick, random_path, roll_random_path};

pub(crate) async fn run_worker(
    client:  Client,
    targets: Arc<Vec<String>>,
    stats:   SharedStats,
    token:   CancellationToken,
) {
    let mut rng     = SmallRng::from_os_rng();
    let mut url_buf = String::with_capacity(256);

    loop {
        // ── Build URL ────────────────────────────────────────────────────────
        url_buf.clear();
        let base = random_pick(&targets, &mut rng);
        url_buf.push_str(base);

        if roll_random_path(&mut rng) {
            let path = random_path(&mut rng);
            if !base.ends_with('/') { url_buf.push('/'); }
            url_buf.push_str(&path);
        }

        let method  = random_pick(HTTP_METHODS, &mut rng);
        let ua      = random_pick(USER_AGENTS, &mut rng);
        let request = build_request(&client, method, &url_buf, ua);

        // ── Fire request — cancellation races the request itself ─────────────
        tokio::select! {
            biased;
            _ = token.cancelled() => break,
            result = async {
                let t0  = Instant::now();
                let res = request.send().await;
                (res, t0.elapsed())
            } => {
                let (result, elapsed) = result;
                match result {
                    Ok(_)  => stats.flush(1, 1, 0, elapsed.as_micros() as u64),
                    Err(_) => stats.flush(1, 0, 1, 0),
                }
            }
        }

        // Catch cancellations that arrived while we were inside request.send(),
        // so we never start a brand-new request after the test has ended.
        if token.is_cancelled() { break; }
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
}

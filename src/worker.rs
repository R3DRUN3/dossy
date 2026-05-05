use std::sync::Arc;
use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use reqwest::Client;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

use crate::stats::SharedStats;
use crate::utils::{HTTP_METHODS, USER_AGENTS, random_pick, random_path, roll_random_path};

pub(crate) async fn run_worker(
    client:  Client,
    targets: Arc<Vec<String>>,
    stats:   SharedStats,
    token:   CancellationToken,
) {
    let mut rng = SmallRng::from_os_rng();

    loop {
        if token.is_cancelled() {
            break;
        }

        // Pick base target, then maybe append a random path
        let base   = random_pick(&targets, &mut rng);
        let url = if roll_random_path(&mut rng) {
            let path = random_path(&mut rng);
            // Avoid double-slash if the base URL already has a trailing slash
            let sep = if base.ends_with('/') { "" } else { "/" };
            format!("{}{}{}", base, sep, path)
        } else {
            base.clone()
        };

        let method = random_pick(HTTP_METHODS, &mut rng);
        let ua     = random_pick(USER_AGENTS,  &mut rng);

        let request = build_request(&client, method, &url, ua);

        let t0 = Instant::now();

        tokio::select! {
            biased;

            _ = token.cancelled() => {
                break;
            }

            result = request.send() => {
                match result {
                    Ok(_)  => stats.record_success(t0.elapsed()),
                    Err(_) => {
                        if !token.is_cancelled() {
                            stats.record_error();
                        }
                    }
                }
            }
        }

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
      .timeout(Duration::from_secs(5))
}

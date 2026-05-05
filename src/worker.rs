use std::sync::Arc;
use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use reqwest::Client;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

use crate::stats::SharedStats;
use crate::utils::{HTTP_METHODS, USER_AGENTS, random_pick};

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

        let url    = random_pick(&targets, &mut rng).clone();
        let method = random_pick(HTTP_METHODS, &mut rng);
        let ua     = random_pick(USER_AGENTS, &mut rng);

        let request = build_request(&client, method, &url, ua);

        let t0 = Instant::now();

        tokio::select! {
            biased; // check cancellation branch first every iteration

            _ = token.cancelled() => {
                // We were cancelled — do NOT record this as an error,
                // the request was intentionally interrupted by shutdown.
                break;
            }

            result = request.send() => {
                match result {
                    Ok(_)  => stats.record_success(t0.elapsed()),
                    Err(_) => {
                        // Only record as error if we were NOT cancelled.
                        // A cancelled token means the error is just the
                        // abort tearing down the in-flight reqwest future.
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

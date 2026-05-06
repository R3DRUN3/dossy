use std::sync::Arc;
use std::time::Instant;

use futures::StreamExt;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use reqwest::Client;
use tokio::task;
use tokio_util::sync::CancellationToken;

use crate::stats::SharedStats;
use crate::utils::{HTTP_METHODS, USER_AGENTS, random_pick, random_path, roll_random_path};

const PIPELINE_DEPTH: usize = 64;

fn worker_threads() -> usize {
    std::env::var("DOSSY_THREADS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(num_cpus::get)
}

/// Passed into every worker: cheaply cloneable, body bytes shared via Arc.
#[derive(Clone)]
pub(crate) struct BodyConfig {
    pub body:         Option<Arc<bytes::Bytes>>,
    pub content_type: String,
}

pub(crate) fn spawn_workers(
    client:      Client,
    targets:     Arc<Vec<String>>,
    stats:       SharedStats,
    token:       CancellationToken,
    concurrency: usize,
    body_cfg:    Arc<BodyConfig>,
) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads())
        .enable_all()
        .build()
        .expect("failed to build worker runtime");

    std::thread::spawn(move || {
        rt.block_on(async move {
            let mut handles = Vec::with_capacity(concurrency);
            for _ in 0..concurrency {
                handles.push(task::spawn(run_worker(
                    client.clone(),
                    Arc::clone(&targets),
                    Arc::clone(&stats),
                    token.clone(),
                    Arc::clone(&body_cfg),
                )));
            }
            futures::future::join_all(handles).await;
        });
    });
}

// ── Owned request arguments ───────────────────────────────────────────────

struct ReqArgs {
    url:    String,
    method: &'static str,
    ua:     &'static str,
}

fn build_args(targets: &[String], rng: &mut SmallRng) -> ReqArgs {
    let mut url = String::with_capacity(256);
    let base = random_pick(targets, rng);
    url.push_str(base);

    if roll_random_path(rng) {
        if !base.ends_with('/') { url.push('/'); }
        url.push_str(&random_path(rng));
    }

    ReqArgs {
        url,
        method: *random_pick(HTTP_METHODS, rng),
        ua:     *random_pick(USER_AGENTS,  rng),
    }
}

// ── Worker loop ───────────────────────────────────────────────────────────

pub(crate) async fn run_worker(
    client:   Client,
    targets:  Arc<Vec<String>>,
    stats:    SharedStats,
    token:    CancellationToken,
    body_cfg: Arc<BodyConfig>,
) {
    let mut rng  = SmallRng::from_os_rng();
    let mut pool = futures::stream::FuturesUnordered::new();

    loop {
        while pool.len() < PIPELINE_DEPTH && !token.is_cancelled() {
            let args = build_args(&targets, &mut rng);
            pool.push(fire_one(client.clone(), args, Arc::clone(&body_cfg)));
        }

        if token.is_cancelled() && pool.is_empty() { break; }

        tokio::select! {
            biased;
            _ = token.cancelled() => {
                while let Some(outcome) = pool.next().await {
                    record(&stats, outcome);
                }
                break;
            }
            Some(outcome) = pool.next() => {
                record(&stats, outcome);
            }
        }
    }
}

// ── Request execution ─────────────────────────────────────────────────────

struct Outcome {
    success:    bool,
    latency_us: u64,
}

async fn fire_one(client: Client, args: ReqArgs, body_cfg: Arc<BodyConfig>) -> Outcome {
    let t0  = Instant::now();
    let res = build_request(&client, args.method, &args.url, args.ua, &body_cfg)
        .send()
        .await;
    let lat = t0.elapsed().as_micros() as u64;

    match res {
        Ok(resp) => {
            let s = resp.status();
            let _ = resp.bytes().await;
            if s.is_success() || s.is_redirection() {
                Outcome { success: true,  latency_us: lat }
            } else {
                Outcome { success: false, latency_us: 0   }
            }
        }
        Err(_) => Outcome { success: false, latency_us: 0 },
    }
}

#[inline(always)]
fn record(stats: &SharedStats, o: Outcome) {
    if o.success {
        stats.flush(1, 1, 0, o.latency_us);
    } else {
        stats.flush(1, 0, 1, 0);
    }
}

fn build_request(
    client:   &Client,
    method:   &str,
    url:      &str,
    ua:       &str,
    body_cfg: &BodyConfig,
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

    let rb = rb
        .header("User-Agent", ua)
        .header("Accept", "*/*")
        .header("Accept-Encoding", "identity");

    // Only attach body on methods that carry one
    match &body_cfg.body {
        Some(body) if matches!(method, "POST" | "PUT" | "PATCH") => {
            rb.header("Content-Type", &body_cfg.content_type)
              .body(reqwest::Body::from((**body).clone()))
        }
        _ => rb,
    }
}

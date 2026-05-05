use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use clap::CommandFactory;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

mod stats;
mod utils;
mod worker;

use stats::Stats;

#[derive(Parser, Debug)]
#[command(
    name       = "dossy",
    version,
    about      = "your server's will to live not found.",
    long_about = "
  ██████╗  ██████╗ ███████╗███████╗██╗   ██╗
  ██╔══██╗██╔═══██╗██╔════╝██╔════╝╚██╗ ██╔╝
  ██║  ██║██║   ██║███████╗███████╗ ╚████╔╝
  ██║  ██║██║   ██║╚════██║╚════██║  ╚██╔╝
  ██████╔╝╚██████╔╝███████║███████║   ██║
  ╚═════╝  ╚═════╝ ╚══════╝╚══════╝   ╚═╝
"
)]
struct Cli {
    /// One or more target URLs (e.g. https://example.com)
    #[arg(short, long, required = true, value_name = "URL", num_args = 1..)]
    targets: Vec<String>,

    /// How long to run the test (seconds)
    #[arg(short, long, default_value_t = 30, value_name = "SECS")]
    duration: u64,

    /// Number of concurrent async workers
    #[arg(short, long, default_value_t = 512, value_name = "N")]
    concurrency: usize,

    /// Suppress progress bar (useful in CI)
    #[arg(short, long)]
    quiet: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let quiet = cli.quiet || !atty::is(atty::Stream::Stdout);

    print_banner();
    println!(
        "{} {} targets | {} concurrency | {} seconds\n",
        "►".cyan().bold(),
        cli.targets.len().to_string().yellow(),
        cli.concurrency.to_string().yellow(),
        cli.duration.to_string().yellow(),
    );
    for t in &cli.targets {
        println!("  {} {}", "•".dimmed(), t.underline());
    }
    println!();

    let client = Client::builder()
        .pool_max_idle_per_host(cli.concurrency * 2)
        .tcp_keepalive(Duration::from_secs(30))
        .tcp_nodelay(true)
        .danger_accept_invalid_certs(false)
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(3))
        .http2_adaptive_window(true)
        .build()?;

    let targets = Arc::new(cli.targets.clone());
    let stats   = Arc::new(Stats::default());
    let token   = CancellationToken::new();

    // ── Spawn workers ────────────────────────────────────────────────────────
    for _ in 0..cli.concurrency {
        tokio::spawn(worker::run_worker(
            client.clone(),
            Arc::clone(&targets),
            Arc::clone(&stats),
            token.clone(),
        ));
    }

    // ── Progress bar ─────────────────────────────────────────────────────────
    let bar = if !quiet {
        let pb = ProgressBar::new(cli.duration);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.cyan} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}s  {msg}",
            )?
            .progress_chars("█▉▊▋▌▍▎▏  "),
        );
        Some(pb)
    } else {
        None
    };

    // ── Tick loop ─────────────────────────────────────────────────────────────
    let stats_ref        = Arc::clone(&stats);
    let mut prev_sent       = 0u64;
    let mut prev_success    = 0u64;
    let mut prev_errors     = 0u64;
    let mut prev_latency_us = 0u64;

    for elapsed in 0..cli.duration {
        sleep(Duration::from_secs(1)).await;
        let snap = stats_ref.snapshot();

        // Per-second deltas
        let delta_sent       = snap.sent.saturating_sub(prev_sent);
        let delta_success    = snap.success.saturating_sub(prev_success);
        let delta_errors     = snap.errors.saturating_sub(prev_errors);
        let delta_latency_us = snap.latency_us_total.saturating_sub(prev_latency_us);

        // Per-second avg latency in ms
        let delta_avg_ms = if delta_success > 0 {
            (delta_latency_us / delta_success) as f64 / 1_000.0
        } else {
            0.0
        };

        prev_sent       = snap.sent;
        prev_success    = snap.success;
        prev_errors     = snap.errors;
        prev_latency_us = snap.latency_us_total;

        let msg = format!(
            "req/s ≈ {:>6}  ✓ {:>8}  ✗ {:>6}  avg {:>7.2}ms",
            delta_sent,
            delta_success,
            delta_errors,
            delta_avg_ms,
        );
        if let Some(ref pb) = bar {
            pb.set_position(elapsed + 1);
            pb.set_message(msg);
        } else {
            println!("[{:>3}s] {}", elapsed + 1, msg);
        }
    }

    // ── Graceful shutdown ────────────────────────────────────────────────────
    token.cancel();
    sleep(Duration::from_millis(50)).await;

    if let Some(pb) = bar {
        pb.finish_and_clear();
    }

    // ── Final report ─────────────────────────────────────────────────────────
    let snap = stats.snapshot();
    println!("\n{}", "═══════════════════════════════════════".cyan());
    println!(" {} Final Report", "dossy".bold().cyan());
    println!("{}", "═══════════════════════════════════════".cyan());
    println!("  {:<22} {}", "Total requests sent:".dimmed(),  snap.sent.to_string().bold());
    println!("  {:<22} {}", "Successful (2xx/3xx):".dimmed(), snap.success.to_string().green().bold());
    println!("  {:<22} {}", "Errors / timeouts:".dimmed(),    snap.errors.to_string().red().bold());
    println!("  {:<22} {:.2} ms", "Avg latency:".dimmed(),    snap.avg_latency_ms);
    println!(
        "  {:<22} {:.0} req/s",
        "Throughput:".dimmed(),
        snap.sent as f64 / cli.duration as f64
    );
    println!(
        "  {:<22} {:.1} %",
        "Success rate:".dimmed(),
        if snap.sent > 0 { snap.success as f64 / snap.sent as f64 * 100.0 } else { 0.0 }
    );
    println!("{}\n", "═══════════════════════════════════════".cyan());

    std::process::exit(0);
}

fn print_banner() {
    println!(
        "{}{}",
        Cli::command().get_long_about().unwrap_or_default().to_string().cyan().bold(),
        Cli::command().get_about().unwrap_or_default().to_string().cyan().bold()
    );
}

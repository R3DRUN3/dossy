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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
        .pool_max_idle_per_host(cli.concurrency)
        .tcp_keepalive(Duration::from_secs(30))
        .tcp_nodelay(true)
        .danger_accept_invalid_certs(false)
        .redirect(reqwest::redirect::Policy::limited(5))
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(3))
        .build()?;

    let targets = Arc::new(cli.targets.clone());
    let stats   = Arc::new(Stats::default());
    let token   = CancellationToken::new();

    // ── Spawn workers ────────────────────────────────────────────────────────
    let mut handles = Vec::with_capacity(cli.concurrency);
    for _ in 0..cli.concurrency {
        let h = tokio::spawn(worker::run_worker(
            client.clone(),
            Arc::clone(&targets),
            Arc::clone(&stats),
            token.clone(),
        ));
        handles.push(h);
    }

    // ── Progress bar ─────────────────────────────────────────────────────────
    let bar = if !cli.quiet {
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

    let stats_ref = Arc::clone(&stats);
    for elapsed in 0..cli.duration {
        sleep(Duration::from_secs(1)).await;
        let snap = stats_ref.snapshot();
        let msg = format!(
            "req/s ≈ {:>6}  ✓ {:>8}  ✗ {:>6}  avg {:>7.2}ms",
            snap.sent / (elapsed + 1),
            snap.success,
            snap.errors,
            snap.avg_latency_ms,
        );
        if let Some(ref pb) = bar {
            pb.set_position(elapsed + 1);
            pb.set_message(msg);
        } else {
            println!("[{:>3}s] {}", elapsed + 1, msg);
        }
    }

    // ── Shutdown ─────────────────────────────────────────────────────────────
    // 1. Signal workers to stop accepting new work
    token.cancel();
    // 2. Hard-abort every task — instant, no waiting for in-flight requests
    for handle in &handles {
        handle.abort();
    }

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

    Ok(())
}

fn print_banner() {
    println!(
    "{}{}",
    Cli::command().get_long_about().unwrap_or_default().to_string().cyan().bold(),
    Cli::command().get_about().unwrap_or_default().to_string().cyan().bold());
}

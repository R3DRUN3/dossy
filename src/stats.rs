use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::Serialize;

#[derive(Default)]
pub(crate) struct Stats {
    pub sent:             AtomicU64,
    pub success:          AtomicU64,
    pub errors:           AtomicU64,
    pub latency_us_total: AtomicU64,
}

impl Stats {
    pub(crate) fn flush(&self, sent: u64, success: u64, errors: u64, latency_us: u64) {
        self.sent            .fetch_add(sent,       Ordering::Relaxed);
        self.success         .fetch_add(success,    Ordering::Relaxed);
        self.errors          .fetch_add(errors,     Ordering::Relaxed);
        self.latency_us_total.fetch_add(latency_us, Ordering::Relaxed);
    }

    pub(crate) fn snapshot(&self) -> StatsSnapshot {
        let sent    = self.sent.load(Ordering::Relaxed);
        let success = self.success.load(Ordering::Relaxed);
        let errors  = self.errors.load(Ordering::Relaxed);
        let lat_us  = self.latency_us_total.load(Ordering::Relaxed);
        let avg_ms  = if success > 0 {
            (lat_us / success) as f64 / 1_000.0
        } else { 0.0 };
        StatsSnapshot { sent, success, errors, avg_latency_ms: avg_ms, latency_us_total: lat_us }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StatsSnapshot {
    pub sent:             u64,
    pub success:          u64,
    pub errors:           u64,
    pub avg_latency_ms:   f64,
    pub latency_us_total: u64,
}

pub(crate) type SharedStats = Arc<Stats>;

// ── Report types ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub(crate) struct Report {
    pub duration_secs:    u64,
    pub concurrency:      usize,
    pub targets:          Vec<String>,
    pub body_provided:    bool,
    pub content_type:     String,
    pub total_sent:       u64,
    pub total_success:    u64,
    pub total_errors:     u64,
    pub avg_latency_ms:   f64,
    pub throughput_rps:   f64,
    pub success_rate_pct: f64,
}

impl Report {
    pub(crate) fn to_csv(&self) -> String {
        let mut out = String::new();
        out.push_str("field,value\n");
        out.push_str(&format!("duration_secs,{}\n",       self.duration_secs));
        out.push_str(&format!("concurrency,{}\n",         self.concurrency));
        out.push_str(&format!("targets,\"{}\"\n",         self.targets.join(" | ")));
        out.push_str(&format!("body_provided,{}\n",       self.body_provided));
        out.push_str(&format!("content_type,{}\n",        self.content_type));
        out.push_str(&format!("total_sent,{}\n",          self.total_sent));
        out.push_str(&format!("total_success,{}\n",       self.total_success));
        out.push_str(&format!("total_errors,{}\n",        self.total_errors));
        out.push_str(&format!("avg_latency_ms,{:.4}\n",   self.avg_latency_ms));
        out.push_str(&format!("throughput_rps,{:.2}\n",   self.throughput_rps));
        out.push_str(&format!("success_rate_pct,{:.2}\n", self.success_rate_pct));
        out
    }
}

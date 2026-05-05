use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Shared, lock-free counters updated from every worker task.
#[derive(Default)]
pub(crate) struct Stats {
    pub sent:             AtomicU64,
    pub success:          AtomicU64,
    pub errors:           AtomicU64,
    pub latency_us_total: AtomicU64,
}

impl Stats {
    pub(crate) fn flush(
        &self,
        sent:       u64,
        success:    u64,
        errors:     u64,
        latency_us: u64,
    ) {
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
        } else {
            0.0
        };
        StatsSnapshot { sent, success, errors, avg_latency_ms: avg_ms, latency_us_total: lat_us }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StatsSnapshot {
    pub sent:             u64,
    pub success:          u64,
    pub errors:           u64,
    pub avg_latency_ms:   f64,
    pub latency_us_total: u64, // exposed so main.rs can compute per-second deltas
}

/// Convenience wrapper so every task only needs one Arc clone.
pub(crate) type SharedStats = Arc<Stats>;

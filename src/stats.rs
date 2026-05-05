use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Shared, lock-free counters updated from every worker task.
#[derive(Default)]
pub(crate) struct Stats {
    pub sent:    AtomicU64,
    pub success: AtomicU64,
    pub errors:  AtomicU64,
    pub latency_us_total: AtomicU64, // microseconds, summed
}

impl Stats {
    pub(crate) fn record_success(&self, latency: Duration) {
        self.sent.fetch_add(1, Ordering::Relaxed);
        self.success.fetch_add(1, Ordering::Relaxed);
        self.latency_us_total
            .fetch_add(latency.as_micros() as u64, Ordering::Relaxed);
    }

    pub(crate) fn record_error(&self) {
        self.sent.fetch_add(1, Ordering::Relaxed);
        self.errors.fetch_add(1, Ordering::Relaxed);
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
        StatsSnapshot { sent, success, errors, avg_latency_ms: avg_ms }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StatsSnapshot {
    pub sent:            u64,
    pub success:         u64,
    pub errors:          u64,
    pub avg_latency_ms:  f64,
}

/// Convenience wrapper so every task only needs one Arc clone.
pub(crate) type SharedStats = Arc<Stats>;

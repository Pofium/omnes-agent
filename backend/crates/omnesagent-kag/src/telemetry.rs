//! Batched access telemetry shared by the memory and knowledge-graph stores
//! (ob2h pillar III/VI parity — Access Telemetry Parity, ADR-O6).
//!
//! Every real read of a stored record (`memories` via `omnesagent-memory`,
//! `graph_nodes` via [`crate::graph::GraphService`]) used to run its own
//! `UPDATE …` against the shared `Mutex<Connection>`. On a busy turn loop that
//! is a per-read blocking write. Here reads are accumulated in memory and
//! applied to the store in one batch every [`ACCESS_FLUSH_INTERVAL`] (or on
//! shutdown), so the agent turn loop never pays for an individual write.
//!
//! Flag-free accumulator: [`AccessTelemetry::try_begin_flush`] hands the flush
//! right to exactly one caller per cadence, so background flushes never overlap.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Flush cadence for accumulated access telemetry (ADR-O6: "раз в 10 с").
pub const ACCESS_FLUSH_INTERVAL: Duration = Duration::from_secs(10);

/// In-memory accumulator of per-key access counts with a time-based flush gate.
///
/// The type is cheap to clone behind an `Arc` and safe to share across the
/// turn loop: [`record`](Self::record) and [`record_many`](Self::record_many)
/// only take a short mutex, never the SQLite connection.
#[derive(Debug)]
pub struct AccessTelemetry {
    interval: Duration,
    counts: Mutex<HashMap<String, i64>>,
    last_flush: Mutex<Instant>,
    flushing: AtomicBool,
}

impl Default for AccessTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessTelemetry {
    /// Accumulator using the production [`ACCESS_FLUSH_INTERVAL`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_interval(ACCESS_FLUSH_INTERVAL)
    }

    /// Accumulator with an explicit cadence (used by tests to force/deny a due
    /// flush without waiting on wall-clock time).
    #[must_use]
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            interval,
            counts: Mutex::new(HashMap::new()),
            last_flush: Mutex::new(Instant::now()),
            flushing: AtomicBool::new(false),
        }
    }

    /// Accumulate a single touch of `key`. Non-blocking: one short mutex bump.
    pub fn record(&self, key: &str) {
        *self.counts.lock().entry(key.to_string()).or_insert(0) += 1;
    }

    /// Accumulate touches for several keys at once.
    pub fn record_many(&self, keys: &[String]) {
        if keys.is_empty() {
            return;
        }
        let mut counts = self.counts.lock();
        for key in keys {
            *counts.entry(key.clone()).or_insert(0) += 1;
        }
    }

    /// Number of distinct keys currently pending a flush.
    #[must_use]
    pub fn pending(&self) -> usize {
        self.counts.lock().len()
    }

    /// True when the flush interval has elapsed and something is pending.
    #[must_use]
    pub fn flush_due(&self) -> bool {
        if self.counts.lock().is_empty() {
            return false;
        }
        self.last_flush.lock().elapsed() >= self.interval
    }

    /// Claim the right to run a background flush. Returns `false` when nothing
    /// is due, or a flush is already in flight (so callers never overlap).
    #[must_use]
    pub fn try_begin_flush(&self) -> bool {
        if !self.flush_due() || self.flushing.swap(true, Ordering::AcqRel) {
            return false;
        }
        true
    }

    /// Drain the accumulated counts into a batch for the store to apply. Does
    /// not update the flush clock — pair it with [`mark_flushed`](Self::mark_flushed)
    /// on success or [`requeue`](Self::requeue) on failure.
    #[must_use]
    pub fn drain(&self) -> Vec<(String, i64)> {
        self.counts.lock().drain().collect()
    }

    /// Record that a drained batch reached the store: reset the cadence clock
    /// and release the in-flight guard.
    pub fn mark_flushed(&self) {
        *self.last_flush.lock() = Instant::now();
        self.flushing.store(false, Ordering::Release);
    }

    /// Put a failed batch back so it is retried on the next cadence (fail-open:
    /// telemetry loss on a transient store error is avoided).
    pub fn requeue(&self, batch: Vec<(String, i64)>) {
        if !batch.is_empty() {
            let mut counts = self.counts.lock();
            for (key, n) in batch {
                *counts.entry(key).or_insert(0) += n;
            }
        }
        self.flushing.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn record_many_accumulates_and_drains() {
        let t = AccessTelemetry::with_interval(Duration::ZERO);
        t.record_many(&keys(&["a", "b"]));
        t.record_many(&keys(&["a"]));
        assert_eq!(t.pending(), 2);

        let mut batch = t.drain();
        batch.sort();
        assert_eq!(batch, vec![("a".to_string(), 2), ("b".to_string(), 1)]);
        assert_eq!(t.pending(), 0);
    }

    #[test]
    fn flush_due_requires_pending_and_elapsed_interval() {
        let t = AccessTelemetry::new();
        assert!(!t.flush_due(), "empty accumulator is never due");
        t.record("a");
        assert!(!t.flush_due(), "fresh touch waits out the interval");

        let immediate = AccessTelemetry::with_interval(Duration::ZERO);
        immediate.record("a");
        assert!(immediate.flush_due());
    }

    #[test]
    fn try_begin_flush_is_exclusive_until_settled() {
        let t = AccessTelemetry::with_interval(Duration::ZERO);
        t.record("a");
        assert!(t.try_begin_flush(), "first claim wins");
        assert!(!t.try_begin_flush(), "second claim is refused while in flight");

        let batch = t.drain();
        t.mark_flushed();
        assert_eq!(batch.len(), 1);

        t.record("b");
        assert!(t.try_begin_flush(), "guard released after a successful flush");
    }

    #[test]
    fn requeue_restores_a_failed_batch() {
        let t = AccessTelemetry::with_interval(Duration::ZERO);
        t.record("a");
        assert!(t.try_begin_flush());
        let batch = t.drain();
        assert_eq!(t.pending(), 0);

        t.requeue(batch);
        assert_eq!(t.pending(), 1);
        assert!(t.try_begin_flush(), "guard released after a failed flush");
    }
}

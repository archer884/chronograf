//! A small, ergonomic stopwatch.
//!
//! The common case — time a block of code — is a single call:
//!
//! ```
//! let sw = chronograf::start();
//! let _sum: u64 = (0..1000).sum();
//! let elapsed = sw.finish();
//! println!("{elapsed:?}");
//! ```
//!
//! For everything else there's [`Sw::stop`], [`Sw::start`], and [`Sw::peek`],
//! all idempotent so you never have to track current state. Backed by
//! [`quanta::Instant`] for fast monotonic time, and generic over an [`Instant`]
//! trait so the clock can be swapped out (e.g. for a fake clock in tests).

use core::{fmt::Debug, time::Duration};

/// Starts and returns a running stopwatch backed by [`quanta::Instant`].
///
/// This is the usual entry point: time a block, then call [`Sw::finish()`].
#[inline]
pub fn start() -> Sw<quanta::Instant> {
    Sw {
        elapsed: Duration::ZERO,
        started: Some(quanta::Instant::now()),
    }
}

/// A monotonic time source used by [`Sw`].
///
/// Abstracts over the clock so a stopwatch can be driven by something other
/// than [`quanta::Instant`] — for example, a fake clock in tests. The only
/// implementation shipped with this crate is `quanta::Instant`, chosen because
/// it is significantly faster than [`std::time::Instant`].
pub trait Instant: Copy + Debug {
    fn now() -> Self;
    fn saturating_duration_since(&self, earlier: Self) -> Duration;
}

impl Instant for quanta::Instant {
    #[inline]
    fn now() -> Self {
        quanta::Instant::now()
    }

    #[inline]
    fn saturating_duration_since(&self, earlier: Self) -> Duration {
        self.saturating_duration_since(earlier)
    }
}

/// A stopwatch: accumulates elapsed [`Duration`] across zero or more run/stop
/// cycles.
///
/// A stopped watch holds only its accumulated elapsed time; a running watch
/// additionally tracks the instant it most recently started. Construct a
/// running watch directly with [`start`], or use [`Sw::new()`] to pick a custom
/// [`Instant`] source.
#[derive(Clone, Debug)]
pub struct Sw<I> {
    elapsed: Duration,
    started: Option<I>,
}

impl<I: Instant> Sw<I> {
    /// Create a new stopwatch without starting it.
    ///
    /// By creating a stopwatch in this way, it is possible to select an alternate
    /// implementation of [`Instant`].
    #[inline]
    pub fn new() -> Self {
        Sw {
            elapsed: Duration::ZERO,
            started: None,
        }
    }

    /// Starts the stopwatch.
    ///
    /// Starting a stopwatch that is already running will have no effect.
    #[inline]
    pub fn start(&mut self) {
        self.started.get_or_insert_with(I::now);
    }

    /// Stops the stopwatch.
    ///
    /// Stopping a stopwatch that has not been started will have no effect.
    #[inline]
    pub fn stop(&mut self) {
        self.elapsed += self
            .started
            .take()
            .map(|earlier| I::now().saturating_duration_since(earlier))
            .unwrap_or(Duration::ZERO);
    }

    /// Returns the total elapsed time without stopping the watch.
    ///
    /// Combines the accumulated `elapsed` with any time accrued in the current
    /// run. Unlike [`stop`](Sw::stop)/[`finish`](Sw::finish), this does not
    /// mutate the watch.
    #[inline]
    pub fn peek(&self) -> Duration {
        let running = self
            .started
            .map(|earlier| I::now().saturating_duration_since(earlier))
            .unwrap_or(Duration::ZERO);
        self.elapsed + running
    }

    /// Consumes the stopwatch, returning the total time elapsed.
    #[inline]
    pub fn finish(mut self) -> Duration {
        self.stop();
        self.elapsed
    }
}

impl<I: Instant> Default for Sw<I> {
    fn default() -> Self {
        Sw::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    /// Deterministic clock backed by a thread-local nanosecond counter.
    ///
    /// Implementing [`Instant`] here also serves as the first proof that the
    /// trait abstraction works for a non-`quanta` time source.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct FakeInstant(u64);

    thread_local! {
        static FAKE_NOW: Cell<u64> = const { Cell::new(0) };
    }

    fn set_now(nanos: u64) {
        FAKE_NOW.set(nanos);
    }

    fn advance(nanos: u64) {
        FAKE_NOW.with(|c| c.set(c.get() + nanos));
    }

    impl Instant for FakeInstant {
        fn now() -> Self {
            FakeInstant(FAKE_NOW.get())
        }

        fn saturating_duration_since(&self, earlier: Self) -> Duration {
            Duration::from_nanos(self.0.saturating_sub(earlier.0))
        }
    }

    #[test]
    fn new_watch_finishes_at_zero() {
        set_now(0);
        let sw = Sw::<FakeInstant>::new();
        assert_eq!(sw.finish(), Duration::ZERO);
    }

    #[test]
    fn start_then_finish_reports_elapsed() {
        set_now(0);
        let mut sw = Sw::<FakeInstant>::new();
        sw.start();
        advance(1_000_000_000);
        assert_eq!(sw.finish(), Duration::from_secs(1));
    }

    #[test]
    fn start_is_idempotent_when_running() {
        set_now(0);
        let mut sw = Sw::<FakeInstant>::new();
        sw.start();
        advance(500);
        sw.start();
        advance(500);
        assert_eq!(sw.finish(), Duration::from_nanos(1_000));
    }

    #[test]
    fn stop_is_idempotent_when_stopped() {
        set_now(0);
        let mut sw = Sw::<FakeInstant>::new();
        sw.start();
        advance(1_000);
        sw.stop();
        sw.stop();
        assert_eq!(sw.peek(), Duration::from_nanos(1_000));
    }

    #[test]
    fn stop_without_start_is_noop() {
        set_now(0);
        let mut sw = Sw::<FakeInstant>::new();
        sw.stop();
        assert_eq!(sw.peek(), Duration::ZERO);
    }

    #[test]
    fn peek_does_not_mutate() {
        set_now(0);
        let mut sw = Sw::<FakeInstant>::new();
        sw.start();
        advance(1_000);
        assert_eq!(sw.peek(), Duration::from_nanos(1_000));
        advance(1_000);
        assert_eq!(sw.peek(), Duration::from_nanos(2_000));
        assert_eq!(sw.finish(), Duration::from_nanos(2_000));
    }

    #[test]
    fn peek_on_stopped_watch_returns_elapsed() {
        set_now(0);
        let mut sw = Sw::<FakeInstant>::new();
        sw.start();
        advance(2_000);
        sw.stop();
        assert_eq!(sw.peek(), Duration::from_nanos(2_000));
    }

    #[test]
    fn laps_accumulate_across_stop_start_cycles() {
        set_now(0);
        let mut sw = Sw::<FakeInstant>::new();
        sw.start();
        advance(1_000);
        sw.stop();
        advance(5_000);
        sw.start();
        advance(2_000);
        sw.stop();
        assert_eq!(sw.peek(), Duration::from_nanos(3_000));
    }

    #[test]
    fn real_quanta_start_smoke_test() {
        let _ = start().finish();
    }
}

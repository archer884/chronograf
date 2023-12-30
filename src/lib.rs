use ::core::{fmt::Debug, time::Duration};

/// Starts and returns a new stopwatch.
#[inline]
pub fn start() -> Sw<quanta::Instant> {
    Sw {
        elapsed: Duration::ZERO,
        started: Some(quanta::Instant::now()),
    }
}

pub trait Instant: Copy + Debug + Sized {
    fn now() -> Self;
    fn checked_add(&self, duration: Duration) -> Option<Self>;
    fn checked_sub(&self, duration: Duration) -> Option<Self>;
    fn saturating_duration_since(&self, earlier: Self) -> Duration;
}

impl Instant for quanta::Instant {
    fn now() -> Self {
        quanta::Instant::now()
    }

    fn checked_add(&self, duration: Duration) -> Option<Self> {
        self.checked_add(duration)
    }

    fn checked_sub(&self, duration: Duration) -> Option<Self> {
        self.checked_sub(duration)
    }

    fn saturating_duration_since(&self, earlier: Self) -> Duration {
        self.saturating_duration_since(earlier)
    }
}

#[derive(Clone, Copy, Debug)]
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
    /// Starting a stopwatch that is already running will have the effect of resetting the stopwatch.
    #[inline]
    pub fn start(&mut self) {
        self.started = Some(I::now())
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

    /// Stops the stopwatch, both consuming the stopwatch and returning the time elapsed.
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

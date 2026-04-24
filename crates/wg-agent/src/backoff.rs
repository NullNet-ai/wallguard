#![allow(dead_code)]

use std::time::Duration;

use rand::Rng;

/// Exponential backoff with ±20% jitter.
///
/// Used for reconnect loops and any retry-with-delay scenario.
pub struct Backoff {
    current:    f64,
    base:       f64,
    max:        f64,
    multiplier: f64,
}

impl Backoff {
    /// Default for agent reconnects: base=1s, max=300s, multiplier=2.0.
    pub fn default_reconnect() -> Self {
        Self::new(1.0, 300.0, 2.0)
    }

    pub fn new(base_secs: f64, max_secs: f64, multiplier: f64) -> Self {
        Self {
            current:    base_secs,
            base:       base_secs,
            max:        max_secs,
            multiplier,
        }
    }

    /// Returns the next delay duration with ±20% jitter applied, then advances
    /// the internal state for the next call.
    pub fn next(&mut self) -> Duration {
        let jitter = rand::thread_rng().gen_range(0.8_f64..1.2_f64);
        let delay  = (self.current * jitter).min(self.max * 1.2);
        self.current = (self.current * self.multiplier).min(self.max);
        Duration::from_secs_f64(delay)
    }

    /// Resets the delay back to the base value (call after a successful connection).
    pub fn reset(&mut self) {
        self.current = self.base;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_exceeds_max_with_jitter() {
        let mut b = Backoff::new(1.0, 10.0, 2.0);
        for _ in 0..30 {
            let d = b.next();
            // Allow for the upper jitter bound (1.2×).
            assert!(
                d <= Duration::from_secs_f64(10.0 * 1.2 + 0.001),
                "delay {d:?} exceeded max×jitter"
            );
        }
    }

    #[test]
    fn reset_returns_to_base() {
        let mut b = Backoff::new(1.0, 300.0, 2.0);
        for _ in 0..10 {
            b.next();
        }
        b.reset();
        let d = b.next();
        // After reset the first delay must be within base × jitter range.
        assert!(d <= Duration::from_secs(3), "post-reset delay too large: {d:?}");
        assert!(d >= Duration::from_millis(700), "post-reset delay too small: {d:?}");
    }

    #[test]
    fn jitter_stays_within_twenty_percent() {
        // Hold current fixed (multiplier=1.0) to isolate jitter.
        let mut b = Backoff::new(10.0, 1000.0, 1.0);
        for _ in 0..200 {
            let d = b.next();
            assert!(d >= Duration::from_secs_f64(8.0),  "below lower jitter: {d:?}");
            assert!(d <= Duration::from_secs_f64(12.0), "above upper jitter: {d:?}");
        }
    }

    proptest::proptest! {
        #[test]
        fn proptest_delay_bounded(
            base in 0.1_f64..10.0_f64,
            max  in 10.0_f64..600.0_f64,
            mult in 1.0_f64..4.0_f64,
            n    in 1_u32..50_u32,
        ) {
            let mut b = Backoff::new(base, max, mult);
            for _ in 0..n {
                let d = b.next();
                // Upper bound includes jitter headroom.
                proptest::prop_assert!(d <= Duration::from_secs_f64(max * 1.2 + 0.001));
                proptest::prop_assert!(d >= Duration::ZERO);
            }
        }
    }
}

use std::time::Instant;

pub(crate) struct Timer {
    interval: u64,
    instant: Instant,
}

impl Timer {
    pub(crate) fn new(interval: u64) -> Self {
        Self {
            interval,
            instant: Instant::now(),
        }
    }

    pub(crate) fn is_expired(&self) -> bool {
        self.instant.elapsed().as_millis() >= 1000 * u128::from(self.interval)
    }

    pub(crate) fn reset(&mut self) {
        self.instant = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer() {
        let mut timer = Timer::new(1);
        assert!(!timer.is_expired());
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(!timer.is_expired());
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(timer.is_expired());
        timer.reset();
        assert!(!timer.is_expired());
    }
}

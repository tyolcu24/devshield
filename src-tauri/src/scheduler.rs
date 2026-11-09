use std::time::{Duration, Instant};
pub const INTERVAL: u64 = 6 * 60 * 60;
pub fn remaining(last: Option<u64>, now: u64) -> Duration {
    Duration::from_secs(
        last.map(|last| INTERVAL.saturating_sub(now.saturating_sub(last)))
            .unwrap_or(0),
    )
}
pub fn backoff(failures: u32) -> Duration {
    Duration::from_secs(
        (60u64.saturating_mul(2u64.saturating_pow(failures.saturating_sub(1)))).min(3600),
    )
}
pub struct Schedule {
    pub due: Instant,
    pub failures: u32,
}
impl Schedule {
    pub fn new(last: Option<u64>) -> Self {
        Self {
            due: Instant::now() + remaining(last, crate::model::now()),
            failures: 0,
        }
    }
    pub fn finished(&mut self, ok: bool) {
        if ok {
            self.failures = 0;
            self.due = Instant::now()
                + Duration::from_secs(INTERVAL + u64::from(std::process::id() % 120));
        } else {
            self.failures = self.failures.saturating_add(1);
            self.due = Instant::now() + backoff(self.failures);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_scan_due_and_sleep_catches_up_once() {
        assert_eq!(remaining(None, 100).as_secs(), 0);
        assert_eq!(remaining(Some(100), 100 + INTERVAL + 5).as_secs(), 0);
        assert_eq!(remaining(Some(100), 101).as_secs(), INTERVAL - 1);
    }
    #[test]
    fn clock_rollback_does_not_trigger_loop() {
        assert_eq!(remaining(Some(100), 50).as_secs(), INTERVAL);
    }
    #[test]
    fn retries_are_bounded() {
        assert_eq!(backoff(1).as_secs(), 60);
        assert_eq!(backoff(100).as_secs(), 3600);
    }
}

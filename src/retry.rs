use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub max_attempts: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            max_attempts: 0,
        }
    }
}

impl RetryPolicy {
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base = self.initial_delay.as_millis() as f64
            * self.multiplier.powi(attempt.saturating_sub(1) as i32);
        let clamped = base.min(self.max_delay.as_millis() as f64);

        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        let jitter = (nanos as f64 / u32::MAX as f64) * clamped * 0.25;
        Duration::from_millis((clamped + jitter) as u64)
    }

    pub fn has_attempts_left(&self, attempt: u32) -> bool {
        self.max_attempts == 0 || attempt < self.max_attempts
    }
}

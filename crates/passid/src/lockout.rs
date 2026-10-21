use serde::{Deserialize, Serialize};

/// Result of asking "may this attempt proceed right now?"
#[derive(Debug, PartialEq, Eq)]
pub enum Gate {
    Open,
    /// Locked; try again in this many seconds.
    Wait(u64),
    /// Too many failures: passcode disabled, use password/passkey/other device.
    FallbackRequired,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LockoutState {
    pub failures: u32,
    pub locked_until: u64, // unix seconds
}

/// Delay applied after the Nth consecutive failure. None = fallback required.
fn delay_for(failures: u32) -> Option<u64> {
    match failures {
        0..=4 => Some(0),
        5 => Some(60),
        6 => Some(5 * 60),
        7 => Some(15 * 60),
        8 | 9 => Some(60 * 60),
        _ => None,
    }
}

impl LockoutState {
    pub fn check(&self, now: u64) -> Gate {
        if delay_for(self.failures).is_none() {
            Gate::FallbackRequired
        } else if now < self.locked_until {
            Gate::Wait(self.locked_until - now)
        } else {
            Gate::Open
        }
    }

    pub fn record_failure(&mut self, now: u64) {
        self.failures = self.failures.saturating_add(1);
        if let Some(d) = delay_for(self.failures) {
            self.locked_until = now + d;
        }
    }

    /// Call after a successful passcode OR a successful fallback login.
    pub fn record_success(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_four_failures_are_free() {
        let mut s = LockoutState::default();
        for _ in 0..4 {
            s.record_failure(1000);
            assert_eq!(s.check(1000), Gate::Open);
        }
    }

    #[test]
    fn escalating_delays() {
        let mut s = LockoutState::default();
        for _ in 0..5 {
            s.record_failure(1000);
        }
        assert_eq!(s.check(1000), Gate::Wait(60));
        assert_eq!(s.check(1030), Gate::Wait(30));
        assert_eq!(s.check(1060), Gate::Open);
        s.record_failure(1060);
        assert_eq!(s.check(1060), Gate::Wait(300));
    }

    #[test]
    fn ten_failures_requires_fallback() {
        let mut s = LockoutState::default();
        for _ in 0..10 {
            s.record_failure(1000);
        }
        assert_eq!(s.check(u64::MAX), Gate::FallbackRequired);
    }

    #[test]
    fn success_resets() {
        let mut s = LockoutState::default();
        for _ in 0..7 {
            s.record_failure(1000);
        }
        s.record_success();
        assert_eq!(s.check(1000), Gate::Open);
        assert_eq!(s.failures, 0);
    }
}

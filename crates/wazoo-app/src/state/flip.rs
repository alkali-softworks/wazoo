/*!
 * ALKALI SOFTWORKS - Wazoo
 *
 * Flip Mode Runtime Countdown State
 */

#[derive(Debug, Clone, Default)]
pub struct FlipState {
    pub countdown: u64,
}

impl FlipState {
    pub fn new(interval_secs: u64) -> Self {
        Self {
            countdown: interval_secs.max(1),
        }
    }

    pub fn reset(&mut self, interval_secs: u64) {
        self.countdown = interval_secs.max(1);
    }
}

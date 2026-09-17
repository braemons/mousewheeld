//! Turning the device's numbers into the host's, across resets.
//!
//! Two jobs, both of them about a clock or a counter that the device is
//! entitled to restart and the host is not.
//!
//! **The clock.** Every sample carries `t_us`, free-running from the board's
//! boot. The host maps it onto `CLOCK_MONOTONIC` — the join key with
//! statemachined's trace and vstimd's vblank timestamps. Without that mapping,
//! "how far did it run during trial 42" cannot be asked.
//!
//! **The counter.** A reset, a reflash or a USB re-enumeration restarts the
//! device's counts at zero. vstimd differences its segment every frame, so a
//! published value that jumps backwards jumps the camera backwards. The offset
//! here is what keeps the published accumulator continuous across a restart the
//! device itself has no memory of.

/// Device microseconds → host monotonic nanoseconds.
///
/// A **minimum filter** rather than an average: every sample's transport delay
/// is positive and variable — USB scheduling, a serial buffer, this thread
/// being descheduled — so the smallest `host − device` seen is the closest
/// thing to the true offset, and averaging would track the jitter instead of
/// rejecting it. The same shape statemachined's device-clock correlation uses.
pub struct ClockCorrelation {
    /// The best (smallest) `host_ns − device_ns` seen in this window.
    offset_ns: Option<i128>,
    samples_in_window: u32,
    /// How many samples the current estimate is allowed to serve before the
    /// window restarts. A board's crystal drifts against the host's, so an
    /// offset held forever would slowly become wrong — this re-estimates a few
    /// times a second at any plausible sample rate.
    window: u32,
    previous_device_us: u64,
}

impl ClockCorrelation {
    pub fn new() -> Self {
        Self {
            offset_ns: None,
            samples_in_window: 0,
            window: 2048,
            previous_device_us: 0,
        }
    }

    /// Feed one arrival and get the host time for it.
    ///
    /// `host_now_ns` is when this line was *read*, which is later than when the
    /// device stamped it by exactly the delay the minimum filter is rejecting.
    pub fn observe(&mut self, device_us: u64, host_now_ns: u64) -> u64 {
        // A device clock that went backwards is a board that restarted; the old
        // offset describes a machine that no longer exists.
        if device_us < self.previous_device_us {
            self.offset_ns = None;
            self.samples_in_window = 0;
        }
        self.previous_device_us = device_us;

        let device_ns = i128::from(device_us) * 1_000;
        let observed = i128::from(host_now_ns) - device_ns;
        match self.offset_ns {
            Some(current) if observed >= current && self.samples_in_window < self.window => {
                self.samples_in_window += 1;
            }
            _ => {
                self.offset_ns = Some(observed);
                self.samples_in_window = 0;
            }
        }
        let offset = self.offset_ns.unwrap_or(observed);
        (device_ns + offset).max(0) as u64
    }
}

impl Default for ClockCorrelation {
    fn default() -> Self {
        Self::new()
    }
}

/// A per-axis accumulator that never steps backwards.
///
/// The published value is `device counts + offset`. On a restart the offset
/// absorbs the difference, so a consumer differencing successive values sees
/// **no movement** across the reset rather than a kilometre of it in the wrong
/// direction.
///
/// Origins are a different number. `zero`, and an arm with `origin: "current"`,
/// move what displacement and the zones are measured from; they never touch
/// this.
pub struct Continuity {
    offsets: Vec<i64>,
    last_raw: Vec<i64>,
    started: bool,
}

impl Continuity {
    pub fn new(axes: usize) -> Self {
        Self {
            offsets: vec![0; axes],
            last_raw: vec![0; axes],
            started: false,
        }
    }

    /// Absorb a restart: whatever the device says next continues from where the
    /// published value already was.
    pub fn device_restarted(&mut self) {
        if !self.started {
            return;
        }
        for axis in 0..self.offsets.len() {
            let published = self.last_raw[axis] + self.offsets[axis];
            // The next raw value is unknown; treat it as zero, which is what a
            // board that has just booted reports. A board that restarts with a
            // non-zero counter corrects itself on the first sample below.
            self.offsets[axis] = published;
            self.last_raw[axis] = 0;
        }
    }

    /// The published counts for one sample.
    pub fn publish(&mut self, raw: &[i64]) -> Vec<i64> {
        let mut published = Vec::with_capacity(raw.len());
        for (axis, value) in raw.iter().enumerate() {
            if axis >= self.offsets.len() {
                published.push(*value);
                continue;
            }
            // A counter that went backwards without a hello is still a restart:
            // the board rebooted faster than the daemon noticed. Same rule as
            // `device_restarted` — the published value carries on from where it
            // was, and the counts the board has made since it came back are
            // movement, not a correction to throw away.
            if self.started && *value < self.last_raw[axis] - RESTART_TOLERANCE {
                self.offsets[axis] += self.last_raw[axis];
            }
            self.last_raw[axis] = *value;
            published.push(*value + self.offsets[axis]);
        }
        self.started = true;
        published
    }
}

/// A wheel run backwards is normal; a counter that fell by more than this
/// between two samples is not a wheel, it is a board that restarted.
const RESTART_TOLERANCE: i64 = 1_000_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_takes_the_smallest_delay_it_has_seen() {
        let mut clock = ClockCorrelation::new();
        // Nothing better than the first arrival yet, so it is the estimate.
        assert_eq!(clock.observe(1_000_000, 2_005_000_000), 2_005_000_000);

        // A prompter arrival: a smaller `host − device`, so it is closer to the
        // true offset and the estimate tightens onto it.
        assert_eq!(clock.observe(1_001_000, 2_005_002_000), 2_005_002_000);

        // A late one is rejected: its host time is 2_007_000_000, and what the
        // mapping returns is the device stamp under the tighter offset.
        assert_eq!(
            clock.observe(1_002_000, 2_007_000_000),
            2_006_002_000,
            "a slow arrival must not drag the estimate out"
        );
    }

    #[test]
    fn a_device_clock_that_restarts_re_estimates_rather_than_reporting_the_past() {
        let mut clock = ClockCorrelation::new();
        clock.observe(9_000_000, 10_000_000_000);
        let after = clock.observe(1_000, 10_500_000_000);
        assert_eq!(after, 10_500_000_000, "a fresh offset, not the old one");
    }

    #[test]
    fn counts_are_published_unchanged_while_nothing_restarts() {
        let mut continuity = Continuity::new(1);
        assert_eq!(continuity.publish(&[100]), vec![100]);
        assert_eq!(continuity.publish(&[250]), vec![250]);
    }

    #[test]
    fn a_reset_never_steps_the_published_value_backwards() {
        let mut continuity = Continuity::new(1);
        continuity.publish(&[10_000]);
        continuity.device_restarted();
        // The board comes back from zero and counts on; the published value
        // continues from where it was.
        assert_eq!(continuity.publish(&[0]), vec![10_000]);
        assert_eq!(continuity.publish(&[500]), vec![10_500]);
    }

    #[test]
    fn a_counter_that_falls_off_a_cliff_is_treated_as_a_restart_too() {
        // No hello arrived: the board rebooted faster than the daemon noticed.
        let mut continuity = Continuity::new(1);
        continuity.publish(&[5_000_000]);
        let published = continuity.publish(&[3]);
        assert_eq!(published, vec![5_000_003]);
    }

    #[test]
    fn running_the_wheel_backwards_is_not_mistaken_for_a_restart() {
        let mut continuity = Continuity::new(1);
        continuity.publish(&[10_000]);
        assert_eq!(continuity.publish(&[9_000]), vec![9_000], "an animal backing up");
    }
}

//! A stateful animation track: one scalar lane that knows its current value,
//! velocity and target, and retargets without discontinuity.
//!
//! [`crate::motion::Curve`] is stateless — it answers "where is this at `t`".
//! A [`Track`] owns the state a running animation needs: it holds a curve, a
//! target and the presented `(value, velocity)`, advances on a caller-supplied
//! clock, and can be retargeted mid-flight. A retarget starts the new segment
//! from the exact current `(value, velocity)`, so interrupting a spring never
//! jumps — the same "merge" behaviour a view-diff retargets property tracks
//! with. This is the piece both consumers need: an engine-side CSS timeline and
//! the reactive layer both drive one or more tracks per animated property.
//!
//! ```
//! use arniko::motion::{Curve, Track, STANDARD};
//!
//! let mut t = Track::new(0.0);
//! t.animate_to(1.0, Curve::Spring(STANDARD), 0.0);
//! assert!(t.is_active());
//! let end = STANDARD.settling_duration_from(0.0, 0.0, 1.0, 1e-3) as f64 + 1e-6;
//! assert!(!t.tick(end), "track should be settled by its settling time");
//! assert!((t.value() - 1.0).abs() < 1e-6);
//! ```

use crate::motion::Curve;

/// The default settle threshold: below this the residual position is invisible,
/// so a track snaps to its target and goes dormant.
pub const SETTLE_EPSILON: f32 = 1e-3;

/// One scalar lane of motion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Track {
    curve: Curve,
    /// Value at the start of the current segment.
    from: f32,
    /// Velocity at the start of the current segment.
    from_velocity: f32,
    /// Where the current segment is heading (a decay projects its own).
    target: f32,
    /// Clock time the current segment started.
    start: f64,
    /// Absolute clock time the current segment is considered settled.
    end: f64,
    /// Last presented value.
    value: f32,
    /// Last presented velocity.
    velocity: f32,
    active: bool,
}

impl Track {
    /// A track at rest holding `initial`.
    pub fn new(initial: f32) -> Track {
        Track {
            curve: Curve::Instant,
            from: initial,
            from_velocity: 0.0,
            target: initial,
            start: 0.0,
            end: 0.0,
            value: initial,
            velocity: 0.0,
            active: false,
        }
    }

    /// The last presented value.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// The last presented velocity (units per second).
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Where the current segment is heading.
    pub fn target(&self) -> f32 {
        self.target
    }

    /// The curve driving the current (or last) segment.
    pub fn curve(&self) -> Curve {
        self.curve
    }

    /// Whether the track still needs ticks.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Jump to `value` immediately with no motion, cancelling any animation.
    pub fn set(&mut self, value: f32, now: f64) {
        self.from = value;
        self.from_velocity = 0.0;
        self.target = value;
        self.value = value;
        self.velocity = 0.0;
        self.curve = Curve::Instant;
        self.start = now;
        self.end = now;
        self.active = false;
    }

    /// Retarget to `target` on `curve`, starting from the current
    /// `(value, velocity)` at `now`.
    ///
    /// Continuous: position and velocity carry over, so retargeting a moving
    /// track never jumps. A `Curve::Decay` ignores `target` and projects its own
    /// rest point from the current velocity.
    pub fn animate_to(&mut self, target: f32, curve: Curve, now: f64) {
        self.from = self.value;
        self.from_velocity = self.velocity;
        self.target = match curve {
            Curve::Decay(d) => self.from + self.from_velocity / d.k(),
            _ => target,
        };
        self.curve = curve;
        self.start = now;
        self.active = true;
        let span = settle_duration(
            curve,
            self.from,
            self.from_velocity,
            self.target,
            SETTLE_EPSILON,
        );
        self.end = now + span as f64;
        // Land immediately for a zero-span curve (Instant, or already settled).
        self.tick(now);
    }

    /// Advance the presented value to clock time `now`.
    ///
    /// Returns `true` while the track is still animating (i.e. the caller should
    /// keep requesting frames). Once settled the value is snapped to the target
    /// and the track goes dormant.
    pub fn tick(&mut self, now: f64) -> bool {
        if !self.active {
            return false;
        }
        let elapsed = (now - self.start).max(0.0) as f32;
        let (x, v) = self
            .curve
            .sample(self.from, self.from_velocity, self.target, elapsed);
        self.value = x;
        self.velocity = v;
        if now >= self.end {
            self.value = self.target;
            self.velocity = 0.0;
            self.active = false;
        }
        self.active
    }
}

/// Seconds a segment starting from `(from, v0)` toward `target` runs before it
/// is settled under `eps`.
fn settle_duration(curve: Curve, from: f32, v0: f32, target: f32, eps: f32) -> f32 {
    match curve {
        Curve::Instant => 0.0,
        Curve::Tween { duration, .. } => duration.max(0.0),
        Curve::Spring(s) => s.settling_duration_from(from, v0, target, eps),
        Curve::Decay(d) => {
            if v0.abs() <= eps {
                0.0
            } else {
                (v0.abs() / eps).ln() / d.k()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::{Decay, STANDARD, STANDARD_EXIT};

    #[test]
    fn new_track_is_at_rest() {
        let t = Track::new(3.0);
        assert_eq!(t.value(), 3.0);
        assert_eq!(t.velocity(), 0.0);
        assert!(!t.is_active());
    }

    #[test]
    fn set_jumps_and_cancels() {
        let mut t = Track::new(0.0);
        t.animate_to(10.0, Curve::Spring(STANDARD), 0.0);
        assert!(t.is_active());
        t.set(4.0, 0.1);
        assert_eq!(t.value(), 4.0);
        assert_eq!(t.target(), 4.0);
        assert!(!t.is_active());
    }

    #[test]
    fn spring_track_reaches_target_and_settles() {
        let mut t = Track::new(0.0);
        t.animate_to(1.0, Curve::Spring(STANDARD), 0.0);
        assert!(t.is_active());
        let end = STANDARD.settling_duration_from(0.0, 0.0, 1.0, SETTLE_EPSILON) as f64;
        // Mid-flight: moving.
        t.tick(end * 0.25);
        assert!(t.value() > 0.0 && t.value() < 1.0, "value={}", t.value());
        assert!(t.velocity() != 0.0);
        // Settled by the settling time.
        assert!(!t.tick(end + 1e-6));
        assert_eq!(t.value(), 1.0);
        assert_eq!(t.velocity(), 0.0);
    }

    #[test]
    fn retarget_is_continuous_in_position_and_velocity() {
        let mut t = Track::new(0.0);
        t.animate_to(1.0, Curve::Spring(STANDARD), 0.0);
        t.tick(0.12);
        let v_before = t.value();
        let vel_before = t.velocity();
        assert!(v_before > 0.0 && v_before < 1.0);

        // Retarget to a new target mid-flight; the presented value/velocity must
        // not jump at the instant of retarget.
        t.animate_to(0.4, Curve::Spring(STANDARD_EXIT), 0.12);
        assert!((t.value() - v_before).abs() < 1e-6);
        assert!((t.velocity() - vel_before).abs() < 1e-6);

        // And it still converges to the new target.
        let end = 0.12
            + STANDARD_EXIT.settling_duration_from(v_before, vel_before, 0.4, SETTLE_EPSILON)
                as f64;
        assert!(!t.tick(end + 1e-6));
        assert_eq!(t.value(), 0.4);
    }

    #[test]
    fn tween_settles_at_its_duration() {
        let mut t = Track::new(0.0);
        t.animate_to(10.0, Curve::linear(0.5), 0.0);
        assert!(t.is_active());
        t.tick(0.25);
        assert!((t.value() - 5.0).abs() < 1e-4, "value={}", t.value());
        assert!(t.is_active());
        assert!(!t.tick(0.5));
        assert_eq!(t.value(), 10.0);
    }

    #[test]
    fn decay_projects_its_own_rest_point() {
        let decay = Decay { rate: 4.0 };
        let mut t = Track::new(0.0);
        // Gain velocity by animating toward a far target briefly.
        t.animate_to(100.0, Curve::Spring(STANDARD), 0.0);
        t.tick(0.05);
        let from = t.value();
        let v0 = t.velocity();
        assert!(v0 > 0.0);

        // Retarget into a fling decay: it ignores the passed target and projects
        // its own rest point at from + v0/k.
        t.animate_to(0.0, Curve::Decay(decay), 0.05);
        assert!(t.target() > from);
        assert!((t.target() - (from + v0 / decay.k())).abs() < 1e-3);

        let end = 0.05 + ((v0.abs() / SETTLE_EPSILON).ln() / decay.k()) as f64;
        assert!(!t.tick(end + 1e-3));
        assert_eq!(t.value(), t.target());
    }

    #[test]
    fn instant_curve_lands_on_the_same_tick() {
        let mut t = Track::new(1.0);
        t.animate_to(9.0, Curve::Instant, 0.0);
        assert!(!t.is_active());
        assert_eq!(t.value(), 9.0);
    }

    #[test]
    fn tick_is_idempotent_after_settle() {
        let mut t = Track::new(0.0);
        t.animate_to(2.0, Curve::Spring(STANDARD), 0.0);
        let end = STANDARD.settling_duration_from(0.0, 0.0, 2.0, SETTLE_EPSILON) as f64 + 1.0;
        assert!(!t.tick(end));
        assert!(!t.tick(end + 5.0));
        assert_eq!(t.value(), 2.0);
    }
}

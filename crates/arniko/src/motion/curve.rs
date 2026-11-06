//! Curves: what a value animates on, and how to sample it.
//!
//! A [`Curve`] is a small descriptor — a spring, an eased tween, a fling decay,
//! or an instant jump — and [`Curve::sample`] evaluates any of them at a time
//! `t`, returning `(position, velocity)`. A driver (an engine-side CSS timeline,
//! or a reactive binding) owns retargeting and time; the curve only knows its
//! own shape.

use crate::motion::{Easing, Spring};

/// The shape of one animation segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve {
    /// Physics. Retargets can start the new spring from the exact current
    /// `(position, velocity)`, so motion stays continuous.
    Spring(Spring),
    /// A fixed-duration eased tween (a CSS transition). Starts from rest; a
    /// driver that wants carried velocity should retarget through a spring.
    Tween { duration: f32, easing: Easing },
    /// Fling inertia: velocity decays exponentially toward a projected rest.
    Decay(Decay),
    /// Jump to the target this frame.
    Instant,
}

impl Curve {
    /// A tween of `duration` seconds on `easing`.
    pub const fn tween(duration: f32, easing: Easing) -> Curve {
        Curve::Tween { duration, easing }
    }

    /// A linear tween of `duration` seconds.
    pub const fn linear(duration: f32) -> Curve {
        Curve::Tween {
            duration,
            easing: Easing::Linear,
        }
    }

    /// An `ease-in-out` tween of `duration` seconds.
    pub const fn ease_in_out(duration: f32) -> Curve {
        Curve::Tween {
            duration,
            easing: Easing::EASE_IN_OUT,
        }
    }

    /// When the animation "logically" completes: a spring's perceptual duration,
    /// a tween's duration, a decay's time constant. Choreography cues on this
    /// rather than on physical rest.
    pub fn logical_duration(&self) -> f32 {
        match *self {
            Curve::Spring(s) => s.duration,
            Curve::Tween { duration, .. } => duration,
            Curve::Decay(d) => 1.0 / d.k(),
            Curve::Instant => 0.0,
        }
    }

    /// Position and velocity `t` seconds after leaving `x0` at velocity `v0`
    /// toward `target`.
    ///
    /// `Spring` and `Decay` use `v0`; `Tween` starts from rest and ignores it;
    /// `Instant` returns `(target, 0)` at any `t`.
    pub fn sample(self, x0: f32, v0: f32, target: f32, t: f32) -> (f32, f32) {
        match self {
            Curve::Spring(s) => s.sample(x0, v0, target, t),
            Curve::Instant => (target, 0.0),
            Curve::Decay(d) => d.sample(x0, v0, t),
            Curve::Tween { duration, easing } => {
                if duration <= 0.0 {
                    return (target, 0.0);
                }
                let u = (t / duration).clamp(0.0, 1.0);
                let (y, slope) = easing.eval_with_slope(u);
                let x = x0 + (target - x0) * y;
                let v = if u <= 0.0 || u >= 1.0 {
                    0.0
                } else {
                    (target - x0) * slope / duration
                };
                (x, v)
            }
        }
    }
}

impl From<Spring> for Curve {
    fn from(s: Spring) -> Curve {
        Curve::Spring(s)
    }
}

impl From<Decay> for Curve {
    fn from(d: Decay) -> Curve {
        Curve::Decay(d)
    }
}

/// Exponential velocity decay: `v(t) = v0·e^{−k·t}`, coming to rest at
/// `x0 + v0/k`. Used for fling inertia after a scroll or drag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decay {
    /// Rate constant `k` in 1/s.
    pub rate: f32,
}

impl Decay {
    /// iOS `UIScrollView.DecelerationRate.normal`: 0.998 per ms.
    pub const IOS_NORMAL: Decay = Decay { rate: 2.002_002 };
    /// iOS `UIScrollView.DecelerationRate.fast`: 0.99 per ms.
    pub const IOS_FAST: Decay = Decay { rate: 10.050_336 };

    /// From a per-millisecond deceleration rate `r` (UIKit): `k = −1000·ln r`.
    pub fn per_ms(r: f32) -> Decay {
        assert!(r > 0.0 && r < 1.0, "deceleration rate must be in (0, 1)");
        Decay {
            rate: -1000.0 * r.ln(),
        }
    }

    /// Effective rate, floored to avoid a zero divisor.
    #[inline]
    pub fn k(self) -> f32 {
        self.rate.max(1e-3)
    }

    /// Position and velocity at `t`. The rest point is `x0 + v0/k`; `target`
    /// plays no part (decay projects its own).
    pub fn sample(self, x0: f32, v0: f32, t: f32) -> (f32, f32) {
        let k = self.k();
        let t = t.max(0.0);
        let e = (-k * t).exp();
        (x0 + (v0 / k) * (1.0 - e), v0 * e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instant_jumps() {
        let c = Curve::Instant;
        assert_eq!(c.sample(0.0, 5.0, 3.0, 0.0), (3.0, 0.0));
        assert_eq!(c.sample(0.0, 5.0, 3.0, 10.0), (3.0, 0.0));
        assert_eq!(c.logical_duration(), 0.0);
    }

    #[test]
    fn spring_curve_delegates() {
        let s = Spring::new(0.3, 0.0);
        let c: Curve = s.into();
        assert_eq!(c.sample(0.0, 0.0, 1.0, 0.15), s.sample(0.0, 0.0, 1.0, 0.15));
        assert_eq!(c.logical_duration(), 0.3);
    }

    #[test]
    fn tween_hits_endpoints_and_ignores_start_velocity() {
        let c = Curve::ease_in_out(0.4);
        let (x, _) = c.sample(2.0, 99.0, 10.0, 0.0);
        assert!((x - 2.0).abs() < 1e-6);
        let (x, v) = c.sample(2.0, 99.0, 10.0, 0.4);
        assert!((x - 10.0).abs() < 1e-6);
        assert_eq!(v, 0.0);
        // Mid-flight it is between.
        let (x, _) = c.sample(2.0, 0.0, 10.0, 0.2);
        assert!(x > 2.0 && x < 10.0);
        assert_eq!(c.logical_duration(), 0.4);
    }

    #[test]
    fn linear_tween_is_linear() {
        let c = Curve::linear(1.0);
        let (x, v) = c.sample(0.0, 0.0, 10.0, 0.25);
        assert!((x - 2.5).abs() < 1e-6);
        assert!((v - 10.0).abs() < 1e-6);
    }

    #[test]
    fn decay_rests_at_projected_point() {
        let d = Decay { rate: 4.0 };
        let (x0, v0) = (0.0, 8.0);
        // Rest point is x0 + v0/k.
        let (x, v) = d.sample(x0, v0, 10.0);
        assert!((x - 2.0).abs() < 1e-3, "{x}");
        assert!(v.abs() < 1e-3);
        // Velocity halves every ln(2)/k seconds.
        let (_, v_half) = d.sample(x0, v0, 2.0_f32.ln() / 4.0);
        assert!((v_half - 4.0).abs() < 1e-3);
    }

    #[test]
    fn decay_per_ms_matches_ios_constants() {
        let normal = Decay::per_ms(0.998);
        assert!((normal.rate - Decay::IOS_NORMAL.rate).abs() < 1e-3);
        let fast = Decay::per_ms(0.99);
        assert!((fast.rate - Decay::IOS_FAST.rate).abs() < 1e-3);
    }
}

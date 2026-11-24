//! Analytic springs parameterised by perceptual duration and bounce.
//!
//! A spring is described the way people tune it — a perceptual `duration` (the
//! undamped period) and a `bounce` in `(-1, 1)` — rather than by raw
//! stiffness/damping. `bounce = 0` is critically damped, positive values
//! overshoot, negative values are overdamped. The mapping is the same one
//! SwiftUI's `Spring(duration:bounce:)` and Flutter's
//! `SpringDescription.withDurationAndBounce` use:
//!
//! ```text
//! ω = 2π / duration                    (mass 1)
//! k = ω²                               (stiffness)
//! ζ = 1 − bounce                       (bounce ≥ 0)
//! ζ = 1 / (1 + bounce)                 (bounce < 0, overdamped)
//! c = 2ζω                              (damping)
//! ```

/// A damped spring with unit mass, described by perceptual duration and bounce.
///
/// `Spring::new` validates its inputs (also in `const` contexts). Use the
/// [`crate::motion`] palette for common feels rather than tuning raw numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// Perceptual duration in seconds: the undamped period, `2π/ω`.
    pub duration: f32,
    /// Damping character in `(-1, 1)`. `0` critically damped, `> 0` overshoots,
    /// `< 0` overdamped.
    pub bounce: f32,
}

impl Spring {
    /// `Spring(duration:bounce:)`. Panics on a non-positive duration or a
    /// bounce outside `(-1, 1)`.
    pub const fn new(duration: f32, bounce: f32) -> Spring {
        assert!(
            duration > 0.0 && duration < 1e4,
            "spring duration must be positive seconds"
        );
        assert!(
            bounce > -1.0 && bounce < 1.0,
            "spring bounce must be in (-1, 1)"
        );
        Spring { duration, bounce }
    }

    /// From a `response` (undamped period, same number as `duration`) and a
    /// damping fraction `ζ`, as SwiftUI's `.spring(response:dampingFraction:)`.
    pub fn from_response(response: f32, damping_fraction: f32) -> Spring {
        Spring::from_omega_zeta(core::f32::consts::TAU / response, damping_fraction)
    }

    /// From angular frequency `ω` (rad/s) and damping ratio `ζ` (`> 0`).
    pub fn from_omega_zeta(omega: f32, zeta: f32) -> Spring {
        assert!(omega > 0.0 && zeta > 0.0, "omega and zeta must be positive");
        let bounce = if zeta <= 1.0 {
            1.0 - zeta
        } else {
            1.0 / zeta - 1.0
        };
        Spring::new(core::f32::consts::TAU / omega, bounce.clamp(-0.999, 0.999))
    }

    /// From physical stiffness, damping and mass, as CSS-physics spring solvers
    /// and Motion's `{ stiffness, damping, mass }` do.
    pub fn from_stiffness_damping(stiffness: f32, damping: f32, mass: f32) -> Spring {
        let omega = (stiffness / mass).sqrt();
        let zeta = damping / (2.0 * (stiffness * mass).sqrt());
        Spring::from_omega_zeta(omega, zeta)
    }

    /// A copy with a different duration, keeping the bounce.
    pub fn with_duration(self, duration: f32) -> Spring {
        Spring::new(duration, self.bounce)
    }

    /// Angular frequency `ω = 2π / duration` (rad/s).
    #[inline]
    pub fn omega(self) -> f32 {
        core::f32::consts::TAU / self.duration
    }

    /// Damping ratio `ζ`.
    #[inline]
    pub fn zeta(self) -> f32 {
        if self.bounce >= 0.0 {
            1.0 - self.bounce
        } else {
            1.0 / (1.0 + self.bounce)
        }
    }

    /// Stiffness `k = ω²` (unit mass).
    pub fn stiffness(self) -> f32 {
        let w = self.omega();
        w * w
    }

    /// Damping coefficient `c = 2ζω` (unit mass).
    pub fn damping(self) -> f32 {
        2.0 * self.zeta() * self.omega()
    }

    /// Position and velocity `t` seconds after leaving `x0` at velocity `v0`
    /// toward `target`. Closed form for every damping regime and exact at any
    /// `t` (including `t < 0`, which returns the start state).
    pub fn sample(self, x0: f32, v0: f32, target: f32, t: f32) -> (f32, f32) {
        let a = x0 - target;
        let t = t.max(0.0);
        if a == 0.0 && v0 == 0.0 {
            return (target, 0.0);
        }
        let omega = self.omega();
        let zeta = self.zeta();

        if (zeta - 1.0).abs() < 1e-4 {
            // Critically damped: (a + b·t)·e^{−ωt}.
            let b = v0 + omega * a;
            let e = (-omega * t).exp();
            let x = (a + b * t) * e;
            let v = (b - omega * (a + b * t)) * e;
            (target + x, v)
        } else if zeta < 1.0 {
            // Under-damped: e^{−σt}(a·cos(ωd·t) + b·sin(ωd·t)).
            let sigma = zeta * omega;
            let wd = omega * (1.0 - zeta * zeta).sqrt();
            let b = (v0 + sigma * a) / wd;
            let e = (-sigma * t).exp();
            let (s, c) = (wd * t).sin_cos();
            let x = e * (a * c + b * s);
            let v = e * ((b * wd - sigma * a) * c - (a * wd + sigma * b) * s);
            (target + x, v)
        } else {
            // Over-damped: c1·e^{r1·t} + c2·e^{r2·t}, both roots negative.
            let root = (zeta * zeta - 1.0).sqrt();
            let r1 = -omega * (zeta - root);
            let r2 = -omega * (zeta + root);
            let c1 = (v0 - r2 * a) / (r1 - r2);
            let c2 = a - c1;
            let (e1, e2) = ((r1 * t).exp(), (r2 * t).exp());
            (target + c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
        }
    }

    /// Seconds until a unit step (`0 → 1` from rest) stays within `epsilon` of
    /// the target in position and within `10·epsilon` in velocity. Numeric and
    /// regime-agnostic; capped at 120 s.
    pub fn settling_duration(self, epsilon: f32) -> f32 {
        assert!(
            epsilon.is_finite() && epsilon > 0.0,
            "epsilon must be positive"
        );
        let vel_eps = epsilon * 10.0;
        let settled = |t: f32| {
            let (x, v) = self.sample(0.0, 0.0, 1.0, t);
            (x - 1.0).abs() <= epsilon && v.abs() <= vel_eps
        };
        if settled(0.0) {
            return 0.0;
        }
        let mut hi = self.duration.max(1e-3);
        while !settled(hi) {
            hi *= 2.0;
            if hi > MAX_SETTLE {
                return MAX_SETTLE;
            }
        }
        let mut lo = 0.0;
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if settled(mid) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        hi
    }

    /// Seconds until an arbitrary retarget from `(from, velocity)` toward
    /// `target` settles under the same thresholds as [`Spring::settling_duration`].
    pub fn settling_duration_from(
        self,
        from: f32,
        velocity: f32,
        target: f32,
        epsilon: f32,
    ) -> f32 {
        assert!(
            epsilon.is_finite() && epsilon > 0.0,
            "epsilon must be positive"
        );
        let vel_eps = epsilon * 10.0;
        let settled = |t: f32| {
            let (x, v) = self.sample(from, velocity, target, t);
            (x - target).abs() <= epsilon && v.abs() <= vel_eps
        };
        if settled(0.0) {
            return 0.0;
        }
        let mut hi = self.duration.max(1e-3);
        while !settled(hi) {
            hi *= 2.0;
            if hi > MAX_SETTLE {
                return MAX_SETTLE;
            }
        }
        let mut lo = 0.0;
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if settled(mid) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        hi
    }
}

/// Upper bound on any reported settling time, in seconds.
const MAX_SETTLE: f32 = 120.0;

/// An enter/exit spring pair. Exits generally run shorter than enters, so a
/// dismissal feels quicker than a presentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringPair {
    pub enter: Spring,
    pub exit: Spring,
}

impl SpringPair {
    pub const fn new(enter: Spring, exit: Spring) -> SpringPair {
        SpringPair { enter, exit }
    }

    /// Instant both ways: enters and leaves in the frame it starts.
    pub const NONE: SpringPair = SpringPair::new(
        Spring {
            duration: 1e-6,
            bounce: 0.0,
        },
        Spring {
            duration: 1e-6,
            bounce: 0.0,
        },
    );

    /// A pair whose exit is `enter` shortened by `ratio` with halved bounce.
    /// `ratio` around `0.8` reads naturally for most motion.
    pub fn from_enter(enter: Spring, ratio: f32) -> SpringPair {
        SpringPair {
            enter,
            exit: Spring::new(enter.duration * ratio, enter.bounce * 0.5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_is_exact_at_start() {
        let s = Spring::new(0.3, 0.25);
        let (x, v) = s.sample(10.0, 2.0, 0.0, 0.0);
        assert_eq!(x, 10.0);
        assert_eq!(v, 2.0);
        // Negative time clamps to the start state.
        let (x, v) = s.sample(10.0, 2.0, 0.0, -1.0);
        assert_eq!(x, 10.0);
        assert_eq!(v, 2.0);
    }

    #[test]
    fn rest_is_stationary() {
        let s = Spring::new(0.3, 0.0);
        let (x, v) = s.sample(5.0, 0.0, 5.0, 0.5);
        assert_eq!((x, v), (5.0, 0.0));
    }

    #[test]
    fn critical_approaches_without_overshoot() {
        let s = Spring::new(0.4, 0.0);
        let mut prev = 0.0;
        for i in 0..=200 {
            let t = i as f32 * 0.01;
            let (x, _) = s.sample(0.0, 0.0, 1.0, t);
            assert!(
                x >= prev - 1e-4,
                "critical spring must not undershoot: {x} < {prev}"
            );
            assert!(
                x <= 1.0 + 1e-4,
                "critically damped spring must not overshoot: {x}"
            );
            prev = x;
        }
        assert!((s.sample(0.0, 0.0, 1.0, 5.0).0 - 1.0).abs() < 1e-3);
    }

    #[test]
    fn underdamped_overshoots_then_settles() {
        let s = Spring::new(0.4, 0.4);
        let peak = (0..400)
            .map(|i| s.sample(0.0, 0.0, 1.0, i as f32 * 0.005).0)
            .fold(f32::MIN, f32::max);
        assert!(
            peak > 1.0 + 0.05,
            "under-damped spring should overshoot, peak={peak}"
        );
        assert!((s.sample(0.0, 0.0, 1.0, 8.0).0 - 1.0).abs() < 1e-3);
    }

    #[test]
    fn overdamped_does_not_overshoot() {
        let s = Spring::new(0.5, -0.5);
        for i in 0..=400 {
            let x = s.sample(0.0, 0.0, 1.0, i as f32 * 0.01).0;
            assert!(x <= 1.0 + 1e-4, "over-damped spring overshot: {x}");
        }
    }

    #[test]
    fn omega_zeta_and_physics_round_trip() {
        let s = Spring::new(0.5, 0.0);
        assert!((s.zeta() - 1.0).abs() < 1e-6);
        assert!((s.omega() - core::f32::consts::TAU / 0.5).abs() < 1e-6);
        assert!((s.stiffness() - s.omega() * s.omega()).abs() < 1e-6);
        assert!((s.damping() - 2.0 * s.zeta() * s.omega()).abs() < 1e-6);

        // from_omega_zeta is the inverse of omega()/zeta().
        let back = Spring::from_omega_zeta(s.omega(), s.zeta());
        assert!((back.duration - s.duration).abs() < 1e-4);
        assert!((back.bounce - s.bounce).abs() < 1e-4);
    }

    #[test]
    fn from_response_matches_swiftui_shape() {
        let s = Spring::from_response(0.5, 1.0);
        assert!((s.duration - 0.5).abs() < 1e-6);
        assert!((s.bounce - 0.0).abs() < 1e-6);
    }

    #[test]
    fn from_stiffness_damping_matches_omega_zeta() {
        // k = ω², c = 2ζω with unit mass.
        let omega = 10.0_f32;
        let zeta = 0.7_f32;
        let s = Spring::from_stiffness_damping(omega * omega, 2.0 * zeta * omega, 1.0);
        assert!((s.omega() - omega).abs() < 1e-3);
        assert!((s.zeta() - zeta).abs() < 1e-3);
    }

    #[test]
    fn settling_duration_bounds_the_curve() {
        for (d, b) in [(0.2, 0.0), (0.3, 0.3), (0.5, -0.4), (0.1, 0.0)] {
            let s = Spring::new(d, b);
            let eps = 1e-3;
            let t = s.settling_duration(eps);
            assert!(t > 0.0 && t.is_finite());
            let (x, v) = s.sample(0.0, 0.0, 1.0, t);
            assert!(
                (x - 1.0).abs() <= eps,
                "not settled in position at t={t}: {x}"
            );
            assert!(
                v.abs() <= eps * 10.0,
                "not settled in velocity at t={t}: {v}"
            );
            // And it stays settled afterwards.
            let (x, v) = s.sample(0.0, 0.0, 1.0, t + 0.5);
            assert!((x - 1.0).abs() <= eps);
            assert!(v.abs() <= eps * 10.0);
        }
    }

    #[test]
    fn settling_duration_from_handles_a_retarget() {
        let s = Spring::new(0.3, 0.1);
        let eps = 1e-3_f32;
        let vel_eps = eps * 10.0;
        let t = s.settling_duration_from(4.0, -3.0, 0.0, eps);
        let (x, v) = s.sample(4.0, -3.0, 0.0, t);
        assert!(x.abs() <= eps, "position not settled at t={t}: {x}");
        assert!(v.abs() <= vel_eps, "velocity not settled at t={t}: {v}");
    }

    #[test]
    fn spring_pair_exit_is_shorter() {
        let p = SpringPair::from_enter(Spring::new(0.4, 0.3), 0.8);
        assert!((p.exit.duration - 0.32).abs() < 1e-6);
        assert!((p.exit.bounce - 0.15).abs() < 1e-6);
    }

    #[test]
    #[should_panic]
    fn zero_duration_panics() {
        let _ = Spring::new(0.0, 0.0);
    }

    #[test]
    #[should_panic]
    fn out_of_range_bounce_panics() {
        let _ = Spring::new(0.3, 1.0);
    }
}

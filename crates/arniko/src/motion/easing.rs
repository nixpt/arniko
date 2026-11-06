//! CSS timing functions: `linear`, `cubic-bezier()` and `steps()`.
//!
//! These map progress `x ∈ [0, 1]` to eased progress and are the timing half of
//! a [`crate::motion::Curve::Tween`]. The cubic-Bézier evaluator follows the
//! same Newton-then-bisection method browsers use (`UnitBezier`) so arniko can
//! later drive real CSS `transition-timing-function` values.

/// A CSS timing function.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    /// `linear`.
    Linear,
    /// `cubic-bezier(x1, y1, x2, y2)`.
    CubicBezier(CubicBezier),
    /// `steps(count, position)`.
    Steps { count: u16, position: StepPosition },
}

impl Easing {
    /// `cubic-bezier(x1, y1, x2, y2)`.
    pub const fn cubic(x1: f32, y1: f32, x2: f32, y2: f32) -> Easing {
        Easing::CubicBezier(CubicBezier::new(x1, y1, x2, y2))
    }

    /// The CSS keyword `ease`.
    pub const EASE: Easing = Easing::cubic(0.25, 0.1, 0.25, 1.0);
    /// The CSS keyword `ease-in`.
    pub const EASE_IN: Easing = Easing::cubic(0.42, 0.0, 1.0, 1.0);
    /// The CSS keyword `ease-out`.
    pub const EASE_OUT: Easing = Easing::cubic(0.0, 0.0, 0.58, 1.0);
    /// The CSS keyword `ease-in-out`.
    pub const EASE_IN_OUT: Easing = Easing::cubic(0.42, 0.0, 0.58, 1.0);

    /// Eased progress at `x`.
    #[inline]
    pub fn eval(&self, x: f32) -> f32 {
        self.eval_with_slope(x).0
    }

    /// `(y, dy/dx)` at `x`. The slope carries velocity when a tween is
    /// retargeted mid-flight.
    #[inline]
    pub fn eval_with_slope(&self, x: f32) -> (f32, f32) {
        match self {
            Easing::Linear => (x, 1.0),
            Easing::CubicBezier(b) => b.eval_with_slope(x),
            Easing::Steps { count, position } => (steps(x, *count, *position), 0.0),
        }
    }
}

/// Where a `steps()` jump lands (CSS `jump-start`, `jump-end`, `jump-none`,
/// `jump-both`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPosition {
    Start,
    End,
    None,
    Both,
}

/// A unit cubic Bézier through `(0,0)`, `(x1,y1)`, `(x2,y2)`, `(1,1)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl CubicBezier {
    /// `x1` and `x2` are clamped to `[0, 1]`, as CSS requires.
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let x1 = if x1 < 0.0 {
            0.0
        } else if x1 > 1.0 {
            1.0
        } else {
            x1
        };
        let x2 = if x2 < 0.0 {
            0.0
        } else if x2 > 1.0 {
            1.0
        } else {
            x2
        };
        CubicBezier { x1, y1, x2, y2 }
    }

    /// Polynomial coefficients `(a, b, c)` of the axis Bézier through
    /// `(0, 0, a1, a2, 1)`.
    #[inline]
    fn coefficients(a1: f32, a2: f32) -> (f32, f32, f32) {
        let c = 3.0 * a1;
        let b = 3.0 * (a2 - a1) - c;
        let a = 1.0 - c - b;
        (a, b, c)
    }

    /// Solves `x(t) = x` (Newton, then bisection) and returns `(y(t), dy/dx)`.
    pub fn eval_with_slope(&self, x: f32) -> (f32, f32) {
        if x <= 0.0 {
            return (0.0, self.slope_at(0.0));
        }
        if x >= 1.0 {
            return (1.0, self.slope_at(1.0));
        }
        let t = self.solve_t(x);
        let (ay, by, cy) = Self::coefficients(self.y1, self.y2);
        let y = ((ay * t + by) * t + cy) * t;
        (y, self.slope_at(t))
    }

    fn slope_at(&self, t: f32) -> f32 {
        let (ax, bx, cx) = Self::coefficients(self.x1, self.x2);
        let (ay, by, cy) = Self::coefficients(self.y1, self.y2);
        let dx = (3.0 * ax * t + 2.0 * bx) * t + cx;
        let dy = (3.0 * ay * t + 2.0 * by) * t + cy;
        if dx.abs() < 1e-6 {
            // Vertical tangent (e.g. x1 = 0 at t = 0): report a steep finite slope.
            return if dy == 0.0 { 0.0 } else { dy.signum() * 1e3 };
        }
        dy / dx
    }

    fn solve_t(&self, x: f32) -> f32 {
        const EPS: f32 = 1e-7;
        let (ax, bx, cx) = Self::coefficients(self.x1, self.x2);
        let sample = |t: f32| ((ax * t + bx) * t + cx) * t;

        let mut t = x;
        for _ in 0..8 {
            let err = sample(t) - x;
            if err.abs() < EPS {
                return t;
            }
            let d = (3.0 * ax * t + 2.0 * bx) * t + cx;
            if d.abs() < 1e-6 {
                break;
            }
            t -= err / d;
        }

        let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
        t = x;
        for _ in 0..40 {
            let v = sample(t);
            if (v - x).abs() < EPS {
                break;
            }
            if x > v {
                lo = t;
            } else {
                hi = t;
            }
            t = 0.5 * (lo + hi);
        }
        t
    }
}

/// The `steps()` function.
fn steps(x: f32, n: u16, pos: StepPosition) -> f32 {
    if x >= 1.0 {
        return 1.0;
    }
    if x <= 0.0 && !matches!(pos, StepPosition::Start | StepPosition::Both) {
        return 0.0;
    }
    let n = n.max(1) as f32;
    let mut step = (x.max(0.0) * n).floor();
    if matches!(pos, StepPosition::Start | StepPosition::Both) {
        step += 1.0;
    }
    let jumps = match pos {
        StepPosition::Start | StepPosition::End => n,
        StepPosition::Both => n + 1.0,
        StepPosition::None => (n - 1.0).max(1.0),
    };
    (step.min(jumps) / jumps).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_is_identity() {
        assert_eq!(Easing::Linear.eval(0.25), 0.25);
        assert_eq!(Easing::Linear.eval_with_slope(0.25), (0.25, 1.0));
    }

    #[test]
    fn css_ease_known_value() {
        // Chrome getComputedTiming for `ease` at 50 % progress is ~0.8024.
        let y = Easing::EASE.eval(0.5);
        assert!((y - 0.802_4).abs() < 1e-3, "{y}");
        assert_eq!(Easing::EASE_IN_OUT.eval(0.5), 0.5_f32);
        assert_eq!(Easing::EASE_OUT.eval(0.0), 0.0);
        assert_eq!(Easing::EASE_OUT.eval(1.0), 1.0);
    }

    #[test]
    fn bezier_endpoints_are_exact() {
        for e in [
            Easing::EASE,
            Easing::EASE_IN,
            Easing::EASE_OUT,
            Easing::EASE_IN_OUT,
        ] {
            assert_eq!(e.eval(0.0), 0.0);
            assert_eq!(e.eval(1.0), 1.0);
        }
    }

    #[test]
    fn bezier_solves_x_exactly() {
        let b = CubicBezier::new(0.2, 0.8, 0.2, 1.0);
        let (ax, bx, cx) = CubicBezier::coefficients(b.x1, b.x2);
        for i in 1..1000 {
            let x = i as f32 / 1000.0;
            let t = b.solve_t(x);
            let back = ((ax * t + bx) * t + cx) * t;
            assert!((back - x).abs() < 1e-5, "x={x} back={back}");
        }
    }

    #[test]
    fn steps_follow_css() {
        let s = |x, n, p| {
            Easing::Steps {
                count: n,
                position: p,
            }
            .eval(x)
        };
        assert_eq!(s(0.0, 4, StepPosition::End), 0.0);
        assert_eq!(s(0.3, 4, StepPosition::End), 0.25);
        assert_eq!(s(0.3, 4, StepPosition::Start), 0.5);
        assert_eq!(s(0.5, 2, StepPosition::None), 1.0);
        assert!((s(0.4, 2, StepPosition::Both) - 1.0 / 3.0).abs() < 1e-6);
        assert_eq!(s(1.0, 3, StepPosition::Both), 1.0);
    }
}

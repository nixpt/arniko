//! A named motion palette.
//!
//! Components and apps pick a feel by name — `STANDARD`, `SPRINGY`, `GENTLE` —
//! rather than tuning raw durations, the way a design system names its shadows
//! and radii. Each entry is a [`Spring`]; related entries have an `_EXIT`
//! variant that is shorter, so a dismissal reads quicker than a presentation.
//! [`SpringPair`] constants bundle the two for presence and open/close toggles.
//!
//! The named-palette idea is borrowed from Rune (`rune-motion`, see
//! `docs/RUNE_BORROW.md`); the values here are arniko's own starting point and
//! can be retuned without changing any call site.

use crate::motion::{Curve, Easing, Spring, SpringPair};

// ── Enter springs ───────────────────────────────────────────────────────────

/// Nothing perceptible: use for reduced-motion snap or a zero-length transition.
pub const INSTANT: Spring = Spring::new(0.10, 0.00);
/// Small, immediate state flips (a checkbox tick).
pub const QUICK: Spring = Spring::new(0.20, 0.00);
/// The default for most UI motion.
pub const STANDARD: Spring = Spring::new(0.30, 0.00);
/// Longer content moves — a panel growing, a layout reflow.
pub const GENTLE: Spring = Spring::new(0.40, 0.00);
/// A drag settling into place; slightly slower than `STANDARD`.
pub const GLIDE: Spring = Spring::new(0.35, 0.00);
/// A little overshoot: a button pop, a toggled knob.
pub const SPRINGY: Spring = Spring::new(0.32, 0.18);
/// Pronounced overshoot for playful, attention-getting motion.
pub const BOUNCY: Spring = Spring::new(0.42, 0.32);
/// Large, slow travel (a camera, a sheet).
pub const DRIFT: Spring = Spring::new(0.80, 0.00);

// ── Exit springs (shorter, lower bounce) ────────────────────────────────────

pub const INSTANT_EXIT: Spring = Spring::new(0.08, 0.00);
pub const QUICK_EXIT: Spring = Spring::new(0.16, 0.00);
pub const STANDARD_EXIT: Spring = Spring::new(0.24, 0.00);
pub const GENTLE_EXIT: Spring = Spring::new(0.32, 0.00);
pub const GLIDE_EXIT: Spring = Spring::new(0.28, 0.00);
pub const SPRINGY_EXIT: Spring = Spring::new(0.256, 0.09);
pub const BOUNCY_EXIT: Spring = Spring::new(0.336, 0.16);
pub const DRIFT_EXIT: Spring = Spring::new(0.64, 0.00);

// ── Enter/exit pairs ────────────────────────────────────────────────────────

pub const STANDARD_PAIR: SpringPair = SpringPair::new(STANDARD, STANDARD_EXIT);
pub const GENTLE_PAIR: SpringPair = SpringPair::new(GENTLE, GENTLE_EXIT);
pub const SPRINGY_PAIR: SpringPair = SpringPair::new(SPRINGY, SPRINGY_EXIT);
pub const BOUNCY_PAIR: SpringPair = SpringPair::new(BOUNCY, BOUNCY_EXIT);

// ── CSS transition curves ───────────────────────────────────────────────────

/// A CSS-style enter transition: `cubic-bezier(.2,.8,.2,1)` over 150 ms.
pub const CSS_ENTER: Curve = Curve::tween(0.15, Easing::cubic(0.2, 0.8, 0.2, 1.0));
/// A CSS-style exit transition: `cubic-bezier(.4,0,1,1)` over 110 ms.
pub const CSS_EXIT: Curve = Curve::tween(0.11, Easing::cubic(0.4, 0.0, 1.0, 1.0));

/// The fade duration used when reduced motion replaces spatial travel, in
/// seconds. Spatial motion snaps; opacity/presence take this instead.
pub const REDUCED_FADE_SECS: f32 = 0.15;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exits_are_shorter_than_enters() {
        for (enter, exit) in [
            (INSTANT, INSTANT_EXIT),
            (QUICK, QUICK_EXIT),
            (STANDARD, STANDARD_EXIT),
            (GENTLE, GENTLE_EXIT),
            (GLIDE, GLIDE_EXIT),
            (SPRINGY, SPRINGY_EXIT),
            (BOUNCY, BOUNCY_EXIT),
            (DRIFT, DRIFT_EXIT),
        ] {
            assert!(
                exit.duration < enter.duration,
                "exit {:?} should be shorter than enter {:?}",
                exit,
                enter
            );
        }
    }

    #[test]
    fn pairs_match_from_enter_ratio() {
        // The named pairs use the same 0.8 shortening as SpringPair::from_enter.
        let close = |a: Spring, b: Spring| {
            (a.duration - b.duration).abs() < 1e-6 && (a.bounce - b.bounce).abs() < 1e-6
        };
        for (pair, enter) in [
            (STANDARD_PAIR, STANDARD),
            (SPRINGY_PAIR, SPRINGY),
            (BOUNCY_PAIR, BOUNCY),
        ] {
            let expected = SpringPair::from_enter(enter, 0.8);
            assert!(
                close(pair.enter, expected.enter) && close(pair.exit, expected.exit),
                "{pair:?} vs {expected:?}"
            );
        }
    }

    #[test]
    fn standard_settles_quickly() {
        // Sanity: the default spring is perceptually settled within ~1 s.
        let t = STANDARD.settling_duration(1e-3);
        assert!(t > 0.0 && t < 1.0, "STANDARD settles in {t}s");
    }

    #[test]
    fn css_curves_are_well_formed() {
        assert_eq!(CSS_ENTER.logical_duration(), 0.15);
        assert_eq!(CSS_EXIT.logical_duration(), 0.11);
        let (x, _) = CSS_ENTER.sample(0.0, 0.0, 1.0, 0.15);
        assert!((x - 1.0).abs() < 1e-6);
    }
}

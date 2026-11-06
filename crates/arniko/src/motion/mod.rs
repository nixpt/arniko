//! Motion math: springs, curves and CSS timing functions.
//!
//! This is the pure-math half of arniko's motion runtime. It borrows the
//! *shape* of the problem from Rune's `rune-motion` crate (see
//! `docs/RUNE_BORROW.md`, track 1) but is an independent implementation: a
//! damped-oscillator sampler, the CSS timing-function evaluator, a small curve
//! enum, and a named motion palette. Nothing here touches the DOM, the reactor
//! or the renderer, so it can drive either an engine-side CSS transition
//! timeline or reactive spring bindings later.
//!
//! Every sampler is closed-form and exact at any `t`, so a driver can sample it
//! at a frame timestamp and retarget from the current `(position, velocity)`
//! without a discontinuity.
//!
//! ```
//! use arniko::motion::STANDARD;
//!
//! // A unit step under a named spring settles to 1 with no residual velocity.
//! let t = STANDARD.settling_duration(1e-3);
//! let (x, v) = STANDARD.sample(0.0, 0.0, 1.0, t);
//! assert!((x - 1.0).abs() < 1e-2);
//! assert!(v.abs() < 1e-1);
//! ```

mod curve;
mod easing;
mod spring;
mod tokens;

pub use curve::{Curve, Decay};
pub use easing::{CubicBezier, Easing, StepPosition};
pub use spring::{Spring, SpringPair};
pub use tokens::*;

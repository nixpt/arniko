//! Engine wiring for the motion runtime: animate a numeric CSS property on a
//! DOM node, driven by the reactive [`Reactor`](crate::reactive::Reactor).
//!
//! [`Reactor::animate`](crate::reactive::Reactor::animate) registers (or
//! retargets) an animation for a `(node, property)` pair using a
//! [`crate::motion::Curve`]; [`Reactor::tick`](crate::reactive::Reactor::tick)
//! advances every animation to a clock time and writes the presented value with
//! `DocumentMutator::set_style_property`. This is track 1 (motion) of
//! `docs/RUNE_BORROW.md` wired into the reactive layer.
//!
//! As Rune does, **nothing animates on the first frame**: the first `animate`
//! for a `(node, property)` sets the value instantly, and only later changes
//! spring. Retargeting mid-flight keeps position and velocity, so interrupting
//! an animation never jumps.

use bliss_dom::DocumentMutator;

use crate::live::{Live, SpringBinding};
use crate::motion::Track;

/// A numeric CSS property an animation can drive, with its unit.
///
/// Most animated properties are lengths (`px`) or unitless (`opacity`, a
/// scale/rotation factor). Use [`CssProperty::px`] / [`CssProperty::unitless`]
/// for others.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CssProperty {
    /// The CSS property name, e.g. `"opacity"` or `"width"`.
    pub name: &'static str,
    /// The unit appended to the value: `""` (unitless) or `"px"`, etc.
    pub unit: &'static str,
}

impl CssProperty {
    /// `opacity` (unitless).
    pub const OPACITY: CssProperty = CssProperty::unitless("opacity");
    /// `width` in `px`.
    pub const WIDTH: CssProperty = CssProperty::px("width");
    /// `height` in `px`.
    pub const HEIGHT: CssProperty = CssProperty::px("height");

    /// A length property in `px`.
    pub const fn px(name: &'static str) -> CssProperty {
        CssProperty { name, unit: "px" }
    }

    /// A unitless property.
    pub const fn unitless(name: &'static str) -> CssProperty {
        CssProperty { name, unit: "" }
    }

    /// The CSS value string for `value`, e.g. `"0.5"` or `"200px"`.
    pub(crate) fn format(self, value: f32) -> String {
        format!("{}{}", value, self.unit)
    }
}

/// One animated `(node, property)` lane. Owned by the reactor.
pub(crate) struct Animation {
    pub node: usize,
    pub property: CssProperty,
    pub track: Track,
    /// The last value written to the DOM, so a settled lane stops writing.
    pub written: Option<f32>,
}

impl Animation {
    /// A new lane settled at `value` (the first set is instant).
    pub fn new(node: usize, property: CssProperty, value: f32) -> Self {
        Animation {
            node,
            property,
            track: Track::new(value),
            written: None,
        }
    }

    /// Advance to `now` and write the presented value if it changed. Returns
    /// `true` if the DOM was written.
    pub fn tick(&mut self, now: f64, mutator: &mut DocumentMutator) -> bool {
        self.track.tick(now);
        let value = self.track.value();
        if self.written != Some(value) {
            self.written = Some(value);
            mutator.set_style_property(self.node, self.property.name, &self.property.format(value));
            true
        } else {
            false
        }
    }
}

/// A `Live<f32>` bound to a CSS property, applied on every [`Reactor::tick`].
///
/// A `Raw` binding writes the source value directly — the "live-only" path: a
/// fast value (a scroll offset, a pointer-driven transform) reaches the DOM
/// without a reactive flush or a view re-run. A `Smoothed` binding runs the
/// source through a [`SpringBinding`], so a jumpy source still animates.
///
/// [`Reactor::tick`]: crate::reactive::Reactor::tick
pub(crate) enum LiveBinding {
    Raw {
        node: usize,
        property: CssProperty,
        source: Live<f32>,
        written: Option<f32>,
    },
    Smoothed {
        node: usize,
        property: CssProperty,
        binding: SpringBinding,
        written: Option<f32>,
    },
}

impl LiveBinding {
    /// The node this binding writes to.
    pub fn node(&self) -> usize {
        match self {
            LiveBinding::Raw { node, .. } | LiveBinding::Smoothed { node, .. } => *node,
        }
    }

    /// The property this binding writes.
    pub fn property(&self) -> CssProperty {
        match self {
            LiveBinding::Raw { property, .. } | LiveBinding::Smoothed { property, .. } => *property,
        }
    }

    /// Read the source (through the spring, if smoothed), write the value if it
    /// changed. Returns `true` if the DOM was written.
    pub fn tick(&mut self, now: f64, mutator: &mut DocumentMutator) -> bool {
        match self {
            LiveBinding::Raw {
                node,
                property,
                source,
                written,
            } => {
                let value = source.get();
                if *written != Some(value) {
                    *written = Some(value);
                    mutator.set_style_property(*node, property.name, &property.format(value));
                    true
                } else {
                    false
                }
            }
            LiveBinding::Smoothed {
                node,
                property,
                binding,
                written,
            } => {
                binding.tick(now);
                let value = binding.value();
                if *written != Some(value) {
                    *written = Some(value);
                    mutator.set_style_property(*node, property.name, &property.format(value));
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Whether this binding still needs frames (a smoothed binding mid-spring).
    pub fn is_animating(&self) -> bool {
        matches!(self, LiveBinding::Smoothed { binding, .. } if binding.is_animating())
    }

    /// The current presented value.
    pub fn value(&self) -> f32 {
        match self {
            LiveBinding::Raw { source, .. } => source.get(),
            LiveBinding::Smoothed { binding, .. } => binding.value(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::{Curve, STANDARD};
    use crate::reactive::Reactor;
    use bliss_dom::{BaseDocument, DocumentConfig, qual_name};

    fn doc_with_div() -> (BaseDocument, usize) {
        let mut doc = BaseDocument::new(DocumentConfig::default());
        let mut mutator = doc.mutate();
        let node = mutator.create_element(qual_name!("div"), vec![]);
        mutator.append_children(0, &[node]);
        drop(mutator);
        (doc, node)
    }

    #[test]
    fn css_property_formats_value_and_unit() {
        assert_eq!(CssProperty::OPACITY.format(0.5), "0.5");
        assert_eq!(CssProperty::WIDTH.format(200.0), "200px");
    }

    #[test]
    fn first_animate_is_instant_then_retarget_springs() {
        let (mut doc, node) = doc_with_div();
        let reactor_anim = CssProperty::OPACITY;
        let mut reactor = Reactor::new();

        // First call: instant (nothing animates on the first frame).
        reactor.animate(node, reactor_anim, 0.0, Curve::Spring(STANDARD), 0.0);
        let mut mutator = doc.mutate();
        assert!(
            reactor.tick(0.0, &mut mutator),
            "first tick writes the value"
        );
        assert_eq!(reactor.animation_value(node, reactor_anim), Some(0.0));
        assert!(!reactor.is_animating());

        // Retarget: springs toward 1.
        reactor.animate(node, reactor_anim, 1.0, Curve::Spring(STANDARD), 0.0);
        assert!(reactor.tick(0.05, &mut mutator));
        let mid = reactor.animation_value(node, reactor_anim).expect("value");
        assert!(mid > 0.0 && mid < 1.0, "mid={mid}");
        assert!(reactor.is_animating());

        // Settles at the target, and then stops writing.
        let end = STANDARD.settling_duration_from(0.0, 0.0, 1.0, 1e-3) as f64 + 1e-3;
        assert!(reactor.tick(end, &mut mutator));
        assert_eq!(reactor.animation_value(node, reactor_anim), Some(1.0));
        assert!(!reactor.is_animating());
        assert!(!reactor.tick(end + 1.0, &mut mutator), "settled: no writes");
    }

    #[test]
    fn retarget_keeps_position_and_velocity() {
        let (mut doc, node) = doc_with_div();
        let prop = CssProperty::WIDTH;
        let mut reactor = Reactor::new();
        let mut mutator = doc.mutate();

        reactor.animate(node, prop, 0.0, Curve::Spring(STANDARD), 0.0);
        reactor.tick(0.0, &mut mutator);
        reactor.animate(node, prop, 100.0, Curve::Spring(STANDARD), 0.0);
        reactor.tick(0.1, &mut mutator);
        let before = reactor.animation_value(node, prop).expect("value");

        // Interrupt toward a nearer target at the same instant: no jump.
        reactor.animate(node, prop, 40.0, Curve::Spring(STANDARD), 0.1);
        reactor.tick(0.1, &mut mutator);
        let after = reactor.animation_value(node, prop).expect("value");
        assert!(
            (after - before).abs() < 1e-4,
            "position jumped: {before} -> {after}"
        );
    }

    #[test]
    fn raw_live_binding_writes_only_on_change() {
        use crate::live::Live;

        let (mut doc, node) = doc_with_div();
        let prop = CssProperty::OPACITY;
        let live = Live::new(0.0_f32);
        let mut reactor = Reactor::new();
        reactor.bind_live(node, prop, live.clone());
        let mut mutator = doc.mutate();

        // First tick writes the initial value.
        assert!(reactor.tick(0.0, &mut mutator));
        assert_eq!(reactor.live_value(node, prop), Some(0.0));
        // Unchanged: no write, and a raw binding is never "animating".
        assert!(!reactor.tick(0.1, &mut mutator));
        assert!(!reactor.is_animating());

        // A live set reaches the DOM on the next tick, with no reactive flush.
        assert!(live.set(1.0));
        assert!(reactor.tick(0.2, &mut mutator));
        assert_eq!(reactor.live_value(node, prop), Some(1.0));
    }

    #[test]
    fn smoothed_live_binding_springs_to_the_source() {
        use crate::live::Live;

        let (mut doc, node) = doc_with_div();
        let prop = CssProperty::WIDTH;
        let live = Live::new(0.0_f32);
        let mut reactor = Reactor::new();
        reactor.bind_live_smoothed(node, prop, live.clone(), STANDARD);
        let mut mutator = doc.mutate();

        reactor.tick(0.0, &mut mutator);
        assert_eq!(reactor.live_value(node, prop), Some(0.0));
        assert!(!reactor.is_animating());

        live.set(100.0);
        // The frame the change is seen retargets (no jump); movement starts the
        // next frame.
        reactor.tick(0.05, &mut mutator);
        reactor.tick(0.10, &mut mutator);
        let mid = reactor.live_value(node, prop).expect("value");
        assert!(mid > 0.0 && mid < 100.0, "mid={mid}");
        assert!(reactor.is_animating());

        // The retarget happened at t = 0.05, so it settles by then + the spring's span.
        let end = 0.05 + STANDARD.settling_duration_from(0.0, 0.0, 100.0, 1e-3) as f64 + 1e-3;
        reactor.tick(end, &mut mutator);
        assert_eq!(reactor.live_value(node, prop), Some(100.0));
        assert!(!reactor.is_animating());
    }

    #[test]
    fn rebinding_replaces_the_source() {
        use crate::live::Live;

        let (mut doc, node) = doc_with_div();
        let prop = CssProperty::OPACITY;
        let first = Live::new(0.25_f32);
        let second = Live::new(0.75_f32);
        let mut reactor = Reactor::new();
        reactor.bind_live(node, prop, first.clone());
        reactor.bind_live(node, prop, second.clone());
        let mut mutator = doc.mutate();

        reactor.tick(0.0, &mut mutator);
        assert_eq!(reactor.live_value(node, prop), Some(0.75));
        // The first source no longer drives the property.
        first.set(0.1);
        assert!(!reactor.tick(0.1, &mut mutator));
    }
}

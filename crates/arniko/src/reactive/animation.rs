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
}

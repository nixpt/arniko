//! Live values: high-frequency state that updates **without** a reactive flush.
//!
//! A `Signal` (in `crate::reactive`, behind the `reactive` feature) `set` marks
//! the reactor dirty, re-runs the bound view and patches the DOM. That is the
//! right cost for most state, but some values change every frame — a scroll
//! offset, a camera pan, a spring's presented value — and re-running the view
//! for each would dominate the frame. A [`Live<T>`] is shared, lock-free state a
//! driver reads directly: `set` returns whether the value changed and wakes the
//! shell for a redraw, but never touches the reactor, so the view, diff and
//! layout are skipped.
//!
//! [`SpringBinding`] smooths a `Live<f32>` onto a spring ([`crate::motion::Track`]),
//! so a fast source still animates with velocity-preserving retargets.
//!
//! This is the live-value channel of `docs/RUNE_BORROW.md` track 3. It is pure
//! state with no DOM or reactor dependency; wiring a live value to a specific
//! style property is a follow-up.
//!
//! ```
//! use arniko::live::Live;
//!
//! let scroll = Live::new(0.0_f32);
//! assert!(scroll.set(120.0));   // changed
//! assert!(!scroll.set(120.0));  // same value: no change, no wake
//! assert_eq!(scroll.get(), 120.0);
//! ```

use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use parking_lot::Mutex;

use crate::motion::{Curve, Spring, Track};

/// A wake callback shared among all clones of a live value. Called from
/// [`Live::set`] when the value changes, so the shell can request a redraw
/// without a reactor flush. Mirrors `reactive::signal::WakeFn`.
pub type WakeFn = Arc<dyn Fn() + Send + Sync>;

/// A value that round-trips through a 32-bit atomic.
///
/// Implemented for [`f32`], [`u32`] and [`i32`] — the numeric types UI values
/// are made of. 64-bit values are out of scope: they would need a 64-bit atomic
/// that some targets (notably wasm) lack.
pub trait LiveValue: Copy + PartialEq + 'static {
    fn to_bits(self) -> u32;
    fn from_bits(bits: u32) -> Self;
}

impl LiveValue for f32 {
    fn to_bits(self) -> u32 {
        self.to_bits()
    }
    fn from_bits(bits: u32) -> Self {
        f32::from_bits(bits)
    }
}

impl LiveValue for u32 {
    fn to_bits(self) -> u32 {
        self
    }
    fn from_bits(bits: u32) -> Self {
        bits
    }
}

impl LiveValue for i32 {
    fn to_bits(self) -> u32 {
        self as u32
    }
    fn from_bits(bits: u32) -> Self {
        bits as i32
    }
}

/// Shared, lock-free high-frequency state. Clone-to-share.
pub struct Live<T: LiveValue> {
    bits: Arc<AtomicU32>,
    waker: Arc<Mutex<Option<WakeFn>>>,
    _marker: PhantomData<T>,
}

impl<T: LiveValue> Clone for Live<T> {
    fn clone(&self) -> Self {
        Live {
            bits: Arc::clone(&self.bits),
            waker: Arc::clone(&self.waker),
            _marker: PhantomData,
        }
    }
}

impl<T: LiveValue> Live<T> {
    /// A new value, read by anyone holding a clone.
    pub fn new(value: T) -> Self {
        Live {
            bits: Arc::new(AtomicU32::new(value.to_bits())),
            waker: Arc::new(Mutex::new(None)),
            _marker: PhantomData,
        }
    }

    /// The current value.
    pub fn get(&self) -> T {
        T::from_bits(self.bits.load(Ordering::Acquire))
    }

    /// Set the value. Returns `true` if it changed; on a change, calls the
    /// waker (if one is attached) so the shell redraws. Never touches the reactor.
    pub fn set(&self, value: T) -> bool {
        let changed = self.bits.swap(value.to_bits(), Ordering::AcqRel) != value.to_bits();
        if changed {
            // Clone the callback out before calling: it may re-enter `set`.
            let waker = self.waker.lock().clone();
            if let Some(waker) = waker {
                waker();
            }
        }
        changed
    }

    /// Set the value from a function of the current one. Returns `true` if
    /// the result changed.
    pub fn update(&self, f: impl FnOnce(T) -> T) -> bool {
        self.set(f(self.get()))
    }

    /// Attach a redraw callback, shared with every clone. Replaces any previous one.
    pub fn set_waker(&self, waker: WakeFn) {
        *self.waker.lock() = Some(waker);
    }

    /// Remove the redraw callback.
    pub fn clear_waker(&self) {
        *self.waker.lock() = None;
    }
}

/// A spring-smoothed view of a [`Live<f32>`].
///
/// Each [`tick`](SpringBinding::tick) reads the source and, if it moved,
/// retargets a [`Track`] from the current `(value, velocity)` toward it — so a
/// jumpy source (a scroll offset snap, a telemetry feed) still animates, and
/// interrupting a running spring never jumps.
pub struct SpringBinding {
    source: Live<f32>,
    spring: Spring,
    track: Track,
}

impl SpringBinding {
    /// Follow `source` on `spring`, starting settled at its current value.
    pub fn new(source: Live<f32>, spring: Spring) -> Self {
        let track = Track::new(source.get());
        SpringBinding {
            source,
            spring,
            track,
        }
    }

    /// The presented (smoothed) value.
    pub fn value(&self) -> f32 {
        self.track.value()
    }

    /// Whether the smoothing is still moving.
    pub fn is_animating(&self) -> bool {
        self.track.is_active()
    }

    /// Advance toward the source value at clock time `now`. Returns `true`
    /// while still animating.
    pub fn tick(&mut self, now: f64) -> bool {
        let target = self.source.get();
        if self.track.target() != target {
            self.track
                .animate_to(target, Curve::Spring(self.spring), now);
        }
        self.track.tick(now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::STANDARD;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn set_reports_change() {
        let live = Live::new(1.0_f32);
        assert_eq!(live.get(), 1.0);
        assert!(live.set(2.0));
        assert!(!live.set(2.0));
        assert_eq!(live.get(), 2.0);
    }

    #[test]
    fn clone_shares_state() {
        let a = Live::new(0_u32);
        let b = a.clone();
        a.set(7);
        assert_eq!(b.get(), 7);
        assert!(!b.set(7));
        assert!(b.set(8));
        assert_eq!(a.get(), 8);
    }

    #[test]
    fn update_applies_a_function() {
        let live = Live::new(10_i32);
        assert!(live.update(|v| v + 5));
        assert_eq!(live.get(), 15);
        assert!(!live.update(|v| v));
    }

    #[test]
    fn waker_fires_only_on_change() {
        let live = Live::new(0_u32);
        let wakes = Arc::new(AtomicUsize::new(0));
        let counter = wakes.clone();
        live.set_waker(Arc::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }));
        live.set(1);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
        live.set(1); // unchanged
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
        live.set(2);
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
        live.clear_waker();
        live.set(3);
        assert_eq!(wakes.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn waker_is_shared_with_clones() {
        let a = Live::new(0_u32);
        let b = a.clone();
        let wakes = Arc::new(AtomicUsize::new(0));
        let counter = wakes.clone();
        a.set_waker(Arc::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }));
        b.set(5);
        assert_eq!(wakes.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn spring_binding_is_idle_at_the_source() {
        let src = Live::new(3.0_f32);
        let mut binding = SpringBinding::new(src.clone(), STANDARD);
        assert_eq!(binding.value(), 3.0);
        assert!(
            !binding.tick(0.0),
            "nothing to do when the source is unchanged"
        );
    }

    #[test]
    fn spring_binding_animates_toward_a_new_source() {
        let src = Live::new(0.0_f32);
        let mut binding = SpringBinding::new(src.clone(), STANDARD);
        src.set(10.0);
        // First tick retargets; mid-flight the presented value is between.
        assert!(binding.tick(0.0));
        binding.tick(0.05);
        let mid = binding.value();
        assert!(mid > 0.0 && mid < 10.0, "mid={mid}");
        // Settled by the spring's settling time.
        let end = STANDARD.settling_duration_from(0.0, 0.0, 10.0, 1e-3) as f64 + 1e-3;
        assert!(!binding.tick(end));
        assert_eq!(binding.value(), 10.0);
    }
}

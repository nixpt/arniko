//! Bubbletea-inspired spinner widget for `ratatui`.
//!
//! Vendored from [`ratatui-cheese`](https://crates.io/crates/ratatui-cheese)
//! v0.7.0 — see `LICENSE-MIT` and `NOTICE` at the root of this crate for
//! full attribution. The source was originally written by Shashank Tomar and
//! is licensed under MIT.
//!
//! The public surface (`Spinner`, `SpinnerState`, `SpinnerType`) is taken
//! verbatim from upstream's `crates/ratatui-cheese/src/spinner.rs`.
//!
//! Tornado wiring: `tornado` re-exports this crate as
//! `tornado::spinner` (when the `spinner` feature is enabled), so
//! downstream apps can write
//! `tornado::spinner::SpinnerState::new(SpinnerType::Dot)` directly.
//!
//! # Tick loop integration
//!
//! `run_app(app, tick_rate)` calls `app.update()` once per iteration
//! followed by `term.draw(|frame| app.draw(frame))`. The app is
//! responsible for measuring `dt = tick_start.elapsed()` between
//! iterations and forwarding it to [`SpinnerState::tick`]:
//!
//! ```ignore
//! // in your TuiApp impl
//! fn draw(&mut self, frame: &mut Frame) {
//!     let dt = self.last_tick.elapsed();
//!     self.spinner_state.tick(dt);
//!     let area = frame.area();
//!     frame.render_stateful_widget(
//!         Spinner::default(),
//!         area,
//!         &mut self.spinner_state,
//!     );
//! }
//! ```
//!
//! `SpinnerState::tick(Duration)` accumulates the interval budget
//! internally, so transient stutter in the host loop does not visibly
//! desync spinner cadence.

pub mod spinner;

pub use spinner::{Spinner, SpinnerState, SpinnerType};

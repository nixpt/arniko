//! # tui-shell — Shared Terminal UI Shell for Exosphere
//!
//! Provides a standard terminal setup, event loop, and panic hook
//! so TUI apps (vortex, teddy, dashboard, voyager) don't each
//! duplicate the same crossterm boilerplate.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use tui_shell::{terminal, event, TuiApp};
//! use ratatui::Frame;
//! use std::time::Duration;
//!
//! struct MyApp { running: bool }
//!
//! impl TuiApp for MyApp {
//!     fn draw(&mut self, frame: &mut Frame) {
//!         // render widgets...
//!     }
//!     fn handle_event(&mut self, event: event::TuiEvent) {
//!         // handle input...
//!     }
//!     fn should_quit(&self) -> bool { !self.running }
//!     fn name(&self) -> &str { "my-app" }
//! }
//!
//! fn main() -> std::io::Result<()> {
//!     run_app(MyApp { running: true }, Duration::from_millis(100))
//! }
//! ```

pub mod event;
pub mod terminal;

#[cfg(feature = "exoshell")]
pub mod exoshell;

use ratatui::Frame;
use std::time::Duration;

/// Trait that TUI applications implement to use the shared shell
pub trait TuiApp {
    /// Render the UI into the frame
    fn draw(&mut self, frame: &mut Frame);

    /// Handle an input event
    fn handle_event(&mut self, event: event::TuiEvent);

    /// Return true when the app should exit
    fn should_quit(&self) -> bool;

    /// Application name (used for ExoShell registration)
    fn name(&self) -> &str {
        "tui-app"
    }

    /// Capabilities this app requests from ExoShell
    fn capabilities(&self) -> Vec<String> {
        vec![]
    }
}

/// Run a TUI application with the standard terminal setup.
///
/// Initializes the terminal, optionally registers with ExoShell (if the `exoshell`
/// feature is enabled and ExoShell is running), enters the draw-event loop,
/// and restores the terminal on exit or panic.
pub fn run_app(app: impl TuiApp, tick_rate: Duration) -> std::io::Result<()> {
    #[cfg(feature = "exoshell")]
    {
        run_app_with_exoshell(app, tick_rate)
    }

    #[cfg(not(feature = "exoshell"))]
    {
        run_app_standalone(app, tick_rate)
    }
}

/// Run without ExoShell integration
fn run_app_standalone(mut app: impl TuiApp, tick_rate: Duration) -> std::io::Result<()> {
    let mut term = terminal::init()?;
    let result = run_loop(&mut term, &mut app, tick_rate);
    terminal::restore();
    result
}

/// Run with optional ExoShell registration
#[cfg(feature = "exoshell")]
fn run_app_with_exoshell(mut app: impl TuiApp, tick_rate: Duration) -> std::io::Result<()> {
    let mut term = terminal::init()?;

    // Try to register with ExoShell (async, via a small runtime)
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let name = app.name().to_string();
    let caps = app.capabilities();
    let exo_state = rt.block_on(exoshell::ExoShellState::try_connect(&name, caps));

    if exo_state.is_connected() {
        tracing::info!(app = %name, "Running under ExoShell");
    }

    let result = run_loop(&mut term, &mut app, tick_rate);

    // Clean disconnect
    rt.block_on(exo_state.disconnect());

    terminal::restore();
    result
}

fn run_loop(
    term: &mut terminal::ExoTerminal,
    app: &mut impl TuiApp,
    tick_rate: Duration,
) -> std::io::Result<()> {
    while !app.should_quit() {
        term.draw(|frame| app.draw(frame))?;
        let evt = event::poll(tick_rate)?;
        app.handle_event(evt);
    }
    Ok(())
}

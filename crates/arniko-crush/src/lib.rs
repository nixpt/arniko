//! # Arniko CRUSH integration
//!
//! Exposes [Arniko](https://github.com/nixpt/arniko) UI components as host
//! capabilities for the CRUSH/CVM1 runtime. A Crush capsule can build rich
//! HTML UI by calling capability functions like `arniko.button` and
//! `arniko.card`, then embed the returned markup in its output.
//!
//! ## Example CASM
//!
//! ```text
//! .func main
//! PUSH_STR "Click me"
//! PUSH_STR "accent"
//! CAP_CALL "arniko.button" 2
//! CAP_CALL "io.print" 1
//! HALT
//! ```
//!
//! The host must register the capabilities. `arniko-crush` exposes its own
//! [`register`] function (rather than a `HostCapsBuilder` method) to avoid a
//! circular dependency on `crush-lang-sdk`:
//!
//! ```rust,no_run
//! use crush_lang_sdk::HostCapsBuilder;
//!
//! let mut host_caps = HostCapsBuilder::new().build();
//! arniko_crush::register(&mut host_caps);
//! ```

pub mod caps;

pub use caps::register;

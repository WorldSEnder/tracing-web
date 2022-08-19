//! A tracing compatible subscriber layer for web platforms.
//!
//! # Example usage
//!
//! ```rust, no_run
//! use tracing_web::{MakeConsoleWriter, performance_layer};
//! use tracing_subscriber::fmt::format::Pretty;
//! use tracing_subscriber::fmt::time::UtcTime;
//! use tracing_subscriber::prelude::*;
//!
//! let fmt_layer = tracing_subscriber::fmt::layer()
//!     .with_ansi(false) // Only partially supported across browsers
//!     .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
//!     .with_writer(MakeConsoleWriter); // write events to the console
//! let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
//!
//! tracing_subscriber::registry()
//!     .with(fmt_layer)
//!     .with(perf_layer)
//!     .init();
//! ```

#![deny(
    missing_docs,
    bare_trait_objects,
    anonymous_parameters,
    elided_lifetimes_in_paths
)]

mod performance_layer;
pub use performance_layer::{
    performance_layer, FormatSpan, FormatSpanFromFields, PerformanceEventsLayer,
};
mod console_writer;
pub use console_writer::{ConsoleWriter, MakeConsoleWriter};

//! A tracing compatible subscriber layer for web platforms.
//!
//! # Example usage
//!
//! ```rust, no_run
//! use tracing_web::{MakeWebConsoleWriter, performance_layer};
//! use tracing_subscriber::fmt::format::Pretty;
//! use tracing_subscriber::fmt::time::UtcTime;
//! use tracing_subscriber::prelude::*;
//!
//! let console_writer = MakeWebConsoleWriter::new();
//! let fmt_layer = tracing_subscriber::fmt::layer()
//!     .with_ansi(false) // Only partially supported across browsers
//!     .without_time()   // std::time is not available in browsers
//!     .with_writer(console_writer); // write events to the console
//!
//! // The performance_layer records span entry and exit, and events to the Performance
//! // Web API. Each event gets a mark, and a measure represents the during between entry
//! // and exit.
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
pub use console_writer::{ConsoleWriter, MakeConsoleWriter, MakeWebConsoleWriter};

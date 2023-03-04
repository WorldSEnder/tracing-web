##  tracing-web

A [`tracing`] compatible subscriber layer for web platforms.

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![Apache licensed][apache-badge]][apache-url]

[Documentation][docs-url]

[`tracing`]: https://crates.io/crates/tracing
[crates-badge]: https://img.shields.io/crates/v/tracing-web.svg
[crates-url]: https://crates.io/crates/tracing-web
[docs-badge]: https://docs.rs/tracing-web/badge.svg
[docs-url]: https://docs.rs/tracing-web
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE-MIT
[apache-badge]: https://img.shields.io/badge/license-Apache-blue.svg
[apache-url]: LICENSE-APACHE

# Overview

`tracing-web` can be used in conjunction with the [`tracing-subscriber`] crate to quickly install a
subscriber that emits messages to the dev-tools console, and events to the [Performance API]. An example
configuration can look like

```rust
use tracing_web::{MakeConsoleWriter, performance_layer};
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .with_timer(UtcTime::rfc_3339()) // see also note below
        .with_writer(MakeConsoleWriter); // write events to the console
    let perf_layer = performance_layer()
        .with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init(); // Install these as subscribers to tracing events

    todo!("write your awesome application");
}
```

Note: To use `UtcTime` on `web` targets, you need to enable the `wasm_bindgen` feature of the `time`
crate, for example by adding the following to your `Cargo.toml`.

```toml
time = { version = "0.3", features = ["wasm-bindgen"] }
```

[`tracing-subscriber`]: https://crates.io/crates/tracing-subscriber
[Performance API]: https://developer.mozilla.org/en-US/docs/Web/API/Performance

# License

This project is dual licensed under the [MIT license] and the [Apache license].

[MIT license]: LICENSE-MIT
[Apache license]: LICENSE-APACHE

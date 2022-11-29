# Contribution Guide

Contributions are welcome! Before starting a significant amount of work, you can also open an issue to discuss it first.

## Setting up your local development environment

To test changes, you can use the sample yew app found in `examples/trace-yew-app`. You need to add the Wasm target and install Trunk to compile this:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Linting

The following command formats the code using Rustfmt:

```bash
cargo +nightly fmt
```

## Writing APIs

When building new APIs, think about what it would be like to use them. Would this API cause confusing and hard to pin error mesages? Would this API integrate well with other APIs? Is it intuitive to use this API?

Below, you can find some useful guidance and best practices on how to write APIs. These are only _guidelines_ and while they are helpful and should be followed where possible, in some cases, it may not be possible to do so.

- [The Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Elegant Library APIs in Rust](https://deterministic.space/elegant-apis-in-rust.html)

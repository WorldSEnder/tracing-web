## Version 0.1.4

- Use `JsValue::from_utf8_lossy` when pushing the log to javascript to save an allocation.

## Version 0.1.3

- Add `MakeWebConsoleWriter`, a more configurable alternative to``MakeConsoleWriter`.
  `MakeWebConsoleWriter::new()` is a drop-in replacement for the old `MakeConsoleWriter` constructor.
- Add `MakeWebConsoleWriter::with_pretty_level()` to print a label denoting the level of the logged event.

## Version 0.1.2

- Change logging method of `Level::TRACE` to `console.debug`.

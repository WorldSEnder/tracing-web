use std::io::Write;

use tracing_core::Level;
use tracing_subscriber::fmt::MakeWriter;
use wasm_bindgen::JsValue;
use web_sys::console;

/// A [`MakeWriter`] emitting the written text to the [`console`].
///
/// The used log method is sensitive to the level the event is emitted with.
///
/// | Level     | Method           |
/// |-----------|------------------|
/// | TRACE     | console.debug    |
/// | DEBUG     | console.debug    |
/// | INFO      | console.info     |
/// | WARN      | console.warn     |
/// | ERROR     | console.error    |
/// | other     | console.log      |
pub struct MakeConsoleWriter;

/// Concrete [`std::io::Write`] implementation returned from [`MakeConsoleWriter`].
pub struct ConsoleWriter {
    buffer: Vec<u8>,
    log: fn(&str),
}

impl Write for ConsoleWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Nothing to-do here, we instead flush on drop
        Ok(())
    }
}

impl Drop for ConsoleWriter {
    fn drop(&mut self) {
        // TODO: it's rather pointless to decoded to utf-8 here,
        //  just to re-encode as utf-16 when crossing wasm-bindgen boundaries
        // we could use TextDecoder directly to produce a
        let message = String::from_utf8_lossy(&self.buffer);
        (self.log)(message.as_ref())
    }
}

fn console_debug(msg: &str) {
    console::debug_1(&JsValue::from(msg))
}
fn console_info(msg: &str) {
    console::info_1(&JsValue::from(msg))
}
fn console_warn(msg: &str) {
    console::warn_1(&JsValue::from(msg))
}
fn console_error(msg: &str) {
    console::error_1(&JsValue::from(msg))
}
fn console_log(msg: &str) {
    console::log_1(&JsValue::from(msg))
}

impl<'a> MakeWriter<'a> for MakeConsoleWriter {
    type Writer = ConsoleWriter;

    fn make_writer(&'a self) -> Self::Writer {
        ConsoleWriter {
            buffer: vec![],
            log: console_log,
        }
    }

    fn make_writer_for(&'a self, meta: &tracing_core::Metadata<'_>) -> Self::Writer {
        let level = meta.level();
        let log_fn = if *level == Level::TRACE || *level == Level::DEBUG {
            // Even though console.trace exists and generates stack traces, it logs with level: info, so leads to verbose logs
            console_debug
        } else if *level == Level::INFO {
            console_info
        } else if *level == Level::WARN {
            console_warn
        } else if *level == Level::ERROR {
            console_error
        } else {
            console_log
        };
        ConsoleWriter {
            buffer: vec![],
            log: log_fn,
        }
    }
}

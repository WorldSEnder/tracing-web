use std::io::Write;

use tracing_core::Level;
use tracing_subscriber::fmt::MakeWriter;
use wasm_bindgen::JsValue;
use web_sys::console;

/// **Discouraged** A [`MakeWriter`] emitting the written text to the [`console`].
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
///
/// ### Note
///
/// Since version `0.1.3`, you should prefer the alternative, more powerful [MakeWebConsoleWriter].
// For now, I have decided against deprecating this. While I do intend to deprecate or even remove it in 0.2, a warning is probably too picky on downstream developers.
// #[deprecated(
//     since = "0.1.3",
//     note = "use `MakeWebConsoleWriter` instead, which provides a more future proof API"
// )]
pub struct MakeConsoleWriter;

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
pub struct MakeWebConsoleWriter {
    use_pretty_label: bool,
}

impl Default for MakeWebConsoleWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl MakeWebConsoleWriter {
    /// Create a default console writer, i.e. no level annotation is shown when logging a message.
    pub fn new() -> Self {
        Self {
            use_pretty_label: false,
        }
    }
    /// Enables an additional label for the log level to be shown.
    ///
    /// It is recommended that you also use [`Layer::with_level(false)`] if you use this option, to avoid the event level being shown twice.
    ///
    /// [`Layer::with_level(false)`]: tracing_subscriber::fmt::Layer::with_level
    pub fn with_pretty_level(mut self) -> Self {
        self.use_pretty_label = true;
        self
    }
}

type LogDispatcher = fn(Level, &str);

/// Concrete [`std::io::Write`] implementation returned by [`MakeConsoleWriter`] and [`MakeWebConsoleWriter`].
pub struct ConsoleWriter {
    buffer: Vec<u8>,
    level: Level,
    log: LogDispatcher,
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
        (self.log)(self.level, message.as_ref())
    }
}

trait LogImpl {
    fn log_simple(level: Level, msg: &str);
    fn log_pretty(level: Level, msg: &str);
}

macro_rules! make_log_impl {
    ($T:ident {
        simple: $s:expr,
        pretty: {
            log: $p:expr, fmt: $f:expr, label_style: $l:expr $(,)?
        } $(,)?
    }) => {
        struct $T;
        impl LogImpl for $T {
            #[inline(always)]
            fn log_simple(_level: Level, msg: &str) {
                $s(&JsValue::from(msg));
            }
            #[inline(always)]
            fn log_pretty(_level: Level, msg: &str) {
                let fmt = JsValue::from(wasm_bindgen::intern($f));
                let label_style = JsValue::from(wasm_bindgen::intern($l));
                let msg_style =
                    JsValue::from(wasm_bindgen::intern("background: inherit; color: inherit;"));
                $p(&fmt, &label_style, &msg_style, &JsValue::from(msg));
            }
        }
    };
}

// Even though console.trace exists and generates stack traces, it logs with level: info, so leads to verbose logs, so log with debug
make_log_impl!(LogLevelTrace { simple: console::debug_1, pretty: { log: console::debug_4, fmt: "%cTRACE%c %s", label_style: "color: white; font-weight: bold; padding: 0 3px; background: #75507B;" } });
make_log_impl!(LogLevelDebug { simple: console::debug_1, pretty: { log: console::debug_4, fmt: "%cDEBUG%c %s", label_style: "color: white; font-weight: bold; padding: 0 3px; background: #3465A4;" } });
make_log_impl!(LogLevelInfo  { simple: console::info_1,  pretty: { log: console::info_4,  fmt: "%cINFO%c %s", label_style: "color: white; font-weight: bold; padding: 0 3px; background: #4E9A06;" } });
make_log_impl!(LogLevelWarn  { simple: console::warn_1,  pretty: { log: console::warn_4,  fmt: "%cWARN%c %s", label_style: "color: white; font-weight: bold; padding: 0 3px; background: #C4A000;" } });
make_log_impl!(LogLevelError { simple: console::error_1, pretty: { log: console::error_4, fmt: "%cERROR%c %s", label_style: "color: white; font-weight: bold; padding: 0 3px; background: #CC0000;" } });
struct LogLevelFallback;
impl LogImpl for LogLevelFallback {
    #[inline(always)]
    fn log_simple(_level: Level, msg: &str) {
        console::log_1(&JsValue::from(msg))
    }

    #[inline(always)]
    fn log_pretty(level: Level, msg: &str) {
        let fmt = JsValue::from(wasm_bindgen::intern("%c%s%c %s"));
        let label_level = JsValue::from(format!("{}", level));
        let label_style = JsValue::from(wasm_bindgen::intern(""));
        let msg_style = JsValue::from(wasm_bindgen::intern(""));
        let msg = JsValue::from(msg);
        console::log_5(&fmt, &label_style, &label_level, &msg_style, &msg)
    }
}

trait LogImplStyle {
    fn get_dispatch<L: LogImpl>(&self) -> LogDispatcher;
}
struct SimpleStyle;
impl LogImplStyle for SimpleStyle {
    #[inline(always)]
    fn get_dispatch<L: LogImpl>(&self) -> LogDispatcher {
        L::log_simple
    }
}
struct PrettyStyle;
impl LogImplStyle for PrettyStyle {
    #[inline(always)]
    fn get_dispatch<L: LogImpl>(&self) -> LogDispatcher {
        L::log_pretty
    }
}

fn select_dispatcher(style: impl LogImplStyle, level: Level) -> LogDispatcher {
    if level == Level::TRACE {
        style.get_dispatch::<LogLevelTrace>()
    } else if level == Level::DEBUG {
        style.get_dispatch::<LogLevelDebug>()
    } else if level == Level::INFO {
        style.get_dispatch::<LogLevelInfo>()
    } else if level == Level::WARN {
        style.get_dispatch::<LogLevelWarn>()
    } else if level == Level::ERROR {
        style.get_dispatch::<LogLevelError>()
    } else {
        style.get_dispatch::<LogLevelFallback>()
    }
}

impl MakeConsoleWriter {
    // "upgrade" to the non-deprecated version of MakeConsoleWriter, mainly to unify code paths.
    fn upgrade(&self) -> MakeWebConsoleWriter {
        MakeWebConsoleWriter {
            use_pretty_label: false,
        }
    }
}
impl<'a> MakeWriter<'a> for MakeConsoleWriter {
    type Writer = ConsoleWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.upgrade().make_writer()
    }

    fn make_writer_for(&'a self, meta: &tracing_core::Metadata<'_>) -> Self::Writer {
        self.upgrade().make_writer_for(meta)
    }
}

impl<'a> MakeWriter<'a> for MakeWebConsoleWriter {
    type Writer = ConsoleWriter;

    fn make_writer(&'a self) -> Self::Writer {
        ConsoleWriter {
            buffer: vec![],
            level: Level::TRACE, // if no level is known, assume the most detailed
            log: if self.use_pretty_label {
                PrettyStyle.get_dispatch::<LogLevelFallback>()
            } else {
                SimpleStyle.get_dispatch::<LogLevelFallback>()
            },
        }
    }

    fn make_writer_for(&'a self, meta: &tracing_core::Metadata<'_>) -> Self::Writer {
        let level = *meta.level();
        let log_fn = if self.use_pretty_label {
            select_dispatcher(PrettyStyle, level)
        } else {
            select_dispatcher(SimpleStyle, level)
        };
        ConsoleWriter {
            buffer: vec![],
            level,
            log: log_fn,
        }
    }
}

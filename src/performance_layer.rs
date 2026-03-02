use std::marker::PhantomData;

use tracing_core::{span, Subscriber};
use tracing_subscriber::{
    field::RecordFields,
    fmt::{FormatFields, FormattedFields},
    layer::Context,
    registry::{Extensions, ExtensionsMut, LookupSpan, SpanRef},
    Layer,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use web_sys::{Performance, PerformanceMarkOptions, PerformanceMeasureOptions};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = _fakeGlobal)]
    type Global;
    #[wasm_bindgen(static_method_of = Global, js_class = "globalThis", getter)]
    fn performance() -> Performance;
}

fn do_mark(p: &Performance, name: &str) -> Result<(), JsValue> {
    p.mark(name)?;
    Ok(())
}

fn do_mark_detailed(p: &Performance, name: &str, details: &str) -> Result<(), JsValue> {
    let opts = PerformanceMarkOptions::new();
    opts.set_detail(&JsValue::from_str(details));
    p.mark_with_mark_options(name, &opts)?;
    Ok(())
}

fn do_measure(p: &Performance, name: &str, start: &str, end: &str) -> Result<(), JsValue> {
    p.measure_with_str_and_end_mark(name, start, end)?;
    Ok(())
}

fn do_measure_detailed(
    p: &Performance,
    name: &str,
    start: &str,
    end: &str,
    details: &str,
) -> Result<(), JsValue> {
    let opts = PerformanceMeasureOptions::new();
    opts.set_detail(&JsValue::from_str(details));
    opts.set_start(start);
    opts.set_end(end);
    p.measure_with_performance_measure_options(name, &opts)?;
    Ok(())
}

thread_local! {
    static PERF: Performance = {
        let performance = Global::performance();
        assert!(!performance.is_undefined(), "browser seems to not support the Performance API");
        performance
    };
}

/// A [`Layer`] that emits span enter, exit and events as [`performance`] marks.
///
/// [`performance`]: https://developer.mozilla.org/en-US/docs/Web/API/Performance
pub struct PerformanceEventsLayer<S, N = ()> {
    fmt_details: N,
    _inner: PhantomData<fn(S)>,
}

impl<S, N> PerformanceEventsLayer<S, N> {
    /// Change the way additional details are attached to performance events.
    ///
    /// The given [`FormatFields`] is used to format a string that is attached to each event.
    /// See the [`mod@tracing_subscriber::fmt::format`] module for an assortment of available formatters.
    pub fn with_details_from_fields<N2>(
        self,
        fmt_fields: N2,
    ) -> PerformanceEventsLayer<S, FormatSpanFromFields<N2>>
    where
        N2: 'static + for<'writer> FormatFields<'writer>,
    {
        self.with_details(FormatSpanFromFields { inner: fmt_fields })
    }
    /// Change the way additional details are attached to performance events.
    ///
    /// See also [`with_details_from_fields`](Self::with_details_from_fields) for compatibility with [`mod@tracing_subscriber::fmt::format`].
    pub fn with_details<N2: FormatSpan>(self, fmt_details: N2) -> PerformanceEventsLayer<S, N2> {
        PerformanceEventsLayer {
            fmt_details,
            _inner: PhantomData,
        }
    }
}

impl<S, N> PerformanceEventsLayer<S, N>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: FormatSpan,
{
    fn template_name(span: &SpanRef<'_, S>, event_name: &str) -> String {
        let span_id = span.id().into_u64();
        let name = span.metadata().name();
        format!("{name} [{span_id}]: {event_name}")
    }
    fn span_enter_name(&self, span: &SpanRef<'_, S>) -> String {
        Self::template_name(span, "span-enter")
    }
    fn span_exit_name(&self, span: &SpanRef<'_, S>) -> String {
        Self::template_name(span, "span-exit")
    }
    fn span_record_name(&self, span: &SpanRef<'_, S>) -> String {
        Self::template_name(span, "span-record")
    }
    fn span_measure_name(&self, span: &SpanRef<'_, S>) -> String {
        Self::template_name(span, "span-measure")
    }
}

impl<S, N> Layer<S> for PerformanceEventsLayer<S, N>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: FormatSpan,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, span: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(span).expect("can't find span, this is a bug");

        self.fmt_details
            .add_details(&mut span.extensions_mut(), attrs);
    }
    fn on_record(&self, span: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>) {
        let span = ctx.span(span).expect("can't find span, this is a bug");
        self.fmt_details
            .record_values(&mut span.extensions_mut(), values);

        let mark_name = self.span_record_name(&span);
        let _ = PERF.with(|p| {
            if let Some(details) = self.fmt_details.find_details(&span.extensions()) {
                do_mark_detailed(p, &mark_name, details)
            } else {
                do_mark(p, &mark_name)
            }
        }); // Ignore errors
    }
    fn on_enter(&self, span: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(span).expect("can't find span, this is a bug");
        let mark_name = self.span_enter_name(&span);
        let _ = PERF.with(|p| {
            if let Some(details) = self.fmt_details.find_details(&span.extensions()) {
                do_mark_detailed(p, &mark_name, details)
            } else {
                do_mark(p, &mark_name)
            }
        }); // Ignore errors
    }
    fn on_exit(&self, span: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(span).expect("can't find span, this is a bug");
        let mark_enter_name = self.span_enter_name(&span);
        let mark_exit_name = self.span_exit_name(&span);
        let mark_measure_name = self.span_measure_name(&span);
        let _ = PERF.with(|p| {
            if let Some(details) = self.fmt_details.find_details(&span.extensions()) {
                do_mark_detailed(p, &mark_exit_name, details)?;
                do_measure_detailed(
                    p,
                    &mark_measure_name,
                    &mark_enter_name,
                    &mark_exit_name,
                    details,
                )?;
            } else {
                do_mark(p, &mark_exit_name)?;
                do_measure(p, &mark_measure_name, &mark_enter_name, &mark_exit_name)?;
            }
            Result::<(), JsValue>::Ok(())
        }); // Ignore errors
    }
    fn on_id_change(&self, _: &span::Id, _: &span::Id, _ctx: Context<'_, S>) {
        web_sys::console::warn_1(&JsValue::from(
            "A span changed id, this is currently not supported",
        ));
        debug_assert!(false, "A span changed id, this is currently not supported");
    }
}

/// Construct a new layer recording performance events.
///
/// The default will not attach any additional field information to the events.
pub fn performance_layer<S>() -> PerformanceEventsLayer<S, ()>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    PerformanceEventsLayer {
        fmt_details: (),
        _inner: PhantomData,
    }
}

/// Determine what additional information will be attached to the performance events.
pub trait FormatSpan: 'static {
    /// Find the details in the extensions of a span that will be recorded with the event.
    fn find_details<'ext>(&self, ext: &'ext Extensions<'_>) -> Option<&'ext str>;
    /// Called when a span is constructed, with its initial attributes.
    ///
    /// This method should insert, for later consumption in [`Self::find_details`], a description of the details.
    fn add_details(&self, ext: &mut ExtensionsMut<'_>, attrs: &span::Attributes<'_>);
    /// Called when a span records some values.
    ///
    /// This method should modify, for later consumption in [`Self::find_details`], the description of the details.
    fn record_values(&self, ext: &mut ExtensionsMut<'_>, values: &span::Record<'_>);
}

impl FormatSpan for () {
    fn find_details<'ext>(&self, _: &'ext Extensions<'_>) -> Option<&'ext str> {
        None
    }
    fn add_details(&self, _: &mut ExtensionsMut<'_>, _: &span::Attributes<'_>) {}
    fn record_values(&self, _: &mut ExtensionsMut<'_>, _: &span::Record<'_>) {}
}

/// An adaptor for Formatters from [`mod@tracing_subscriber::fmt::format`] as a [`FormatSpan`].
///
/// Uses [`FormattedFields`] to store the details attachement, so it might reuse an existing extension
/// for logging, to save some work visiting the recorded fields.
pub struct FormatSpanFromFields<N> {
    inner: N,
}
impl<N> FormatSpanFromFields<N>
where
    N: 'static + for<'writer> FormatFields<'writer>,
{
    fn add_formatted_fields(&self, ext: &mut ExtensionsMut<'_>, fields: impl RecordFields) {
        if ext.get_mut::<FormattedFields<N>>().is_none() {
            let mut fmt_fields = FormattedFields::<N>::new(String::new());
            if self
                .inner
                .format_fields(fmt_fields.as_writer(), fields)
                .is_ok()
            {
                ext.insert(fmt_fields);
            }
        }
    }
}

impl<N> FormatSpan for FormatSpanFromFields<N>
where
    N: 'static + for<'writer> FormatFields<'writer>,
{
    fn find_details<'ext>(&self, ext: &'ext Extensions<'_>) -> Option<&'ext str> {
        let fields = ext.get::<FormattedFields<N>>()?;
        Some(&fields.fields)
    }

    fn add_details(&self, ext: &mut ExtensionsMut<'_>, attrs: &span::Attributes<'_>) {
        self.add_formatted_fields(ext, attrs);
    }

    fn record_values(&self, ext: &mut ExtensionsMut<'_>, values: &span::Record<'_>) {
        if let Some(fields) = ext.get_mut::<FormattedFields<N>>() {
            let _ = self.inner.add_fields(fields, values);
        } else {
            self.add_formatted_fields(ext, values);
        }
    }
}

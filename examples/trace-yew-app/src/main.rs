use tracing::Span;
use tracing_subscriber::{
    fmt::{
        format::{FmtSpan, Pretty},
        time::UtcTime,
    },
    prelude::*,
};
use yew::{function_component, html, Html};

#[function_component]
fn App() -> Html {
    html! {
        <div>
        <p>{"This web app shows timings of components and tracing with tracing-web"}</p>
        </div>
    }
}

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(tracing_web::MakeConsoleWriter)
        .with_span_events(FmtSpan::ACTIVE);
    let perf_layer = tracing_web::performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    tracing::debug_span!("top-level", i = 5).in_scope(|| {
        let message = "debug message";
        tracing::debug!(msg = ?message, "Hello, world!");
        Span::current().record("i", 7);
    });
    yew::Renderer::<App>::new().render();
}

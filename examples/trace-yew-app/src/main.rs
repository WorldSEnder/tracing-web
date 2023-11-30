use tracing::Span;
use tracing_subscriber::{
    fmt::format::{FmtSpan, Pretty},
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
        .without_time()
        .with_writer(tracing_web::MakeWebConsoleWriter::new().with_pretty_level())
        .with_level(false)
        .with_span_events(FmtSpan::ACTIVE);
    let perf_layer = tracing_web::performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    tracing::debug_span!("top-level", i = 5).in_scope(|| {
        tracing::trace!("This is a trace message.");
        let message = "debug message";
        tracing::debug!(msg = ?message, "Hello, world!");
        tracing::warn!("This is a sample warning.");
        tracing::error!("This shows up as an error.");
        tracing::info!("This contains an informational message.");
        Span::current().record("i", 7);
    });
    yew::Renderer::<App>::new().render();
}

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("panic: {}", panic_info);
        tracing::error!(?panic_info, "application panic");
    }));
}

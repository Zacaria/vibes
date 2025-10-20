use anyhow::Result;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

pub struct TelemetryGuard;

impl TelemetryGuard {
    pub fn init(level: &str) -> Result<Self> {
        if tracing::dispatcher::has_been_set() {
            return Ok(Self);
        }
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));
        let fmt_layer = fmt::layer().with_target(false).compact();
        let subscriber = Registry::default().with(filter).with(fmt_layer);
        tracing::subscriber::set_global_default(subscriber)?;
        Ok(Self)
    }
}

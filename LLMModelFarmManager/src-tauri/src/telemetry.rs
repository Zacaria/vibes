use std::sync::Once;

use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};

static INIT: Once = Once::new();

pub fn init() {
    INIT.call_once(|| {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
        fmt()
            .with_env_filter(env_filter)
            .with_max_level(Level::TRACE)
            .with_target(false)
            .init();
    });
}

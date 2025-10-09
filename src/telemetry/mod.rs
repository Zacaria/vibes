use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).try_init().ok();
    std::panic::set_hook(Box::new(|panic| {
        tracing::error!("panic: {}", panic);
    }));
    Ok(())
}

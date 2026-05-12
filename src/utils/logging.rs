//! Logging setup.

use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

/// Initializes process logging from `RUST_LOG`.
pub(crate) fn init() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("openwhatsapp=info,warn"));

    let _ = fmt().with_env_filter(filter).try_init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_can_be_called_more_than_once() {
        init().unwrap();
        init().unwrap();
    }
}

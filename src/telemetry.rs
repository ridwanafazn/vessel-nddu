//! Inisialisasi tracing.
//!
//! Sebelumnya `tracing_subscriber` ada di Cargo.toml tetapi tidak pernah
//! diinisialisasi, sehingga semua `tracing::info!` hilang.

use tracing_subscriber::{fmt, EnvFilter};

pub fn init(level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let _ = fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .compact()
        .try_init();
}

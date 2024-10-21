use std::{env, path::Path};
use tracing::info;

use anyhow::Context;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, Registry};

use crate::ro_cell::RoCell;

static _GUARD: RoCell<WorkerGuard> = RoCell::new();

pub(super) struct Logs;

impl Logs {
    pub(super) fn start() -> anyhow::Result<()> {
        let state_dir = Path::new(&env::var("HOME").unwrap()).join(".local/state/lazycmd");

        std::fs::create_dir_all(&state_dir)
            .with_context(|| format!("failed to create state directory: {state_dir:?}"))?;

        let appender = tracing_appender::rolling::never(state_dir, "lazycmd.log");
        let (handle, guard) = tracing_appender::non_blocking(appender);

        // let filter = EnvFilter::from_default_env();
        let subscriber = Registry::default().with(
            fmt::layer()
                .pretty()
                .with_writer(handle)
                .with_ansi(cfg!(debug_assertions)),
        );

        tracing::subscriber::set_global_default(subscriber)
            .context("setting default subscriber failed")?;

        _GUARD.init(guard);

        info!("sdf");
        Ok(())
    }
}

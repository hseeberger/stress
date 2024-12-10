mod config;
mod error;
mod telemetry;

use crate::{
    config::{Config, ConfigExt, MainConfig},
    error::log_error,
};
use anyhow::{Context, Result};
use rand::{random, thread_rng, Rng};
use std::{panic, time::Duration};
use time::OffsetDateTime;
use tokio::time::sleep;
use tokio_postgres::NoTls;
use tracing::{error, info};

/// The entry point into the application.
pub async fn main() -> Result<()> {
    // Load configuration first, because needed for tracing initialization.
    let MainConfig {
        config,
        telemetry_config,
    } = MainConfig::load()
        .context("load configuration")
        .inspect_err(log_error)?;

    // Initialize tracing.
    telemetry::init(telemetry_config).inspect_err(log_error)?;

    // Replace the default panic hook with one that uses structured logging at ERROR level.
    panic::set_hook(Box::new(|panic| error!(%panic, "process panicked")));

    // Run and log any error.
    run(config).await.inspect_err(|error| {
        error!(
            error = format!("{error:#}"),
            backtrace = %error.backtrace(),
            "process exited with ERROR"
        )
    })
}

async fn run(config: Config) -> Result<()> {
    info!(?config, "starting");

    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=postgres",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
        .execute(
            "CREATE TABLE IF NOT EXISTS test (id VARCHAR(64) PRIMARY KEY, state BYTEA NOT NULL)",
            &[],
        )
        .await
        .context("create table")?;

    const LEN: usize = 8 * 1_024 * 1_024;
    let mut bytes = Vec::<u8>::with_capacity(LEN);
    for _ in 0..LEN {
        bytes.push(random());
    }

    let _ = client
        .execute(
            "INSERT INTO test (id, state) VALUES('state', $1)",
            &[&bytes.as_slice()],
        )
        .await;

    for n in 0..100_000 {
        for _ in 0..1_000 {
            let index = thread_rng().gen_range(0..LEN);
            bytes[index] = random();
        }

        client
            .execute(
                "UPDATE test SET state = $1 WHERE id = 'state'",
                &[&bytes.as_slice()],
            )
            .await
            .context("update state")?;

        if n % 100 == 0 {
            println!("{} saved {n} blocks", OffsetDateTime::now_utc());
        }
    }

    sleep(Duration::from_secs(1)).await;
    Ok(())
}

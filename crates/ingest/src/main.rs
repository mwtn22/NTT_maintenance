//! Ingest CLI.
//!
//! Usage:
//!   ingest migrate                       Apply database migrations
//!   ingest run-once [--source <code>]    Run one ingest cycle (all, or one source)
//!   ingest scheduler                     Poll all sources on their intervals

use std::time::Duration;
use storage::Repo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;
    let repo = Repo::connect(&database_url).await?;

    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("run-once");

    match cmd {
        "migrate" => {
            repo.migrate().await?;
            tracing::info!("migrations applied");
        }
        "run-once" => {
            repo.migrate().await?;
            let source_flag = arg_value(&args, "--source");
            let total = match source_flag {
                Some(code) => {
                    let source = connectors::source_by_org_code(&code)
                        .ok_or_else(|| anyhow::anyhow!("unknown source: {code}"))?;
                    ingest::run_source(&repo, source.as_ref()).await?
                }
                None => ingest::run_all(&repo).await?,
            };
            tracing::info!(events = total, "run-once complete");
        }
        "scheduler" => {
            repo.migrate().await?;
            run_scheduler(repo).await?;
        }
        other => {
            anyhow::bail!("unknown command: {other}");
        }
    }
    Ok(())
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

/// Per-source polling loop with exponential backoff on consecutive failures.
/// Each source runs independently; a broken source does not block others.
async fn run_scheduler(repo: Repo) -> anyhow::Result<()> {
    let mut handles = Vec::new();
    for source in connectors::all_sources() {
        let repo = repo.clone();
        let base_interval = source.meta().poll_interval_sec.max(60) as u64;
        handles.push(tokio::spawn(async move {
            loop {
                // Read current failure count to determine wait before next attempt.
                let failures = match repo.upsert_source(&source.meta()).await {
                    Ok(id) => repo
                        .get_consecutive_failures(id)
                        .await
                        .unwrap_or(0),
                    Err(_) => 0,
                };
                let wait = ingest::backoff_interval_secs(base_interval, failures);
                if failures > 0 {
                    tracing::warn!(
                        org_code = %source.meta().org_code,
                        consecutive_failures = failures,
                        wait_secs = wait,
                        "backing off due to repeated failures"
                    );
                }
                tokio::time::sleep(Duration::from_secs(wait)).await;
                if let Err(e) = ingest::run_source(&repo, source.as_ref()).await {
                    tracing::error!(
                        org_code = %source.meta().org_code,
                        error = %e,
                        "scheduled ingest failed"
                    );
                }
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
    Ok(())
}

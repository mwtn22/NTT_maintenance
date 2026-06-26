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

/// Minimal per-source polling loop. Each source is polled on its own interval.
async fn run_scheduler(repo: Repo) -> anyhow::Result<()> {
    let mut handles = Vec::new();
    for source in connectors::all_sources() {
        let repo = repo.clone();
        let interval = source.meta().poll_interval_sec.max(60) as u64;
        handles.push(tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(interval));
            loop {
                ticker.tick().await;
                if let Err(e) = ingest::run_source(&repo, source.as_ref()).await {
                    tracing::error!(error = %e, "scheduled ingest failed");
                }
            }
        }));
    }
    for h in handles {
        let _ = h.await;
    }
    Ok(())
}

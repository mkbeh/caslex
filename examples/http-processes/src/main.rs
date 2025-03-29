//! Run with
//!
//! ```not_rust
//! LOG_LEVEL=trace TRACE_LOG_LEVEL=trace cargo run -p example-http-processes
//! ```

#![allow(clippy::exit)]

use std::{env, sync::OnceLock};

use async_trait::async_trait;
use caslex_extra::{cleanup_resources, setup_application};
use caslex_http::server::{Config, Process, Server};
use tokio_util::sync::CancellationToken;

static SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() {
    // Setup application defaults.
    setup_application(SERVICE_NAME);

    let config = Config::parse();

    let ps = DummyProcess::new();
    let processes: Vec<&'static dyn Process> = vec![ps];

    // The router parameter can be omitted and the server will be started only with liveness and
    // readiness endpoints.
    let result = Server::new(config).processes(&processes).run().await;

    // Cleanup application resources.
    cleanup_resources();

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            println!("failed to start server: {e}");
            std::process::exit(1);
        }
    }
}

#[derive(Default)]
struct DummyProcess {
    process_num: usize,
}

impl DummyProcess {
    fn new() -> &'static Self {
        static INSTANCE: OnceLock<DummyProcess> = OnceLock::new();
        INSTANCE.get_or_init(DummyProcess::default)
    }
}

#[async_trait]
impl Process for DummyProcess {
    async fn pre_run(&self) -> anyhow::Result<()> {
        println!("pre run process {}", self.process_num);
        Ok(())
    }

    async fn run(&self, token: CancellationToken) -> anyhow::Result<()> {
        const DELAY_SECS: u64 = 5;
        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    println!("process {} stopped", self.process_num);
                    return Ok(());
                }

                _ = tokio::time::sleep(std::time::Duration::from_secs(DELAY_SECS)) => {
                    println!("process {} timed out", self.process_num);
                }
            }
        }
    }
}

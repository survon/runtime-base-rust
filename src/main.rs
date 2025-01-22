mod error;
mod event_bus;
mod module_manager;
mod orchestrator;
mod module;
mod runtime;

use runtime::{Runtime, RuntimeConfig};

#[tokio::main]
async fn main() -> error::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let config = RuntimeConfig {
        modules_dir: "modules".to_string(),
        max_concurrent_tasks: 10,
    };

    let runtime = Runtime::new(config).await?;
    runtime.load_modules().await?;
    runtime.run().await?;

    Ok(())
}

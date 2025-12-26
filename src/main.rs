use crate::app::App;

pub mod app;
pub mod ui;
pub mod module;
pub mod util;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let _ = &*util::log::LOGGER;
    log_info!("Survon runtime starting...");

    tracing_subscriber::fmt::init();
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().await?.run(terminal).await;
    ratatui::restore();
    result
}

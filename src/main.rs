use crate::app::App;

pub mod app;
pub mod event;
pub mod ui;
pub mod bus;
pub mod database;
pub mod modules;
pub mod llm;
pub mod knowledge;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().await?.run(terminal).await;
    ratatui::restore();
    result
}

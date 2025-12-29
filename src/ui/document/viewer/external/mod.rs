mod new;
mod can_launch_external;
mod show_document_external;
mod launch_browser_with_file;
mod create_document_html;
mod launch_browser;
mod command_exists;

#[derive(Debug)]
pub struct ExternalViewer {
    temp_dir: std::path::PathBuf,
}

mod can_launch_external;
mod command_exists;
mod create_document_html;
mod launch_browser;
mod launch_browser_with_file;
mod new;
mod show_document_external;

#[derive(Debug)]
pub struct ExternalViewer {
    temp_dir: std::path::PathBuf,
}

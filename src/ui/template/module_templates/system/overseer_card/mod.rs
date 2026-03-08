mod get_view_data;
mod render_all_devices;
mod render_archived_modules;
mod render_config_editor;
mod render_install_registry;
mod render_main_menu;
mod render_manage_modules;
mod render_overview_cta;
mod render_pending_trust;
mod trait_default;
mod trait_ui_template;

use color_eyre::owo_colors::OwoColorize;
use ratatui::{prelude::*, widgets::Widget};

use crate::ui::template::UiTemplate;

#[derive(Debug)]
pub struct OverseerCard;

struct ViewData<'a> {
    current_view: &'a str,
    selected_index: usize,
    status_message: Option<&'a str>,
    border_color: Color,
    module_list: Vec<String>,
    installed_modules: Vec<String>,
    known_devices: Vec<String>,
    pending_devices: Vec<String>,
    pending_count: usize,
    known_count: usize,
    registry_count: usize,
    installed_count: usize,
    archived_count: usize,
    is_scanning: bool,
    scan_countdown: u8,
    has_status: bool,
}

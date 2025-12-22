mod _make_is_scanning_component;
mod _make_status_component;
mod _make_help_component;
mod _make_empty_message_component;
mod get_view_data;
mod render_overview_cta;
mod render_main_menu;
mod render_pending_trust;
mod render_all_devices;
mod render_install_registry;
mod render_manage_modules;
mod render_archived_modules;
mod render_config_editor;
mod trait_ui_template;
mod trait_default;

use color_eyre::owo_colors::OwoColorize;
use ratatui::{
    prelude::*,
    widgets::Widget,
};

use crate::ui::template::UiTemplate;

#[derive(Debug)]
pub struct WastelandManagerCard;

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

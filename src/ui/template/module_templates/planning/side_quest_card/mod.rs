mod get_view_data;
mod render_create_quest;
mod render_overview_cta;
mod render_quest_detail;
mod render_quest_list;
mod trait_ui_template;
mod trait_default;

use ratatui::{
    prelude::*,
    widgets::{Widget},
};

use crate::ui::template::UiTemplate;

#[derive(Debug)]
pub struct SideQuestCard;

struct ViewData<'a> {
    current_view: &'a str,
    border_color: Color,
    selected_index: usize,
    quests: Vec<String>,
    quest_count: usize,
    status_message: Option<&'a str>,
    has_status: bool,
    create_step: &'a str,
    form_title: &'a str,
    form_description: &'a str,
    form_topic: &'a str,
    form_urgency: &'a str,
    available_topics: Vec<String>,
    urgency_options: Vec<String>,
    selected_quest_title: &'a str,
    selected_quest_description: &'a str,
    selected_quest_topic: &'a str,
    selected_quest_urgency: &'a str,
    selected_quest_trigger: &'a str,
}

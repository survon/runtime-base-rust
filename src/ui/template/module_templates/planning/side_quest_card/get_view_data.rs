use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Color,
};

use crate::module::Module;
use super::{SideQuestCard, ViewData};

impl SideQuestCard {
    pub(super) fn get_view_data<'a>(
        &self,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        module: &'a mut Module
    ) -> ViewData<'a> {
        let current_view = module
            .config
            .bindings
            .get("current_view")
            .and_then(|v| v.as_str())
            .unwrap_or("QuestList");

        let border_color = if is_selected { Color::White } else { Color::Magenta };

        let selected_index = module
            .config
            .bindings
            .get("selected_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let quests = module
            .config
            .bindings
            .get("quests")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let quest_count = quests.len();

        let status_message = module
            .config
            .bindings
            .get("status_message")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let has_status = status_message.is_some();

        let create_step = module
            .config
            .bindings
            .get("create_step")
            .and_then(|v| v.as_str())
            .unwrap_or("Title");

        let form_title = module
            .config
            .bindings
            .get("form_title")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let form_description = module
            .config
            .bindings
            .get("form_description")
            .and_then(|v| v.as_str())
            .unwrap_or("(none)");

        let form_topic = module
            .config
            .bindings
            .get("form_topic")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let form_urgency = module
            .config
            .bindings
            .get("form_urgency")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let available_topics = module
            .config
            .bindings
            .get("available_topics")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let urgency_options = module
            .config
            .bindings
            .get("urgency_options")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let selected_quest_title = module
            .config
            .bindings
            .get("selected_quest_title")
            .and_then(|v| v.as_str())
            .unwrap_or("Quest Details");

        let selected_quest_description = module
            .config
            .bindings
            .get("selected_quest_description")
            .and_then(|v| v.as_str())
            .unwrap_or("No description");

        let selected_quest_topic = module
            .config
            .bindings
            .get("selected_quest_topic")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let selected_quest_urgency = module
            .config
            .bindings
            .get("selected_quest_urgency")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let selected_quest_trigger = module
            .config
            .bindings
            .get("selected_quest_trigger")
            .and_then(|v| v.as_str())
            .unwrap_or("No deadline");


        ViewData {
            current_view,
            border_color,
            selected_index,
            quests,
            quest_count,
            status_message,
            has_status,
            create_step,
            form_title,
            form_description,
            form_topic,
            form_urgency,
            available_topics,
            urgency_options,
            selected_quest_title,
            selected_quest_description,
            selected_quest_topic,
            selected_quest_urgency,
            selected_quest_trigger,
        }
    }
}

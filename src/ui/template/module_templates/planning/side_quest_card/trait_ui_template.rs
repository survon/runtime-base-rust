use ratatui::{
    buffer::Buffer,
    layout::Rect,
};

use crate::modules::Module;
use crate::ui::template::UiTemplate;
use super::{SideQuestCard, ViewData};

impl UiTemplate for SideQuestCard {
    fn render_overview_cta(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        self.render_overview_cta(is_selected, area, buf, module);
    }

    fn render_detail(&self, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let is_selected = false;
        let ViewData { current_view, .. } = self.get_view_data(false, area, buf, module);

        match current_view {
            "QuestList" => self.render_quest_list(is_selected, area, buf, module),
            "CreateQuest" => self.render_create_quest(area, buf, module),
            "QuestDetail" => self.render_quest_detail(area, buf, module),
            _ => self.render_quest_list(is_selected, area, buf, module),
        }
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["current_view", "quests", "selected_index"]
    }

    fn docs(&self) -> &'static str {
        "Side Quest manager - Track activities and experiences you want to do 'someday'. \
         Create quests with urgency levels and optional deadlines."
    }
}

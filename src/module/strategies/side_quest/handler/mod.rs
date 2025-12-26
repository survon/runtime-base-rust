// src/modules/side_quest/handler.rs

mod new;
mod load_quests;
mod handle_quest_list_key;
mod handle_create_quest_key;
mod handle_detail_key;
mod reset_form;
mod save_quest;
mod publish_calendar_event;

use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use std::any::Any;
use chrono::{DateTime, Utc};

use crate::module::{
    trait_module_handler::ModuleHandler,
    Module,
};
use crate::util::{
    database::Database,
    io::{bus::MessageBus, event::AppEvent},
};
use crate::module::strategies::side_quest::database::SideQuestDatabase;
use super::{QuestUrgency, SideQuest};

#[derive(Debug, Clone, PartialEq)]
enum SideQuestView {
    QuestList,      // Main list of quests
    CreateQuest,    // Form to create new quest
    QuestDetail,    // View individual quest details
}

#[derive(Debug, Clone, PartialEq)]
enum CreateStep {
    Title,
    Description,
    Topic,
    Urgency,
    TriggerDate,
    Confirm,
}

#[derive(Debug)]
pub struct SideQuestHandler {
    current_view: SideQuestView,
    selected_index: usize,
    quests: Vec<SideQuest>,
    database: Database,
    message_bus: MessageBus,

    // Create form state
    create_step: CreateStep,
    form_title: String,
    form_description: String,
    form_topic: String,
    form_urgency: QuestUrgency,
    form_trigger_date: Option<DateTime<Utc>>,

    // Available topics (could be from DB later)
    available_topics: Vec<String>,

    status_message: Option<String>,
}


impl ModuleHandler for SideQuestHandler {
    fn handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        match self.current_view {
            SideQuestView::QuestList => self.handle_quest_list_key(key_code),
            SideQuestView::CreateQuest => self.handle_create_quest_key(key_code),
            SideQuestView::QuestDetail => self.handle_detail_key(key_code),
        }
    }

    fn handle_event(&mut self, _event: &AppEvent, _module: &mut Module) -> Result<bool> {
        Ok(false)
    }

    fn update_bindings(&mut self, module: &mut Module) {
        // Update view state
        module.config.bindings.insert(
            "current_view".to_string(),
            serde_json::json!(format!("{:?}", self.current_view)),
        );

        module.config.bindings.insert(
            "selected_index".to_string(),
            serde_json::json!(self.selected_index),
        );

        // Quest list data
        let quest_summaries: Vec<String> = self.quests
            .iter()
            .map(|q| q.display_summary())
            .collect();

        module.config.bindings.insert(
            "quests".to_string(),
            serde_json::json!(quest_summaries),
        );

        module.config.bindings.insert(
            "quest_count".to_string(),
            serde_json::json!(self.quests.len()),
        );

        // Create form state
        module.config.bindings.insert(
            "create_step".to_string(),
            serde_json::json!(format!("{:?}", self.create_step)),
        );

        module.config.bindings.insert(
            "form_title".to_string(),
            serde_json::json!(self.form_title),
        );

        module.config.bindings.insert(
            "form_description".to_string(),
            serde_json::json!(self.form_description),
        );

        module.config.bindings.insert(
            "form_topic".to_string(),
            serde_json::json!(self.form_topic),
        );

        module.config.bindings.insert(
            "form_urgency".to_string(),
            serde_json::json!(self.form_urgency.as_str()),
        );

        module.config.bindings.insert(
            "available_topics".to_string(),
            serde_json::json!(self.available_topics),
        );

        let urgency_options: Vec<String> = QuestUrgency::all()
            .iter()
            .map(|u| u.as_str().to_string())
            .collect();

        module.config.bindings.insert(
            "urgency_options".to_string(),
            serde_json::json!(urgency_options),
        );

        // Selected quest detail
        if let Some(quest) = self.quests.get(self.selected_index) {
            module.config.bindings.insert(
                "selected_quest_title".to_string(),
                serde_json::json!(quest.title),
            );

            module.config.bindings.insert(
                "selected_quest_description".to_string(),
                serde_json::json!(quest.description.clone().unwrap_or_default()),
            );

            module.config.bindings.insert(
                "selected_quest_topic".to_string(),
                serde_json::json!(quest.topic),
            );

            module.config.bindings.insert(
                "selected_quest_urgency".to_string(),
                serde_json::json!(quest.urgency.as_str()),
            );

            let trigger_str = quest.trigger_date
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "No deadline".to_string());

            module.config.bindings.insert(
                "selected_quest_trigger".to_string(),
                serde_json::json!(trigger_str),
            );
        }

        // Status message
        if let Some(status) = &self.status_message {
            module.config.bindings.insert(
                "status_message".to_string(),
                serde_json::json!(status),
            );
        } else {
            module.config.bindings.insert(
                "status_message".to_string(),
                serde_json::json!(""),
            );
        }
    }

    fn module_type(&self) -> &str {
        "side_quest"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

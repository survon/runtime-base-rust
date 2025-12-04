// src/modules/side_quest/handler.rs

use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use std::any::Any;
use chrono::{DateTime, Utc, Duration};

use crate::modules::{
    Module,
    module_handler::ModuleHandler,
    side_quest::database::SideQuestDatabase,
};
use crate::util::{
    io::{event::AppEvent, bus::MessageBus},
    database::Database,
};
use crate::{log_debug, log_error, log_info};
use super::{SideQuest, QuestUrgency};

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

impl SideQuestHandler {
    pub fn new(database: Database, message_bus: MessageBus) -> Self {
        let mut handler = Self {
            current_view: SideQuestView::QuestList,
            selected_index: 0,
            quests: Vec::new(),
            database: database.clone(),
            message_bus,
            create_step: CreateStep::Title,
            form_title: String::new(),
            form_description: String::new(),
            form_topic: String::new(),
            form_urgency: QuestUrgency::Casual,
            form_trigger_date: None,
            available_topics: vec![
                "outdoor".to_string(),
                "food".to_string(),
                "learning".to_string(),
                "fitness".to_string(),
                "social".to_string(),
                "creative".to_string(),
                "adventure".to_string(),
                "hobby".to_string(),
            ],
            status_message: None,
        };

        // Load quests from database
        handler.load_quests();

        handler
    }

    fn load_quests(&mut self) {
        match self.database.get_active_side_quests() {
            Ok(quests) => {
                self.quests = quests;
                log_info!("Loaded {} side quests", self.quests.len());
            }
            Err(e) => {
                log_error!("Failed to load side quests: {}", e);
                self.status_message = Some(format!("Error loading quests: {}", e));
            }
        }
    }

    fn handle_quest_list_key(&mut self, key_code: KeyCode) -> Option<AppEvent> {
        match key_code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                None
            }
            KeyCode::Down => {
                if self.selected_index < self.quests.len().saturating_sub(1) {
                    self.selected_index += 1;
                }
                None
            }
            KeyCode::Char('n') => {
                // Start creating new quest
                self.current_view = SideQuestView::CreateQuest;
                self.create_step = CreateStep::Title;
                self.reset_form();
                self.status_message = Some("Enter quest title...".to_string());
                None
            }
            KeyCode::Char('c') => {
                // Complete selected quest
                if let Some(quest) = self.quests.get_mut(self.selected_index) {
                    if let Err(e) = self.database.complete_side_quest(quest.id) {
                        self.status_message = Some(format!("Error: {}", e));
                    } else {
                        quest.complete();
                        self.status_message = Some(format!("✓ Completed: {}", quest.title));
                        self.load_quests();
                    }
                }
                None
            }
            KeyCode::Char('d') => {
                // Delete selected quest
                if let Some(quest) = self.quests.get(self.selected_index) {
                    if let Err(e) = self.database.delete_side_quest(quest.id) {
                        self.status_message = Some(format!("Error: {}", e));
                    } else {
                        self.status_message = Some(format!("Deleted: {}", quest.title));
                        self.load_quests();
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                    }
                }
                None
            }
            KeyCode::Enter => {
                // View quest details
                if self.quests.get(self.selected_index).is_some() {
                    self.current_view = SideQuestView::QuestDetail;
                }
                None
            }
            KeyCode::Esc => Some(AppEvent::Back),
            _ => None,
        }
    }

    fn handle_create_quest_key(&mut self, key_code: KeyCode) -> Option<AppEvent> {
        match self.create_step {
            CreateStep::Title => {
                match key_code {
                    KeyCode::Char(c) => {
                        self.form_title.push(c);
                        None
                    }
                    KeyCode::Backspace => {
                        self.form_title.pop();
                        None
                    }
                    KeyCode::Enter if !self.form_title.is_empty() => {
                        self.create_step = CreateStep::Description;
                        self.status_message = Some("Enter description (optional, Enter to skip)...".to_string());
                        None
                    }
                    KeyCode::Esc => {
                        self.current_view = SideQuestView::QuestList;
                        None
                    }
                    _ => None,
                }
            }
            CreateStep::Description => {
                match key_code {
                    KeyCode::Char(c) => {
                        self.form_description.push(c);
                        None
                    }
                    KeyCode::Backspace => {
                        self.form_description.pop();
                        None
                    }
                    KeyCode::Enter => {
                        self.create_step = CreateStep::Topic;
                        self.status_message = Some("Select topic...".to_string());
                        None
                    }
                    KeyCode::Esc => {
                        self.current_view = SideQuestView::QuestList;
                        None
                    }
                    _ => None,
                }
            }
            CreateStep::Topic => {
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        None
                    }
                    KeyCode::Down => {
                        if self.selected_index < self.available_topics.len().saturating_sub(1) {
                            self.selected_index += 1;
                        }
                        None
                    }
                    KeyCode::Enter => {
                        if let Some(topic) = self.available_topics.get(self.selected_index) {
                            self.form_topic = topic.clone();
                            self.create_step = CreateStep::Urgency;
                            self.selected_index = 1; // Default to Casual
                            self.status_message = Some("Select urgency...".to_string());
                        }
                        None
                    }
                    KeyCode::Esc => {
                        self.current_view = SideQuestView::QuestList;
                        None
                    }
                    _ => None,
                }
            }
            CreateStep::Urgency => {
                let urgencies = QuestUrgency::all();
                match key_code {
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        None
                    }
                    KeyCode::Down => {
                        if self.selected_index < urgencies.len().saturating_sub(1) {
                            self.selected_index += 1;
                        }
                        None
                    }
                    KeyCode::Enter => {
                        if let Some(urgency) = urgencies.get(self.selected_index) {
                            self.form_urgency = urgency.clone();
                            self.create_step = CreateStep::TriggerDate;
                            self.status_message = Some("Set trigger date (Enter to skip)...".to_string());
                        }
                        None
                    }
                    KeyCode::Esc => {
                        self.current_view = SideQuestView::QuestList;
                        None
                    }
                    _ => None,
                }
            }
            CreateStep::TriggerDate => {
                match key_code {
                    KeyCode::Char('1') => {
                        // 1 week from now
                        self.form_trigger_date = Some(Utc::now() + Duration::weeks(1));
                        self.create_step = CreateStep::Confirm;
                        None
                    }
                    KeyCode::Char('2') => {
                        // 1 month from now
                        self.form_trigger_date = Some(Utc::now() + Duration::weeks(4));
                        self.create_step = CreateStep::Confirm;
                        None
                    }
                    KeyCode::Char('3') => {
                        // 3 months from now
                        self.form_trigger_date = Some(Utc::now() + Duration::weeks(12));
                        self.create_step = CreateStep::Confirm;
                        None
                    }
                    KeyCode::Enter => {
                        // Skip trigger date
                        self.form_trigger_date = None;
                        self.create_step = CreateStep::Confirm;
                        None
                    }
                    KeyCode::Esc => {
                        self.current_view = SideQuestView::QuestList;
                        None
                    }
                    _ => None,
                }
            }
            CreateStep::Confirm => {
                match key_code {
                    KeyCode::Char('y') | KeyCode::Enter => {
                        self.save_quest();
                        None
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        self.current_view = SideQuestView::QuestList;
                        None
                    }
                    _ => None,
                }
            }
        }
    }

    fn handle_detail_key(&mut self, key_code: KeyCode) -> Option<AppEvent> {
        match key_code {
            KeyCode::Esc => {
                self.current_view = SideQuestView::QuestList;
                None
            }
            KeyCode::Char('c') => {
                // Complete quest from detail view
                if let Some(quest) = self.quests.get_mut(self.selected_index) {
                    if let Err(e) = self.database.complete_side_quest(quest.id) {
                        self.status_message = Some(format!("Error: {}", e));
                    } else {
                        quest.complete();
                        self.status_message = Some(format!("✓ Completed: {}", quest.title));
                        self.current_view = SideQuestView::QuestList;
                        self.load_quests();
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn reset_form(&mut self) {
        self.form_title = String::new();
        self.form_description = String::new();
        self.form_topic = String::new();
        self.form_urgency = QuestUrgency::Casual;
        self.form_trigger_date = None;
        self.selected_index = 0;
    }

    fn save_quest(&mut self) {
        let description = if self.form_description.is_empty() {
            None
        } else {
            Some(self.form_description.clone())
        };

        match self.database.create_side_quest(
            &self.form_title,
            description,
            &self.form_topic,
            &self.form_urgency,
            self.form_trigger_date,
        ) {
            Ok(quest_id) => {
                self.status_message = Some(format!("✓ Created: {}", self.form_title));

                // Publish calendar event if trigger date exists
                if let Some(date) = self.form_trigger_date {
                    self.publish_calendar_event(quest_id, date);
                }

                self.load_quests();
                self.current_view = SideQuestView::QuestList;
                self.reset_form();
            }
            Err(e) => {
                log_error!("Failed to create quest: {}", e);
                self.status_message = Some(format!("Error: {}", e));
            }
        }
    }

    fn publish_calendar_event(&self, quest_id: i64, trigger_date: DateTime<Utc>) {
        let payload = serde_json::json!({
            "event_type": "side_quest_deadline",
            "quest_id": quest_id,
            "title": self.form_title,
            "date": trigger_date.to_rfc3339(),
            "source": "side_quest"
        });

        let message = crate::util::io::bus::BusMessage::new(
            "calendar.event.create".to_string(),
            payload.to_string(),
            "side_quest".to_string(),
        );

        let bus = self.message_bus.clone();
        tokio::spawn(async move {
            let _ = bus.publish(message).await;
        });

        log_info!("Published calendar event for quest: {}", self.form_title);
    }
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

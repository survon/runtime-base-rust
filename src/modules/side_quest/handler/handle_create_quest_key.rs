use chrono::{Duration, Utc};
use crossterm::event::KeyCode;

use crate::{
    modules::side_quest::{
        QuestUrgency,
        handler::{CreateStep, SideQuestHandler, SideQuestView}
    },
    util::io::event::AppEvent,
};

impl SideQuestHandler {
    pub(super) fn handle_create_quest_key(&mut self, key_code: KeyCode) -> Option<AppEvent> {
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
                        Some(AppEvent::NoOp)
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
                        Some(AppEvent::NoOp)
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
                        Some(AppEvent::NoOp)
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
                        Some(AppEvent::NoOp)
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
                        Some(AppEvent::NoOp)
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
                        Some(AppEvent::NoOp)
                    }
                    _ => None,
                }
            }
        }
    }
}

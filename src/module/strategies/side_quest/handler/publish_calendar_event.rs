use chrono::{DateTime, Utc};

use crate::log_info;
use crate::module::strategies::side_quest::handler::SideQuestHandler;

impl SideQuestHandler {
    pub(in crate::module) fn publish_calendar_event(&self, quest_id: i64, trigger_date: DateTime<Utc>) {
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

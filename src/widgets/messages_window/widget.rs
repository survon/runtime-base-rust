// src/widgets/messages_window/widget.rs
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use tokio::sync::mpsc;
use color_eyre::Result;

use super::state::{MessagesState, MessagesIntent, MessagesEvent};
use crate::util::io::{
    bus::{BusMessage, MessageBus},
    get_all_event_message_topics,
};
use crate::ui::style::dim_unless_focused;
use crate::log_debug;

#[derive(Debug)]
pub struct MessagesWidget {
    // State (read-only copy)
    current_state: MessagesState,

    // Intent sender (write-only)
    intent_tx: mpsc::UnboundedSender<MessagesIntent>,

    // Event receiver (for state updates)
    event_rx: mpsc::UnboundedReceiver<BusMessage>,

    // UI-specific state (not messages state)
    visible_lines: usize,

    // Message receivers (for incoming bus messages to display)
    message_receivers: Vec<mpsc::UnboundedReceiver<BusMessage>>,
}

impl MessagesWidget {
    pub async fn new(
        message_bus: &MessageBus,
        intent_tx: mpsc::UnboundedSender<MessagesIntent>,
    ) -> Result<Self> {
        // Subscribe to state changes
        let event_rx = message_bus.subscribe("messages.state".to_string()).await;

        // Subscribe to all relevant topics
        let mut topics = get_all_event_message_topics();
        topics.push("scheduler_event".to_string());

        let mut message_receivers = Vec::new();
        for topic in topics {
            let receiver = message_bus.subscribe(topic).await;
            message_receivers.push(receiver);
        }

        Ok(Self {
            current_state: MessagesState::default(),
            intent_tx,
            event_rx,
            visible_lines: 10,
            message_receivers,
        })
    }

    pub fn poll_state(&mut self) {
        // Poll for state updates
        while let Ok(msg) = self.event_rx.try_recv() {
            if let Ok(event) = serde_json::from_str::<MessagesEvent>(&msg.payload) {
                if let MessagesEvent::StateChanged(state) = event {
                    self.current_state = state;
                }
            }
        }

        // Poll all message receivers for new messages to add
        for receiver in &mut self.message_receivers {
            while let Ok(msg) = receiver.try_recv() {
                log_debug!("📨 MessagesWidget received: topic={}, source={}, payload={}",
                    msg.topic, msg.source, msg.payload);
                let _ = self.intent_tx.send(MessagesIntent::AddMessage(msg));
            }
        }
    }

    pub fn scroll_up(&self) {
        let _ = self.intent_tx.send(MessagesIntent::ScrollUp);
    }

    pub fn scroll_down(&self) {
        let _ = self.intent_tx.send(MessagesIntent::ScrollDown);
    }

    pub fn scroll_to_bottom(&self) {
        let _ = self.intent_tx.send(MessagesIntent::ScrollToBottom);
    }

    fn is_at_bottom(&self) -> bool {
        let max_scroll = self.current_state.messages.len().saturating_sub(self.visible_lines);
        self.current_state.scroll_offset >= max_scroll
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, is_focused: Option<bool>) {
        // Poll for updates
        self.poll_state();

        // Update visible lines based on area
        self.visible_lines = (area.height.saturating_sub(2)) as usize;
        let _ = self.intent_tx.send(MessagesIntent::SetVisibleLines(self.visible_lines));

        // Clamp scroll offset to valid range
        let max_scroll = self.current_state.messages.len().saturating_sub(self.visible_lines);
        let clamped_offset = self.current_state.scroll_offset.min(max_scroll);

        let content = if self.current_state.messages.is_empty() {
            Text::from("Message bus activity will appear here...\n\nWaiting for messages...\nPress Tab to focus this panel, then ↑/↓ to scroll")
        } else {
            let total = self.current_state.messages.len();
            let end = total.saturating_sub(clamped_offset);
            let start = end.saturating_sub(self.visible_lines);

            let lines: Vec<String> = self.current_state.messages[start..end]
                .iter()
                .map(|msg| {
                    // Special formatting for scheduler events
                    if msg.topic == "scheduler_event" {
                        format_scheduler_event(msg)
                    } else {
                        format!(
                            "[{}] {}: {}",
                            msg.source,
                            msg.topic,
                            msg.payload
                        )
                    }
                })
                .collect();

            Text::from(lines.join("\n"))
        };

        let title = if self.current_state.messages.is_empty() {
            "Message Bus".to_string()
        } else {
            let total = self.current_state.messages.len();
            let end = total.saturating_sub(clamped_offset);
            let start = end.saturating_sub(self.visible_lines);

            if clamped_offset == 0 {
                format!("Message Bus ({}/{}) [LIVE]", total.min(self.visible_lines), total)
            } else {
                format!(
                    "Message Bus ({}-{}/{}) → {}",
                    start + 1,
                    end,
                    total,
                    clamped_offset
                )
            }
        };

        let text_style = dim_unless_focused(is_focused, Style::default().fg(Color::White));
        let border_style = dim_unless_focused(is_focused, Style::default().fg(Color::Yellow));

        let messages_widget = Paragraph::new(content)
            .block(
                Block::bordered()
                    .title(title)
                    .border_type(BorderType::Rounded)
                    .style(border_style)
            )
            .style(text_style)
            .wrap(Wrap { trim: true });

        messages_widget.render(area, buf);
    }
}

/// Format scheduler events with nice icons and structure
fn format_scheduler_event(msg: &BusMessage) -> String {
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg.payload) {
        let event = data.get("event").and_then(|v| v.as_str()).unwrap_or("unknown");
        let device_id = data.get("device_id").and_then(|v| v.as_str()).unwrap_or("?");

        match event {
            "command_queued" => {
                let priority = data.get("priority").and_then(|v| v.as_str()).unwrap_or("?");
                let action = data.get("action").and_then(|v| v.as_str()).unwrap_or("?");
                let queue_size = data.get("queue_size").and_then(|v| v.as_u64()).unwrap_or(0);
                format!("📥 [{}] Queued {} {}: {} in queue", device_id, priority, action, queue_size)
            }
            "cmd_window_open" => {
                let duration = data.get("duration").and_then(|v| v.as_u64()).unwrap_or(0);
                format!("🟢 [{}] CMD WINDOW OPEN ({}s)", device_id, duration)
            }
            "cmd_window_imminent" => {
                let seconds = data.get("seconds").and_then(|v| v.as_u64()).unwrap_or(0);
                format!("🟡 [{}] CMD window in {}s", device_id, seconds)
            }
            "cmd_window_scheduled" => {
                let seconds = data.get("seconds").and_then(|v| v.as_u64()).unwrap_or(0);
                format!("⏰ [{}] CMD window in {}s", device_id, seconds)
            }
            "batch_start" => {
                let count = data.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                format!("📤 [{}] Sending {} command(s)...", device_id, count)
            }
            "command_sent" => {
                let action = data.get("action").and_then(|v| v.as_str()).unwrap_or("?");
                format!("✅ [{}] Sent: {}", device_id, action)
            }
            "command_sent_critical" => {
                let action = data.get("action").and_then(|v| v.as_str()).unwrap_or("?");
                format!("⚡ [{}] CRITICAL sent: {}", device_id, action)
            }
            "commands_expired" => {
                let count = data.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                format!("⏳ [{}] {} expired", device_id, count)
            }
            "batch_complete" => {
                let count = data.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
                format!("✅ [{}] Batch done: {} sent", device_id, count)
            }
            "error" => {
                let error = data.get("error").and_then(|v| v.as_str()).unwrap_or("?");
                format!("❌ [{}] {}", device_id, error)
            }
            _ => {
                format!("📡 [{}] {}", device_id, event)
            }
        }
    } else {
        format!("[scheduler] {}", msg.payload)
    }
}

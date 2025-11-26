use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use crate::log_debug;
use crate::util::io::{
    get_all_event_message_topics,
    bus::{BusMessage, BusReceiver, MessageBus}
};
use crate::ui::style::{dim_unless_focused};

#[derive(Debug)]
pub struct MessagesPanel {
    // Main receiver that gets ALL messages
    main_receiver: Option<BusReceiver>,
    // Topic-specific receivers (kept for backward compatibility)
    receivers: Vec<BusReceiver>,
    recent_messages: Vec<BusMessage>,
    max_messages: usize,
    scroll_offset: usize,
    visible_lines: usize,
}

impl MessagesPanel {
    pub fn new() -> Self {
        Self {
            main_receiver: None,
            receivers: Vec::new(),
            recent_messages: Vec::new(),
            max_messages: 100,
            scroll_offset: 0,
            visible_lines: 10,
        }
    }

    /// Subscribe to the main receiver to get ALL messages
    pub fn subscribe_main(&mut self, receiver: BusReceiver) {
        log_debug!("MessagesPanel: Subscribed to main receiver (ALL messages)");
        self.main_receiver = Some(receiver);
    }

    /// Subscribe to all topics (or specific topics)
    pub async fn subscribe(&mut self, message_bus: &MessageBus) {
        let receiver = message_bus.subscribe("navigation".to_string()).await;
        self.receivers.push(receiver);
    }

    /// Subscribe to multiple topics
    pub async fn subscribe_topics(&mut self, message_bus: &MessageBus, topics: Vec<String>) {
        for topic in topics {
            let receiver = message_bus.subscribe(topic).await;
            self.receivers.push(receiver);
        }
    }

    /// Subscribe to ALL bus topics for maximum observability (useful for debugging)
    pub async fn subscribe_all(&mut self, message_bus: &MessageBus) {
        let topics = get_all_event_message_topics();
        self.subscribe_topics(message_bus, topics).await;
    }

    /// Scroll up in message history
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll down in message history
    pub fn scroll_down(&mut self) {
        let max_scroll = self.recent_messages.len().saturating_sub(self.visible_lines);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    /// Scroll to the bottom (most recent messages)
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.recent_messages.len().saturating_sub(self.visible_lines);
    }

    /// Check if we're at the bottom of the message list
    fn is_at_bottom(&self) -> bool {
        let max_scroll = self.recent_messages.len().saturating_sub(self.visible_lines);
        self.scroll_offset >= max_scroll
    }

    /// Poll for new messages (call this in your event loop)
    pub fn poll_messages(&mut self) {
        let was_at_bottom = self.is_at_bottom();
        let mut new_messages = false;

        // Poll main receiver FIRST (gets ALL messages)
        if let Some(receiver) = &mut self.main_receiver {
            while let Ok(message) = receiver.try_recv() {
                log_debug!("📨 MessagesPanel (MAIN) received: topic={}, source={}, payload={}",
                    message.topic, message.source, message.payload);

                self.recent_messages.push(message);
                new_messages = true;

                if self.recent_messages.len() > self.max_messages {
                    self.recent_messages.remove(0);
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                }
            }
        }

        // Also poll topic-specific receivers (for backward compatibility)
        for receiver in &mut self.receivers {
            while let Ok(message) = receiver.try_recv() {
                log_debug!("📨 MessagesPanel (topic) received: topic={}, source={}, payload={}",
                    message.topic, message.source, message.payload);

                self.recent_messages.push(message);
                new_messages = true;

                if self.recent_messages.len() > self.max_messages {
                    self.recent_messages.remove(0);
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                }
            }
        }

        // Auto-scroll to bottom when new messages arrive (only if already at bottom)
        if new_messages && was_at_bottom {
            self.scroll_to_bottom();
        }
    }

    pub fn add_message(&mut self, message: BusMessage) {
        let was_at_bottom = self.is_at_bottom();

        log_debug!("📨 MessagesPanel received: topic={}, source={}, payload={}",
        message.topic, message.source, message.payload);

        self.recent_messages.push(message);

        if self.recent_messages.len() > self.max_messages {
            self.recent_messages.remove(0);
            if self.scroll_offset > 0 {
                self.scroll_offset -= 1;
            }
        }

        if was_at_bottom {
            self.scroll_to_bottom();
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, is_focused: bool) {
        // Update visible lines based on available area (subtract borders)
        self.visible_lines = (area.height.saturating_sub(2)) as usize;

        let content = if self.recent_messages.is_empty() {
            Text::from("Message bus activity will appear here...\n\nWaiting for messages...\nPress Tab to focus this panel, then ↑/↓ to scroll")
        } else {
            let total = self.recent_messages.len();
            let end = total.saturating_sub(self.scroll_offset);
            let start = end.saturating_sub(self.visible_lines);

            let lines: Vec<String> = self.recent_messages[start..end]
                .iter()
                .map(|msg| {
                    format!(
                        "[{}] {}: {}",
                        msg.source,
                        msg.topic,
                        msg.payload
                    )
                })
                .collect();

            Text::from(lines.join("\n"))
        };

        let title = if self.recent_messages.is_empty() {
            "Message Bus".to_string()
        } else {
            let total = self.recent_messages.len();
            let end = total.saturating_sub(self.scroll_offset);
            let start = end.saturating_sub(self.visible_lines);

            if self.scroll_offset == 0 {
                format!("Message Bus ({}/{}) [LIVE]", total.min(self.visible_lines), total)
            } else {
                format!(
                    "Message Bus ({}-{}/{}) ↑ {}",
                    start + 1,
                    end,
                    total,
                    self.scroll_offset
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

// Standalone render function for backward compatibility
pub fn render_recent_messages_panel(area: Rect, buf: &mut Buffer) {
    let mut panel = MessagesPanel::new();
    panel.render(area, buf, false);
}

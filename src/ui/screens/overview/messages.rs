use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use crate::util::bus::{BusMessage, BusReceiver, MessageBus};
use crate::ui::style::{dim_unless_focused};

#[derive(Debug)]
pub struct MessagesPanel {
    receivers: Vec<BusReceiver>,
    recent_messages: Vec<BusMessage>,
    max_messages: usize,
    scroll_offset: usize,
    visible_lines: usize,
}

impl MessagesPanel {
    pub fn new() -> Self {
        Self {
            receivers: Vec::new(),
            recent_messages: Vec::new(),
            max_messages: 100,  // Keep more history for scrolling
            scroll_offset: 0,
            visible_lines: 10,
        }
    }

    /// Subscribe to all topics (or specific topics)
    pub async fn subscribe(&mut self, message_bus: &MessageBus) {
        let receiver = message_bus.subscribe("navigation".to_string()).await;
        self.receivers.push(receiver);
    }

    /// Subscribe to multiple topics
    pub async fn subscribe_topics(&mut self, message_bus: &MessageBus, topics: Vec<String>) {
        // Subscribe to ALL topics
        for topic in topics {
            let receiver = message_bus.subscribe(topic).await;
            self.receivers.push(receiver);
        }
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

        // Poll ALL receivers
        for receiver in &mut self.receivers {
            // Try to receive all pending messages without blocking
            while let Ok(message) = receiver.try_recv() {
                self.recent_messages.push(message);

                // Keep only the most recent messages
                if self.recent_messages.len() > self.max_messages {
                    self.recent_messages.remove(0);
                    // Adjust scroll offset if we removed a message
                    if self.scroll_offset > 0 {
                        self.scroll_offset -= 1;
                    }
                }
            }
        }

        // Auto-scroll to bottom when new messages arrive (only if already at bottom)
        if was_at_bottom {
            self.scroll_to_bottom();
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, is_focused: bool) {
        // Update visible lines based on available area (subtract borders)
        self.visible_lines = (area.height.saturating_sub(2)) as usize;

        let content = if self.recent_messages.is_empty() {
            Text::from("Message bus activity will appear here...\n\nWaiting for messages...\n\nPress ← or → to test!\nPress Tab to focus this panel, then ↑/↓ to scroll")
        } else {
            // Calculate which messages to show
            let total = self.recent_messages.len();
            let start = self.scroll_offset;
            let end = (start + self.visible_lines).min(total);

            // Format messages in the visible range (reversed to show most recent at top)
            let lines: Vec<String> = self.recent_messages[start..end]
                .iter()
                .rev()
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

        // Show scroll indicator in title
        let title = if self.recent_messages.is_empty() {
            "Message Bus".to_string()
        } else {
            let total = self.recent_messages.len();
            let viewing_end = (self.scroll_offset + self.visible_lines).min(total);
            format!(
                "Message Bus ({}-{}/{})",
                self.scroll_offset + 1,
                viewing_end,
                total
            )
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
            .style(text_style)  // Use the style with dim
            .wrap(Wrap { trim: true });

        messages_widget.render(area, buf);
    }
}

// Standalone render function for backward compatibility
pub fn render_recent_messages_panel(area: Rect, buf: &mut Buffer) {
    let mut panel = MessagesPanel::new();
    panel.render(area, buf, false);
}

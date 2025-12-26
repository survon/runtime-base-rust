// src/ui/module_templates/control/toggle_switch.rs
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Borders, ListItem, Paragraph, Widget};
use ratatui::layout::{Alignment, Layout, Constraint, Direction};
use serde_json::Value;
use std::collections::HashMap;

use crate::module::Module;
use crate::ui::template::UiTemplate;

#[derive(Debug)]
pub struct ToggleSwitch;

struct ViewData<'a> {
    state: bool,
    label: &'a str,
    description: &'a str,
    status_color: Color,
    status_text: &'a str,
    switch_visual: &'a str,
    border_color: Color,
    block: Block<'a>,
    inner: Rect
}

fn get_str<'a>(bindings: &'a HashMap<String, Value>, key: &str, default: &'a str) -> &'a str {
    bindings
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
}

struct ToggleConfig<'a> {
    on_label: &'a str,
    off_label: &'a str,
    state: bool,
    label: &'a str,
}

impl<'a> ToggleConfig<'a> {
    fn from_bindings(bindings: &'a HashMap<String, Value>) -> Self {
        Self {
            on_label: get_str(bindings, "toggle_on_label", "ON"),
            off_label: get_str(bindings, "toggle_off_label", "OFF"),
            state: bindings
                .get("state")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            label: get_str(bindings, "label", "Toggle"),
        }
    }
}

impl ToggleSwitch {
    fn get_view_data<'a>(
        &self,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        module: &'a mut Module
    ) -> ViewData<'a> {
        // Get the state from module bindings
        let state = module
            .config
            .bindings
            .get("state")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Get optional label
        let label = module
            .config
            .bindings
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Get optional description
        let description = module
            .config
            .bindings
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Determine colors based on state
        let toggle_on_label = get_str(&module.config.bindings, "toggle_on_label", "ON");
        let toggle_off_label = get_str(&module.config.bindings, "toggle_off_label", "OFF");

        let (status_color, status_text, switch_visual) = if state {
            (Color::Green, toggle_on_label, "[ ────── ● ]")
        } else {
            (Color::Red, toggle_off_label, "[ ● ────── ]")
        };

        let border_color = if is_selected { Color:: White } else { status_color };

        // Create main container
        let block = Block::default()
            .title(format!(" {} ", module.config.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);

        ViewData {
            state,
            label,
            description,
            status_color,
            status_text,
            switch_visual,
            border_color,
            block,
            inner
        }
    }
}

impl UiTemplate for ToggleSwitch {
    fn render_overview_cta(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let ViewData {
            label,
            description,
            status_color,
            status_text,
            switch_visual,
            block,
            inner,
            ..
        } = self.get_view_data(is_selected, area, buf, module);

        Widget::render(block, area, buf);

        // Split inner area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Label
                Constraint::Length(3),  // Switch visual
                Constraint::Length(2),  // Status text
                Constraint::Min(1),     // Description
            ])
            .split(inner);

        // Render label if provided
        if !label.is_empty() {
            let label_widget = Paragraph::new(label)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center);
            Widget::render(label_widget, chunks[0], buf);
        }

        // Render switch visual
        let switch_widget = Paragraph::new(switch_visual)
            .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(switch_widget, chunks[1], buf);

        // Render status text
        let status_widget = Paragraph::new(status_text)
            .style(Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(status_widget, chunks[2], buf);

        // Render description if provided
        if !description.is_empty() {
            let desc_widget = Paragraph::new(description)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            Widget::render(desc_widget, chunks[3], buf);
        }
    }

    fn render_detail(&self, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let ViewData {
            label,
            description,
            status_color,
            status_text,
            switch_visual,
            block,
            inner,
            ..
        } = self.get_view_data(false, area, buf, module);

        Widget::render(block, area, buf);

        // Split inner area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // Label
                Constraint::Length(3),  // Switch visual
                Constraint::Length(2),  // Status text
                Constraint::Min(1),     // Description
            ])
            .split(inner);

        // Render label if provided
        if !label.is_empty() {
            let label_widget = Paragraph::new(label)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center);
            Widget::render(label_widget, chunks[0], buf);
        }

        // Render switch visual
        let switch_widget = Paragraph::new(switch_visual)
            .style(Style::default().fg(status_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(switch_widget, chunks[1], buf);

        // Render status text
        let status_widget = Paragraph::new(status_text)
            .style(Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(status_widget, chunks[2], buf);

        // Render description if provided
        if !description.is_empty() {
            let desc_widget = Paragraph::new(description)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            Widget::render(desc_widget, chunks[3], buf);
        }
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["state"]
    }

    fn docs(&self) -> &'static str {
        "Toggle switch display showing ON/OFF state. Required: 'state' (boolean). Optional: 'label' (string), 'description' (string). Green when ON, red when OFF."
    }
}

impl Default for ToggleSwitch {
    fn default() -> Self {
        Self
    }
}

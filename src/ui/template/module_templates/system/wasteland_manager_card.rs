// src/ui/module_templates/system/wasteland_manager_card.rs
use crate::modules::Module;
use crate::ui::template::UiTemplate;
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};

#[derive(Debug)]
pub struct WastelandManagerCard;

impl UiTemplate for WastelandManagerCard {
    fn render(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let current_view = module
            .config
            .bindings
            .get("current_view")
            .and_then(|v| v.as_str())
            .unwrap_or("Main");

        let selected_index = module
            .config
            .bindings
            .get("selected_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let status_message = module
            .config
            .bindings
            .get("status_message")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let border_color = if is_selected { Color::White } else { Color::Cyan };

        match current_view {
            "Main" => {
                self.render_main_menu(area, buf, border_color, selected_index, status_message, module)
            }
            "PendingTrust" => {
                self.render_pending_trust(area, buf, border_color, selected_index, status_message, module)
            }
            "AllDevices" => {
                self.render_all_devices(area, buf, border_color, selected_index, status_message, module)
            }
            "InstallRegistry" => {
                self.render_install_registry(area, buf, border_color, selected_index, status_message, module)
            }
            "ManageModules" => {
                self.render_manage_modules(area, buf, border_color, selected_index, status_message, module)
            }
            "ArchivedModules" => {
                self.render_archived_modules(area, buf, border_color, selected_index, status_message, module)
            }
            _ => {
                self.render_main_menu(area, buf, border_color, selected_index, status_message, module)
            }
        }
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["current_view", "selected_index"]
    }

    fn docs(&self) -> &'static str {
        "Wasteland Manager interface for managing modules, trusting BLE devices, and installing from registry. \
         Integrates device trust management with pending and known devices."
    }
}

impl WastelandManagerCard {
    fn render_main_menu(
        &self,
        area: Rect,
        buf: &mut Buffer,
        border_color: Color,
        selected_index: usize,
        status_message: Option<&str>,
        module: &Module,
    ) {
        // Get counts for menu items
        let pending_count = module
            .config
            .bindings
            .get("pending_devices")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let known_count = module
            .config
            .bindings
            .get("known_devices")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let registry_count = module
            .config
            .bindings
            .get("module_list")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let installed_count = module
            .config
            .bindings
            .get("installed_modules")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let archived_count = module
            .config
            .bindings
            .get("archived_modules")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        let is_scanning = module
            .config
            .bindings
            .get("is_scanning")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let scan_countdown = module
            .config
            .bindings
            .get("scan_countdown")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;

        let menu_items = vec![
            format!("⚠️  Trust Pending Devices ({})", pending_count),
            format!("📡 Manage All Devices ({})", known_count),
            format!("📦 Install from Registry ({})", registry_count),
            format!("⚙️  Manage Installed Modules ({})", installed_count),
            format!("📚 View Archived Modules ({})", archived_count),
            "← Back".to_string(),
        ];

        let has_status = status_message.is_some();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_status {
                vec![
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Menu
                    Constraint::Length(3),  // Status/Scan
                    Constraint::Length(3),  // Help
                ]
            } else {
                vec![
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Menu
                    Constraint::Length(3),  // Help
                ]
            })
            .split(area);

        // Title with alert indicator if pending devices
        let title_text = if pending_count > 0 {
            format!("⚙️  Wasteland Manager ⚠️  {} Pending", pending_count)
        } else {
            "⚙️  Wasteland Manager".to_string()
        };

        let title_color = if pending_count > 0 { Color::Yellow } else { Color::Cyan };

        let title = Paragraph::new(title_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(title_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Menu list
        let list_items: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if i == selected_index { "▶ " } else { "  " };
                ListItem::new(format!("{}{}", prefix, item)).style(style)
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(" Main Menu ")
            );
        Widget::render(list, chunks[1], buf);

        // Status message if present
        let help_index = if has_status {
            if is_scanning {
                let scan_msg = format!("🔍 SCANNING FOR DEVICES... {} seconds remaining", scan_countdown);
                let scan_widget = Paragraph::new(scan_msg)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Blue))
                            .title(" Scan in Progress ")
                    )
                    .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center);
                Widget::render(scan_widget, chunks[2], buf);
            } else if let Some(status) = status_message {
                let status_widget = Paragraph::new(status)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Yellow))
                            .title(" Status ")
                    )
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center);
                Widget::render(status_widget, chunks[2], buf);
            }
            3
        } else {
            2
        };

        // Help
        let help = Paragraph::new("↑/↓: Navigate • Enter: Select • 's': Scan for Devices • 'r': Refresh • Esc: Back")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        Widget::render(help, chunks[help_index], buf);
    }

    fn render_pending_trust(
        &self,
        area: Rect,
        buf: &mut Buffer,
        border_color: Color,
        selected_index: usize,
        status_message: Option<&str>,
        module: &Module,
    ) {
        let pending_devices = module
            .config
            .bindings
            .get("pending_devices")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let is_scanning = module
            .config
            .bindings
            .get("is_scanning")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let scan_countdown = module
            .config
            .bindings
            .get("scan_countdown")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;

        let has_status = status_message.is_some() || is_scanning;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_status {
                vec![
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Device list
                    Constraint::Length(3),  // Status/Scan
                    Constraint::Length(3),  // Help
                ]
            } else {
                vec![
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Device list
                    Constraint::Length(3),  // Help
                ]
            })
            .split(area);

        // Title with alert styling
        let title = Paragraph::new(format!("⚠️  New Devices Discovered ({})", pending_devices.len()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Device list
        if pending_devices.is_empty() {
            let empty_msg = Paragraph::new("No pending devices.\n\nNew devices will appear here when discovered.\n\nThey need to be trusted before registration.")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                )
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            Widget::render(empty_msg, chunks[1], buf);
        } else {
            let list_items: Vec<ListItem> = pending_devices
                .iter()
                .enumerate()
                .map(|(i, device)| {
                    let style = if i == selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let prefix = if i == selected_index { "▶ " } else { "  " };
                    ListItem::new(format!("{}{}", prefix, device)).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow))
                        .title(" Select device to trust ")
                );
            Widget::render(list, chunks[1], buf);
        }

        // Status message if present
        let help_index = if has_status {
            if is_scanning {
                let scan_msg = format!("🔍 SCANNING... {} seconds", scan_countdown);
                let scan_widget = Paragraph::new(scan_msg)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Blue))
                            .title(" Scan in Progress ")
                    )
                    .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center);
                Widget::render(scan_widget, chunks[2], buf);
            } else if let Some(status) = status_message {
                let status_widget = Paragraph::new(status)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Yellow))
                            .title(" Status ")
                    )
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center);
                Widget::render(status_widget, chunks[2], buf);
            }
            3
        } else {
            2
        };

        // Help
        let help = Paragraph::new("↑/↓: Select • Enter: Trust & Register • 'i': Ignore • 's': Scan Now • 'v': View All • Esc: Back")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        Widget::render(help, chunks[help_index], buf);
    }

    fn render_all_devices(
        &self,
        area: Rect,
        buf: &mut Buffer,
        border_color: Color,
        selected_index: usize,
        status_message: Option<&str>,
        module: &Module,
    ) {
        let known_devices = module
            .config
            .bindings
            .get("known_devices")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let is_scanning = module
            .config
            .bindings
            .get("is_scanning")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let scan_countdown = module
            .config
            .bindings
            .get("scan_countdown")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;

        let has_status = status_message.is_some() || is_scanning;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_status {
                vec![
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Device list
                    Constraint::Length(3),  // Status/Scan
                    Constraint::Length(3),  // Help
                ]
            } else {
                vec![
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Device list
                    Constraint::Length(3),  // Help
                ]
            })
            .split(area);

        // Title
        let title = Paragraph::new(format!("📡 All Known Devices ({})", known_devices.len()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Device list
        if known_devices.is_empty() {
            let empty_msg = Paragraph::new("No devices discovered yet.\n\nDevices will appear here when they're in range.\n\nPress 'r' to refresh scanning.")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                )
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            Widget::render(empty_msg, chunks[1], buf);
        } else {
            let list_items: Vec<ListItem> = known_devices
                .iter()
                .enumerate()
                .map(|(i, device)| {
                    // Parse device string format: "✓ Device Name (MAC) RSSI: -65 dBm"
                    let is_trusted = device.starts_with('✓');
                    let is_untrusted = device.starts_with('✗');

                    let style = if i == selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(if is_trusted { Color::Green } else { Color::Gray })
                            .add_modifier(Modifier::BOLD)
                    } else if is_trusted {
                        Style::default().fg(Color::Green)
                    } else if is_untrusted {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Gray)
                    };

                    let prefix = if i == selected_index { "▶ " } else { "  " };
                    ListItem::new(format!("{}{}", prefix, device)).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                        .title(" All Known Devices ")
                );
            Widget::render(list, chunks[1], buf);
        }

        // Status message if present
        let help_index = if has_status {
            if is_scanning {
                let scan_msg = format!("🔍 SCANNING... {} seconds", scan_countdown);
                let scan_widget = Paragraph::new(scan_msg)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Blue))
                            .title(" Scan in Progress ")
                    )
                    .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center);
                Widget::render(scan_widget, chunks[2], buf);
            } else if let Some(status) = status_message {
                let status_widget = Paragraph::new(status)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Yellow))
                            .title(" Status ")
                    )
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center);
                Widget::render(status_widget, chunks[2], buf);
            }
            3
        } else {
            2
        };

        // Help
        let help = Paragraph::new("↑/↓: Navigate • 't': Toggle Trust • 'd': Delete • 's': Scan Now • 'p': Pending • Esc: Back")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        Widget::render(help, chunks[help_index], buf);
    }

    fn render_install_registry(
        &self,
        area: Rect,
        buf: &mut Buffer,
        border_color: Color,
        selected_index: usize,
        status_message: Option<&str>,
        module: &Module,
    ) {
        let module_list = module
            .config
            .bindings
            .get("module_list")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(1),     // Module list
                Constraint::Length(3),  // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new(format!("📦 Registry Modules ({})", module_list.len()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Module list
        let list_items: Vec<ListItem> = module_list
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if i == selected_index { "▶ " } else { "  " };
                ListItem::new(format!("{}{}", prefix, item)).style(style)
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(" Select module to install ")
            );
        Widget::render(list, chunks[1], buf);

        // Help
        let help = Paragraph::new("↑/↓: Navigate • Enter: Install • Esc: Back")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        Widget::render(help, chunks[2], buf);
    }

    fn render_manage_modules(
        &self,
        area: Rect,
        buf: &mut Buffer,
        border_color: Color,
        selected_index: usize,
        status_message: Option<&str>,
        module: &Module,
    ) {
        let installed_modules = module
            .config
            .bindings
            .get("installed_modules")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(1),     // Module list
                Constraint::Length(3),  // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new(format!("⚙️  Installed Modules ({})", installed_modules.len()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Module list
        let list_items: Vec<ListItem> = installed_modules
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if i == selected_index { "▶ " } else { "  " };
                ListItem::new(format!("{}{}", prefix, item)).style(style)
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(" Manage installed modules ")
            );
        Widget::render(list, chunks[1], buf);

        // Help
        let help = Paragraph::new("↑/↓: Navigate • 'a': Archive Module • Esc: Back")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        Widget::render(help, chunks[2], buf);
    }

    fn render_archived_modules(
        &self,
        area: Rect,
        buf: &mut Buffer,
        border_color: Color,
        selected_index: usize,
        status_message: Option<&str>,
        module: &Module,
    ) {
        let archived_modules = module
            .config
            .bindings
            .get("archived_modules")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(1),     // Module list
                Constraint::Length(3),  // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new(format!("📚 Archived Modules ({})", archived_modules.len()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Module list
        if archived_modules.is_empty() {
            let empty_msg = Paragraph::new("No archived modules.")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                )
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            Widget::render(empty_msg, chunks[1], buf);
        } else {
            let list_items: Vec<ListItem> = archived_modules
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let style = if i == selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };

                    let prefix = if i == selected_index { "↩ " } else { "  " };
                    ListItem::new(format!("{}{}", prefix, item)).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                        .title(" Select module to restore ")
                );
            Widget::render(list, chunks[1], buf);
        }

        // Help
        let help = Paragraph::new("↑/↓: Navigate • Enter: Restore Module • Esc: Back")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        Widget::render(help, chunks[2], buf);
    }
}

impl Default for WastelandManagerCard {
    fn default() -> Self {
        Self
    }
}

use crate::app::{App, OverviewFocus};
use crate::module::ModuleManagerView;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap},
};

#[derive(Debug, Default)]
pub struct CouncilUiState {
    pub advisors: Vec<CouncilAdvisor>,
    pub messages: Vec<CouncilMessage>,
    pub selected_advisor: usize,
    pub selected_message: usize,
}

#[derive(Debug, Clone)]
pub struct CouncilAdvisor {
    pub id: String,
    pub name: String,
    pub expertise: String,
    pub online: bool,
}

#[derive(Debug, Clone)]
pub struct CouncilMessage {
    pub advisor: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct CouncilScreen;

impl CouncilScreen {
    pub fn new() -> Self {
        Self
    }

    pub fn init_state(&mut self) -> CouncilUiState {
        CouncilUiState {
            advisors: vec![
                CouncilAdvisor {
                    id: "advisor_hardware".to_string(),
                    name: "Hardware Expert".to_string(),
                    expertise: "Device management".to_string(),
                    online: true,
                },
                CouncilAdvisor {
                    id: "advisor_knowledge".to_string(),
                    name: "Knowledge Base".to_string(),
                    expertise: "Information retrieval".to_string(),
                    online: true,
                },
                CouncilAdvisor {
                    id: "advisor_monitor".to_string(),
                    name: "System Monitor".to_string(),
                    expertise: "System health".to_string(),
                    online: false,
                },
                CouncilAdvisor {
                    id: "advisor_security".to_string(),
                    name: "Security Specialist".to_string(),
                    expertise: "Security & access".to_string(),
                    online: true,
                },
            ],
            messages: vec![
                CouncilMessage {
                    advisor: "System".to_string(),
                    content: "🚀 Council initialized - advisors ready".to_string(),
                    timestamp: 0,
                },
                CouncilMessage {
                    advisor: "Hardware Expert".to_string(),
                    content: "📊 All devices connected and operational".to_string(),
                    timestamp: 1,
                },
                CouncilMessage {
                    advisor: "System Monitor".to_string(),
                    content: "⚡ Performance optimal".to_string(),
                    timestamp: 2,
                },
            ],
            selected_advisor: 0,
            selected_message: 0,
        }
    }

    pub fn render(app: &mut App, area: Rect, buf: &mut Buffer) {
        let constraints = [
            Constraint::Length(3), // Header
            Constraint::Min(1),    // Main content
            Constraint::Length(3), // Footer
        ];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Render header
        Self::render_header(chunks[0], buf);

        // Render main content
        Self::render_main_content(app, chunks[1], buf);

        // Render footer
        Self::render_footer(app, chunks[2], buf);
    }

    fn render_header(area: Rect, buf: &mut Buffer) {
        let header = Paragraph::new(" 💬 Council Chat ")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title_style(Style::default().add_modifier(Modifier::BOLD)),
            )
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
            .alignment(Alignment::Center);

        header.render(area, buf);
    }

    fn render_main_content(app: &mut App, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Advisors list
                Constraint::Percentage(70), // Chat area
            ])
            .split(area);

        // Render advisors list
        Self::render_advisors_list(app, main_layout[0], buf);

        // Render chat area
        Self::render_chat_area(app, main_layout[1], buf);
    }

    fn render_advisors_list(app: &mut App, area: Rect, buf: &mut Buffer) {
        let state = &app.council_state;

        let items: Vec<ListItem> = state
            .advisors
            .iter()
            .enumerate()
            .map(|(idx, advisor)| {
                let status = if advisor.online { "🟢" } else { "🔴" };
                let line = format!("{} {}: {}", status, advisor.name, advisor.expertise);
                let style = if idx == state.selected_advisor {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if advisor.online {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                ListItem::new(Span::raw(line)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
                    .title(" Advisors "),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::REVERSED)
                    .bg(Color::Blue),
            );

        list.render(area, buf);
    }

    fn render_chat_area(app: &mut App, area: Rect, buf: &mut Buffer) {
        let state = &app.council_state;

        let message_lines: Vec<Line> = state
            .messages
            .iter()
            .map(|msg| {
                let prefix = format!("[{}]: ", msg.advisor);
                Line::from(vec![Span::raw(prefix), Span::raw(&msg.content)])
            })
            .collect();

        let chat_area = Paragraph::new(message_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
                    .title(" Chat History "),
            )
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));

        chat_area.render(area, buf);
    }

    fn render_footer(app: &mut App, area: Rect, buf: &mut Buffer) {
        let help_text = " [Tab] Switch advisors  [Enter] Send message  [Esc] Back  [q] Quit ";
        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
                    .title(" Controls "),
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);

        help.render(area, buf);
    }

    pub fn handle_key_event(app: &mut App, key: crossterm::event::KeyCode) -> bool {
        match key {
            crossterm::event::KeyCode::Tab => {
                let state = &mut app.council_state;
                if !state.advisors.is_empty() {
                    state.selected_advisor = (state.selected_advisor + 1) % state.advisors.len();
                }
                true
            }
            crossterm::event::KeyCode::Enter => {
                let state = &mut app.council_state;
                if let Some(advisor) = state.advisors.get(state.selected_advisor) {
                    let msg = CouncilMessage {
                        advisor: "You".to_string(),
                        content: "Query sent to council...".to_string(),
                        timestamp: state.messages.len() as u64,
                    };
                    state.messages.push(msg);
                }
                true
            }
            crossterm::event::KeyCode::Up => {
                let state = &mut app.council_state;
                if !state.messages.is_empty() {
                    state.selected_message = state.selected_message.saturating_sub(1);
                }
                true
            }
            crossterm::event::KeyCode::Down => {
                let state = &mut app.council_state;
                if !state.messages.is_empty() {
                    state.selected_message =
                        (state.selected_message + 1).min(state.messages.len() - 1);
                }
                true
            }
            crossterm::event::KeyCode::Esc => {
                app.mode = crate::app::AppMode::Overview;
                true
            }
            _ => false,
        }
    }
}

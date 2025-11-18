use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use color_eyre::Result;
use std::path::{Path,PathBuf};
use std::time::Duration;
use std::sync::Arc;
use gag::Gag;
use std::collections::HashMap;
use ratatui::{Frame, layout::Rect};

use crate::event::{AppEvent, Event, EventHandler};
use crate::modules::{Module, ModuleManager};
use crate::bus::{MessageBus, BusMessage, BusReceiver};
use crate::database::{Database, ChatMessage};
use crate::modules::llm::handler::LlmHandler;

use crate::ui::{
    document_viewer::{DocumentViewer, DocumentContent, DocumentManager},
    external_viewer::ExternalViewer,
    module_detail,
    splash::SplashScreen,
    messages::MessagesPanel
};

#[derive(Debug, PartialEq, Clone)]
pub enum ModuleSource {
    Wasteland,
    Core,
}

#[derive(Debug, PartialEq)]
pub enum OverviewFocus {
    WastelandModules,
    Messages,
    CoreModules
}

#[derive(Debug, PartialEq)]
pub enum AppMode {
    Splash,
    Overview,
    ModuleDetail(ModuleSource, usize),
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Current app mode/screen
    pub mode: AppMode,

    pub needs_redraw: bool,
    pub splash_screen: Option<SplashScreen>,

    /// Module managers
    pub wasteland_module_manager: ModuleManager,
    pub core_module_manager: ModuleManager,

    /// Message bus
    pub message_bus: MessageBus,
    /// Bus receiver for incoming messages
    pub bus_receiver: BusReceiver,
    /// Database for persistent storage
    pub database: Database,
    /// Event handler.
    pub events: EventHandler,

    pub document_manager: DocumentManager,
    pub messages_panel: MessagesPanel,
    pub overview_focus: OverviewFocus,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> Result<Self> {

        let core_modules_path = std::path::PathBuf::from("./modules/core/");
        let core_modules_namespace= "core".to_string();
        let mut core_module_manager = ModuleManager::new(core_modules_path, core_modules_namespace);

        let wasteland_modules_path = std::path::PathBuf::from("./modules/wasteland/");
        let wasteland_modules_namespace= "wasteland".to_string();
        let mut wasteland_module_manager = ModuleManager::new(wasteland_modules_path, wasteland_modules_namespace);

        let (message_bus, bus_receiver) = MessageBus::new();
        let database = Database::new_implied_all_schemas()?;

        // Discover modules on startup
        if let Err(e) = wasteland_module_manager.discover_modules() {
            eprintln!("Failed to discover wasteland modules: {}", e);
        }
        if let Err(e) = core_module_manager.discover_modules() {
            eprintln!("Failed to discover core modules: {}", e);
        }

        // Subscribe module manager to events
        wasteland_module_manager.subscribe_to_events(&message_bus).await;
        core_module_manager.subscribe_to_events(&message_bus).await;

        // Initialize handlers for core modules (LLM, etc.)
        if let Err(e) = core_module_manager.initialize_module_handlers(&database, message_bus.get_sender()).await {
            eprintln!("Failed to initialize module handlers: {}", e);
        }

        // Knowledge ingestion...
        let ingester = crate::knowledge::KnowledgeIngester::new(&database);
        if ingester.should_reingest()? {
            ingester.ingest_all_knowledge()?;
        }

        let mut messages_panel = MessagesPanel::new();
        messages_panel.subscribe_topics(
            &message_bus,
            vec![
                "com_input".to_string(),
                "sensor_data".to_string(),
                "navigation".to_string(),
                "app.event.increment".to_string(),
                "app.event.decrement".to_string(),
            ]
        ).await;

        Ok(Self {
            running: true,
            mode: AppMode::Splash,
            needs_redraw: false,
            splash_screen: Some(SplashScreen::new()),
            wasteland_module_manager,
            core_module_manager,
            message_bus,
            bus_receiver,
            database,
            events: EventHandler::new(),
            document_manager: DocumentManager::new()?,
            messages_panel,
            overview_focus: OverviewFocus::CoreModules,
        })
    }

    async fn publish_event(&self, topic: &str, payload: &str) {
        let msg = BusMessage::new(
            topic.to_string(),
            payload.to_string(),
            "survon_tui".to_string()
        );
        let _ = self.message_bus.publish(msg).await;
    }

    pub fn has_active_blinks(&self) -> bool {
        self.wasteland_module_manager.has_active_blinks() ||
            self.core_module_manager.has_active_blinks()
    }

    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
    }

    async fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Tick => Ok(self.handle_tick()),
            Event::Crossterm(event) => self.handle_crossterm_event(event),
            Event::App(app_event) => self.handle_app_event(app_event).await,
        }
    }

    fn handle_tick(&mut self) -> bool {
        // During splash, we need continuous redraws for animation
        let should_animate = matches!(self.mode, AppMode::Splash) ||
            (matches!(self.mode, AppMode::Overview) && self.has_active_blinks());

        should_animate
    }

    fn handle_crossterm_event(&mut self, event: crossterm::event::Event) -> Result<bool> {
        match event {
            crossterm::event::Event::Key(key_event) => {
                if matches!(self.mode, AppMode::Splash) {
                    self.mode = AppMode::Overview;
                    self.splash_screen = None;
                    return Ok(true);
                }

                self.handle_key_events(key_event)?;
                use std::io::Write;
                std::io::stdout().flush()?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Publish an AppEvent to the message bus
    async fn publish_app_event(&self, event: &AppEvent) -> Result<()> {
        let (topic, payload) = match event {
            AppEvent::Increment => ("increment", String::new()),
            AppEvent::Decrement => ("decrement", String::new()),
            AppEvent::Select => ("select", String::new()),
            AppEvent::Back => ("back", String::new()),
            AppEvent::RefreshModules => ("refresh_modules", String::new()),
            AppEvent::Quit => ("quit", String::new()),
            AppEvent::OpenDocument(path) => ("open_document", path.clone()),
            AppEvent::CloseDocument => ("close_document", String::new()),
            AppEvent::ScrollDocumentUp => ("scroll_document_up", String::new()),
            AppEvent::ScrollDocumentDown => ("scroll_document_down", String::new()),
            AppEvent::SendCommand(topic, cmd) => ("send_command", format!("{}:{}", topic, cmd)),
            AppEvent::ChatSubmit => ("chat_submit", String::new()),

        };

        self.message_bus.publish_app_event(topic, &payload).await
    }

    async fn handle_app_event(&mut self, app_event: AppEvent) -> Result<bool> {
        // First, publish to message bus for modules/components to react
        self.publish_app_event(&app_event).await?;

        // Then handle local state changes that App owns
        match app_event {
            // Navigation events - App owns UI state
            AppEvent::Increment => {
                self.wasteland_module_manager.next_module();
                Ok(true)
            }
            AppEvent::Decrement => {
                self.wasteland_module_manager.prev_module();
                Ok(true)
            }
            AppEvent::Select => Ok(self.handle_select()),
            AppEvent::Back => {
                self.back_to_overview();
                Ok(true)
            }

            // System events
            AppEvent::Quit => {
                self.quit();
                Ok(false)
            }
            AppEvent::RefreshModules => {
                self.handle_refresh_modules().await;
                Ok(true)
            }

            // Document events
            AppEvent::OpenDocument(file_path) => {
                self.document_manager.open_document(file_path);
                Ok(true)
            }
            AppEvent::CloseDocument => {
                self.handle_close_document();
                Ok(true)
            }
            AppEvent::ScrollDocumentUp => {
                self.document_manager.scroll_document_up();
                Ok(true)
            }
            AppEvent::ScrollDocumentDown => {
                self.document_manager.scroll_document_down();
                Ok(true)
            }

            // Command pass-through - publishing is enough, modules will handle
            AppEvent::SendCommand(_, _) => Ok(false),

            // ChatSubmit needs special handling because it's async
            AppEvent::ChatSubmit => {
                self.handle_chat_submit().await;
                Ok(true)
            }
        }
    }

    fn handle_select(&mut self) -> bool {
        match self.overview_focus {
            OverviewFocus::WastelandModules => {
                let source = ModuleSource::Wasteland;
                let module_index = self.wasteland_module_manager.selected_module;

                match self.wasteland_module_manager.select_current_module() {
                    Some(_) => {
                        self.mode = AppMode::ModuleDetail(source, module_index);
                        true
                    }
                    None => {
                        self.mode = AppMode::Overview;
                        true
                    }
                }
            },
            OverviewFocus::CoreModules => {
                let source = ModuleSource::Core;
                let module_index = self.core_module_manager.selected_module;

                match self.core_module_manager.select_current_module() {
                    Some(_module) => {
                        self.mode = AppMode::ModuleDetail(source, module_index);
                        true
                    }
                    None => {
                        self.mode = AppMode::Overview;
                        true
                    }
                }
            },
            _ => false,
        }
    }

    pub fn toggle_overview_focus(&mut self) {
        self.overview_focus = match self.overview_focus {
            OverviewFocus::WastelandModules => OverviewFocus::Messages,
            OverviewFocus::Messages => OverviewFocus::CoreModules,
            OverviewFocus::CoreModules => OverviewFocus::WastelandModules,
        };
    }

    async fn handle_refresh_modules(&mut self) {
        self.wasteland_module_manager.refresh_modules().await;
        self.core_module_manager.refresh_modules().await;

        // Re-initialize handlers after refresh
        if let Err(e) = self.core_module_manager.initialize_module_handlers(
            &self.database,
            self.message_bus.get_sender()
        ).await {
            eprintln!("Failed to re-initialize handlers: {}", e);
        }
    }

    fn handle_close_document(&mut self) {
        self.document_manager.close_document();
    }

    fn render_current_mode(&mut self, frame: &mut Frame) {
        match &self.mode {
            AppMode::Splash => self.render_splash(frame),
            AppMode::Overview => self.render_widget_mode(frame),
            AppMode::ModuleDetail(source, module_idx) => {
                self.render_module_detail(frame, source.clone(), *module_idx)
            },
        }
    }

    fn render_splash(&mut self, frame: &mut Frame) {
        if let Some(splash) = &mut self.splash_screen {
            let area = frame.area();
            let buf = frame.buffer_mut();
            splash.render(area, buf);
        }
    }

    fn render_widget_mode(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn render_module_detail(&mut self, frame: &mut Frame, source: ModuleSource, module_idx: usize) {
        let area = frame.area();
        let buf = frame.buffer_mut();

        // First render chrome with immutable reference
        let module_manager_ref = match source {
            ModuleSource::Wasteland => &self.wasteland_module_manager,
            ModuleSource::Core => &self.core_module_manager,
        };
        module_detail::render_module_detail_chrome(module_manager_ref, module_idx, area, buf);

        // Update bindings and render template content
        let content_area = module_detail::get_content_area(area);

        let module_manager = match source {
            ModuleSource::Wasteland => &mut self.wasteland_module_manager,
            ModuleSource::Core => &mut self.core_module_manager,
        };

        // Let handler update bindings before render
        module_manager.update_module_bindings(module_idx);

        if let Some(module) = module_manager.get_modules_mut().get_mut(module_idx) {
            let is_selected = false;
            if let Err(e) = module.render(is_selected, content_area, buf) {
                self.render_template_error(frame, content_area, e);
            }
        }
    }

    fn render_template_error(&self, frame: &mut Frame, area: Rect, error: String) {
        use ratatui::widgets::{Block, BorderType, Paragraph, Wrap};
        use ratatui::style::{Color, Style};
        use ratatui::layout::Alignment;
        use ratatui::text::Line;

        let error_lines = vec![
            Line::from(""),
            Line::from("⚠️  Template Rendering Error").style(Style::default().fg(Color::Red)),
            Line::from(""),
            Line::from(error.clone()).style(Style::default().fg(Color::Yellow)),
            Line::from(""),
            Line::from("Check your module's config.yml:").style(Style::default().fg(Color::Gray)),
            Line::from("  - Is the 'template' field correct?").style(Style::default().fg(Color::Gray)),
            Line::from("  - Are all required bindings present?").style(Style::default().fg(Color::Gray)),
        ];

        let error_widget = Paragraph::new(error_lines)
            .block(
                Block::bordered()
                    .title("Error")
                    .border_type(BorderType::Rounded)
                    .style(Style::default().fg(Color::Red))
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(error_widget, area);
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let mut needs_redraw = true;

        while self.running {
            if matches!(self.mode, AppMode::Splash) {
                if let Some(splash) = &self.splash_screen {
                    if splash.is_complete() {
                        self.mode = AppMode::Overview;
                        self.splash_screen = None;
                        needs_redraw = true;
                    }
                }
            }

            if needs_redraw || self.needs_redraw {
                terminal.draw(|frame| {
                    self.render_current_mode(frame);
                })?;
                needs_redraw = false;
                self.needs_redraw = false;
            }

            // Poll for events from subscribed topics
            self.messages_panel.poll_messages();
            self.wasteland_module_manager.poll_events();
            self.core_module_manager.poll_events();

            tokio::select! {
                event = self.events.next() => {
                    if let Ok(event) = event {
                        if self.handle_event(event).await? {
                            needs_redraw = true;
                        }
                    } else if let Err(e) = event {
                        eprintln!("Event error: {}", e);
                    }
                }
                message = self.bus_receiver.recv() => {
                    if let Some(msg) = message {
                        self.handle_bus_message(msg);
                        needs_redraw = true;
                    }
                }
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<()> {
        let key_code = key_event.code;

        match key_code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Left if key_event.modifiers == KeyModifiers::SHIFT => {
                self.events.send(AppEvent::Decrement)
            },
            KeyCode::Right if key_event.modifiers == KeyModifiers::SHIFT => {
                self.events.send(AppEvent::Increment)
            },
            _ => {}
        }

        match &self.mode {
            AppMode::Splash => {},
            AppMode::Overview => {
                match key_code {
                    KeyCode::Tab => {
                        self.toggle_overview_focus();
                        return Ok(());
                    }
                    KeyCode::Up | KeyCode::Down if matches!(self.overview_focus, OverviewFocus::Messages) => {
                        if key_code == KeyCode::Up {
                            self.messages_panel.scroll_up();
                        } else {
                            self.messages_panel.scroll_down();
                        }
                        return Ok(());
                    }
                    _ => {}
                }
            },
            AppMode::ModuleDetail(source, module_idx) => {
                let module_manager = match source {
                    ModuleSource::Core => &mut self.core_module_manager,
                    ModuleSource::Wasteland => &mut self.wasteland_module_manager,
                };

                // Let the module's handler handle the key
                if let Some(event) = module_manager.handle_key_for_module(*module_idx, key_code) {
                    self.events.send(event);
                } else {
                    // Default module navigation keys
                    match key_code {
                        KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                        KeyCode::Backspace | KeyCode::Char('h') => self.events.send(AppEvent::Back),
                        KeyCode::Char('r') => self.events.send(AppEvent::RefreshModules),
                        KeyCode::Char('1') => {
                            self.events.send(AppEvent::SendCommand("com_input".to_string(), "close_gate".to_string()));
                        }
                        _ => {}
                    }
                }
            },
        }

        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn back_to_overview(&mut self) {
        self.mode = AppMode::Overview;
    }

    pub fn send_command(&self, topic: String, command: String) {
        if let Err(e) = self.message_bus.send_command(topic, command, "survon_tui".to_string()) {
            eprintln!("Failed to send command: {}", e);
        }
    }

    pub fn handle_bus_message(&mut self, message: BusMessage) {
        // Log message to database
        if let Err(e) = self.database.log_bus_message(&message.topic, &message.payload, &message.source) {
            eprintln!("Failed to log bus message: {}", e);
        }
    }

    // Method 1: Get LLM engine
    pub fn get_llm_engine(&self) -> Option<&crate::modules::llm::LlmEngine> {
        self.core_module_manager
            .get_handler("llm")
            .and_then(|h| h.as_any().downcast_ref::<LlmHandler>())
            .and_then(|llm_handler| llm_handler.get_engine())
    }

    // Method 2: Get mutable LLM handler
    fn get_llm_handler_mut(&mut self) -> Option<&mut LlmHandler> {
        self.core_module_manager
            .get_handler_mut("llm")
            .and_then(|h| h.as_any_mut().downcast_mut::<LlmHandler>())
    }

    // Method 3: Handle chat submit
    async fn handle_chat_submit(&mut self) {
        // Get the current LLM module name if we're in that view
        let module_name = if let AppMode::ModuleDetail(ModuleSource::Core, idx) = &self.mode {
            self.core_module_manager
                .get_modules()
                .get(*idx)
                .map(|m| m.config.name.clone())
        } else {
            None
        };

        if let Some(module_name) = module_name {
            // Collect the knowledge module names first (before any mutable borrows)
            let knowledge_module_names: Vec<String> = self.core_module_manager
                .get_knowledge_modules()
                .iter()
                .map(|m| m.config.name.clone())
                .collect();

            // Get the input text before borrowing the handler mutably
            let input_text = self.core_module_manager
                .get_handler("llm")
                .and_then(|h| h.as_any().downcast_ref::<LlmHandler>())
                .map(|llm_handler| llm_handler.get_manager().chat_manager.get_input().to_string());

            if let Some(input) = input_text {
                if !input.trim().is_empty() {
                    // Now get mutable access to process the message
                    if let Some(llm_handler) = self.core_module_manager
                        .get_handler_mut("llm")
                        .and_then(|h| h.as_any_mut().downcast_mut::<LlmHandler>())
                    {
                        // Process without needing module_manager reference
                        // The handler already has access to the engine which has the database
                        if let Err(e) = llm_handler.submit_message(
                            module_name,
                            knowledge_module_names
                        ).await {
                            eprintln!("Failed to submit chat message: {}", e);
                        }
                    }
                }
            }
        }
    }
}

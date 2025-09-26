use crate::event::{AppEvent, Event, EventHandler};
use crate::module::{Module, ModuleManager};
use crate::bus::{MessageBus, BusMessage, BusReceiver};
use crate::database::{Database, ChatMessage};
use crate::llm::{LlmEngine, create_llm_strategy};
use crate::ui::document_viewer::{DocumentViewer, DocumentContent};
use crate::ui::external_viewer::ExternalViewer;
use crate::ui::document_popup_widget::DocumentPopupWidget;
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

use crate::ui;

#[derive(Debug, PartialEq)]
pub enum AppMode {
    Overview,
    ModuleDetail(usize),
    LlmChat(usize), // Index of the LLM module
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Current app mode/screen
    pub mode: AppMode,

    pub chat_scroll_offset: usize,
    pub document_scroll_offset: usize,

    /// Selected module index
    pub selected_module: usize,
    /// Module manager
    pub module_manager: ModuleManager,
    /// Message bus
    pub message_bus: MessageBus,
    /// Bus receiver for incoming messages
    pub bus_receiver: BusReceiver,
    /// Database for persistent storage
    pub database: Database,
    /// LLM engine (optional - only created when LLM modules are available)
    pub llm_engine: Option<LlmEngine>,
    /// Current chat input buffer for LLM interface
    pub chat_input: String,
    /// Event handler.
    pub events: EventHandler,

    pub document_viewer: Option<DocumentViewer>,
    pub external_viewer: Option<Arc<ExternalViewer>>,
    pub show_document_popup: Option<String>, // File path to show

    pub current_link_index: Option<usize>,
    pub available_links: Vec<String>,
    pub cached_document: Option<(String, DocumentContent)>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> Result<Self> {
        let mut module_manager = ModuleManager::new();
        let (message_bus, bus_receiver) = MessageBus::new();
        let database = Database::new_implied_all_schemas()?;

        // Discover modules on startup
        if let Err(e) = module_manager.discover_modules() {
            eprintln!("Failed to discover modules: {}", e);
        }

        // Add knowledge ingestion here
        let ingester = crate::knowledge::KnowledgeIngester::new(&database);
        if ingester.should_reingest()? {
            ingester.ingest_all_knowledge()?;
        }

        // Initialize LLM engine if LLM modules are available
        let llm_engine = Self::create_llm_engine_if_available(
            &module_manager,
            &database,
            message_bus.get_sender(),
        ).await?;

        Ok(Self {
            running: true,
            mode: AppMode::Overview,
            chat_scroll_offset: 0,
            document_scroll_offset: 0,
            selected_module: 0,
            module_manager,
            message_bus,
            bus_receiver,
            database,
            llm_engine,
            chat_input: String::new(),
            events: EventHandler::new(),
            document_viewer: Some(DocumentViewer::new()),
            external_viewer: Some(Arc::new(ExternalViewer::new()?)),
            show_document_popup: None,
            current_link_index: None,
            available_links: Vec::new(),
            cached_document: None,
        })
    }

    pub fn scroll_document_up(&mut self) {
        if self.document_scroll_offset > 0 {
            self.document_scroll_offset -= 1;
        }
    }

    pub fn scroll_document_down(&mut self) {
        self.document_scroll_offset += 1;
    }

    pub fn scroll_document_page_up(&mut self) {
        for _ in 0..10 {
            self.scroll_document_up();
        }
    }

    pub fn scroll_document_page_down(&mut self) {
        for _ in 0..10 {
            self.scroll_document_down();
        }
    }

    pub fn cycle_document_links(&mut self) {
        self.cycle_document_links_direction(1);
    }

    pub fn cycle_document_links_backward(&mut self) {
        self.cycle_document_links_direction(-1);
    }

    fn cycle_document_links_direction(&mut self, direction: i32) {
        if !self.available_links.is_empty() {
            match self.current_link_index {
                None => self.current_link_index = Some(0),
                Some(index) => {
                    let len = self.available_links.len() as i32;
                    let new_index = if direction > 0 {
                        (index as i32 + 1) % len
                    } else {
                        (index as i32 - 1 + len) % len
                    };
                    self.current_link_index = Some(new_index as usize);
                }
            }
        }
    }

    pub fn clear_document_links(&mut self) {
        self.current_link_index = None;
        self.available_links.clear();
    }

    pub fn set_available_links(&mut self, links: Vec<String>) {
        self.available_links = links;
        self.current_link_index = None;
    }

    fn update_available_links(&mut self) {
        let mut links = Vec::new();

        if let Some(llm_engine) = &self.llm_engine {
            if let Ok(messages) = llm_engine.get_chat_history(50) {
                for msg in messages {
                    if msg.role == "assistant" {
                        // Extract file paths from assistant messages
                        for line in msg.content.lines() {
                            if line.contains("(from ./") {
                                if let Some(start) = line.find("(from ") {
                                    if let Some(end) = line[start..].find(')') {
                                        let file_path = &line[start + 6..start + end];
                                        links.push(file_path.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.set_available_links(links);
    }

    pub fn scroll_chat_up(&mut self) {
        if self.chat_scroll_offset > 0 {
            self.chat_scroll_offset -= 1;
        }
    }

    pub fn scroll_chat_down(&mut self) {
        if let Some(llm_engine) = &self.llm_engine {
            if let Ok(messages) = llm_engine.get_chat_history(50) {
                let total_lines = self.calculate_chat_content_lines(&messages);
                let visible_lines = self.get_chat_visible_lines(); // You'll need to add this
                let max_scroll = total_lines.saturating_sub(visible_lines);

                if self.chat_scroll_offset < max_scroll {
                    self.chat_scroll_offset += 1;
                }
            }
        }
    }

    pub fn get_chat_scroll_offset(&self) -> usize {
        self.chat_scroll_offset
    }

    fn calculate_chat_content_lines(&self, messages: &[ChatMessage]) -> usize {
        let mut line_count = 0;
        for msg in messages {
            let content_lines: Vec<&str> = msg.content.lines().collect();
            line_count += content_lines.len() + 1; // +1 for spacing
        }
        line_count
    }

    fn get_chat_visible_lines(&self) -> usize {
        // This should match the chat history area height from your layout
        // You'll need to calculate this based on your UI layout
        20 // Placeholder - adjust based on actual chat area height
    }

    pub fn get_cached_document(&self, file_path: &str) -> Option<&DocumentContent> {
        if let Some((cached_path, content)) = &self.cached_document {
            if cached_path == file_path {
                Some(content)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn cache_document(&mut self, file_path: String, content: DocumentContent) {
        self.cached_document = Some((file_path, content));
    }

    pub fn open_document(&mut self, file_path: String) {
        eprintln!("DEBUG: open_document called with: {}", file_path);

        // Parse and cache document first
        if let Some(viewer) = &self.document_viewer {
            if self.get_cached_document(&file_path).is_none() {
                if let Ok(content) = viewer.view_document(Path::new(&file_path)) {
                    self.cache_document(file_path.clone(), content);
                }
            }
        }

        if let Some(external_viewer) = &self.external_viewer {
            if external_viewer.as_ref().can_launch_wry() {
                eprintln!("Trying wry viewer...");
                if let Some((_, content)) = &self.cached_document {
                    if let Ok(_) = external_viewer.as_ref().show_document_wry(&file_path, content) {
                        eprintln!("Wry viewer launched");
                        return;
                    }
                }
            }
        }
        eprintln!("Using TUI fallback");

        // Tier 3: Fallback to TUI popup
        self.show_document_popup = Some(file_path);
        self.document_scroll_offset = 0;
    }

    pub fn close_document(&mut self) {
        self.show_document_popup = None;
        self.current_link_index = None;
    }
    async fn create_llm_engine_if_available(
        module_manager: &ModuleManager,
        _database: &Database,
        bus_sender: crate::bus::BusSender,
    ) -> Result<Option<LlmEngine>> {
        let llm_modules = module_manager.get_modules_by_type("llm");
        if let Some(llm_module) = llm_modules.first() {
            let model_name = llm_module.config.model.as_deref().unwrap_or("knowledge");

            println!("Found LLM module: {} with model: {}", llm_module.config.name, model_name);

            let (strategy, _) = create_llm_strategy(model_name).await;
            let session_id = format!("session_{}",
                                     std::time::SystemTime::now()
                                         .duration_since(std::time::UNIX_EPOCH)
                                         .unwrap()
                                         .as_secs()
            );

            let database = Database::new_implied_all_schemas()?;
            let engine = LlmEngine::new(strategy, database, bus_sender, session_id);
            Ok(Some(engine))
        } else {
            println!("No LLM modules found. Create an LLM module to enable AI chat.");
            Ok(None)
        }
    }

    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let mut needs_redraw = true;

        while self.running {
            if needs_redraw {
                let mut temp_scroll_offset = self.document_scroll_offset.clone();
                let has_popup = self.show_document_popup.is_some();

                terminal.draw(|frame| {
                    static mut RENDER_COUNT: u32 = 0;
                    unsafe {
                        RENDER_COUNT += 1;
                        if RENDER_COUNT % 100 == 0 { // Only print every 100th render
                            eprintln!("Render count: {}", RENDER_COUNT);
                        }
                    }

                    // Render main app
                    frame.render_widget(&self, frame.area());

                    // Render document popup if active
                    if let Some(file_path) = &self.show_document_popup {
                        if let Some((_, content)) = &self.cached_document {
                            if let Some(popup_file) = &self.show_document_popup {
                                let popup_widget = DocumentPopupWidget::new(content, popup_file);
                                frame.render_stateful_widget(
                                    popup_widget,
                                    frame.area(),
                                    &mut temp_scroll_offset
                                );
                            }
                        }
                    }
                })?;

                if has_popup {
                    self.document_scroll_offset = temp_scroll_offset;
                }

                // save power
                needs_redraw = false;
            }

            // Handle events with timeout so we can process bus messages
            tokio::select! {
                event = self.events.next() => {
                    match event {
                        Ok(event) => {
                            match event {
                                Event::Tick => {}, // Don't redraw on tick
                                Event::Crossterm(event) => match event {
                                    crossterm::event::Event::Key(key_event) => {
                                        self.handle_key_events(key_event)?;
                                        needs_redraw = true;
                                        use std::io::Write;
                                        std::io::stdout().flush()?;
                                    }
                                    _ => {}
                                },
                                Event::App(app_event) => match app_event {
                                    AppEvent::Increment => { self.next_module(); needs_redraw = true; }
                                    AppEvent::Decrement => { self.prev_module(); needs_redraw = true; }
                                    AppEvent::Quit => self.quit(),
                                    AppEvent::Select => { self.select_current_module(); needs_redraw = true; }
                                    AppEvent::Back => { self.back_to_overview(); needs_redraw = true; }
                                    AppEvent::RefreshModules => { self.refresh_modules().await; needs_redraw = true; }
                                    AppEvent::SendCommand(topic, command) => self.send_command(topic, command),
                                    AppEvent::EnterChat => { self.enter_llm_chat(); needs_redraw = true; }
                                    AppEvent::ChatInput(ch) => { self.handle_chat_input(ch); needs_redraw = true; }
                                    AppEvent::ChatSubmit => { self.submit_chat_message().await; needs_redraw = true; }
                                    AppEvent::ChatBackspace => { self.chat_backspace(); needs_redraw = true; }
                                    AppEvent::ScrollChatUp => { self.scroll_chat_up(); needs_redraw = true; }
                                    AppEvent::ScrollChatDown => { self.scroll_chat_down(); needs_redraw = true; }
                                    AppEvent::OpenDocument(file_path) => { self.open_document(file_path); needs_redraw = true; }
                                    AppEvent::CloseDocument => { self.close_document(); needs_redraw = true; }
                                    AppEvent::ScrollDocumentUp => { self.scroll_document_up(); needs_redraw = true; }
                                    AppEvent::ScrollDocumentDown => { self.scroll_document_down(); needs_redraw = true; }
                                },
                            }
                        }
                        Err(e) => eprintln!("Event error: {}", e),
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
        let start = std::time::Instant::now();

        // Handle document viewing mode
        if self.show_document_popup.is_some() {
            match key_event.code {
                KeyCode::Esc => {
                    self.close_document();
                    return Ok(());
                }
                KeyCode::Up => {
                    self.scroll_document_up();
                    return Ok(());
                }
                KeyCode::Down=> {
                    self.scroll_document_down();
                    return Ok(());
                }
                KeyCode::PageUp => {
                    self.scroll_document_page_up();
                    return Ok(());
                }
                KeyCode::PageDown => {
                    self.scroll_document_page_down();
                    return Ok(());
                }
                KeyCode::Char('e') => {
                    // Force external viewer for current document
                    if let Some(file_path) = &self.show_document_popup {
                        if let Some((_, content)) = &self.cached_document {
                            if let Some(external_viewer) = &self.external_viewer {
                                let path = file_path.clone();
                                let content_clone = content.clone();
                                let external_viewer = external_viewer.clone();

                                tokio::spawn(async move {
                                    if let Err(e) = external_viewer.show_document_external(&path, &content_clone).await {
                                        eprintln!("External viewer error: {}", e);
                                    }
                                });
                                self.close_document();
                            }
                        }
                    }
                    return Ok(());
                }
                _ => {}
            }
            return Ok(());
        }
        // Handle chat input mode
        if matches!(self.mode, AppMode::LlmChat(_)) {
            match key_event.code {
                KeyCode::Esc => self.events.send(AppEvent::Back),
                KeyCode::Tab => self.cycle_document_links(),
                KeyCode::BackTab => self.cycle_document_links_backward(),
                KeyCode::Enter => {
                    if let Some(index) = self.current_link_index {
                        if let Some(file_path) = self.available_links.get(index) {
                            self.events.send(AppEvent::OpenDocument(file_path.clone()));
                            return Ok(());
                        }
                    }
                    // No link selected, proceed with normal chat submit
                    self.events.send(AppEvent::ChatSubmit)
                },
                KeyCode::Backspace => self.events.send(AppEvent::ChatBackspace),
                KeyCode::Char(ch) => self.events.send(AppEvent::ChatInput(ch)),
                KeyCode::PageUp | KeyCode::Up => self.events.send(AppEvent::ScrollChatUp),
                KeyCode::PageDown | KeyCode::Down => self.events.send(AppEvent::ScrollChatDown),
                _ => {}
            }
            return Ok(());
        }

        // Handle normal navigation
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Up | KeyCode::Char('k') => self.events.send(AppEvent::Decrement),
            KeyCode::Down | KeyCode::Char('j') => self.events.send(AppEvent::Increment),
            KeyCode::Enter | KeyCode::Char(' ') => self.events.send(AppEvent::Select),
            KeyCode::Backspace | KeyCode::Char('h') => self.events.send(AppEvent::Back),
            KeyCode::Char('r') => self.events.send(AppEvent::RefreshModules),
            KeyCode::Char('c') => self.events.send(AppEvent::EnterChat),
            KeyCode::Char('1') => {
                // Quick command for gate close
                self.events.send(AppEvent::SendCommand("com_input".to_string(), "close_gate".to_string()));
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn next_module(&mut self) {
        let module_count = self.module_manager.get_modules().len();
        if module_count > 0 {
            self.selected_module = (self.selected_module + 1) % module_count;
        }
    }

    pub fn prev_module(&mut self) {
        let module_count = self.module_manager.get_modules().len();
        if module_count > 0 {
            self.selected_module = if self.selected_module == 0 {
                module_count - 1
            } else {
                self.selected_module - 1
            };
        }
    }

    pub fn select_current_module(&mut self) {
        if !self.module_manager.get_modules().is_empty() {
            let module = &self.module_manager.get_modules()[self.selected_module];
            // If it's an LLM module, go directly to chat interface
            if module.config.module_type == "llm" && self.llm_engine.is_some() {
                self.mode = AppMode::LlmChat(self.selected_module);
            } else {
                self.mode = AppMode::ModuleDetail(self.selected_module);
            }
        }
    }

    pub fn enter_llm_chat(&mut self) {
        // Find first LLM module and enter chat mode
        for (i, module) in self.module_manager.get_modules().iter().enumerate() {
            if module.config.module_type == "llm" && self.llm_engine.is_some() {
                self.mode = AppMode::LlmChat(i);
                self.update_available_links();
                return;
            }
        }
    }

    pub fn back_to_overview(&mut self) {
        self.mode = AppMode::Overview;
        self.chat_input.clear(); // Clear chat input when leaving chat
    }

    pub async fn refresh_modules(&mut self) {
        if let Err(e) = self.module_manager.discover_modules() {
            eprintln!("Failed to refresh modules: {}", e);
        }

        // Recreate LLM engine if needed
        if let Ok(new_engine) = Self::create_llm_engine_if_available(
            &self.module_manager,
            &self.database,
            self.message_bus.get_sender(),
        ).await {
            self.llm_engine = new_engine;
        }

        // Reset selection if it's out of bounds
        let module_count = self.module_manager.get_modules().len();
        if self.selected_module >= module_count && module_count > 0 {
            self.selected_module = 0;
        }
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

    pub fn handle_chat_input(&mut self, ch: char) {
        self.chat_input.push(ch);
    }

    pub fn chat_backspace(&mut self) {
        self.chat_input.pop();
    }

    pub async fn submit_chat_message(&mut self) {
        if self.chat_input.trim().is_empty() {
            return;
        }

        // Move the llm_engine check and clear links before the borrow
        let has_llm_engine = self.llm_engine.is_some();
        if !has_llm_engine {
            return;
        }

        let query = self.chat_input.clone();
        self.chat_input.clear();
        self.clear_document_links();

        // Get current module for context
        let module_name = if let AppMode::LlmChat(idx) = self.mode {
            self.module_manager.get_modules()
                .get(idx)
                .map(|m| m.config.name.clone())
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            "llm".to_string()
        };

        let recent_messages = self.get_recent_bus_messages();

        // Now borrow llm_engine immutably for the async call
        if let Some(llm_engine) = &self.llm_engine {
            match llm_engine.process_user_query(
                query,
                module_name,
                &self.module_manager,
                recent_messages,
            ).await {
                Ok(_response) => {
                    // After getting response, extract file links from chat history
                    self.update_available_links();
                }
                Err(e) => {
                    eprintln!("LLM processing error: {}", e);
                }
            }
        }
    }

    fn get_recent_bus_messages(&self) -> Vec<BusMessage> {
        // For now return empty - in full implementation would query database
        // for recent bus messages to provide context to LLM
        Vec::new()
    }

    pub fn get_current_module(&self) -> Option<&Module> {
        self.module_manager.get_modules().get(self.selected_module)
    }

    pub fn get_modules(&self) -> &[Module] {
        self.module_manager.get_modules()
    }

    pub fn get_llm_engine(&self) -> Option<&LlmEngine> {
        self.llm_engine.as_ref()
    }

    pub fn get_chat_input(&self) -> &str {
        &self.chat_input
    }
}

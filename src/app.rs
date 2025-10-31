use crate::event::{AppEvent, Event, EventHandler};
use crate::module::{Module, ModuleManager};
use crate::bus::{MessageBus, BusMessage, BusReceiver};
use crate::database::{Database, ChatMessage};
use crate::llm::{LlmEngine, ChatManager, create_llm_strategy, create_llm_engine_if_available};
use crate::ui::document_viewer::{DocumentViewer, DocumentContent, DocumentManager};
use crate::ui::external_viewer::ExternalViewer;
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

    pub document_manager: DocumentManager,
    pub chat_manager: ChatManager,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> Result<Self> {
        let mut module_manager = ModuleManager::new();
        let (message_bus, bus_receiver) = MessageBus::new();
        let database = Database::new_implied_all_schemas()?;
        let chat_manager = ChatManager::new();

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
        let llm_engine = create_llm_engine_if_available(
            &module_manager,
            &database,
            message_bus.get_sender(),
        ).await?;

        Ok(Self {
            running: true,
            mode: AppMode::Overview,
            module_manager,
            message_bus,
            bus_receiver,
            database,
            llm_engine,
            chat_input: String::new(),
            events: EventHandler::new(),
            document_manager: DocumentManager::new()?,
            chat_manager,
        })
    }





    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let mut needs_redraw = true;

        while self.running {
            if needs_redraw {

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
                })?;

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
                                    AppEvent::Increment => { self.module_manager.next_module(); needs_redraw = true; }
                                    AppEvent::Decrement => { self.module_manager.prev_module(); needs_redraw = true; }
                                    AppEvent::Quit => self.quit(),
                                    AppEvent::Select => {
                                        match self.module_manager.select_current_module() {
                                            Some(module) => {
                                                if module.config.module_type == "llm" && self.llm_engine.is_some() {
                                                    self.mode = AppMode::LlmChat(self.module_manager.selected_module);
                                                } else {
                                                    self.mode = AppMode::ModuleDetail(self.module_manager.selected_module);
                                                }
                                                needs_redraw = true;
                                            }
                                            None => {
                                                self.mode = AppMode::Overview;
                                                needs_redraw = true;
                                            }
                                        }
                                    }
                                    AppEvent::Back => { self.back_to_overview(); needs_redraw = true; }
                                    AppEvent::RefreshModules => {
                                        self.module_manager.refresh_modules().await;
                                        if let Ok(new_engine) = create_llm_engine_if_available(
                                            &self.module_manager,
                                            &self.database,
                                            self.message_bus.get_sender(),
                                        ).await {
                                            self.llm_engine = new_engine;
                                        }
                                        needs_redraw = true;
                                    }
                                    AppEvent::SendCommand(topic, command) => self.send_command(topic, command),
                                    AppEvent::EnterChat => { self.enter_llm_chat(); needs_redraw = true; }
                                    AppEvent::ChatInput(ch) => { self.handle_chat_input(ch); needs_redraw = true; }
                                    AppEvent::ChatSubmit => { self.submit_chat_message().await; needs_redraw = true; }
                                    AppEvent::ChatBackspace => { self.chat_backspace(); needs_redraw = true; }
                                    AppEvent::ScrollChatUp => { self.chat_manager.scroll_chat_up(); needs_redraw = true; }
                                    AppEvent::ScrollChatDown => {
                                        if let Some(engine) = &self.llm_engine {
                                            self.chat_manager.scroll_chat_down(engine);
                                        }
                                        needs_redraw = true;
                                    }
                                    AppEvent::OpenDocument(file_path) => { self.document_manager.open_document(file_path); needs_redraw = true; }
                                    AppEvent::CloseDocument => { self.document_manager.close_document(); self.chat_manager.current_link_index = None; needs_redraw = true; }
                                    AppEvent::ScrollDocumentUp => { self.document_manager.scroll_document_up(); needs_redraw = true; }
                                    AppEvent::ScrollDocumentDown => { self.document_manager.scroll_document_down(); needs_redraw = true; }
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

        // Handle chat input mode
        if matches!(self.mode, AppMode::LlmChat(_)) {
            match key_event.code {
                KeyCode::Esc => self.events.send(AppEvent::Back),
                KeyCode::Tab => self.chat_manager.cycle_document_links(),
                KeyCode::BackTab => self.chat_manager.cycle_document_links_backward(),
                KeyCode::Enter => {
                    if let Some(index) = self.chat_manager.current_link_index {
                        if let Some(file_path) = self.chat_manager.available_links.get(index) {
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


    pub fn enter_llm_chat(&mut self) {
        // Find first LLM module and enter chat mode
        for (i, module) in self.module_manager.get_modules().iter().enumerate() {
            if module.config.module_type == "llm" {
                if let Some(engine) = &self.llm_engine {
                    self.mode = AppMode::LlmChat(i);
                    self.chat_manager.update_available_links(engine);
                    return;
                }
            }
        }
    }

    pub fn back_to_overview(&mut self) {
        self.mode = AppMode::Overview;
        self.chat_input.clear(); // Clear chat input when leaving chat
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
        self.chat_manager.clear_document_links();

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

        if let Some(llm_engine) = &self.llm_engine {
            match llm_engine.process_user_query(
                query,
                module_name,
                &self.module_manager,
                recent_messages,
            ).await {
                Ok(_response) => {
                    // After getting response, extract file links from chat history
                    self.chat_manager.update_available_links(llm_engine);
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

    pub fn get_llm_engine(&self) -> Option<&LlmEngine> {
        self.llm_engine.as_ref()
    }

    pub fn get_chat_input(&self) -> &str {
        &self.chat_input
    }
}

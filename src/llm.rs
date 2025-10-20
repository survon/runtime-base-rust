/// ./src/llm.rs

use crate::database::{Database, ChatMessage};
use crate::module::ModuleManager;
use crate::bus::{BusMessage, BusSender};
use tokio::time::{timeout, Duration};
use color_eyre::Result;
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};
use llama_cpp::standard_sampler::StandardSampler;
use std::collections::HashMap;
use std::path::PathBuf;
use gag::Gag;

/// Embedded LLM using llama_cpp with your existing Phi-3 model
pub struct EmbeddedLlmStrategy {
    model: LlamaModel,
    model_info: String,
}

impl std::fmt::Debug for EmbeddedLlmStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddedLlmStrategy")
            .field("model", &"<LlamaModel>")
            .field("model_info", &self.model_info)
            .finish()
    }
}

impl EmbeddedLlmStrategy {
    pub async fn new() -> Result<Self> {
        let model_path = PathBuf::from("bundled/models/phi3-mini.gguf");

        if !model_path.exists() {
            return Err(color_eyre::eyre::eyre!(
                "Model not found at {}. Please download it:\n\
                mkdir -p bundled/models\n\
                curl -L \"https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf\" -o bundled/models/phi3-mini.gguf",
                model_path.display()
            ));
        }

        println!("Model file size: {} bytes", std::fs::metadata(&model_path)?.len());

        let params = LlamaParams::default();

        let model = LlamaModel::load_from_file(&model_path, params)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to load model: {}", e))?;

        let model_info = "Hermes-2-Pro-Mistral-7B (via llama_cpp)".to_string();

        Ok(Self {
            model,
            model_info,
        })
    }

    pub async fn process_query(&self, query: &str, context: &LlmContext) -> Result<String> {
        let print_gag = Gag::stdout().map_err(|e| color_eyre::eyre::eyre!("Gag error: {}", e))?;

        // Create a session with explicit, conservative parameters
        let session_params = SessionParams {
            n_ctx: 512,        // Smaller context to avoid memory issues
            n_batch: 32,       // Smaller batch size
            n_threads: 4,       // Conservative thread count
            n_threads_batch: 2,
            ..Default::default()
        };

        let mut session = self.model.create_session(session_params)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create session: {}", e))?;

        // Build the prompt with context
        let prompt = self.build_prompt(query, context);

        // Feed the prompt to the model
        session.advance_context(&prompt)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to advance context: {}", e))?;

        // Generate response
        let sampler = StandardSampler::default();
        let max_tokens = 256; // Keep reasonable for Pi performance

        let mut response = String::new();

        // Get the completion handle and check for errors
        let completion_result = session.start_completing_with(sampler, max_tokens); // No timeout (source: https://docs.rs/llama_cpp/0.3.1/llama_cpp/context/struct.LlamaContext.html#method.start_completing_with)
        let mut completion_handle = match completion_result {
            Ok(handle) => handle.into_strings(),
            Err(e) => return Err(color_eyre::eyre::eyre!("Failed to start completion: {}", e)),
        };

        // Collect tokens
        let mut token_count = 0;
        let start_time = std::time::Instant::now();
        while let Some(token) = completion_handle.next() { // token is String (source: https://docs.rs/llama_cpp/0.3.1/llama_cpp/completion/struct.CompletionHandle.html#method.into_strings)
            if start_time.elapsed() > Duration::from_secs(30) {
                break; // Timeout after 30 seconds
            }
            response.push_str(&token);
            token_count += 1;

            // Stop if we get a good response length or hit limits
            if token_count > 100 && (response.contains('\n') || response.len() > 1000) {
                break;
            }

            if token_count >= max_tokens {
                break;
            }
        }

        // Clean up the response
        let cleaned_response = self.clean_response(&response);

        drop(print_gag);
        Ok(cleaned_response)
    }

    // fn build_prompt(&self, query: &str, context: &LlmContext) -> String {
    //     format!("Q: {}\nA:", query)  // Much simpler prompt
    // }
    fn build_prompt(&self, query: &str, context: &LlmContext) -> String {
        let mut prompt = String::new();

        // System message
        prompt.push_str("<|system|>\nYou are Survon, a helpful homestead assistant. Give direct, concise answers.\n<|user|>\n");

        // Add context if available
        if context.has_survival_knowledge() {
            prompt.push_str("I have access to survival knowledge. ");
        }

        prompt.push_str(query);
        prompt.push_str("\n<|assistant|>\n");

        prompt
    }

    fn clean_response(&self, response: &str) -> String {
        let cleaned = response
            .chars()
            .filter(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace() || c.is_ascii_punctuation()) // Strip tokens/control chars (source: https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_alphabetic)
            .collect::<String>()
            .trim()
            .replace("<|end|>", "")
            .replace("<|assistant|>", "")
            .replace("<|user|>", "")
            .replace("<|system|>", "")
            .lines()
            .take(10)  // Limit to reasonable length
            .filter(|line| !line.trim().is_empty()) // Skip empty (source: https://doc.rust-lang.org/std/primitive.str.html#method.trim)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
            .chars()
            .take(500) // Hard cap length for DB (adjust if your schema has limit)
            .collect::<String>();

        if cleaned.is_empty() || cleaned.len() < 10 {
            "I understand your question, but I'm having trouble generating a response right now. Could you try rephrasing your question?".to_string()
        } else {
            cleaned
        }
    }

    pub fn get_model_info(&self) -> &str {
        &self.model_info
    }
}

/// Mock LLM implementation for fallback when model isn't available
#[derive(Debug)]
pub struct MockLlmStrategy {
    response_patterns: HashMap<String, Vec<String>>,
}

impl MockLlmStrategy {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Initialize response patterns for your homestead/survival use case
        patterns.insert("survival".to_string(), vec![
            "For survival situations, remember the rule of threes: 3 minutes without air, 3 hours without shelter in harsh conditions, 3 days without water, 3 weeks without food. Focus on immediate threats first.".to_string(),
        ]);

        patterns.insert("shelter".to_string(), vec![
            "For emergency shelter: Find natural windbreaks, insulate from ground, prioritize staying dry. Lean-to structures work well with available materials.".to_string(),
        ]);

        patterns.insert("fire".to_string(), vec![
            "Fire basics: Tinder (fine dry material), kindling (pencil-thick), fuel wood (arm-thick and larger). Build a platform, create airflow, start small and build up.".to_string(),
        ]);

        patterns.insert("water".to_string(), vec![
            "Water procurement: Collect from moving sources when possible, purify by boiling 1+ minutes, use purification tablets, or improvised filtration.".to_string(),
        ]);

        patterns.insert("gate".to_string(), vec![
            "Gate systems respond to 'open_gate' and 'close_gate' commands via the message bus. Check power supply and sensor alignment for troubleshooting.".to_string(),
        ]);

        patterns.insert("monitor".to_string(), vec![
            "Monitoring systems track environmental conditions and system status. Set appropriate alert thresholds and check sensor calibration regularly.".to_string(),
        ]);

        Self { response_patterns: patterns }
    }

    pub async fn process_query(&self, query: &str, context: &LlmContext) -> Result<String> {
        let query_lower = query.to_lowercase();

        // Check for pattern matches
        for (key, responses) in &self.response_patterns {
            if query_lower.contains(key) {
                let response = &responses[0];
                return Ok(format!("{} (Download hermes model for full AI capabilities)", response));
            }
        }

        // Context-aware responses
        if query_lower.contains("help") || query_lower.contains("what can you do") {
            let mut response = String::from("I'm Survon's survival and homestead assistant! I can help with:\n• Survival and wilderness skills\n• Homestead management\n• System monitoring\n• Gate and device control");

            if !context.knowledge_modules.is_empty() {
                response.push_str(&format!("\n\nActive knowledge modules: {}", context.knowledge_modules.join(", ")));
            }

            response.push_str("\n\nDownload the hermes model for full AI capabilities!");
            return Ok(response);
        }

        // Default response
        Ok(format!("I understand you're asking about '{}'. Download the hermes model for full AI responses, or ask about specific topics like 'shelter', 'fire', 'water', 'gate control', or 'monitoring'.", query))
    }

    pub fn get_model_info(&self) -> &str {
        "Mock LLM (Download hermes for AI)"
    }
}

/// Knowledge search strategy that queries the FTS5 database
#[derive(Debug)]
pub struct KnowledgeSearchStrategy {
    database: Database,
}

impl KnowledgeSearchStrategy {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn process_query(&self, query: &str, context: &LlmContext) -> Result<String> {
        let knowledge_results = self.database.search_knowledge(query, &[], 5)
            .map_err(|e| color_eyre::eyre::eyre!("Knowledge search failed: {}", e))?;

        if knowledge_results.is_empty() {
            return Ok(format!(
                "No specific knowledge found for '{}'. Try different keywords.",
                query
            ));
        }

        let mut response = format!("Found {} result{}:\n\n",
                                   knowledge_results.len(),
                                   if knowledge_results.len() == 1 { "" } else { "s" }
        );

        for (i, chunk) in knowledge_results.iter().enumerate() {
            let body_preview = if chunk.body.len() > 300 {
                format!("{}...", chunk.body.chars().take(300).collect::<String>())
            } else {
                chunk.body.clone()
            };

            // Extract page number from metadata
            let page_info = if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&chunk.metadata) {
                if let Some(page_num) = metadata.get("page_number").and_then(|v| v.as_u64()) {
                    format!("#page={}", page_num)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            response.push_str(&format!(
                "{}. **{}** (from {}{})\n{}\n\n",
                i + 1,
                chunk.title,
                chunk.source_file,
                page_info,  // Add page fragment to the link
                body_preview
            ));
        }

        Ok(response)
    }

    pub fn get_model_info(&self) -> &str {
        "Knowledge Search (FTS5)"
    }
}

/// Strategy pattern for different LLM implementations
#[derive(Debug)]
pub enum LlmStrategy {
    Embedded(EmbeddedLlmStrategy),
    Mock(MockLlmStrategy),
    Knowledge(KnowledgeSearchStrategy),
}

impl LlmStrategy {
    pub async fn process_query(&self, query: &str, context: &LlmContext) -> Result<String> {
        match self {
            LlmStrategy::Embedded(strategy) => strategy.process_query(query, context).await,
            LlmStrategy::Mock(strategy) => strategy.process_query(query, context).await,
            LlmStrategy::Knowledge(strategy) => strategy.process_query(query, context).await,
        }
    }

    pub fn get_model_info(&self) -> &str {
        match self {
            LlmStrategy::Embedded(strategy) => strategy.get_model_info(),
            LlmStrategy::Mock(strategy) => strategy.get_model_info(),
            LlmStrategy::Knowledge(strategy) => strategy.get_model_info(),
        }
    }
}

/// Context provided to LLM for processing queries
#[derive(Debug)]
pub struct LlmContext {
    pub knowledge_modules: Vec<String>,
    pub recent_messages: Vec<BusMessage>,
    pub module_states: std::collections::HashMap<String, String>,
}

impl LlmContext {
    pub fn new() -> Self {
        Self {
            knowledge_modules: Vec::new(),
            recent_messages: Vec::new(),
            module_states: std::collections::HashMap::new(),
        }
    }

    pub fn has_knowledge_about(&self, topic: &str) -> bool {
        self.knowledge_modules.iter()
            .any(|module| module.to_lowercase().contains(&topic.to_lowercase()))
    }

    pub fn has_survival_knowledge(&self) -> bool {
        self.knowledge_modules.iter()
            .any(|module| module.to_lowercase().contains("survival"))
    }

    pub fn get_monitoring_threshold(&self) -> Option<f64> {
        Some(50.0)
    }

    pub fn add_knowledge_module(&mut self, name: String) {
        self.knowledge_modules.push(name);
    }

    pub fn update_recent_messages(&mut self, messages: Vec<BusMessage>) {
        self.recent_messages = messages;
    }
}

/// Main LLM engine that coordinates between strategy and context
#[derive(Debug)]
pub struct LlmEngine {
    strategy: LlmStrategy,
    database: Database,
    bus_sender: BusSender,
    session_id: String,
}

impl LlmEngine {
    pub fn new(
        strategy: LlmStrategy,
        database: Database,
        bus_sender: BusSender,
        session_id: String,
    ) -> Self {
        Self {
            strategy,
            database,
            bus_sender,
            session_id,
        }
    }

    pub async fn process_user_query(
        &self,
        query: String,
        module_name: String,
        module_manager: &ModuleManager,
        recent_messages: Vec<BusMessage>,
    ) -> Result<String> {
        // Build context from available modules and recent activity
        let mut context = LlmContext::new();

        // Add knowledge modules to context
        for knowledge_module in module_manager.get_knowledge_modules() {
            context.add_knowledge_module(knowledge_module.config.name.clone());
        }

        // Add recent bus messages for context
        context.update_recent_messages(recent_messages);

        // Store user message in database
        let user_message = ChatMessage::new_user(
            self.session_id.clone(),
            query.clone(),
            module_name.clone(),
        );
        self.database.insert_chat_message(user_message)?;

        // Process query through LLM strategy
        let result = self.strategy.process_query(&query, &context).await;
        let response = result?;

        // Process query through LLM strategy
        // let response = self.strategy.process_query(&query, &context).await?;

        // Store assistant response in database
        let assistant_message = ChatMessage::new_assistant(
            self.session_id.clone(),
            response.clone(),
            module_name.clone(),
        );
        match self.database.insert_chat_message(assistant_message) {
            Ok(_) => {}, // Success, continue
            Err(e) => println!("Insert failed (continuing): {:?}", e), // Log but don't bail (source: https://doc.rust-lang.org/std/io/struct.Error.html)
        }

        // Send response through message bus
        let bus_message = BusMessage::new(
            "llm_response".to_string(),
            response.clone(),
            "llm_engine".to_string(),
        );
        self.bus_sender.send(bus_message)?;

        Ok(response)
    }

    pub fn get_chat_history(&self, limit: usize) -> Result<Vec<ChatMessage>> {
        Ok(self.database.get_chat_history(&self.session_id, limit)?)
    }

    pub fn get_model_info(&self) -> &str {
        self.strategy.get_model_info()
    }
}

/// Factory for creating LLM strategies
/// Factory for creating LLM strategies
pub async fn create_llm_strategy(model_name: &str) -> (LlmStrategy, Option<Database>) {
    match model_name {
        "knowledge" => {
            match Database::new_implied_all_schemas() {
                Ok(database) => {
                    let strategy = KnowledgeSearchStrategy::new(database);
                    (LlmStrategy::Knowledge(strategy), None)
                }
                Err(e) => {
                    eprintln!("Failed to create databases: {}", e);
                    (LlmStrategy::Mock(MockLlmStrategy::new()), None)
                }
            }
        }
        "embedded" => {
            match EmbeddedLlmStrategy::new().await {
                Ok(strategy) => (LlmStrategy::Embedded(strategy), None),
                Err(e) => {
                    eprintln!("Failed to load hermes model: {}", e);
                    eprintln!("Falling back to mock LLM.");
                    (LlmStrategy::Mock(MockLlmStrategy::new()), None)
                }
            }
        }
        _ => (LlmStrategy::Mock(MockLlmStrategy::new()), None),
    }
}

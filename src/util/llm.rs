// src/util/llm.rs
//! LLM Utility - Provides language model processing as a core utility

use color_eyre::Result;
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};
use llama_cpp::standard_sampler::StandardSampler;
use std::path::PathBuf;
use gag::Gag;
use tokio::time::Duration;

use crate::util::database::{Database};
use crate::modules::llm::database::{LlmDatabase, ChatMessage, KnowledgeChunk};

use crate::{log_error, log_debug};

/// Main LLM service that provides language model functionality
#[derive(Clone)]
pub struct LlmService {
    database: Database,
    strategy: LlmStrategyType,
}

#[derive(Clone, Debug)]
pub enum LlmStrategyType {
    Embedded,
    Knowledge,
    Mock,
}

impl LlmService {
    /// Create a new LLM service with the specified strategy
    pub fn new(database: Database, strategy: LlmStrategyType) -> Self {
        Self { database, strategy }
    }

    /// Create from model name string
    pub async fn from_model_name(database: Database, model_name: &str) -> Result<Self> {
        let strategy = match model_name {
            "embedded" => {
                // Check if model file exists
                let model_path = PathBuf::from("bundled/models/phi3-mini.gguf");
                if model_path.exists() {
                    LlmStrategyType::Embedded
                } else {
                    log_error!("Embedded model not found, falling back to knowledge search");
                    LlmStrategyType::Knowledge
                }
            }
            "knowledge" => LlmStrategyType::Knowledge,
            _ => LlmStrategyType::Mock,
        };

        Ok(Self::new(database, strategy))
    }

    /// Process a user query and return a response
    pub async fn process_query(
        &self,
        session_id: &str,
        module_name: &str,
        query: &str,
        knowledge_module_names: &[String],
    ) -> Result<String> {
        // Store user message
        let user_message = ChatMessage::new_user(
            session_id.to_string(),
            query.to_string(),
            module_name.to_string(),
        );
        self.database.insert_chat_message(user_message)?;

        // Build context by searching knowledge base
        let knowledge_context = self.search_knowledge(query, knowledge_module_names)?;

        // Process query based on strategy
        let response = match self.strategy {
            LlmStrategyType::Embedded => {
                // Embedded model can synthesize/summarize the knowledge
                self.process_with_embedded(query, &knowledge_context).await?
            }
            LlmStrategyType::Knowledge => {
                // Knowledge-only mode: provide detailed extracts
                self.process_with_knowledge_synthesis(query, &knowledge_context)?
            }
            LlmStrategyType::Mock => {
                self.process_with_mock(query, &knowledge_context)?
            }
        };

        // Store assistant response
        let assistant_message = ChatMessage::new_assistant(
            session_id.to_string(),
            response.clone(),
            module_name.to_string(),
        );

        // Use unwrap_or for error handling to continue even if insert fails
        let _ = self.database.insert_chat_message(assistant_message);

        Ok(response)
    }

    /// Search knowledge base for relevant information
    fn search_knowledge(
        &self,
        query: &str,
        knowledge_module_names: &[String],
    ) -> Result<Vec<KnowledgeChunk>> {
        // Extract domains from knowledge module names if available
        let domains: Vec<String> = knowledge_module_names
            .iter()
            .filter_map(|name| {
                // Try to extract domain from module name
                // e.g., "survival_knowledge" -> "survival"
                if name.contains("_knowledge") {
                    Some(name.replace("_knowledge", ""))
                } else {
                    None
                }
            })
            .collect();

        // Search with more results for better synthesis
        let results = self.database.search_knowledge(query, &domains, 10)?;

        log_debug!("Knowledge search for '{}' found {} results", query, results.len());

        // Debug: print what we found
        if !results.is_empty() {
            log_debug!("Top results:");
            for (i, chunk) in results.iter().take(3).enumerate() {
                log_debug!("  {}. {} ({})", i+1, chunk.title, chunk.source_file);
                log_debug!("     First 100 chars: {}", &chunk.body[..chunk.body.len().min(100)]);
            }
        }

        Ok(results)
    }

    /// Process query using embedded LLM model
    async fn process_with_embedded(
        &self,
        query: &str,
        knowledge_context: &[KnowledgeChunk],
    ) -> Result<String> {
        let model_path = PathBuf::from("bundled/models/phi3-mini.gguf");

        let print_gag = Gag::stdout()
            .map_err(|e| color_eyre::eyre::eyre!("Gag error: {}", e))?;

        let params = LlamaParams::default();
        let model = LlamaModel::load_from_file(&model_path, params)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to load model: {}", e))?;

        // Increase context size for better synthesis
        let session_params = SessionParams {
            n_ctx: 2048,  // Increased from 512 for more context
            n_batch: 64,   // Increased batch size
            n_threads: 4,
            n_threads_batch: 2,
            ..Default::default()
        };

        let mut session = model.create_session(session_params)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create session: {}", e))?;

        // Build prompt with knowledge context
        let prompt = self.build_embedded_prompt(query, knowledge_context);

        session.advance_context(&prompt)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to advance context: {}", e))?;

        let sampler = StandardSampler::default();
        let max_tokens = 512;  // Increased from 256 for longer responses

        let mut response = String::new();
        let completion_result = session.start_completing_with(sampler, max_tokens);
        let mut completion_handle = match completion_result {
            Ok(handle) => handle.into_strings(),
            Err(e) => return Err(color_eyre::eyre::eyre!("Failed to start completion: {}", e)),
        };

        let mut token_count = 0;
        let start_time = std::time::Instant::now();

        while let Some(token) = completion_handle.next() {
            if start_time.elapsed() > Duration::from_secs(60) {  // Increased timeout
                break;
            }
            response.push_str(&token);
            token_count += 1;

            // Allow longer responses for synthesis
            if token_count >= max_tokens {
                break;
            }

            // Stop on natural ending
            if response.contains("<|end|>") || response.contains("<|user|>") {
                break;
            }
        }

        drop(print_gag);

        let cleaned = self.clean_llm_response(&response);

        // If we got a good response and have knowledge context, append source citations
        if !knowledge_context.is_empty() && cleaned.len() > 50 {
            let mut with_sources = cleaned;
            with_sources.push_str("\n\n---\n📚 **Sources**: ");

            // Collect unique sources with page info
            let mut sources = Vec::new();
            let mut seen_sources = std::collections::HashSet::new();

            for chunk in knowledge_context {
                let filename = std::path::Path::new(&chunk.source_file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&chunk.source_file);

                // Parse metadata for page number
                let page_info = if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&chunk.metadata) {
                    if let Some(page_num) = metadata.get("page_number").and_then(|v| v.as_u64()) {
                        format!("{}#page={}", chunk.source_file, page_num)
                    } else {
                        chunk.source_file.clone()
                    }
                } else {
                    chunk.source_file.clone()
                };

                if !seen_sources.contains(filename) {
                    sources.push(page_info);
                    seen_sources.insert(filename.to_string());
                }
            }

            // Format sources as clickable links
            for (i, path) in sources.iter().enumerate() {
                if i > 0 {
                    with_sources.push_str(", ");
                }
                with_sources.push_str(&format!("(from {})", path));
            }

            with_sources.push_str("\n\n💡 *Press Tab to cycle through source links, Enter to open. Ask follow-up questions anytime!*");

            Ok(with_sources)
        } else {
            Ok(cleaned)
        }
    }

    /// Process query using knowledge search only - with better synthesis
    fn process_with_knowledge_synthesis(
        &self,
        query: &str,
        knowledge_context: &[KnowledgeChunk],
    ) -> Result<String> {
        if knowledge_context.is_empty() {
            return Ok(self.generate_no_knowledge_response(query));
        }

        // Synthesize knowledge into a natural conversational response
        let synthesis = self.synthesize_knowledge_into_answer(query, knowledge_context);

        // Add source citations at the end
        let mut response = synthesis;
        response.push_str("\n\n---\n");
        response.push_str("📚 **Sources**: ");

        // Collect unique sources with page info
        let mut sources = Vec::new();
        let mut seen_sources = std::collections::HashSet::new();

        for chunk in knowledge_context {
            let filename = std::path::Path::new(&chunk.source_file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&chunk.source_file);

            // Parse metadata for page number
            let page_info = if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&chunk.metadata) {
                if let Some(page_num) = metadata.get("page_number").and_then(|v| v.as_u64()) {
                    format!("{}#page={}", chunk.source_file, page_num)
                } else {
                    chunk.source_file.clone()
                }
            } else {
                chunk.source_file.clone()
            };

            if !seen_sources.contains(filename) {
                sources.push((filename.to_string(), page_info));
                seen_sources.insert(filename.to_string());
            }
        }

        // Format sources as links
        for (i, (_name, path)) in sources.iter().enumerate() {
            if i > 0 {
                response.push_str(", ");
            }
            response.push_str(&format!("(from {})", path));
        }

        response.push_str("\n\n💡 *Press Tab to cycle through source links, Enter to open. Ask follow-up questions to learn more!*");

        Ok(response)
    }

    /// Synthesize knowledge chunks into a natural, conversational answer
    fn synthesize_knowledge_into_answer(&self, query: &str, chunks: &[KnowledgeChunk]) -> String {
        // Combine all relevant content with better filtering
        let mut all_content = String::new();
        let query_keywords = self.extract_keywords(query);

        log_debug!("Query keywords: {:?}", query_keywords);

        // Score and filter chunks by relevance to query
        let mut scored_chunks: Vec<(f32, &KnowledgeChunk)> = chunks.iter()
            .map(|chunk| {
                let score = self.score_chunk_relevance(chunk, &query_keywords);
                (score, chunk)
            })
            .collect();

        // Sort by relevance score
        scored_chunks.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        log_debug!("Chunk relevance scores:");
        for (score, chunk) in scored_chunks.iter().take(5) {
            log_debug!("  {:.2}: {} - {}", score, chunk.title, &chunk.body[..chunk.body.len().min(80)]);
        }

        // Take top 5 most relevant chunks
        for (_, chunk) in scored_chunks.iter().take(5) {
            all_content.push_str(&chunk.body);
            all_content.push_str("\n\n");
        }

        if all_content.trim().is_empty() {
            return self.generate_no_knowledge_response(query);
        }

        // Identify key topics from the query
        let query_lower = query.to_lowercase();
        let is_how_to = query_lower.contains("how") || query_lower.contains("build")
            || query_lower.contains("make") || query_lower.contains("create");
        let is_what = query_lower.contains("what") || query_lower.contains("explain");
        let is_why = query_lower.contains("why");

        // Extract key information based on query type
        let response = if is_how_to {
            self.generate_how_to_response(&all_content, query)
        } else if is_what {
            self.generate_what_response(&all_content, query)
        } else if is_why {
            self.generate_why_response(&all_content, query)
        } else {
            self.generate_general_response(&all_content, query)
        };

        response
    }

    /// Extract keywords from query for relevance scoring
    fn extract_keywords(&self, query: &str) -> Vec<String> {
        query.to_lowercase()
            .split_whitespace()
            .filter(|word| {
                // Filter out common question words
                !matches!(*word, "how" | "do" | "i" | "a" | "the" | "is" | "to" | "can" | "you" | "what" | "when" | "where" | "why")
            })
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| s.len() > 2)
            .collect()
    }

    /// Score a chunk's relevance to query keywords
    fn score_chunk_relevance(&self, chunk: &KnowledgeChunk, keywords: &[String]) -> f32 {
        let content_lower = format!("{} {}", chunk.title, chunk.body).to_lowercase();
        let mut score = 0.0;

        for keyword in keywords {
            let keyword_count = content_lower.matches(keyword.as_str()).count() as f32;

            // Title matches worth more
            if chunk.title.to_lowercase().contains(keyword) {
                score += 5.0;
            }

            // Body matches
            score += keyword_count;

            // Bonus for keyword in first 200 chars (likely more relevant)
            if content_lower[..content_lower.len().min(200)].contains(keyword) {
                score += 2.0;
            }
        }

        // Penalize very short chunks (likely not useful)
        if chunk.body.len() < 100 {
            score *= 0.5;
        }

        score
    }

    fn generate_how_to_response(&self, content: &str, query: &str) -> String {
        let mut response = String::new();

        // Try to identify the main topic
        let topic = self.extract_topic(query);

        // Opening statement
        response.push_str(&format!("Here's how to {}:\n\n", topic.to_lowercase()));

        // Look for numbered steps, bullet points, or sequential information
        let steps = self.extract_steps_from_content(content);

        if !steps.is_empty() {
            log_debug!("Found {} steps", steps.len());
            for (i, step) in steps.iter().enumerate() {
                response.push_str(&format!("**{}. {}**\n\n", i + 1, step));
            }
        } else {
            log_debug!("No clear steps found, using paragraph format");
            // If no clear steps, provide informative paragraphs
            let paragraphs = self.extract_relevant_paragraphs(content, 4);

            if paragraphs.is_empty() {
                // Fallback: split content into chunks
                let words: Vec<&str> = content.split_whitespace().collect();
                let chunk_size = 50;
                for (i, chunk) in words.chunks(chunk_size).take(4).enumerate() {
                    let para = chunk.join(" ");
                    if para.len() > 50 {
                        response.push_str(&format!("**Step {}**: {}\n\n", i + 1, para));
                    }
                }
            } else {
                for (i, para) in paragraphs.iter().enumerate() {
                    response.push_str(&format!("**{}**: {}\n\n", i + 1, para));
                }
            }
        }

        // Add a practical tip if available
        if let Some(tip) = self.extract_tip_from_content(content) {
            response.push_str(&format!("💡 **Tip**: {}\n", tip));
        }

        response
    }

    fn generate_what_response(&self, content: &str, query: &str) -> String {
        let mut response = String::new();
        let topic = self.extract_topic(query);

        // Definition/explanation style
        response.push_str(&format!("**About {}:**\n\n", topic));

        // Extract relevant explanatory content
        let paragraphs = self.extract_relevant_paragraphs(content, 4);
        for para in paragraphs {
            response.push_str(&para);
            response.push_str("\n\n");
        }

        response
    }

    fn generate_why_response(&self, content: &str, query: &str) -> String {
        let mut response = String::new();
        let topic = self.extract_topic(query);

        response.push_str(&format!("**Why {}:**\n\n", topic.to_lowercase()));

        // Look for explanatory content
        let paragraphs = self.extract_relevant_paragraphs(content, 3);
        for para in paragraphs {
            response.push_str(&para);
            response.push_str("\n\n");
        }

        response
    }

    fn generate_general_response(&self, content: &str, query: &str) -> String {
        let mut response = String::new();
        let topic = self.extract_topic(query);

        response.push_str(&format!("**Regarding {}:**\n\n", topic));

        let paragraphs = self.extract_relevant_paragraphs(content, 4);
        for para in paragraphs {
            response.push_str(&para);
            response.push_str("\n\n");
        }

        response
    }

    fn extract_topic(&self, query: &str) -> String {
        // Remove common question words to get the core topic
        let topic = query
            .to_lowercase()
            .replace("how do i ", "")
            .replace("how to ", "")
            .replace("what is ", "")
            .replace("what are ", "")
            .replace("why do ", "")
            .replace("why does ", "")
            .replace("tell me about ", "")
            .replace("explain ", "")
            .replace("?", "")
            .trim()
            .to_string();

        // Capitalize first letter
        let mut chars = topic.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    fn extract_steps_from_content(&self, content: &str) -> Vec<String> {
        let mut steps = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_step = String::new();
        let mut in_step_section = false;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Detect if we're entering a procedural section
            if trimmed.to_uppercase().contains("STEP")
                || trimmed.to_uppercase().contains("METHOD")
                || (trimmed.to_uppercase() == trimmed && trimmed.len() < 50 && trimmed.contains("BUILD"))
            {
                in_step_section = true;
                continue;
            }

            // Detect step indicators
            let is_step_start =
                // Numbered: "1.", "2)", etc
                (trimmed.chars().next().map(|c| c.is_numeric()).unwrap_or(false)
                    && trimmed.chars().nth(1).map(|c| c == '.' || c == ')').unwrap_or(false))
                    // Keywords
                    || trimmed.starts_with("First")
                    || trimmed.starts_with("Second")
                    || trimmed.starts_with("Third")
                    || trimmed.starts_with("Next")
                    || trimmed.starts_with("Then")
                    || trimmed.starts_with("Finally")
                    // Bullets
                    || trimmed.starts_with("•")
                    || trimmed.starts_with("-")
                    || trimmed.starts_with("*");

            if is_step_start || in_step_section {
                // Save previous step
                if !current_step.is_empty() && current_step.len() > 20 {
                    steps.push(self.clean_step_text(&current_step));
                    current_step = String::new();
                }

                // Start new step
                if is_step_start {
                    current_step = trimmed.to_string();
                    in_step_section = true;
                } else if in_step_section {
                    current_step.push(' ');
                    current_step.push_str(trimmed);
                }
            } else if in_step_section && !current_step.is_empty() {
                // Continue current step
                current_step.push(' ');
                current_step.push_str(trimmed);
            }

            // Limit step length
            if current_step.len() > 400 {
                steps.push(self.clean_step_text(&current_step));
                current_step = String::new();
            }

            // Stop if we've collected enough or hit a new section
            if steps.len() >= 8 || (trimmed.to_uppercase() == trimmed && trimmed.len() > 10 && trimmed.len() < 50) {
                if !current_step.is_empty() {
                    steps.push(self.clean_step_text(&current_step));
                }
                break;
            }
        }

        // Save last step
        if !current_step.is_empty() && current_step.len() > 20 {
            steps.push(self.clean_step_text(&current_step));
        }

        // Filter and return
        steps.into_iter()
            .filter(|s| s.len() > 30 && s.split_whitespace().count() > 5)
            .take(7)
            .collect()
    }

    fn clean_step_text(&self, text: &str) -> String {
        text.trim()
            .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ')' || c == '•' || c == '-')
            .trim()
            .to_string()
    }

    fn extract_relevant_paragraphs(&self, content: &str, max_paragraphs: usize) -> Vec<String> {
        let paragraphs: Vec<String> = content
            .split("\n\n")
            .map(|p| p.trim().to_string())
            .filter(|p| {
                // More strict filtering
                let len = p.len();
                let has_substance = p.split_whitespace().count() > 10;
                let not_header = !p.chars().all(|c| c.is_uppercase() || c.is_whitespace() || c.is_ascii_punctuation());

                len > 100 && len < 1000 && has_substance && not_header
            })
            .take(max_paragraphs)
            .collect();

        // If we didn't find good paragraphs, try splitting by sentences
        if paragraphs.is_empty() {
            content
                .split('.')
                .map(|s| s.trim().to_string())
                .filter(|s| s.len() > 50 && s.len() < 500)
                .take(max_paragraphs)
                .map(|s| format!("{}.", s))
                .collect()
        } else {
            paragraphs
        }
    }

    fn extract_tip_from_content(&self, content: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();

        for line in lines {
            let lower = line.to_lowercase();
            if lower.contains("tip:")
                || lower.contains("note:")
                || lower.contains("important:")
                || lower.contains("remember")
            {
                return Some(line.trim().to_string());
            }
        }

        None
    }

    fn generate_no_knowledge_response(&self, query: &str) -> String {
        let topic = self.extract_topic(query);
        let query_lower = query.to_lowercase();

        // Try to identify the domain
        let suggested_domain = if query_lower.contains("fire") || query_lower.contains("shelter")
            || query_lower.contains("water") || query_lower.contains("food") {
            "survival"
        } else if query_lower.contains("plant") || query_lower.contains("garden")
            || query_lower.contains("crop") {
            "agriculture"
        } else if query_lower.contains("build") || query_lower.contains("construct") {
            "construction"
        } else if query_lower.contains("cook") || query_lower.contains("recipe") {
            "cooking"
        } else {
            "general homesteading"
        };

        format!(
            "I don't have enough information in my knowledge base to answer your question about {}.\n\n\
            **What I need:**\n\
            I could better answer this question if you added more knowledge documents about {}. \
            Try adding PDF guides, manuals, or books covering this topic to your knowledge modules.\n\n\
            **To add knowledge:**\n\
            1. Place PDF/text files in `modules/wasteland/knowledge_*/knowledge/`\n\
            2. Restart the app to re-ingest the knowledge base\n\
            3. Ask your question again!\n\n\
            I learn from survival manuals, homesteading guides, technical documentation, and reference books. \
            The more comprehensive the sources, the better I can help!",
            topic.to_lowercase(),
            suggested_domain
        )
    }

    /// Process query using mock responses
    fn process_with_mock(
        &self,
        query: &str,
        knowledge_context: &[KnowledgeChunk],
    ) -> Result<String> {
        let query_lower = query.to_lowercase();

        // If we have knowledge context, use the synthesis method
        if !knowledge_context.is_empty() {
            return self.process_with_knowledge_synthesis(query, knowledge_context);
        }

        // Pattern-based responses for when no knowledge is found
        let response = if query_lower.contains("help") || query_lower.contains("what can you do") {
            "I'm Survon's assistant! I can help with:\n\
            • Searching your knowledge modules\n\
            • Answering questions about homesteading and survival\n\
            • System information and monitoring\n\n\
            Try asking specific questions, and I'll search the knowledge base for answers!"
        } else if query_lower.contains("survival") {
            "For survival information, I can search through your survival knowledge modules. \
            Try asking about specific topics like 'shelter building' or 'water purification'."
        } else if query_lower.contains("gate") {
            "Gate systems respond to 'open_gate' and 'close_gate' commands via the message bus. \
            Check power supply and sensor alignment for troubleshooting."
        } else {
            &format!(
                "I understand you're asking about '{}'. I'll search the knowledge base for relevant information. \
                If no results are found, try rephrasing your question or ask about specific topics.",
                query
            )
        };

        Ok(response.to_string())
    }

    /// Build prompt for embedded LLM with knowledge context
    fn build_embedded_prompt(&self, query: &str, knowledge_context: &[KnowledgeChunk]) -> String {
        let mut prompt = String::new();

        prompt.push_str("<|system|>\n");
        prompt.push_str("You are Survon, a knowledgeable and helpful homestead assistant. ");
        prompt.push_str("Your role is to answer questions conversationally, like a friend sharing expertise.\n\n");

        if !knowledge_context.is_empty() {
            prompt.push_str("INSTRUCTIONS:\n");
            prompt.push_str("- Read the provided knowledge excerpts carefully\n");
            prompt.push_str("- Synthesize the information into a natural, conversational answer\n");
            prompt.push_str("- Write as if you're explaining to a friend, not listing facts\n");
            prompt.push_str("- If it's a 'how-to' question, provide clear steps\n");
            prompt.push_str("- If it's a 'what is' question, explain clearly\n");
            prompt.push_str("- Be practical and actionable\n");
            prompt.push_str("- Don't mention the sources in your answer (they'll be cited separately)\n\n");

            prompt.push_str("KNOWLEDGE BASE:\n");
            prompt.push_str("---\n");

            for (i, chunk) in knowledge_context.iter().enumerate().take(5) {
                // Include substantial context per chunk
                let content = if chunk.body.len() > 600 {
                    &chunk.body[..600]
                } else {
                    &chunk.body
                };

                prompt.push_str(&format!("[Excerpt {}]:\n{}\n\n", i + 1, content));
            }

            prompt.push_str("---\n\n");
        } else {
            prompt.push_str("INSTRUCTIONS:\n");
            prompt.push_str("- Answer based on general homesteading and survival knowledge\n");
            prompt.push_str("- Be conversational and practical\n");
            prompt.push_str("- If you don't have enough information, say so honestly\n\n");
        }

        prompt.push_str("<|user|>\n");
        prompt.push_str(query);
        prompt.push_str("\n<|assistant|>\n");

        prompt
    }

    /// Clean up LLM response
    fn clean_llm_response(&self, response: &str) -> String {
        let cleaned = response
            .replace("<|end|>", "")
            .replace("<|assistant|>", "")
            .replace("<|user|>", "")
            .replace("<|system|>", "")
            .trim()
            .to_string();

        // Split into lines and clean each
        let lines: Vec<String> = cleaned
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .take(30)  // Allow more lines for synthesis
            .collect();

        let result = lines.join("\n");

        if result.is_empty() || result.len() < 20 {
            "I understand your question, but I'm having trouble generating a response. \
            Could you try rephrasing?".to_string()
        } else {
            result
        }
    }

    /// Get chat history for a session
    pub fn get_chat_history(&self, session_id: &str, limit: usize) -> Result<Vec<ChatMessage>> {
        Ok(self.database.get_chat_history(session_id, limit)?)
    }

    /// Get model info string
    pub fn get_model_info(&self) -> String {
        match self.strategy {
            LlmStrategyType::Embedded => "Phi-3 Mini (Embedded)".to_string(),
            LlmStrategyType::Knowledge => "Knowledge Search (FTS5)".to_string(),
            LlmStrategyType::Mock => "Mock LLM (Pattern-based)".to_string(),
        }
    }
}

impl std::fmt::Debug for LlmService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmService")
            .field("strategy", &self.strategy)
            .field("database", &"<Database>")
            .finish()
    }
}

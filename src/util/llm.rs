// src/util/llm.rs
//! Hybrid approach: Smart search + tiny LLM for humanizing output

use color_eyre::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use llama_cpp::{LlamaModel, LlamaParams, SessionParams};
use llama_cpp::standard_sampler::StandardSampler;
use gag::Gag;
use tokio::time::Duration;
use serde::{Deserialize, Serialize};

use crate::util::database::Database;
use crate::modules::llm::database::{LlmDatabase, ChatMessage, KnowledgeChunk};
use crate::{log_error, log_debug};

/// LLM service with optional lightweight summarizer
#[derive(Clone)]
pub struct LlmService {
    database: Database,
    use_summarizer: bool,
    model_path: Option<PathBuf>,
    remote_endpoint: Option<String>,
    remote_model: Option<String>,
}

impl LlmService {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            use_summarizer: false,
            model_path: None,
            remote_model: None,
            remote_endpoint: None,
        }
    }

    pub fn new_remote(database: Database, endpoint: String, model: String) -> Self {
        Self {
            database,
            use_summarizer: false,
            model_path: None,
            remote_endpoint: Some(endpoint),
            remote_model: Some(model),
        }
    }

    pub async fn from_model_name(database: Database, model_name: &str) -> Result<Self> {
        let (use_summarizer, model_path) = match model_name {
            "search" => (false, None),
            "summarizer" | "hybrid" => {
                // Look for a tiny model (Q3_K_S or smaller)
                let path = PathBuf::from("bundled/models/phi3-mini.gguf");
                if path.exists() {
                    (true, Some(path))
                } else {
                    log_error!("Summarizer model not found, falling back to search-only");
                    (false, None)
                }
            }
            _ => (false, None),
        };

        Ok(Self {
            database,
            use_summarizer,
            model_path,
            remote_model: None,
            remote_endpoint: None,
        })
    }

    /// Process a user query
    pub async fn process_query(
        &self,
        session_id: &str,
        module_name: &str,
        query: &str,
        knowledge_module_names: &[String],
    ) -> Result<String> {
        // Check if using remote endpoint
        if let (Some(endpoint), Some(model)) = (&self.remote_endpoint, &self.remote_model) {
            return self.query_remote_llm(session_id, module_name, query, endpoint, model).await;
        }

        // Store user message
        let user_message = ChatMessage::new_user(
            session_id.to_string(),
            query.to_string(),
            module_name.to_string(),
        );
        self.database.insert_chat_message(user_message)?;

        // Extract search terms and search
        let search_terms = self.extract_search_terms(query);
        log_debug!("Search terms: {:?}", search_terms);

        let knowledge_context = self.search_knowledge_smart(&search_terms, knowledge_module_names)?;

        // Generate response
        let response = if knowledge_context.is_empty() {
            self.generate_no_results_response(query)
        } else if self.use_summarizer && self.model_path.is_some() {
            // Use tiny LLM to humanize the search results
            self.summarize_with_tiny_llm(query, &knowledge_context).await?
        } else {
            // Direct search results
            self.generate_answer_from_chunks(query, &knowledge_context)
        };

        // Store assistant response
        let assistant_message = ChatMessage::new_assistant(
            session_id.to_string(),
            response.clone(),
            module_name.to_string(),
        );
        let _ = self.database.insert_chat_message(assistant_message);

        Ok(response)
    }

    async fn query_remote_llm(
        &self,
        session_id: &str,
        module_name: &str,
        query: &str,
        endpoint: &str,
        model: &str,
    ) -> Result<String> {
        // Store user message
        let user_message = ChatMessage::new_user(
            session_id.to_string(),
            query.to_string(),
            module_name.to_string(),
        );
        self.database.insert_chat_message(user_message)?;

        // Build request
        let payload = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "user", "content": query}
            ],
            "stream": false
        });

        let url = format!("{}/api/chat", endpoint);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await?;

        #[derive(Deserialize)]
        struct OllamaResponse {
            message: OllamaMessage,
        }

        #[derive(Deserialize)]
        struct OllamaMessage {
            content: String,
        }

        let ollama_response: OllamaResponse = response.json().await?;
        let assistant_content = ollama_response.message.content;

        // Store assistant response
        let assistant_message = ChatMessage::new_assistant(
            session_id.to_string(),
            assistant_content.clone(),
            module_name.to_string(),
        );
        self.database.insert_chat_message(assistant_message)?;

        Ok(assistant_content)
    }

    /// Summarize search results using a tiny LLM
    async fn summarize_with_tiny_llm(
        &self,
        query: &str,
        chunks: &[KnowledgeChunk],
    ) -> Result<String> {
        let model_path = match &self.model_path {
            Some(p) => p,
            None => return Ok(self.generate_answer_from_chunks(query, chunks)),
        };

        // Suppress llama.cpp output
        let print_gag = Gag::stdout()
            .map_err(|e| color_eyre::eyre::eyre!("Gag error: {}", e))?;

        // Load model with minimal settings for fast inference
        let params = LlamaParams::default();
        let model = LlamaModel::load_from_file(model_path, params)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to load model: {}", e))?;

        // Tiny context window - we only need to summarize short excerpts
        let session_params = SessionParams {
            n_ctx: 1024,      // Small context = faster
            n_batch: 32,      // Small batch = less memory
            n_threads: 2,     // Only 2 threads
            n_threads_batch: 1,
            ..Default::default()
        };

        let mut session = model.create_session(session_params)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to create session: {}", e))?;

        // Build a MINIMAL prompt
        let prompt = self.build_summarizer_prompt(query, chunks);
        log_debug!("Prompt length: {} chars", prompt.len());

        session.advance_context(&prompt)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to advance context: {}", e))?;

        let sampler = StandardSampler::default();
        let max_tokens = 200;  // Short summary only!

        let mut response = String::new();
        let completion_result = session.start_completing_with(sampler, max_tokens);
        let mut completion_handle = match completion_result {
            Ok(handle) => handle.into_strings(),
            Err(e) => return Err(color_eyre::eyre::eyre!("Failed to start completion: {}", e)),
        };

        let start_time = std::time::Instant::now();
        while let Some(token) = completion_handle.next() {
            if start_time.elapsed() > Duration::from_secs(15) {  // Quick timeout
                break;
            }
            response.push_str(&token);

            if response.len() > 1000 {  // Cap response length
                break;
            }

            // Stop on natural ending
            if response.contains("<|end|>") || response.contains("<|user|>") {
                break;
            }
        }

        drop(print_gag);

        let cleaned = self.clean_llm_response(&response);

        // Add source citations
        let mut final_response = cleaned;
        final_response.push_str("\n\n---\n📚 **Sources**: ");

        let sources = self.extract_unique_sources(chunks);
        for (i, source) in sources.iter().enumerate() {
            if i > 0 {
                final_response.push_str(", ");
            }
            final_response.push_str(&format!("(from {})", source));
        }

        final_response.push_str("\n\n💡 *Press Tab to cycle source links, Enter to open.*");

        Ok(final_response)
    }

    /// Build minimal prompt for summarization
    fn build_summarizer_prompt(&self, query: &str, chunks: &[KnowledgeChunk]) -> String {
        let mut prompt = String::new();

        prompt.push_str("<|system|>\nYou are a helpful assistant that answers questions concisely based on provided excerpts.\n");
        prompt.push_str("Rules:\n");
        prompt.push_str("- Answer in 2-4 sentences maximum\n");
        prompt.push_str("- Use natural, conversational language\n");
        prompt.push_str("- Only use information from the excerpts\n");
        prompt.push_str("- If excerpts don't fully answer the question, say so\n\n");

        // Add ONLY the most relevant snippets (keep it tiny)
        prompt.push_str("EXCERPTS:\n");
        for (i, chunk) in chunks.iter().take(3).enumerate() {
            // Take only first 300 chars of each chunk
            let snippet = if chunk.body.len() > 300 {
                &chunk.body[..300]
            } else {
                &chunk.body
            };
            prompt.push_str(&format!("[{}] {}\n\n", i+1, snippet));
        }

        prompt.push_str("<|user|>\n");
        prompt.push_str(query);
        prompt.push_str("\n<|assistant|>\n");

        prompt
    }

    /// Clean LLM response
    fn clean_llm_response(&self, response: &str) -> String {
        response
            .replace("<|end|>", "")
            .replace("<|assistant|>", "")
            .replace("<|user|>", "")
            .replace("<|system|>", "")
            .trim()
            .to_string()
    }

    /// Extract search terms from query
    fn extract_search_terms(&self, query: &str) -> Vec<String> {
        let stopwords = [
            "how", "do", "i", "a", "the", "is", "to", "can", "you", "what", "when",
            "where", "why", "does", "will", "would", "could", "should", "make",
            "build", "create", "get", "find", "tell", "me", "about", "my", "your",
            "explain", "show", "help", "with", "from"
        ];

        query.to_lowercase()
            .split_whitespace()
            .filter(|word| {
                let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
                clean.len() > 2 && !stopwords.contains(&clean)
            })
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .collect()
    }

    /// Smart multi-strategy search
    fn search_knowledge_smart(
        &self,
        search_terms: &[String],
        knowledge_module_names: &[String],
    ) -> Result<Vec<KnowledgeChunk>> {
        if search_terms.is_empty() {
            return Ok(Vec::new());
        }

        let domains: Vec<String> = knowledge_module_names
            .iter()
            .filter_map(|name| {
                if name.contains("_knowledge") {
                    Some(name.replace("_knowledge", ""))
                } else {
                    None
                }
            })
            .collect();

        // Strategy 1: Phrase search
        let phrase_query = search_terms.join(" ");
        let mut results = self.database.search_knowledge(&phrase_query, &domains, 15)?;
        log_debug!("Phrase search: {} results", results.len());

        // Strategy 2: OR search if needed
        if results.len() < 5 && search_terms.len() > 1 {
            let or_query = search_terms.join(" OR ");
            let or_results = self.database.search_knowledge(&or_query, &domains, 20)?;
            log_debug!("OR search: {} results", or_results.len());

            for chunk in or_results {
                if !results.iter().any(|r| r.id == chunk.id) {
                    results.push(chunk);
                }
            }
        }

        // Strategy 3: Individual terms
        if results.len() < 5 {
            for term in search_terms.iter().take(2) {
                let term_results = self.database.search_knowledge(term, &domains, 10)?;
                for chunk in term_results {
                    if !results.iter().any(|r| r.id == chunk.id) {
                        results.push(chunk);
                    }
                }
            }
        }

        // Score and rank
        let scored_results = self.score_and_rank_chunks(&results, search_terms);
        Ok(scored_results.into_iter().take(6).collect())
    }

    /// Score and rank chunks by relevance
    fn score_and_rank_chunks(
        &self,
        chunks: &[KnowledgeChunk],
        search_terms: &[String],
    ) -> Vec<KnowledgeChunk> {
        let mut scored: Vec<(f32, KnowledgeChunk)> = chunks
            .iter()
            .map(|chunk| {
                let score = self.calculate_relevance_score(chunk, search_terms);
                (score, chunk.clone())
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().map(|(_, chunk)| chunk).collect()
    }

    /// Calculate relevance score
    fn calculate_relevance_score(&self, chunk: &KnowledgeChunk, search_terms: &[String]) -> f32 {
        let title_lower = chunk.title.to_lowercase();
        let body_lower = chunk.body.to_lowercase();
        let mut score = 0.0;

        for term in search_terms {
            let term_lower = term.to_lowercase();
            score += title_lower.matches(&term_lower).count() as f32 * 10.0;
            score += body_lower.matches(&term_lower).count() as f32;

            let intro = body_lower[..body_lower.len().min(200)].to_string();
            score += intro.matches(&term_lower).count() as f32 * 3.0;
        }

        if chunk.body.len() < 100 {
            score *= 0.3;
        }

        score
    }

    /// Generate answer from chunks (fallback for non-summarizer mode)
    fn generate_answer_from_chunks(
        &self,
        query: &str,
        chunks: &[KnowledgeChunk],
    ) -> String {
        let topic = self.extract_topic(query);
        let mut response = format!("**Regarding {}:**\n\n", topic);

        let snippets = self.extract_relevant_snippets(chunks, 4);
        for snippet in snippets {
            response.push_str(&snippet);
            response.push_str("\n\n");
        }

        // Add sources
        response.push_str("\n---\n📚 **Sources**: ");
        let sources = self.extract_unique_sources(chunks);
        for (i, source) in sources.iter().enumerate() {
            if i > 0 {
                response.push_str(", ");
            }
            response.push_str(&format!("(from {})", source));
        }

        response.push_str("\n\n💡 *Press Tab to cycle source links, Enter to open.*");
        response
    }

    /// Extract relevant snippets
    fn extract_relevant_snippets(&self, chunks: &[KnowledgeChunk], max: usize) -> Vec<String> {
        let mut snippets = Vec::new();

        for chunk in chunks.iter().take(max * 2) {
            let sentences: Vec<&str> = chunk.body
                .split('.')
                .map(|s| s.trim())
                .filter(|s| s.len() > 30 && s.len() < 300)
                .collect();

            for sentence in sentences.iter().take(2) {
                if !sentence.is_empty() {
                    snippets.push(format!("{}.", sentence));
                }
            }

            if snippets.len() >= max {
                break;
            }
        }

        snippets.into_iter().take(max).collect()
    }

    /// Extract unique sources
    fn extract_unique_sources(&self, chunks: &[KnowledgeChunk]) -> Vec<String> {
        let mut sources = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for chunk in chunks {
            let source_path = if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&chunk.metadata) {
                if let Some(page_num) = metadata.get("page_number").and_then(|v| v.as_u64()) {
                    format!("{}#page={}", chunk.source_file, page_num)
                } else {
                    chunk.source_file.clone()
                }
            } else {
                chunk.source_file.clone()
            };

            let filename = std::path::Path::new(&chunk.source_file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&chunk.source_file);

            if !seen.contains(filename) {
                sources.push(source_path);
                seen.insert(filename.to_string());
            }

            if sources.len() >= 5 {
                break;
            }
        }

        sources
    }

    /// Extract topic from query
    fn extract_topic(&self, query: &str) -> String {
        let topic = query
            .to_lowercase()
            .replace("how do i ", "")
            .replace("how to ", "")
            .replace("what is ", "")
            .replace("why ", "")
            .replace("?", "")
            .trim()
            .to_string();

        let mut chars = topic.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// Generate no results response
    fn generate_no_results_response(&self, query: &str) -> String {
        let topic = self.extract_topic(query);

        format!(
            "**No information found about '{}'**\n\n\
            I couldn't find relevant information in the knowledge base.\n\n\
            **Try:**\n\
            • Rephrasing with different keywords\n\
            • Adding PDFs/guides to `modules/wasteland/knowledge_*/knowledge/`\n\
            • Restarting to re-ingest knowledge base",
            topic.to_lowercase()
        )
    }

    pub fn get_chat_history(&self, session_id: &str, limit: usize) -> Result<Vec<ChatMessage>> {
        Ok(self.database.get_chat_history(session_id, limit)?)
    }

    pub fn get_model_info(&self) -> String {
        if self.use_summarizer {
            "Smart Search + Lightweight Summarizer".to_string()
        } else {
            "Smart Search (FTS5)".to_string()
        }
    }
}

impl std::fmt::Debug for LlmService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmService")
            .field("mode", &if self.use_summarizer { "hybrid" } else { "search" })
            .finish()
    }
}

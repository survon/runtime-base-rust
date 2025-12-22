use crate::{
    log_debug,
    modules::llm::database::{
        execute_search,
        sanitize_fts5_query,
        KnowledgeChunk
    },
    util::database::Database,
};

impl Database {
    pub(super) fn _search_knowledge(&self, query: &str, domains: &[String], limit: usize) -> rusqlite::Result<Vec<KnowledgeChunk>> {
        let clean_query = sanitize_fts5_query(query);
        if clean_query.trim().is_empty() {
            return Ok(Vec::new());
        }

        log_debug!("Searching knowledge with query: '{}' (sanitized from '{}')", clean_query, query);
        if !domains.is_empty() {
            log_debug!("Filtering by domains: {:?}", domains);
        }

        // Try different search strategies
        let mut results = Vec::new();

        // Strategy 1: Try exact phrase match with AND
        results = execute_search(self, &clean_query, domains, limit * 2)?;
        log_debug!("Strategy 1 (AND search): found {} results", results.len());

        // Strategy 2: If no results, try OR search
        if results.is_empty() && clean_query.contains(' ') {
            let or_query = clean_query.split_whitespace().collect::<Vec<_>>().join(" OR ");
            log_debug!("Strategy 2 (OR search): trying '{}'", or_query);
            results = execute_search(self, &or_query, domains, limit * 2)?;
            log_debug!("Strategy 2 (OR search): found {} results", results.len());
        }

        // Strategy 3: If still no results, try each word individually
        if results.is_empty() {
            let words: Vec<&str> = clean_query.split_whitespace().collect();
            log_debug!("Strategy 3 (individual words): trying {} words", words.len());
            for word in &words {
                let word_results = execute_search(self, word, domains, limit)?;
                log_debug!("  Word '{}': found {} results", word, word_results.len());
                if !word_results.is_empty() {
                    results.extend(word_results);
                    break; // Use first successful word
                }
            }
        }

        // Filter results by relevance for OR queries
        if clean_query.contains(" OR ") {
            let keywords: Vec<&str> = clean_query.split(" OR ").collect();
            results = results.into_iter()
                .filter(|chunk| {
                    let content_lower = format!("{} {}", chunk.title, chunk.body).to_lowercase();
                    let matches = keywords.iter().filter(|&&keyword| content_lower.contains(keyword)).count();
                    matches >= 2 || keywords.len() == 1
                })
                .take(limit)
                .collect();
        }

        log_debug!("Final results: {} chunks", results.len());
        Ok(results.into_iter().take(limit).collect())
    }
}

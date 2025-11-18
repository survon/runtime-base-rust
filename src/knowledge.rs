use crate::database::{Database, KnowledgeChunk};
use color_eyre::Result;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub module_type: String,
    pub domain: Option<String>,
}

pub struct KnowledgeIngester<'a> {
    database: &'a Database,
    modules_dir: PathBuf,
}

impl<'a> KnowledgeIngester<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self {
            database,
            modules_dir: PathBuf::from("./modules/wasteland"),
        }
    }

    pub fn should_reingest(&self) -> Result<bool> {
        let current_checksum = self.calculate_modules_checksum()?;

        // Check stored checksum in database
        match self.database.get_module_state("knowledge_checksum") {
            Ok(Some(stored_checksum)) => {
                println!("stored_checksum is {}", stored_checksum);
                let stored: u64 = stored_checksum.parse().unwrap_or(0);
                Ok(current_checksum != stored)
            }
            _ => Ok(true) // No checksum stored, need to ingest
        }
    }

    pub fn ingest_all_knowledge(&self) -> Result<()> {
        println!("Starting knowledge ingestion...");

        // Clear existing knowledge
        self.database.clear_knowledge()?;

        let mut total_chunks = 0;

        // Find all knowledge modules
        for module_dir in fs::read_dir(&self.modules_dir)? {
            let module_dir = module_dir?;
            if !module_dir.file_type()?.is_dir() {
                continue;
            }

            let config_path = module_dir.path().join("config.yml");
            if !config_path.exists() {
                continue;
            }

            // Parse module config
            let config_content = fs::read_to_string(&config_path)?;
            let config: ModuleConfig = serde_yaml::from_str(&config_content)?;

            if config.module_type != "knowledge" {
                continue;
            }

            println!("Processing knowledge module: {}", config.name);

            let knowledge_dir = module_dir.path().join("knowledge");
            if knowledge_dir.exists() {
                let chunks = self.process_knowledge_directory(&knowledge_dir, &config)?;
                total_chunks += chunks;
            }
        }

        // Store new checksum
        let checksum = self.calculate_modules_checksum()?;
        self.database.save_module_state("knowledge_checksum", &checksum.to_string())?;

        println!("Knowledge ingestion complete. Processed {} chunks.", total_chunks);
        Ok(())
    }

    fn process_knowledge_directory(&self, knowledge_dir: &Path, config: &ModuleConfig) -> Result<usize> {
        let mut chunk_count = 0;

        // Recursively process all files in knowledge directory and subdirectories
        self.process_directory_recursive(knowledge_dir, config, &mut chunk_count)?;

        Ok(chunk_count)
    }

    fn process_directory_recursive(&self, dir: &Path, config: &ModuleConfig, chunk_count: &mut usize) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively process subdirectories
                self.process_directory_recursive(&path, config, chunk_count)?;
            } else if path.is_file() {
                let chunks = self.process_file(&path, config)?;
                *chunk_count += chunks.len();

                // Insert chunks into database
                for chunk in chunks {
                    self.database.insert_knowledge_chunk(chunk)?;
                }
            }
        }
        Ok(())
    }

    fn process_file(&self, file_path: &Path, config: &ModuleConfig) -> Result<Vec<KnowledgeChunk>> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        println!("  Processing file: {}", file_path.display());

        match extension.as_str() {
            "txt" => self.process_text_file(file_path, config),
            "html" | "htm" => self.process_html_file(file_path, config),
            "pdf" => self.process_pdf_file(file_path, config),
            "rtf" => self.process_rtf_file(file_path, config),
            "doc" | "docx" => self.process_doc_file(file_path, config),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" => self.process_image_file(file_path, config),
            _ => {
                println!("    Unsupported file type: {}", extension);
                Ok(vec![])
            }
        }
    }

    fn process_text_file(&self, file_path: &Path, config: &ModuleConfig) -> Result<Vec<KnowledgeChunk>> {
        let content = fs::read_to_string(file_path)?;
        let chunks = self.chunk_text(&content, file_path, config);
        Ok(chunks)
    }

    fn process_html_file(&self, file_path: &Path, config: &ModuleConfig) -> Result<Vec<KnowledgeChunk>> {
        let content = fs::read_to_string(file_path)?;
        // Simple HTML stripping - remove tags
        let text_content = content
            .lines()
            .map(|line| {
                // Basic HTML tag removal
                let mut result = String::new();
                let mut in_tag = false;
                for ch in line.chars() {
                    match ch {
                        '<' => in_tag = true,
                        '>' => in_tag = false,
                        _ if !in_tag => result.push(ch),
                        _ => {}
                    }
                }
                result
            })
            .collect::<Vec<_>>()
            .join("\n");

        let chunks = self.chunk_text(&text_content, file_path, config);
        Ok(chunks)
    }

    fn chunk_text_with_images(&self, content: &str, file_path: &Path, config: &ModuleConfig, metadata: String) -> Vec<KnowledgeChunk> {
        let mut chunks = Vec::new();
        let paragraphs: Vec<&str> = content.split("\n\n").filter(|p| !p.trim().is_empty()).collect();

        for (index, paragraph) in paragraphs.iter().enumerate() {
            let inferred_domains = self.infer_domains_from_content(paragraph);
            let primary_domain = inferred_domains.first().unwrap_or(&"general".to_string()).clone();

            let chunk = KnowledgeChunk {
                id: None,
                source_file: file_path.to_string_lossy().to_string(),
                domain: primary_domain,
                category: config.name.clone(),
                title: self.extract_title(paragraph, index),
                body: paragraph.trim().to_string(),
                chunk_index: index as i32,
                metadata: metadata.clone(), // Use the provided metadata with image mappings
            };
            chunks.push(chunk);
        }

        chunks
    }

    fn process_pdf_file(&self, file_path: &Path, config: &ModuleConfig) -> Result<Vec<KnowledgeChunk>> {
        use pdf::file::FileOptions;

        let cache_dir = PathBuf::from("./.cache/knowledge");
        std::fs::create_dir_all(&cache_dir)?;

        let mut all_chunks = Vec::new();

        // Try to parse PDF page by page
        if let Ok(pdf_file) = FileOptions::cached().open(file_path) {
            for page_num in 0..pdf_file.num_pages() {
                if let Ok(page) = pdf_file.get_page(page_num) {
                    // Extract text from this specific page
                    // Note: This is simplified - you might need to use a different method
                    let page_text = format!("Page {} content", page_num + 1); // Placeholder

                    if page_text.trim().is_empty() {
                        continue;
                    }

                    // Create chunks for this page
                    let paragraphs: Vec<&str> = page_text.split("\n\n")
                        .filter(|p| !p.trim().is_empty())
                        .collect();

                    for (para_index, paragraph) in paragraphs.iter().enumerate() {
                        let inferred_domains = self.infer_domains_from_content(paragraph);
                        let primary_domain = inferred_domains.first()
                            .unwrap_or(&"general".to_string())
                            .clone();

                        let chunk = KnowledgeChunk {
                            id: None,
                            source_file: file_path.to_string_lossy().to_string(),
                            domain: primary_domain,
                            category: config.name.clone(),
                            title: self.extract_title(paragraph, para_index),
                            body: paragraph.trim().to_string(),
                            chunk_index: ((page_num * 100) + (para_index as u32)) as i32,
                            metadata: serde_json::json!({
                            "file_type": "pdf",
                            "full_path": file_path.to_string_lossy(),
                            "page_number": page_num + 1,  // Store 1-indexed page number
                            "paragraph_index": para_index,
                        }).to_string(),
                        };
                        all_chunks.push(chunk);
                    }
                }
            }
            Ok(all_chunks)
        } else {
            // Fallback to old method if PDF parsing fails
            let text = pdf_extract::extract_text(file_path)?;
            let chunks = self.chunk_text(&text, file_path, config);
            Ok(chunks)
        }
    }

    fn extract_pdf_with_images(&self, file_path: &Path, cache_dir: &Path) -> Result<(String, HashMap<String, String>)> {
        // For now, just extract text - image extraction requires additional PDF parsing
        match pdf_extract::extract_text(file_path) {
            Ok(content) => {
                println!("    Extracted text from PDF: {} characters", content.len());

                // TODO: Add actual image extraction using a crate like `pdf` or `poppler`
                // For now, return text with empty image mappings
                let image_mappings = HashMap::new();

                Ok((content, image_mappings))
            }
            Err(e) => {
                println!("    Failed to extract text from PDF: {}", e);
                Ok((String::new(), HashMap::new()))
            }
        }
    }

    fn process_rtf_file(&self, _file_path: &Path, _config: &ModuleConfig) -> Result<Vec<KnowledgeChunk>> {
        println!("    RTF processing not implemented yet");
        Ok(vec![])
    }

    fn process_doc_file(&self, _file_path: &Path, _config: &ModuleConfig) -> Result<Vec<KnowledgeChunk>> {
        println!("    DOC/DOCX processing not implemented yet");
        Ok(vec![])
    }

    fn process_image_file(&self, _file_path: &Path, _config: &ModuleConfig) -> Result<Vec<KnowledgeChunk>> {
        println!("    Image OCR processing not implemented yet");
        Ok(vec![])
    }

    fn infer_domains_from_content(&self, content: &str) -> Vec<String> {
        let domain_keywords = vec![
            // Agriculture & Farming
            ("agriculture", vec!["farm", "farming", "crop", "crops", "harvest", "plant", "planting", "seed", "seeds", "soil", "fertilizer", "irrigation", "livestock", "cattle", "sheep", "goat", "pig", "chicken", "poultry", "dairy", "barn", "pasture", "field", "acre", "tractor", "plow", "cultivate", "organic", "pesticide", "herbicide", "compost", "manure", "ranch", "grazing"]),

            // Construction & Building
            ("construction", vec!["build", "building", "construction", "house", "home", "foundation", "concrete", "cement", "lumber", "wood", "frame", "framing", "roof", "roofing", "wall", "walls", "door", "window", "floor", "flooring", "plumbing", "electrical", "wiring", "insulation", "drywall", "paint", "siding", "brick", "stone", "nail", "screw", "hammer", "saw", "drill", "level", "measure", "blueprint", "permit", "contractor", "carpenter", "mason", "architect"]),

            // Survival & Emergency Preparedness
            ("survival", vec!["survival", "emergency", "preparedness", "wilderness", "rescue", "danger", "crisis", "disaster", "shelter", "fire", "water", "food", "hunting", "fishing", "trap", "snare", "forage", "edible", "poisonous", "first", "aid", "medical", "wound", "injury", "compass", "navigation", "map", "signal", "whistle", "knife", "rope", "cordage", "tarp", "sleeping", "bag", "backpack", "kit", "supplies", "cache", "stockpile", "bunker", "prepper", "shtf"]),

            // Food & Cooking
            ("food", vec!["cook", "cooking", "recipe", "food", "eat", "meal", "kitchen", "stove", "oven", "pan", "pot", "knife", "cutting", "board", "ingredient", "spice", "herb", "salt", "pepper", "oil", "butter", "meat", "vegetable", "fruit", "bread", "bake", "baking", "roast", "fry", "boil", "steam", "grill", "marinade", "sauce", "soup", "stew", "preserve", "canning", "pickle", "smoke", "cure", "ferment", "nutrition", "vitamin", "protein", "carbohydrate"]),

            // Health & Medicine
            ("health", vec!["health", "medicine", "medical", "doctor", "nurse", "hospital", "clinic", "treatment", "therapy", "drug", "medication", "pill", "dose", "symptom", "disease", "illness", "infection", "virus", "bacteria", "antibiotic", "vaccine", "immune", "fever", "pain", "headache", "nausea", "wound", "bandage", "surgery", "operation", "diagnosis", "patient", "recovery", "healing", "prevention", "hygiene", "sanitation", "exercise", "fitness", "diet", "nutrition", "mental", "stress", "anxiety", "depression"]),

            // Electronics & Technology
            ("electronics", vec!["electronic", "electronics", "circuit", "voltage", "current", "resistance", "capacitor", "resistor", "transistor", "diode", "led", "wire", "cable", "battery", "power", "solar", "generator", "motor", "sensor", "arduino", "raspberry", "pi", "microcontroller", "computer", "programming", "code", "software", "hardware", "digital", "analog", "signal", "frequency", "amplifier", "oscilloscope", "multimeter", "soldering", "pcb", "component", "relay", "switch", "button"]),

            // Mechanical & Engineering
            ("mechanical", vec!["mechanical", "engineering", "machine", "engine", "motor", "gear", "bearing", "shaft", "pump", "valve", "pipe", "hydraulic", "pneumatic", "pressure", "force", "torque", "leverage", "pulley", "belt", "chain", "spring", "bolt", "nut", "washer", "gasket", "seal", "lubrication", "oil", "grease", "maintenance", "repair", "troubleshoot", "calibrate", "alignment", "tolerance", "specification", "material", "steel", "aluminum", "plastic", "rubber"]),

            // Energy & Power
            ("energy", vec!["energy", "power", "electricity", "electrical", "grid", "utility", "solar", "wind", "hydro", "hydroelectric", "turbine", "generator", "alternator", "transformer", "inverter", "battery", "storage", "fuel", "gas", "gasoline", "diesel", "propane", "natural", "coal", "nuclear", "renewable", "sustainable", "efficiency", "conservation", "consumption", "load", "demand", "supply", "voltage", "amperage", "watt", "kilowatt", "megawatt"]),

            // Water & Plumbing
            ("water", vec!["water", "plumbing", "pipe", "pipes", "faucet", "tap", "valve", "pump", "well", "spring", "stream", "river", "lake", "pond", "reservoir", "tank", "cistern", "filter", "filtration", "purification", "chlorine", "disinfection", "pressure", "flow", "drain", "drainage", "sewer", "septic", "waste", "treatment", "irrigation", "sprinkler", "hose", "leak", "repair", "installation", "maintenance", "quality", "testing", "contamination", "clean", "potable"]),

            // Transportation & Vehicles
            ("transportation", vec!["vehicle", "car", "truck", "motorcycle", "bike", "bicycle", "boat", "ship", "plane", "aircraft", "helicopter", "train", "engine", "motor", "transmission", "brake", "tire", "wheel", "steering", "suspension", "fuel", "gas", "diesel", "maintenance", "repair", "oil", "change", "battery", "alternator", "starter", "radiator", "cooling", "heating", "air", "conditioning", "exhaust", "muffler", "carburetor", "injection", "spark", "plug"]),

            // Communication & Networking
            ("communication", vec!["radio", "antenna", "frequency", "ham", "amateur", "broadcast", "transmit", "receive", "signal", "modulation", "amplifier", "repeater", "satellite", "internet", "network", "wifi", "ethernet", "cable", "fiber", "optic", "router", "switch", "modem", "protocol", "tcp", "ip", "dns", "server", "client", "wireless", "cellular", "phone", "telephone", "voip", "encryption", "security", "firewall", "vpn", "bandwidth", "latency"]),

            // Security & Safety
            ("security", vec!["security", "safety", "protection", "guard", "alarm", "camera", "surveillance", "monitor", "sensor", "detector", "lock", "key", "access", "control", "gate", "fence", "barrier", "perimeter", "intrusion", "theft", "burglar", "fire", "smoke", "carbon", "monoxide", "emergency", "evacuation", "procedure", "protocol", "risk", "assessment", "hazard", "danger", "warning", "sign", "label", "certification", "compliance", "regulation", "standard"]),

            // Crafts & Manufacturing
            ("crafts", vec!["craft", "crafts", "making", "handmade", "diy", "workshop", "tool", "tools", "woodworking", "metalworking", "welding", "soldering", "cutting", "drilling", "sanding", "finishing", "polish", "stain", "varnish", "glue", "adhesive", "joint", "connection", "assembly", "fabrication", "manufacturing", "production", "quality", "precision", "measurement", "template", "jig", "fixture", "clamp", "vise", "bench", "lathe", "mill"]),

            // Gardening & Horticulture
            ("gardening", vec!["garden", "gardening", "plant", "plants", "flower", "flowers", "vegetable", "vegetables", "fruit", "fruits", "tree", "trees", "shrub", "grass", "lawn", "landscape", "landscaping", "soil", "compost", "fertilizer", "mulch", "seed", "seedling", "transplant", "prune", "trim", "water", "irrigation", "pest", "disease", "organic", "greenhouse", "nursery", "harvest", "bloom", "pollination", "propagation", "cutting", "grafting"]),

            // Weather & Climate
            ("weather", vec!["weather", "climate", "temperature", "humidity", "pressure", "barometric", "wind", "rain", "snow", "ice", "storm", "hurricane", "tornado", "lightning", "thunder", "forecast", "prediction", "meteorology", "atmosphere", "cloud", "precipitation", "evaporation", "condensation", "front", "high", "low", "seasonal", "drought", "flood", "freeze", "frost", "heat", "cold", "measurement", "instrument", "thermometer", "barometer", "anemometer"]),

            // Finance & Economics
            ("finance", vec!["money", "finance", "financial", "economy", "economic", "cost", "price", "budget", "expense", "income", "profit", "loss", "investment", "savings", "bank", "banking", "loan", "credit", "debt", "interest", "rate", "tax", "taxes", "insurance", "currency", "dollar", "cash", "payment", "purchase", "sale", "market", "business", "trade", "commerce", "accounting", "bookkeeping", "record", "transaction", "receipt", "invoice"]),

            // Education & Learning
            ("education", vec!["education", "learning", "teach", "teaching", "student", "teacher", "school", "university", "college", "course", "class", "lesson", "study", "research", "book", "textbook", "manual", "guide", "instruction", "tutorial", "training", "skill", "knowledge", "information", "data", "fact", "theory", "practice", "exercise", "test", "exam", "grade", "degree", "certificate", "diploma", "curriculum", "syllabus", "homework", "assignment", "project"]),

            // General Homestead
            ("homestead", vec!["homestead", "homesteading", "rural", "country", "property", "land", "acreage", "self", "sufficient", "sustainable", "independence", "off", "grid", "cabin", "cottage", "barn", "shed", "outbuilding", "fence", "fencing", "gate", "path", "driveway", "maintenance", "upkeep", "repair", "improvement", "renovation", "upgrade", "planning", "design", "layout", "zoning", "permit", "inspection", "neighbor", "community", "local"])
        ];

        let content_lower = content.to_lowercase();
        let mut domain_scores: Vec<(String, usize)> = Vec::new();

        for (domain, keywords) in domain_keywords {
            let score = keywords.iter()
                .filter(|&&keyword| content_lower.contains(keyword))
                .count();

            if score > 0 {
                domain_scores.push((domain.to_string(), score));
            }
        }

        // Sort by score, take top domains
        domain_scores.sort_by(|a, b| b.1.cmp(&a.1));
        let found_domains: Vec<String> = domain_scores.into_iter()
            .take(3) // Top 3 domains
            .map(|(domain, _)| domain)
            .collect();

        if found_domains.is_empty() {
            vec!["general".to_string()]
        } else {
            found_domains
        }
    }

    fn chunk_text(&self, content: &str, file_path: &Path, config: &ModuleConfig) -> Vec<KnowledgeChunk> {
        let mut chunks = Vec::new();
        let paragraphs: Vec<&str> = content.split("\n\n").filter(|p| !p.trim().is_empty()).collect();

        for (index, paragraph) in paragraphs.iter().enumerate() {
            let inferred_domains = self.infer_domains_from_content(paragraph); // <-- Using it here
            let primary_domain = inferred_domains.first().unwrap_or(&"general".to_string()).clone();

            let chunk = KnowledgeChunk {
                id: None,
                source_file: file_path.to_string_lossy().to_string(),
                domain: primary_domain, // Store primary domain
                category: config.name.clone(),
                title: self.extract_title(paragraph, index),
                body: paragraph.trim().to_string(),
                chunk_index: index as i32,
                metadata: serde_json::json!({
                    "file_type": file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("unknown"),
                    "paragraph_index": index,
                    "full_path": file_path.to_string_lossy(),
                    "all_domains": inferred_domains  // Store all inferred domains
                }).to_string(),
            };
            chunks.push(chunk);
        }

        chunks
    }

    fn extract_title(&self, paragraph: &str, index: usize) -> String {
        let first_line = paragraph.lines().next().unwrap_or("");
        if first_line.len() < 100 && (first_line.contains(':') || first_line.chars().all(|c| c.is_uppercase() || c.is_whitespace())) {
            first_line.to_string()
        } else {
            format!("Section {}", index + 1)
        }
    }

    fn calculate_modules_checksum(&self) -> Result<u64> {
        let mut hasher = DefaultHasher::new();

        if !self.modules_dir.exists() {
            return Ok(0);
        }

        for module_dir in fs::read_dir(&self.modules_dir)? {
            let module_dir = module_dir?;
            if !module_dir.file_type()?.is_dir() {
                continue;
            }

            let knowledge_dir = module_dir.path().join("knowledge");
            if knowledge_dir.exists() {
                self.hash_directory_recursive(&knowledge_dir, &mut hasher)?;
            }
        }

        Ok(hasher.finish())
    }

    fn hash_directory_recursive(&self, dir: &Path, hasher: &mut DefaultHasher) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively hash subdirectories
                self.hash_directory_recursive(&path, hasher)?;
            } else if path.is_file() {
                // Hash file path and modified time
                path.hash(hasher);
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                            duration.as_secs().hash(hasher);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

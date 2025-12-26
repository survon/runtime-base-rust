mod as_str;
mod color_code;
mod all;

use serde::{Deserialize, Serialize};

// TODO add a test to ensure all these files are synced up in their understanding of what's in this enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestUrgency {
    Chill,      // Someday, no rush
    Casual,     // Would be cool to do soon
    Moderate,   // Should probably do this
    Pressing,   // Getting time sensitive
    Critical,   // Offer expires or deadline approaching
}


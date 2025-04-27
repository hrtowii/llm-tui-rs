// use chrono::DateTime;
// use chrono::Utc;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assistant {
    pub model: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant(Assistant),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

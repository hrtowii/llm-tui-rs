use crate::chat_structs::Message;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatBranch {
    pub id: usize,
    pub name: String,
    pub messages: Vec<Message>,
}

impl ChatBranch {
    pub fn load_all(path: &Path) -> Result<Vec<ChatBranch>> {
        if !path.exists() {
            // create empty file
            let mut f = fs::File::create(path)?;
            f.write_all(b"[]")?;
        }
        let data = fs::read_to_string(path)?;
        let branches = serde_json::from_str(&data)?;
        Ok(branches)
    }

    pub fn save_all(path: &Path, branches: &Vec<ChatBranch>) -> Result<()> {
        let s = serde_json::to_string_pretty(branches)?;
        fs::write(path, s)?;
        Ok(())
    }
}

// how I can implement a settings page that
// 1. allows me to change the MODEL of openai model and
// 2: configure the backend.
//
// I was thinking of making a public enum of model backends and letting the user select them,
// then build the llmbuilder different depending on that.
//
// And then fetch the respective AI list and present them as a vec paragraph in the settings page-
// -depending on the backend selected.
//
// But how do I link it together
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AIBackend {
    OpenAI,
    Anthropic,
    Google,
    Groq,
    Ollama,
    XAi,
    Phind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AISettings {
    pub backend: AIBackend,
    pub model: String,
    pub api_key: Option<String>, // override
    pub temperature: f32,
    pub max_tokens: usize,
}

// there has to be a better way to do this...

impl AIBackend {
    pub fn load_all(path: &Path) -> Result<Self> {
        if !path.exists() {
            let mut f = fs::File::create(path)?;
            f.write_all(b"[]")?;
        }
        let data = fs::read_to_string(path)?;
        let items = serde_json::from_str(&data)?;
        Ok(items)
    }

    pub fn write_all(path: &Path, list: &AIBackend) -> Result<()> {
        let s = serde_json::to_string_pretty(list)?;
        fs::write(path, s)?;
        Ok(())
    }
}

impl AISettings {
    pub fn load_all(path: &Path) -> Result<Self> {
        if !path.exists() {
            let mut f = fs::File::create(path)?;
            f.write_all(b"[]")?;
        }
        let data = fs::read_to_string(path)?;
        let items = serde_json::from_str(&data)?;
        Ok(items)
    }

    pub fn write_all(path: &Path, list: &AISettings) -> anyhow::Result<()> {
        let s = serde_json::to_string_pretty(list)?;
        fs::write(path, s)?;
        Ok(())
    }
}

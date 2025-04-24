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
#[derive(Debug, Clone)]
pub enum AIBackend {
    OpenAI,
    Anthropic,
    Google,
    Groq,
    Ollama,
    XAi,
    Phind,
}

#[derive(Debug, Clone)]
pub struct AISettings {
    pub backend: AIBackend,
    pub model: String,
    pub api_key: Option<String>, // override
    pub temperature: f32,
    pub max_tokens: usize,
}

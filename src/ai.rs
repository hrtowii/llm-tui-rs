// use allms::{
//     llm::{
//         AnthropicModels, AwsBedrockModels, DeepSeekModels, GoogleModels, LLMModel, MistralModels,
//         OpenAIModels, PerplexityModels,
//     },
//     Completions,
// };
// https://github.com/graniet/llm/blob/main/examples/multi_backend_example.rs
use crate::ai_backend::{AIBackend, AISettings};
use crate::chat_structs::{Message, Role};
use llm::{
    builder::{LLMBackend, LLMBuilder}, // Builder pattern components
    chat::ChatMessage,
};

pub async fn run_ai(
    chat_history: Option<Vec<Message>>,
    prompt: &str,
    settings: &AISettings,
) -> Result<String, Box<dyn std::error::Error>> {
    // check settings.json
    let mut builder = LLMBuilder::new()
        .backend(match settings.backend {
            AIBackend::OpenAI => LLMBackend::OpenAI,
            AIBackend::Anthropic => LLMBackend::Anthropic,
            AIBackend::Google => LLMBackend::Google,
            AIBackend::Groq => LLMBackend::Groq,
            AIBackend::Ollama => LLMBackend::Ollama,
            AIBackend::XAi => LLMBackend::XAI,
            AIBackend::Phind => LLMBackend::Phind,
        })
        .model(&settings.model)
        .temperature(settings.temperature)
        .max_tokens(settings.max_tokens as u32);

    if let Some(key) = &settings.api_key {
        builder = builder.api_key(key);
    } else {
        let api_key_string;
        match &settings.backend {
            AIBackend::OpenAI => api_key_string = "OPENAI_API_KEY",
            AIBackend::Anthropic => api_key_string = "ANTHROPIC_API_KEY",
            AIBackend::Google => api_key_string = "GOOGLE_API_KEY",
            AIBackend::Groq => api_key_string = "GROQ_API_KEY",
            AIBackend::Ollama => {
                builder = builder.base_url(
                    &std::env::var("OLLAMA_URL").unwrap_or("http://127.0.0.1:11434".into()),
                );
                api_key_string = "OLLAMA_URL";
            }
            AIBackend::XAi => api_key_string = "XAI_API_KEY",
            // AIBackend::Phind => api_key_string = "ANTHROPIC_API_KEY",
            _ => api_key_string = "",
        }
        builder = builder.api_key(std::env::var(api_key_string).unwrap_or(String::new()));
    }

    let llm = builder.build()?;

    let mut messages = vec![
        ChatMessage::user()
            .content("You are a friendly chatbot.")
            .build(),
    ];
    // for loop through the chat_history vec if it exists, chat_history[x].role => user() / assistant(), .message -> pass to .content()

    if let Some(history) = chat_history {
        for message in &history {
            let chat_msg = match message.role {
                Role::User => ChatMessage::user().content(message.content.clone()).build(),
                Role::Assistant => ChatMessage::assistant()
                    .content(message.content.clone())
                    .build(),
            };
            messages.push(chat_msg);
        }
        messages.push(ChatMessage::user().content(prompt).build());
    }

    match llm.chat(&messages).await {
        Ok(text) => Ok(text.to_string()),
        Err(e) => Ok(e.to_string()),
    }
}

pub async fn generate_chat_title(
    chat_history: Option<Vec<Message>>,
    settings: &AISettings,
) -> Result<String, Box<dyn std::error::Error>> {
    let prompt = "Given our past conversation, come up with an appropriate short title / topic for it. Just give the title, nothing else.";
    run_ai(chat_history, prompt, settings).await
}

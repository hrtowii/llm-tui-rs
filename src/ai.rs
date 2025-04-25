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
use anyhow::Result;
use llm::{
    builder::LLMBuilder, // Builder pattern components
    chat::ChatMessage,
};

pub async fn run_ai(
    chat_history: Option<&[Message]>,
    prompt: &str,
    settings: &AISettings,
) -> Result<String> {
    // check settings.json
    let mut builder = LLMBuilder::new()
        .backend(settings.backend.into())
        .model(&settings.model)
        .temperature(settings.temperature)
        .max_tokens(u32::try_from(settings.max_tokens)?);

    if let Some(key) = &settings.api_key {
        builder = builder.api_key(key);
    } else {
        if settings.backend == AIBackend::Ollama {
            builder = builder.base_url(
                std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".into()),
            );
        }
        builder = builder.api_key(
            std::env::var(settings.backend.to_env_var()).unwrap_or_else(|_| String::new()),
        );
    }

    let llm = builder.build()?;

    let mut messages = vec![
        ChatMessage::user()
            .content("You are a friendly chatbot.")
            .build(),
    ];
    // for loop through the chat_history vec if it exists, chat_history[x].role => user() / assistant(), .message -> pass to .content()

    if let Some(history) = chat_history {
        for message in history {
            let chat_msg = match message.role {
                Role::User => ChatMessage::user().content(&message.content).build(),
                Role::Assistant => ChatMessage::assistant().content(&message.content).build(),
            };
            messages.push(chat_msg);
        }
        messages.push(ChatMessage::user().content(prompt).build());
    }

    llm.chat(&messages)
        .await
        .map(|x| {
            if settings.backend == AIBackend::Google {
                x.text().unwrap()
            } else {
                x.to_string()
            }
        })
        .map_err(Into::into)
}

pub async fn generate_chat_title(
    chat_history: Option<&[Message]>,
    settings: &AISettings,
) -> Result<String> {
    let prompt = "Given our past conversation, come up with an appropriate short title / topic for it. Just give the title, nothing else.";
    run_ai(chat_history, prompt, settings).await
}

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
    }

    let llm = builder.build()?;

    //[TODO]: make the model and the backend configurable via settings

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
    // let messages = vec![
    //     ChatMessage::user()
    //         .content("Tell me that you love cats")
    //         .build(),
    //     ChatMessage::assistant()
    //         .content("I am an assistant, I cannot love cats but I can love dogs")
    //         .build(),
    //     ChatMessage::user()
    //         .content("Tell me that you love dogs in 2000 chars")
    //         .build(),
    // ];

    // Send chat request and handle the response
    match llm.chat(&messages).await {
        Ok(text) => Ok(text.to_string()),
        Err(e) => Ok(e.to_string()),
    }
}

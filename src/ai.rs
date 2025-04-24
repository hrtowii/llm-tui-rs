// use allms::{
//     llm::{
//         AnthropicModels, AwsBedrockModels, DeepSeekModels, GoogleModels, LLMModel, MistralModels,
//         OpenAIModels, PerplexityModels,
//     },
//     Completions,
// };
// https://github.com/graniet/llm/blob/main/examples/multi_backend_example.rs
use crate::chat_structs::{Message, Role};
use llm::{
    builder::{LLMBackend, LLMBuilder}, // Builder pattern components
    chat::ChatMessage,
};

pub async fn run_ai(
    chat_history: Option<Vec<Message>>,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let openai_llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key(std::env::var("OPENAI_API_KEY").unwrap_or("sk-OPENAI".into()))
        .model("gpt-4.1")
        .build()?;

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
    match openai_llm.chat(&messages).await {
        Ok(text) => Ok(text.to_string()),
        Err(e) => Ok(e.to_string()),
    }
}

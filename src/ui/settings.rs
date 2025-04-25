use crate::ai_backend::{AIBackend, AISettings};
use crate::app::CurrentScreen;
use crate::ui::MainMenu;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::path::PathBuf;

pub struct Config {
    pub ai_settings: AISettings,
    pub available_models: Vec<String>,
    // fetch the models somehow,
    // maybe v1/models but for every API? so each backend has to have a
    // model_api field
    pub selected_field: usize, // Track which setting is selected
    pub temp_input: String,    // Temporary buffer for temperature input
    pub tokens_input: String,  // Temporary buffer for max_tokens
}

impl Widget for &Config {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let fields = [
            format!("Backend: {:?}", self.ai_settings.backend),
            format!("Model: {}", self.ai_settings.model),
            format!(
                "API Key: {}",
                self.ai_settings.api_key.as_deref().unwrap_or("<none>")
            ),
            format!("Temperature: {}", self.temp_input),
            format!("Max Tokens: {}", self.tokens_input),
        ];

        let items: Vec<Line> = fields
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let style = if i == self.selected_field {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                Line::from(Span::styled(text, style))
            })
            .collect();

        Paragraph::new(items)
            .block(Block::default().title("Settings").borders(Borders::ALL))
            .render(area, buf);
    }
}

impl CurrentScreen {
    pub fn handle_settings(&mut self, key: KeyEvent) {
        let CurrentScreen::Settings(settings) = self else {
            return;
        };
        match key.code {
            KeyCode::Up => {
                settings.selected_field = settings.selected_field.saturating_sub(1);
            }
            KeyCode::Down => {
                settings.selected_field = (settings.selected_field + 1) % 5;
            }
            KeyCode::Left | KeyCode::Right if settings.selected_field == 0 => {
                // Cycle through backend options
                let backend = &mut settings.ai_settings.backend;
                *backend = match (&backend, key.code) {
                    (_, KeyCode::Left) => match backend {
                        AIBackend::OpenAI => AIBackend::Phind,
                        AIBackend::Anthropic => AIBackend::OpenAI,
                        AIBackend::Google => AIBackend::Anthropic,
                        AIBackend::Groq => AIBackend::Google,
                        AIBackend::Ollama => AIBackend::Groq,
                        AIBackend::XAi => AIBackend::Ollama,
                        AIBackend::Phind => AIBackend::XAi,
                    },
                    _ => match backend {
                        AIBackend::OpenAI => AIBackend::Anthropic,
                        AIBackend::Anthropic => AIBackend::Google,
                        AIBackend::Google => AIBackend::Groq,
                        AIBackend::Groq => AIBackend::Ollama,
                        AIBackend::Ollama => AIBackend::XAi,
                        AIBackend::XAi => AIBackend::Phind,
                        AIBackend::Phind => AIBackend::OpenAI,
                    },
                };
                AISettings::write_all(&PathBuf::from("settings.json"), &settings.ai_settings).ok();
            }
            KeyCode::Char(c) => {
                match settings.selected_field {
                    1 => settings.ai_settings.model.push(c),
                    2 => settings
                        .ai_settings
                        .api_key
                        .get_or_insert(String::new())
                        .push(c),
                    3 => settings.temp_input.push(c),
                    4 => settings.tokens_input.push(c),
                    _ => {}
                }
                // Update actual settings when valid
                if settings.selected_field == 3 {
                    if let Ok(temp) = settings.temp_input.parse() {
                        settings.ai_settings.temperature = temp;
                    }
                }
                if settings.selected_field == 4 {
                    if let Ok(tokens) = settings.tokens_input.parse() {
                        settings.ai_settings.max_tokens = tokens;
                    }
                }
                AISettings::write_all(&PathBuf::from("settings.json"), &settings.ai_settings).ok();
            }
            KeyCode::Backspace => {
                match settings.selected_field {
                    1 => _ = settings.ai_settings.model.pop(),
                    2 => _ = settings.ai_settings.api_key.as_mut().and_then(String::pop),
                    3 => _ = settings.temp_input.pop(),
                    4 => _ = settings.tokens_input.pop(),
                    _ => {}
                }
                AISettings::write_all(&PathBuf::from("settings.json"), &settings.ai_settings).ok();
            }
            KeyCode::Esc => {
                *self = CurrentScreen::MainMenu(MainMenu { selected: 0 });
            }
            _ => {}
        }
    }
}

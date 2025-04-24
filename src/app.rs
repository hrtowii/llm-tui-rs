use crate::ai::run_ai;
use crate::ai_backend::{AIBackend, AISettings};
use crate::chat_branch::ChatBranch;
use crate::chat_structs::{Message, Role};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::path::PathBuf;
// use std::collections::VecDeque;

pub enum CurrentScreen {
    MainMenu(MainMenu),
    ChatView(ChatView),
    Settings(Settings),
    Exit(Exit),
}

pub struct MainMenu {
    pub selected: usize,
}

pub struct ChatView {
    pub title: String,
    pub messages: Option<Vec<Message>>,
    pub input_buffer: String,
    // the sidebar fields:
    pub branches: Vec<ChatBranch>,
    pub selected_branch: usize,
    pub show_sidebar: bool,

    // where we persist them:
    pub storage_path: std::path::PathBuf,
}

pub struct Settings {
    pub ai_settings: AISettings,
    pub available_models: Vec<String>,
    // fetch the models somehow,
    // maybe v1/models but for every API? so each backend has to have a
    // model_api field
    pub selected_field: usize, // Track which setting is selected
    temp_input: String,        // Temporary buffer for temperature input
    tokens_input: String,      // Temporary buffer for max_tokens
}

pub struct Exit {
    pub data: String,
}

// -- Input Handling

impl CurrentScreen {
    pub async fn on_key(&mut self, key: KeyEvent) {
        match self {
            CurrentScreen::MainMenu(menu) => match key.code {
                KeyCode::Char('j') => {
                    menu.selected = (menu.selected + 1) % 3;
                }
                KeyCode::Char('k') => {
                    menu.selected = menu.selected.checked_sub(1).unwrap_or(2);
                }
                KeyCode::Enter => {
                    let storage_path = PathBuf::from("chats.json");
                    let mut branches = ChatBranch::load_all(&storage_path).unwrap();
                    if branches.is_empty() {
                        branches.push(ChatBranch {
                            id: 0,
                            name: "Default Chat".into(),
                            messages: Vec::new(),
                        })
                    }
                    *self = match menu.selected {
                        0 => {
                            let mut chat_view = ChatView {
                                title: "New Chat".to_string(),
                                input_buffer: String::new(),
                                branches,
                                selected_branch: 0,
                                show_sidebar: false,
                                messages: Some(Vec::new()),
                                storage_path,
                            };
                            // load messages for selected branch
                            chat_view.messages = Some(
                                chat_view.branches[chat_view.selected_branch]
                                    .messages
                                    .clone(),
                            );
                            CurrentScreen::ChatView(chat_view)
                        }
                        1 => {
                            let storage_path = PathBuf::from("settings.json");
                            let settings =
                                AISettings::load_all(&storage_path).unwrap_or(AISettings {
                                    backend: AIBackend::OpenAI,
                                    model: "gpt-3.5-turbo".to_string(),
                                    api_key: None,
                                    temperature: 0.7,
                                    max_tokens: 2048,
                                });
                            CurrentScreen::Settings(Settings {
                                ai_settings: settings,
                                available_models: vec![String::new()], // fetch the models somehow,
                                selected_field: 0,
                                temp_input: String::new(),
                                tokens_input: String::new(),
                            })
                        }
                        _ => CurrentScreen::Exit(Exit {
                            data: "Bye!".into(),
                        }),
                    };
                }
                _ => {}
            },
            CurrentScreen::ChatView(chat) => {
                let storage_path = PathBuf::from("settings.json");
                let settings = AISettings::load_all(&storage_path).unwrap_or(AISettings {
                    backend: AIBackend::OpenAI,
                    model: "gpt-3.5-turbo".to_string(),
                    api_key: None,
                    temperature: 0.7,
                    max_tokens: 2048,
                }); // sidebar selection
                if chat.show_sidebar {
                    // sidebar is active: j/k/Enter/Esc
                    match key.code {
                        KeyCode::Char('j') => {
                            chat.selected_branch = (chat.selected_branch + 1) % chat.branches.len();
                        }
                        KeyCode::Char('k') => {
                            chat.selected_branch = chat
                                .selected_branch
                                .checked_sub(1)
                                .unwrap_or(chat.branches.len() - 1);
                        }
                        KeyCode::Enter => {
                            // switch to that chat branch
                            chat.messages =
                                Some(chat.branches[chat.selected_branch].messages.clone());
                            chat.show_sidebar = false;
                        }
                        KeyCode::Tab | KeyCode::Esc => {
                            chat.show_sidebar = false;
                        }
                        _ => {}
                    }
                    return;
                }

                // normal chat view
                match key.code {
                    KeyCode::Tab => {
                        // toggle sidebar
                        chat.show_sidebar = true;
                    }
                    KeyCode::Char(c) => {
                        chat.input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        chat.input_buffer.pop();
                    }
                    KeyCode::Enter => {
                        let user_input = chat.input_buffer.trim();
                        if !user_input.is_empty() {
                            chat.messages.as_mut().unwrap().push(Message {
                                role: Role::User,
                                content: user_input.to_string(),
                            });
                            match run_ai(chat.messages.clone(), user_input, &settings).await {
                                Ok(reply) => {
                                    chat.messages.as_mut().unwrap().push(Message {
                                        role: Role::Assistant,
                                        content: reply,
                                    });
                                }
                                Err(e) => {
                                    chat.messages.as_mut().unwrap().push(Message {
                                        role: Role::Assistant,
                                        content: format!("AI Error: {}", e),
                                    });
                                }
                            }
                            // persist back to branch
                            if let Some(branch) = chat.branches.get_mut(chat.selected_branch) {
                                branch.messages = chat.messages.as_mut().unwrap().clone();
                                let _ = ChatBranch::save_all(&chat.storage_path, &chat.branches);
                            }
                            // Clear input
                            chat.input_buffer.clear();
                            // Optionally scroll up if too many
                            // if chat.messages.len() > 100 {
                            //     chat.messages.pop_front();
                            // }
                        }
                    }
                    KeyCode::Esc => {
                        *self = CurrentScreen::MainMenu(MainMenu { selected: 0 });
                    }
                    _ => {}
                }
            }
            CurrentScreen::Settings(settings) => match key.code {
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
                    AISettings::write_all(&PathBuf::from("settings.json"), &settings.ai_settings)
                        .ok();
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
                    AISettings::write_all(&PathBuf::from("settings.json"), &settings.ai_settings)
                        .ok();
                }
                KeyCode::Backspace => {
                    match settings.selected_field {
                        1 => _ = settings.ai_settings.model.pop(),
                        2 => _ = settings.ai_settings.api_key.as_mut().and_then(|k| k.pop()),
                        3 => _ = settings.temp_input.pop(),
                        4 => _ = settings.tokens_input.pop(),
                        _ => {}
                    }
                    AISettings::write_all(&PathBuf::from("settings.json"), &settings.ai_settings)
                        .ok();
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    *self = CurrentScreen::MainMenu(MainMenu { selected: 0 });
                }
                _ => {}
            },
            CurrentScreen::Exit(_) => {}
        }
    }
}

// https://forum.ratatui.rs/t/multiple-screens-in-ratatui/82/4
// Implement Widget for each screen type

use rascii_art::{RenderOptions, render_to};
impl Widget for &MainMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut buffer = String::new();
        // 1) Define your menu labels in the same order as `selected` (0,1,2)
        let menu_labels = ["Chat View", "Settings", "Exit"];

        // 2) Turn each label into a line, prefixing the selected one with ">>"
        let lines: Vec<Line> = menu_labels
            .iter()
            .enumerate()
            .map(|(idx, label)| {
                let prefix = if idx == self.selected { ">>" } else { "  " };
                Line::from(Span::raw(format!("{} {}", prefix, label)))
            })
            .collect();

        // 3) Create a Paragraph from those lines, add a border/title, and render it.
        render_to(
            r"/Users/ibarahime/dev/llm-tui-rs/src/wingedstrawberry.png",
            &mut buffer,
            &RenderOptions::new().width(50).colored(false), // .charset(&[".", ",", "-", "*", "£", "$", "#"]),
        )
        .unwrap();
        Paragraph::new(lines).render(area, buf);
        Paragraph::new(buffer).render(area, buf);
    }
}
impl Widget for &ChatView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Layout:  [messages box]
        //          [input box]
        // [TODO]: sidebar component containing past chats. storage -> ??? idk
        // if sidebar is ON, split horizontally:
        // Decide whether we need to carve off a left‐hand pane.
        let chat_area = if self.show_sidebar {
            // 1) split horizontally: left is 30 cols, right is rest
            let h = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(30), Constraint::Min(0)])
                .split(area);

            // 2) Render sidebar in h[0]
            let items: Vec<Line> = self
                .branches
                .iter()
                .enumerate()
                .map(|(i, branch)| {
                    let prefix = if i == self.selected_branch {
                        "▶"
                    } else {
                        " "
                    };
                    Line::from(Span::raw(format!("{} {}", prefix, branch.name)))
                })
                .collect();

            Paragraph::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Chats (j/k, Enter)"),
                )
                .render(h[0], buf);

            // Return the *right* pane as the actual chat area
            h[1]
        } else {
            // No sidebar: the full area is our chat area
            area
        };

        // Now split chat_area vertically into messages + input
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(chat_area);
        // Message area: render each message as one line, distinguishing User/AI
        let mut lines = Vec::new();
        if let Some(messages) = &self.messages {
            for msg in messages {
                let prefix = match msg.role {
                    Role::User => "You: ",
                    Role::Assistant => "Assistant: ",
                };
                if let Role::Assistant = msg.role {
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Assistant: ",
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(&msg.content),
                    ]));
                } else {
                    lines.push(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(Color::Green)),
                        Span::raw(&msg.content),
                    ]));
                }
            }
        }

        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title.clone()),
            )
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((
                (if let Some(messages) = &self.messages {
                    if messages.len() > (chunks[0].height as usize) {
                        messages.len() - (chunks[0].height as usize)
                    } else {
                        0
                    }
                } else {
                    0
                }) as u16,
                0,
            ))
            .render(chunks[0], buf);

        // Input area: always bottom
        let input_line = format!("> {}", self.input_buffer);
        Paragraph::new(vec![Line::from(input_line)])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Input (Esc=back, tab=sidebar)"),
            )
            .render(chunks[1], buf);
    }
}

impl Widget for &Settings {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let fields = vec![
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
            .block(
                Block::default()
                    .title("Settings (q to quit)")
                    .borders(Borders::ALL),
            )
            .render(area, buf);
    }
}

// impl Widget for &Exit {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         Paragraph::new(self.data.as_str())
//             .block(Block::default().title("Settings").borders(Borders::ALL))
//             .render(area, buf);
//     }
// }

// Implement Widget for the CurrentScreen enum
impl Widget for &CurrentScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // You can render common elements here (like a header or footer)
        // For example:
        let header_area = Rect { height: 1, ..area };
        Paragraph::new("llm-tui :3").render(header_area, buf);

        let content_area = Rect {
            y: area.y + 1,
            height: area.height - 1,
            ..area
        };

        // Then delegate to the specific screen
        match self {
            CurrentScreen::MainMenu(screen) => screen.render(content_area, buf),
            CurrentScreen::ChatView(screen) => screen.render(content_area, buf),
            CurrentScreen::Settings(screen) => screen.render(content_area, buf),
            CurrentScreen::Exit(..) => (),
        }
    }
}

// pub struct Project {
//     pub name: String,
//     pub chats: Vec<ChatBranch>,
// }
//
// pub struct ChatBranch {
//     pub root: Chat,
//     pub current: ChatId,
//     pub branches: HashMap<ChatId, Chat>,
// }
//
// pub struct Chat {
//     pub id: ChatId,
//     pub messages: Vec<Message>,
//     pub parent: Option<ChatId>,
// }

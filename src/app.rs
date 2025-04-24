use crate::ai::{generate_chat_title, run_ai};
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
use tui_markdown::from_str;

pub enum CurrentScreen {
    MainMenu(MainMenu),
    ChatView(ChatView),
    Settings(Config),
    Exit(Exit),
}

pub struct MainMenu {
    pub selected: usize,
}

pub enum SidebarInputMode {
    NewBranch, // User is naming a new branch
    Renaming,  // User is renaming an existing branch
}

pub struct ChatView {
    pub messages: Option<Vec<Message>>,
    pub input_buffer: String,
    // the sidebar fields:
    pub branches: Vec<ChatBranch>,
    pub selected_branch: usize,
    pub show_sidebar: bool,

    // where we persist them:
    pub storage_path: PathBuf,
    // renaming and creating new chat branches
    pub sidebar_input_mode: Option<SidebarInputMode>,
    pub sidebar_input_buffer: String,
}

pub struct Config {
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
    fn handle_main_menu(&mut self, key: KeyEvent) {
        let CurrentScreen::MainMenu(menu) = self else {
            return
        };
        match key.code {
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
                        name: "Default Chat".to_string(),
                        messages: Vec::new(),
                    });
                }
                *self = match menu.selected {
                    0 => {
                        let mut chat_view = ChatView {
                            input_buffer: String::new(),
                            branches,
                            selected_branch: 0,
                            show_sidebar: false,
                            messages: Some(Vec::new()),
                            storage_path,
                            sidebar_input_mode: None,
                            sidebar_input_buffer: String::new(),
                        };
                        // load messages for selected branch
                        chat_view.messages = Some(
                            chat_view.branches[chat_view.selected_branch]
                                .messages.clone()
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
                        CurrentScreen::Settings(Config {
                            ai_settings: settings,
                            available_models: vec![String::new()], // fetch the models somehow,
                            selected_field: 0,
                            temp_input: String::new(),
                            tokens_input: String::new(),
                        })
                    }
                    _ => CurrentScreen::Exit(Exit {
                        data: "Bye!".to_string(),
                    }),
                }
            },
            _ => {}
        }
    }

    fn handle_settings(&mut self, key: KeyEvent) {
        let CurrentScreen::Settings(settings) = self else {
            return
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
                    }
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
                    2 => _ = settings.ai_settings.api_key.as_mut().and_then(String::pop),
                    3 => _ = settings.temp_input.pop(),
                    4 => _ = settings.tokens_input.pop(),
                    _ => {}
                }
                AISettings::write_all(&PathBuf::from("settings.json"), &settings.ai_settings)
                    .ok();
            }
            KeyCode::Esc => {
                *self = CurrentScreen::MainMenu(MainMenu { selected: 0 });
            }
            _ => {}
        }
    }

    fn handle_chat_view_sidebar(chat: &mut ChatView, key: KeyEvent) -> bool {
        if let Some(input_mode) = &mut chat.sidebar_input_mode {
            // Handle input for renaming or new branch
            match key.code {
                KeyCode::Enter => {
                    let new_name = chat.sidebar_input_buffer.trim();
                    if !new_name.is_empty() {
                        match input_mode {
                            SidebarInputMode::NewBranch => {
                                // Create new branch with custom name
                                let new_branch = ChatBranch {
                                    id: chat.branches.len(),
                                    name: new_name.to_string(),
                                    messages: Vec::new(),
                                };
                                chat.branches.push(new_branch);
                                chat.selected_branch = chat.branches.len() - 1;
                                chat.messages = Some(Vec::new());
                                ChatBranch::save_all(
                                    &chat.storage_path,
                                    &chat.branches,
                                ).unwrap();
                            }
                            SidebarInputMode::Renaming => {
                                // Rename selected branch
                                if let Some(branch) =
                                    chat.branches.get_mut(chat.selected_branch)
                                {
                                    branch.name = new_name.to_string();
                                    ChatBranch::save_all(
                                        &chat.storage_path,
                                        &chat.branches,
                                    ).unwrap();
                                }
                            }
                        }
                    }
                    // Reset input mode
                    chat.sidebar_input_mode = None;
                    chat.sidebar_input_buffer.clear();
                }
                KeyCode::Char(c) => chat.sidebar_input_buffer.push(c),
                KeyCode::Backspace => {
                    chat.sidebar_input_buffer.pop();
                }
                KeyCode::Esc => {
                    chat.sidebar_input_mode = None;
                    chat.sidebar_input_buffer.clear();
                }
                _ => {}
            }
            false
        } else {
            // sidebar is active: j/k/Enter/Esc/n
            match key.code {
                KeyCode::Char('j') => {
                    chat.selected_branch =
                        (chat.selected_branch + 1) % chat.branches.len();
                }
                KeyCode::Char('k') => {
                    chat.selected_branch = chat
                        .selected_branch
                        .checked_sub(1)
                        .unwrap_or(chat.branches.len() - 1);
                }
                KeyCode::Enter => {
                    // switch to that chat branch
                    chat.messages = Some( // clone, otherwise we get a self-referential struct
                        chat.branches[chat.selected_branch].messages.clone()
                    );
                    chat.show_sidebar = false;
                }
                KeyCode::Char('n') => {
                    chat.sidebar_input_mode = Some(SidebarInputMode::NewBranch);
                    chat.sidebar_input_buffer.clear();
                }
                KeyCode::Char('r') => {
                    // Start renaming (if branches exist)
                    if !chat.branches.is_empty() {
                        chat.sidebar_input_mode = Some(SidebarInputMode::Renaming);
                        chat.sidebar_input_buffer =
                            chat.branches[chat.selected_branch].name.to_string();
                    }
                }
                KeyCode::Tab | KeyCode::Esc => {
                    chat.show_sidebar = false;
                }
                _ => {}
            }
            true
        }
    }

    async fn handle_chat_view(&mut self, key: KeyEvent) {
        let CurrentScreen::ChatView(chat) = self else {
            return
        };
        let storage_path = PathBuf::from("settings.json");
        let settings = AISettings::load_all(&storage_path).unwrap_or(AISettings {
            backend: AIBackend::OpenAI,
            model: "gpt-3.5-turbo".to_string(),
            api_key: None,
            temperature: 0.7,
            max_tokens: 2048,
        }); // sidebar selection
        if chat.show_sidebar && Self::handle_chat_view_sidebar(chat, key) {
            return;
        }

        // normal chat view
        match key.code {
            KeyCode::Tab => {
                // toggle sidebar
                chat.show_sidebar = true;
            }
            KeyCode::Char(c) => {
                if !chat.show_sidebar {
                    chat.input_buffer.push(c);
                }
            }
            KeyCode::Backspace => {
                chat.input_buffer.pop();
            }
            KeyCode::Enter => {
                if !chat.show_sidebar {
                    let user_input = chat.input_buffer.trim();
                    if !user_input.is_empty() {
                        chat.messages.as_mut().unwrap().push(Message {
                            role: Role::User,
                            content: user_input.to_string(),
                        });
                        let content = match run_ai(chat.messages.as_deref(), user_input, &settings).await {
                            Ok(reply) => reply,
                            Err(e) => format!("AI Error: {e}")
                        };
                        chat.messages.as_mut().unwrap().push(
                            Message {
                                role: Role::Assistant,
                                content
                            }
                        );
                        // persist back to branch
                        if let Some(branch) = chat.branches.get_mut(chat.selected_branch) {
                            branch.messages = chat.messages.as_deref().unwrap().to_vec();
                            // idk how to make this behavior tbh
                            if branch.name == "Default Chat" || branch.name.is_empty() {
                                if let Ok(new_title) = generate_chat_title(
                                    Some(&branch.messages),
                                    &settings,
                                ).await {
                                    branch.name = new_title;
                                }
                            }
                        }

                        ChatBranch::save_all(&chat.storage_path, &chat.branches).unwrap();
                        // Clear input
                        chat.input_buffer.clear();
                        // Optionally scroll up if too many
                        // if chat.messages.len() > 100 {
                        //     chat.messages.pop_front();
                        // }
                    }
                }
            }
            KeyCode::Esc => {
                *self = CurrentScreen::MainMenu(MainMenu { selected: 0 });
            }
            _ => {}
        }
    }

    pub async fn on_key(&mut self, key: KeyEvent) {
        match self {
            CurrentScreen::MainMenu(_) => self.handle_main_menu(key),
            CurrentScreen::ChatView(_) => self.handle_chat_view(key).await,
            CurrentScreen::Settings(_) => self.handle_settings(key),
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
                Line::from(Span::raw(format!("{prefix} {label}")))
            })
            .collect();

        // 3) Create a Paragraph from those lines, add a border/title, and render it.
        render_to(
            r"wingedstrawberry.png",
            &mut buffer,
            &RenderOptions::new().width(50).colored(false), // .charset(&[".", ",", "-", "*", "£", "$", "#"]),
        )
        .unwrap();
        Paragraph::new(lines).render(area, buf);
        // Paragraph::new(buffer).render(area, buf);
    }
}

fn iter_messages<'a>(messages: &'a [Message], lines: &mut Vec<Line<'a>>) {
    for msg in messages {
        let prefix = match msg.role {
            Role::User => Span::styled(
                "You: ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Role::Assistant => Span::styled(
                "Assistant: ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        };

        let markdown = from_str(&msg.content);
        // idk how this works but i like deepseek
        // Text contains Lines which contains Spans, so loop through the lines and add the spans to the string.
        for (i, line) in markdown.lines.into_iter().enumerate() {
            let mut spans = Vec::with_capacity(line.spans.len() + 1);

            // Add prefix only to the first line
            if i == 0 {
                spans.push(prefix.clone());
            } else {
                // Add indentation for wrapped lines
                spans.push(Span::from("".repeat(prefix.width())));
            }

            spans.extend(line.spans);
            lines.push(Line::from(spans));
        }
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
                    let is_selected = i == self.selected_branch;
                    let prefix = if is_selected { "▶" } else { " " };

                    let mut text = format!("{} {}", prefix, branch.name);
                    let mut style = Style::default();

                    if is_selected
                        && matches!(self.sidebar_input_mode, Some(SidebarInputMode::Renaming))
                    {
                        text = format!("{} {}", prefix, self.sidebar_input_buffer);
                        style = style.bg(Color::DarkGray);
                    }

                    Line::from(Span::styled(text, style))
                })
                .collect();

            let mut all_lines = items;

            if let Some(SidebarInputMode::NewBranch) = self.sidebar_input_mode {
                let input_line = Line::from(Span::styled(
                    format!("▶ {}", self.sidebar_input_buffer),
                    Style::default().bg(Color::DarkGray),
                ));
                all_lines.push(input_line);
            }

            Paragraph::new(all_lines)
                .block(Block::default().borders(Borders::ALL).title(
                    match self.sidebar_input_mode {
                        Some(SidebarInputMode::NewBranch) => "New Chat (Enter: save, Esc: cancel)",
                        Some(SidebarInputMode::Renaming) => "Renaming (Enter: save, Esc: cancel)",
                        None => "Chats (n: new, r: rename)",
                    },
                ))
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
            iter_messages(messages, &mut lines);
        }

        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.branches[self.selected_branch].name.as_str()),
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
            format!("Max Tokens: {}", self.tokens_input)
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
            CurrentScreen::Exit(_) => (),
        }
    }
}

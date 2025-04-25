use crate::CurrentScreen;
use crate::ai::{generate_chat_title, run_ai};
use crate::ai_backend::{AIBackend, AISettings};
use crate::chat_branch::ChatBranch;
use crate::chat_structs::{Message, Role};
use crate::ui::MainMenu;
use anyhow::{Context, Result, bail};
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

    pub scroll: usize,
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

                    let mut text = format!("{prefix} {branch}", branch = branch.name);
                    let mut style = Style::default();

                    if is_selected
                        && matches!(self.sidebar_input_mode, Some(SidebarInputMode::Renaming))
                    {
                        text = format!("{prefix} {input}", input = self.sidebar_input_buffer);
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
                u16::try_from(
                    self.messages
                        .as_ref()
                        .and_then(|x| x.len().checked_sub(chunks[0].height as usize))
                        .unwrap_or(self.scroll),
                )
                .unwrap(),
                self.scroll.try_into().unwrap(),
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

impl CurrentScreen {
    pub fn handle_chat_view_sidebar(chat: &mut ChatView, key: KeyEvent) -> Result<bool> {
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
                                ChatBranch::save_all(&chat.storage_path, &chat.branches)?;
                            }
                            SidebarInputMode::Renaming => {
                                // Rename selected branch
                                if let Some(branch) = chat.branches.get_mut(chat.selected_branch) {
                                    branch.name = new_name.to_string();
                                    ChatBranch::save_all(&chat.storage_path, &chat.branches)?;
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
            Ok(false)
        } else {
            // sidebar is active: j/k/Enter/Esc/n
            match key.code {
                KeyCode::Char('j') => {
                    chat.selected_branch = (chat.selected_branch + 1) % chat.branches.len();
                }
                KeyCode::Char('k') => {
                    chat.selected_branch =
                        (chat.branches.len() + chat.selected_branch - 1) % chat.branches.len();
                }
                KeyCode::Enter => {
                    // switch to that chat branch
                    chat.messages = Some(
                        // clone, otherwise we get a self-referential struct
                        chat.branches[chat.selected_branch].messages.clone(),
                    );
                    chat.show_sidebar = false;
                }
                KeyCode::Char('n') => {
                    chat.sidebar_input_mode = Some(SidebarInputMode::NewBranch);
                    // chat.sidebar_input_buffer.clear();
                    chat.sidebar_input_buffer = "Default Chat".to_string();
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
            Ok(true)
        }
    }

    pub async fn handle_chat_view(&mut self, key: KeyEvent) -> Result<()> {
        let CurrentScreen::ChatView(chat) = self else {
            bail!("Not in chat view");
        };
        let storage_path = PathBuf::from("settings.json");
        let settings = AISettings::load_all(&storage_path).unwrap_or(AISettings {
            backend: AIBackend::OpenAI,
            model: "gpt-3.5-turbo".to_string(),
            api_key: None,
            temperature: 0.7,
            max_tokens: 2048,
        }); // sidebar selection
        if chat.show_sidebar && Self::handle_chat_view_sidebar(chat, key)? {
            return Ok(());
        }

        // normal chat view
        match key.code {
            KeyCode::Tab => {
                // toggle sidebar
                chat.show_sidebar = true;
            }
            KeyCode::Up => {
                if chat.scroll >= 5 {
                    chat.scroll -= 5;
                }
            }
            KeyCode::Down => {
                chat.scroll += 5;
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
                        chat.messages
                            .as_mut()
                            .context("No messages found")?
                            .push(Message {
                                role: Role::User,
                                content: user_input.to_string(),
                            });
                        let content =
                            match run_ai(chat.messages.as_deref(), user_input, &settings).await {
                                Ok(reply) => reply,
                                Err(e) => format!("AI Error: {e}"),
                            };
                        chat.messages
                            .as_mut()
                            .context("No messages found")?
                            .push(Message {
                                role: Role::Assistant,
                                content,
                            });
                        // persist back to branch
                        if let Some(branch) = chat.branches.get_mut(chat.selected_branch) {
                            branch.messages = chat
                                .messages
                                .as_deref()
                                .context("No messages found")?
                                .to_vec();
                            // idk how to make this behavior tbh
                            if branch.name == "Default Chat" || branch.name.is_empty() {
                                if let Ok(new_title) =
                                    generate_chat_title(Some(&branch.messages), &settings).await
                                {
                                    branch.name = new_title;
                                }
                            }
                        }

                        ChatBranch::save_all(&chat.storage_path, &chat.branches)?;
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
        Ok(())
    }
}

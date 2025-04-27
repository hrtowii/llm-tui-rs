use crate::{
    ai_backend::{AIBackend, AISettings},
    app::{CurrentScreen, Exit},
    chat_branch::ChatBranch,
    ui::{ChatView, Config},
};
use anyhow::{Result, bail};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use std::path::PathBuf;
use tokio::sync::mpsc::unbounded_channel;

pub struct MainMenu {
    pub selected: usize,
}

impl Widget for &MainMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let mut buffer = String::new();
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
        // render_to(
        //     r"wingedstrawberry.png",
        //     &mut buffer,
        //     &RenderOptions::new().width(50).colored(false), // .charset(&[".", ",", "-", "*", "£", "$", "#"]),
        // )
        // .unwrap();
        Paragraph::new(lines).render(area, buf);
        // Paragraph::new(buffer).render(area, buf);
    }
}

impl CurrentScreen {
    pub fn handle_main_menu(&mut self, key: KeyEvent) -> Result<()> {
        let CurrentScreen::MainMenu(menu) = self else {
            bail!("Not in main menu");
        };
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                menu.selected = (menu.selected + 1) % 3;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                menu.selected = (3 + menu.selected - 1) % 3;
            }
            KeyCode::Enter => {
                let storage_path = PathBuf::from("chats.json");
                let mut branches = ChatBranch::load_all(&storage_path)?;
                if branches.is_empty() {
                    branches.push(ChatBranch {
                        id: 0,
                        name: "Default Chat".to_string(),
                        messages: Vec::new(),
                    });
                }
                *self = match menu.selected {
                    0 => {
                        let (ai_tx, ai_rx) = unbounded_channel::<ChatBranch>();

                        let mut chat_view = ChatView {
                            input_buffer: String::new(),
                            branches,
                            selected_branch: 0,
                            show_sidebar: false,
                            messages: Some(Vec::new()),
                            storage_path,
                            sidebar_input_mode: None,
                            sidebar_input_buffer: String::new(),
                            scroll: 0,
                            ai_tx,
                            ai_rx,
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
                        let settings = AISettings::load_all(&storage_path).unwrap_or(AISettings {
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
            }
            _ => {}
        }
        Ok(())
    }
}

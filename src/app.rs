use crate::ui::{ChatView, Config, MainMenu};
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget},
};

use anyhow::Result;

pub enum CurrentScreen {
    MainMenu(MainMenu),
    ChatView(ChatView),
    Settings(Config),
    Exit(Exit),
}

pub struct Exit {
    pub data: String,
}

// -- Input Handling

impl CurrentScreen {
    pub async fn on_key(&mut self, key: KeyEvent) -> Result<()> {
        match self {
            CurrentScreen::MainMenu(_) => self.handle_main_menu(key)?,
            CurrentScreen::ChatView(_) => self.handle_chat_view(key).await?,
            CurrentScreen::Settings(_) => self.handle_settings(key),
            CurrentScreen::Exit(_) => {}
        }
        Ok(())
    }
}

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

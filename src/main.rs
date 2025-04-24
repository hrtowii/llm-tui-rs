#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{DefaultTerminal, Frame};
mod ai;
mod ai_backend;
mod app;
mod chat_branch;
mod chat_structs;
use app::CurrentScreen;
use app::MainMenu;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal).await;
    ratatui::restore();
    result
}
async fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut current_screen = CurrentScreen::MainMenu(MainMenu { selected: 0 });
    loop {
        terminal.draw(|f| render(f, &current_screen))?;
        if let Event::Key(key_event) = event::read()? {
            // universal quit on 'q'
            if key_event.code == KeyCode::Char('q') {
                break Ok(());
            }
            // delegate to the current screen
            current_screen.on_key(key_event).await;
        }
    }
}

fn render(frame: &mut Frame, current_screen: &CurrentScreen) {
    frame.render_widget(current_screen, frame.area());
}

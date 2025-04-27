#![warn(clippy::all, clippy::pedantic)]

use color_eyre::{Result, eyre::eyre};
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame};
mod ai;
mod ai_backend;
mod app;
mod chat_branch;
mod chat_structs;
use app::CurrentScreen;
mod ui;
use std::sync::{Arc, Mutex};
use tokio::{
    task,
    time::{Duration, interval},
};
use ui::MainMenu;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal).await;
    ratatui::restore();
    result
}

async fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // let mut current_screen = CurrentScreen::MainMenu(MainMenu { selected: 0 });
    // okay so I have to run drain_ai every second in a different thread and share its data without blowing up, how do I do this?
    // https://itsallaboutthebit.com/arc-mutex/
    let shared = Arc::new(Mutex::new(CurrentScreen::MainMenu(MainMenu {
        selected: 0,
    })));

    // spawn the “drainer” task
    {
        let shared = Arc::clone(&shared);
        task::spawn(async move {
            let mut ticker = interval(Duration::from_millis(10));
            loop {
                ticker.tick().await;
                let mut guard = shared.lock().unwrap();
                if let CurrentScreen::ChatView(chat) = &mut *guard {
                    let _ = chat.drain_ai();
                }
            }
        });
    }
    loop {
        // terminal.draw(|f| render(f, &current_screen))?;
        {
            let guard = shared.lock().unwrap();
            terminal.draw(|f| render(f, &*guard))?;
        }
        // if let Event::Key(key_event) = event::read()? {
        // ^^^ this makes it block for the next keypress, so new draws / updated structs will block until a key is pressed, no good
        // fix: poll for keypresses, fall to next draw after 50 ms

        // delegate to the current screen
        if crossterm::event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = crossterm::event::read()? {
                let mut guard = shared.lock().unwrap();
                guard
                    .on_key(key_event)
                    .await
                    .map_err(|err| eyre!(Box::new(err)))?;
                if let CurrentScreen::Exit(_) = &*guard {
                    break Ok(());
                }
            }
        }
    }
}

fn render(frame: &mut Frame, current_screen: &CurrentScreen) {
    frame.render_widget(current_screen, frame.area());
}

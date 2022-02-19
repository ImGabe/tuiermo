use crate::app::{App, AppResult, InputMode};
use crossterm::event::{Event, KeyCode, KeyEvent};
use tui_input::backend::crossterm as input_backend;
use tui_input::{Input, InputResponse};
use unidecode::unidecode;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.input_mode {
        InputMode::Normal => handle_key_events_normal(key_event, app),
        InputMode::Editing => handle_key_events_insert(key_event, app),
    }

    Ok(())
}

fn handle_key_events_normal(key_event: KeyEvent, app: &mut App) {
    match key_event.code {
        // Press 'e' to enter edit mode.
        KeyCode::Char('e') => {
            app.input_mode = InputMode::Editing;
        }
        // Press 'q' to exit.
        KeyCode::Char('q') => {
            app.running = false;
        }
        _ => {}
    }
}
fn handle_key_events_insert(key_event: KeyEvent, app: &mut App) {
    let resp = input_backend::to_input_request(Event::Key(key_event))
        .and_then(|req| app.input.handle(req));

    match resp {
        Some(InputResponse::StateChanged(_)) => {}
        Some(InputResponse::Submitted) => {
            let input: String = unidecode(app.input.value().into()).to_lowercase();

            if input.len() != 5 {
                return;
            }

            app.guesses.push(input);
            app.input = Input::default();
        }
        Some(InputResponse::Escaped) => {
            app.input_mode = InputMode::Normal;
        }
        None => {}
    }
}

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, str::Chars};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use tui_input::backend::crossterm as input_backend;
use tui_input::{Input, InputResponse};
use unidecode::unidecode;

enum InputMode {
    Normal,
    Editing,
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: Input,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    wordles: Vec<String>,
    // Answer
    wordle: String,
}

impl Default for App {
    fn default() -> App {
        App {
            input: Input::default(),
            input_mode: InputMode::Normal,
            wordles: Vec::new(),
            wordle: get_random_word(),
        }
    }
}

// read file and get one random word
fn get_random_word() -> String {
    unidecode("acaso")
}

// // read file and get one random word

// fn get_random_word() -> String {
//     let mut file = File::open("words.txt").unwrap();
//     let mut buffer = String::new();
//     file.read_to_string(&mut buffer).unwrap();
//     let words: Vec<&str> = buffer.split_whitespace().collect();
//     let mut rng = rand::thread_rng();
//     let index = rng.gen_range(0..=words.len());

//     words[index].to_string()
// }

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::default();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    // Press 'e' to enter edit mode.
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    // Press 'q' to exit.
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing => {
                    let resp = input_backend::to_input_request(Event::Key(key))
                        .and_then(|req| app.input.handle(req));

                    match resp {
                        Some(InputResponse::StateChanged(_)) => {}
                        Some(InputResponse::Submitted) => {
                            let input: String = unidecode(app.input.value().into()).to_lowercase();

                            if input.len() != 5 {
                                continue;
                            }

                            app.wordles.push(input);
                            app.input = Input::default();
                        }
                        Some(InputResponse::Escaped) => {
                            app.input_mode = InputMode::Normal;
                        }
                        None => {}
                    }
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Pressione "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" para sair, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" para começar a escrever."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Pressione "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" para parar de escrever, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" para enviar o \"tuiermo\"."),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);

    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = (app.input.cursor() as u16).max(width) - width;
    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .scroll((0, scroll))
        .block(Block::default().borders(Borders::ALL).title("Tuiermo"));
    f.render_widget(input, chunks[1]);

    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + (app.input.cursor() as u16).min(width) + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let messages: Vec<ListItem> = app
        .wordles
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let is_correct = compare_arrays(&app.wordle, m.chars());

            let content = if is_correct {
                // let size = f.size();
                // let block = Block::default().title("Block").borders(Borders::ALL);
                // f.render_widget(block, chunks[1]);

                vec![Spans::from(vec![
                    Span::raw(format!("{}: ", i)),
                    Span::styled(m, Style::default().fg(Color::Green)),
                    Span::raw(" "),
                    Span::styled("✔", Style::default().fg(Color::Green)),
                ])]
            } else {
                let mut spans = vec![Span::raw(format!("{}: ", i))];
                let mut word = guess_status(&app.wordle, m);

                spans.append(&mut word);

                vec![Spans::from(spans)]
            };

            ListItem::new(content)
        })
        .collect();

    let messages = List::new(messages).block(
        Block::default()
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .title("Tuiermos"),
    );
    f.render_widget(messages, chunks[2]);
}

fn guess_status<'a>(word: &str, guess: &str) -> Vec<Span<'a>> {
    let mut word_clone = word.to_string();
    let mut list: [Span; 5] = [
        Span::raw(" "),
        Span::raw(" "),
        Span::raw(" "),
        Span::raw(" "),
        Span::raw(" "),
    ];

    for (i, letter) in guess.chars().enumerate() {
        if !word.contains(letter) {
            list[i] = Span::styled(letter.to_string(), Style::default().fg(Color::Gray));
            continue;
        }

        if letter == word.chars().nth(i).unwrap() {
            list[i] = Span::styled(letter.to_string(), Style::default().fg(Color::Green));
            word_clone = word_clone.replace(letter, "");
        }
    }

    for (i, letter) in guess.chars().enumerate() {
        if word_clone.contains(letter) {
            list[i] = Span::styled(letter.to_string(), Style::default().fg(Color::Yellow));
            word_clone = word_clone.replace(letter, "");
        }

        if list[i] == Span::raw(" ") {
            list[i] = Span::styled(letter.to_string(), Style::default().fg(Color::Gray));
        }
    }

    list.to_vec()
}

// compare array of chars
fn compare_arrays(a: &str, b: Chars) -> bool {
    if a == b.as_str() {
        return true;
    }

    false
}

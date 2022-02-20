use rand::Rng;
use std::{
    error,
    fs::File,
    io::{Error, Read},
};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use tui_input::Input;
use unidecode::unidecode;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    /// Current value of the input box
    pub input: Input,
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub guesses: Vec<String>,
    /// Answer
    pub wordle: String,
    /// The game is running
    pub running: bool,
}

impl Default for App {
    fn default() -> App {
        App {
            input: Input::default(),
            input_mode: InputMode::Normal,
            guesses: Vec::new(),
            wordle: match get_random_word() {
                Ok(word) => word,
                Err(err) => {
                    panic!("Não foi possível obter uma palavra aleatória: {}", err);
                }
            },
            running: true,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Renders the user interface widgets.
    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
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
            .split(frame.size());

        let (msg, style) = match self.input_mode {
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
        frame.render_widget(help_message, chunks[0]);

        let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor
        let scroll = (self.input.cursor() as u16).max(width) - width;
        let input = Paragraph::new(self.input.value())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .scroll((0, scroll))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .title("Tuiermo"),
            );
        frame.render_widget(input, chunks[1]);

        match self.input_mode {
            InputMode::Normal =>
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                {}

            InputMode::Editing => {
                // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                frame.set_cursor(
                    // Put cursor past the end of the input text
                    chunks[1].x + (self.input.cursor() as u16).min(width) + 1,
                    // Move one line down, from the border to the input line
                    chunks[1].y + 1,
                )
            }
        }

        let messages: Vec<ListItem> = self
            .guesses
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let wordle_unicode = unidecode(&self.wordle);
                let is_correct = &wordle_unicode == m;

                let content = if is_correct {
                    vec![Spans::from(vec![
                        Span::raw(format!("{}: ", i)),
                        Span::styled(m, Style::default().fg(Color::Green)),
                        Span::raw(" "),
                        Span::styled("✔", Style::default().fg(Color::Green)),
                    ])]
                } else {
                    let mut spans = vec![Span::raw(format!("{}: ", i))];
                    let mut word = guessing_status(&wordle_unicode, m);

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
        frame.render_widget(messages, chunks[2]);
    }
}

// read file and get one random word

// fn get_random_word() -> Result<String, Error> {
//     let mut file = File::open("words.txt")?;
//     let mut buffer = String::new();
//     file.read_to_string(&mut buffer)?;
//     let words: Vec<&str> = buffer.split_whitespace().collect();
//     let mut rng = rand::thread_rng();
//     let index = rng.gen_range(0..=words.len());

//     Ok(words[index].to_string())
// }

fn get_random_word() -> Result<String, Error> {
    Ok("áéíóú".to_string())
}

fn guessing_status<'a>(word: &str, guess: &str) -> Vec<Span<'a>> {
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

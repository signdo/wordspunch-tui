use std::{
    collections::HashMap, error::Error, fs, path::PathBuf
};

use ratatui::{
    prelude::*,
    backend::Backend,
    Frame, Terminal
};
use text::ToText;

use crate::{
    data::LearningData,
    word::{Level, Word}
};

#[derive(Debug)]
pub struct App {
    title: String,
    data_path: PathBuf,
    learning_data: LearningData,
    words_map: HashMap<String, Word>,
    cur_word: (String, Word),
    cur_show_chinese: bool,
    cur_selected_level: Level,
    cur_input: Vec<char>,
    cur_done: bool,
    exit: bool,
}

impl App {
    pub fn new(data_path: PathBuf, file_path: Option<PathBuf>) -> App {
        let title;
        let content = if let Some(path) = file_path {
            title = path.to_str().unwrap_or_default().to_owned();
            fs::read_to_string(&path)
                .unwrap_or_default()
                .lines().filter_map(|line| {
                    if line.trim().is_empty() {
                        None
                    } else {
                        Some(line.trim().to_owned())
                    }
                })
                .collect()
        } else {
            title = String::from(" Learning Data ");
            Vec::new()
        };

        App {
            title,
            learning_data: LearningData::default().read_file(&data_path),
            data_path,
            words_map: App::content_to_words_map(content),
            cur_word: (String::new(), Word::default()),
            cur_show_chinese: false,
            cur_selected_level: Level::Unselected,
            cur_input: Vec::new(),
            cur_done: true,
            exit: false,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>, count: usize, max_proficiency: isize) -> Result<(), Box<dyn Error>> {
        if self.words_map.is_empty() {
            self.words_map = self.learning_data.get_words_map(count, max_proficiency);
        }

        let mut words_map = self.words_map.clone();
        let mut words_map_iter = words_map.iter();

        while !self.exit {
            if self.cur_done {
                if let Some((key, value)) = words_map_iter.next() {
                    self.cur_word = (key.clone(), value.clone());
                    self.cur_done = false;
                    self.cur_show_chinese = false;
                    self.cur_input.clear();
                    self.cur_selected_level = Level::Unselected;
                } else {
                    self.learning_data.update(&self.words_map)
                        .save_file(&self.data_path)?;

                    let repeat_words_map:HashMap<String, Word> = self.words_map.iter().filter_map(|(key, value)| {
                        if value.proficiency <= 3 {
                            Some((key.clone(), value.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();

                    if repeat_words_map.is_empty() {
                        self.exit = true;
                    } else {
                        self.words_map = repeat_words_map;
                        words_map = self.words_map.clone();
                        words_map_iter = words_map.iter();
                    }
                    continue;
                }
            }
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }

        Ok(())
    }
}

impl App {
    fn draw(&self, frame: &mut Frame) {
        use ratatui::{
            layout::{Constraint, Layout},
            widgets::{
                block::Position, block::Title,
                Block, Borders, Paragraph, Wrap,
                Padding,
            },
        };

        let layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(22),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .split(frame.area());

        let main_layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(90),
            Constraint::Fill(1),
        ])
        .split(layout[1]);

        let option_layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(90),
            Constraint::Fill(1),
        ])
        .split(layout[2]);

        let hor_split_layout = Layout::horizontal([
            Constraint::Length(60),
            Constraint::Fill(1),
        ])
        .split(main_layout[1]);

        let status_block = Block::new()
            .title(" Learning Status ".to_owned())
            .padding(Padding::symmetric(2, 1))
            .borders(Borders::ALL)
            .border_style(Style::new().blue());
        let status_paragraph = Paragraph::new(Text::from(vec![
            Line::from("Word: ".to_owned() + &self.cur_word.0),
            Line::from("Proficiency: ".to_owned() + &self.cur_word.1.proficiency.to_string()),
            Line::from("Last Level: ".to_owned() + &self.cur_word.1.last_level.to_string()),
            Line::default(),
            Line::from("------------------------".to_owned()),
            Line::default(),
            Line::from("Words Total: ".to_owned() + &self.learning_data.get_words_count().to_string()),
            Line::from("Finished Total: ".to_owned() + &self.learning_data.get_finished_words_count().to_string()),
            Line::from("Current Count: ".to_owned() + &self.words_map.len().to_string()),
        ]))
            .wrap(Wrap {trim: true})
            .block(status_block);
        frame.render_widget(status_paragraph, hor_split_layout[1]);

        let ver_split_layout = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(hor_split_layout[0]);

        let en_block = Block::default()
            .title(" ".to_owned() + &self.title + " ")
            .padding(Padding::symmetric(2, 1))
            .borders(Borders::ALL)
            .border_style(Style::new().yellow());
        let word = self.cur_word.0.clone();
        let en_paragraph = Paragraph::new(if !self.cur_show_chinese {
                self.build_text()
            } else {
                word.to_text()
            })
            .wrap(Wrap {trim: true})
            .block(en_block);
        frame.render_widget(en_paragraph, ver_split_layout[0]);

        let cn_block = Block::default()
            .title(" Result ".to_owned())
            .title(Title::from(" <Esc>: Quit, <Tab>: Clear/Switch, <Enter>: Result ")
                .alignment(Alignment::Center)
                .position(Position::Bottom)
            )
            .padding(Padding::symmetric(2, 1))
            .borders(Borders::ALL)
            .border_style(Style::new().red());
        let cn_paragraph = Paragraph::new(if self.cur_show_chinese {
                self.cur_word.1.chinese.clone()
            } else {
                " ".to_owned()
            })
            .wrap(Wrap {trim: true})
            .block(cn_block);
        frame.render_widget(cn_paragraph, ver_split_layout[1]);

        let option_layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(20),
            Constraint::Length(1),
            Constraint::Length(20),
            Constraint::Length(1),
            Constraint::Length(20),
            Constraint::Length(1),
            Constraint::Length(20),
            Constraint::Fill(1),
        ])
        .split(option_layout[1]);

        let mut span_repeat = Span::from("Repeat");
        let mut span_hard = Span::from("Hard");
        let mut span_normal = Span::from("Normal");
        let mut span_simple = Span::from("Simple");

        match self.cur_selected_level {
            Level::Repeat => span_repeat.style = Style::new().on_yellow(),
            Level::Hard => span_hard.style = Style::new().on_yellow(),
            Level::Normal => span_normal.style = Style::new().on_yellow(),
            Level::Simple => span_simple.style = Style::new().on_yellow(),
            _ => {}
        }

        if self.cur_show_chinese {
            frame.render_widget(
                Paragraph::new(span_repeat).block(Block::bordered()),
                option_layout[1]
            );
            frame.render_widget(
                Paragraph::new(span_hard).block(Block::bordered()),
                option_layout[3]
            );
            frame.render_widget(
                Paragraph::new(span_normal).block(Block::bordered()),
                option_layout[5]
            );
            frame.render_widget(
                Paragraph::new(span_simple).block(Block::bordered()),
                option_layout[7]
            );
        }
    }

    fn handle_event(&mut self) -> Result<(), Box<dyn Error>> {
        use ratatui::crossterm::event::{read, Event, KeyCode, KeyEventKind};

        match read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match key_event.code {
                KeyCode::Esc => {
                    self.learning_data.update(&self.words_map)
                        .save_file(&self.data_path)?;
                    self.exit = true;
                }
                KeyCode::Tab => {
                    self.cur_input.clear();
                    if self.cur_show_chinese {
                        self.cur_selected_level = match self.cur_selected_level {
                            Level::Unselected => Level::Repeat,
                            Level::Repeat => Level::Hard,
                            Level::Hard => Level::Normal,
                            Level::Normal => Level::Simple,
                            Level::Simple => Level::Repeat,
                        }
                    }
                }
                KeyCode::Backspace => {
                    self.cur_input.pop();
                }
                KeyCode::Enter => {
                    self.cur_input.clear();
                    if self.cur_show_chinese {
                        match self.cur_selected_level {
                            Level::Unselected => self.cur_done = false,
                            _ => {
                                if let Some(word) = self.words_map.get_mut(&self.cur_word.0) {
                                    word.last_level = self.cur_selected_level;

                                    if word.last_level.eq(&Level::Repeat) {
                                        word.proficiency = Level::Repeat as isize;
                                    } else {
                                        word.proficiency += self.cur_selected_level as isize;
                                    }
                                }
                                self.cur_done = true;
                            }
                        }
                        self.cur_show_chinese = false;
                    } else {
                        self.cur_show_chinese = true;
                    }
                }
                KeyCode::Char(ch) => {
                    if self.cur_input.len() < self.cur_word.0.len() {
                        self.cur_input.push(ch);
                    }
                }
                _ => {}
            }
            _ => {}
        }
        Ok(())
    }

    fn content_to_words_map(content_lines: Vec<String>) -> HashMap<String, Word> {
        let mut words_map = HashMap::new();

        for line in content_lines {
            let splited_line: Vec<String> = line.split(":").filter_map(|span| {
                if span.trim().is_empty() {
                    None
                } else {
                    Some(span.trim().to_owned())
                }
            }).collect();

            // if line format valid
            if splited_line.len() == 2 {
                words_map.insert(splited_line[0].clone(), splited_line[1].clone());
            }
        }
        words_map.iter()
            .map(|(key, value)| {
                let value = Word {
                    proficiency: 0,
                    chinese: value.clone(),
                    last_level: Level::Unselected,
                };
                (key.clone(), value)
            })
            .collect()
    }

    /// build show-input line text
    fn build_text(&self) -> Text {
        use ratatui::style::{Style, Stylize};

        let mut text = Text::default();
        let mut index = 0;
        let mut build_line = Line::default();

        let cur_word_chars: Vec<char> = self.cur_word.0.chars().collect();

        while index < cur_word_chars.len() {
            if let Some(ch) = cur_word_chars.get(index) {
                if let Some(ch_input) = self.cur_input.get(index) {

                    build_line.push_span(
                        if ch.eq(ch_input) {
                            Span::styled(
                                ch.to_string(),
                                Style::new().green().bold()
                            )
                        } else {
                            Span::styled(
                                ch.to_string(),
                                Style::new().on_red()
                            )
                        }
                    );
                    index += 1;

                } else {
                    build_line.push_span(Span::styled(
                        ch.to_string(),
                        Style::new()
                    ));
                    index += 1;
                }

            } else {
                break;
            }
        }

        // set style for cursor position
        if let Some(span) = build_line.spans.get_mut(self.cur_input.len()) {
            span.style = Style::new().underlined();
        }
        text.lines.push(build_line);

        text
    }
}

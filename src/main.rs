use std::io;
use std::fs;

use crossterm::event::KeyEventKind;
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::*;
use ratatui::widgets::Widget;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

#[derive(Debug)]
pub struct App {
    exit: bool,
    active_block: ActiveBlock,
    popup: bool,
    popup_input: String,
    dirs: Vec<String>,
    dirs_state: ListState,
}

#[derive(Debug, Clone, PartialEq)]
enum ActiveBlock {
    Left,
    Right,
}

impl App {
    pub fn new() -> Self {
        let mut dirs_state = ListState::default();
        dirs_state.select(Some(0));

        let dirs = App::load_dirs();
        Self {
            active_block: ActiveBlock::Left,
            exit: false,
            popup: false,
            popup_input: String::new(),
            dirs,
            dirs_state,
        }
    }
    fn load_dirs()-> Vec<String> {
        if let Ok(content) = fs::read_to_string("dirs.txt") {
                    content.lines().map(|s| s.to_string()).collect()
                } else {
                    Vec::new()
                }
    }

    fn save_dirs(&self) {
        let data = self.dirs.join("\n");
        let _ = fs::write("dirs.txt", data);
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            match event::read()? {
                Event::Key(key_event) => self.handle_key_event(key_event)?,
                _ => {}
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        self.save_dirs();
        Ok(())
    }

    fn main_chunks(&self, area: Rect) -> [Rect; 2] {
        let footer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(footer_layout[0]);

        [chunks[0], chunks[1]]
    }

    fn left_inner_chunks(&self, left_area: Rect) -> [Rect; 2] {
        let inner_left = Block::default().borders(Borders::ALL).inner(left_area);
        let inner_chunk_l = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
            .split(inner_left);

        [inner_chunk_l[0], inner_chunk_l[1]]
    }

    fn left_block(&self) -> Block {
        let style = if self.active_block == ActiveBlock::Left {
            Style::default().fg(Color::Blue)
        } else {
            Style::default()
        };

        Block::default().border_style(style).borders(Borders::ALL)
    }

    fn right_block(&self) -> Block {
        let style = if self.active_block == ActiveBlock::Right {
            Style::default().fg(Color::Blue)
        } else {
            Style::default()
        };

        Block::default()
            .title("Alchemy")
            .border_style(style)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
    }

    fn footer(&self, area: Rect) -> Paragraph<'_> {
        let footer_text = match self.active_block {
            ActiveBlock::Left => " Press a to add directory | j, k to navigate | Enter to select",
            ActiveBlock::Right => ".............",
        };

        Paragraph::new(footer_text)
            .style(Style::default())
            .alignment(Alignment::Left)
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(&*self, frame.area());
        let chunks = self.main_chunks(frame.area());
        let inner_chunk_l = self.left_inner_chunks(chunks[0]);


        let dirs_list: Vec<ListItem> = self
            .dirs
            .iter()
            .map(|i| ListItem::new(i.as_str()))
            .collect();

        let list = List::new(dirs_list)
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::White))
            .highlight_symbol("=> ");

        frame.render_stateful_widget(list, inner_chunk_l[1], &mut self.dirs_state);

    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Tab => {
                    self.active_block = match self.active_block {
                        ActiveBlock::Left => ActiveBlock::Right,
                        ActiveBlock::Right => ActiveBlock::Left,
                    }
                }
                _ => {}
            }
        }

        if self.active_block == ActiveBlock::Left && key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Char('a') => {
                    if !self.popup {
                        self.popup = true;
                        self.popup_input.clear();
                    } else {
                        self.popup_input.push('a');
                    }
                }
                KeyCode::Char(c) if self.popup => {
                    self.popup_input.push(c);
                }
                KeyCode::Backspace if self.popup => {
                    self.popup_input.pop();
                }
                KeyCode::Enter if self.popup => {
                    if !self.popup_input.is_empty() {
                        self.dirs.push(self.popup_input.clone());
                    }
                    self.popup_input.clear();
                    self.popup = false;
                }
                _ => {}
            }
        }
        if self.active_block == ActiveBlock::Left && !self.dirs.is_empty() {
            match key_event.code {
                KeyCode::Char('j') => {
                    let i = match self.dirs_state.selected() {
                        Some(i) => {
                            if i + 1 >= self.dirs.len() { i } else { i + 1 }
                        }
                        None => 0,
                    };
                    self.dirs_state.select(Some(i));
                }
                KeyCode::Char('k') => {
                    let i = match self.dirs_state.selected() {
                        Some(i) => {
                            if i == 0 { 0 } else { i - 1 }
                        }
                        None => 0,
                    };
                    self.dirs_state.select(Some(i));
                }
                KeyCode::Char('d') => {
                    if let Some(selected) = self.dirs_state.selected() {
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
       let chunks = self.main_chunks(area);

        let left_block = self.left_block();
        let inner_left = {
            let temp_block = self.left_block(); // create a temp just for inner()
            temp_block.inner(chunks[0])
        };
        let right_block = self.right_block();
        left_block.render(chunks[0], buf);
        right_block.render(chunks[1], buf);

        let inner_chunk_l = self.left_inner_chunks(chunks[0]);
        let inner_top_l = Paragraph::new(Line::from("Directory"))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        let inner_bot_l = Block::default().borders(Borders::NONE);

        inner_top_l.render(inner_chunk_l[0], buf);
        inner_bot_l.render(inner_chunk_l[1], buf);

        let footer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        self.footer(footer_layout[1]).render(footer_layout[1], buf);
        //for popup
        if self.popup {
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage((100 - 20) / 2),
                    Constraint::Percentage(20),
                    Constraint::Percentage((100 - 20) / 2),
                ])
                .split(inner_left);
            let popup_block = Block::default()
                .title("Enter a Directory path")
                .title_alignment(Alignment::Center)
                .borders(Borders::NONE)
                .style(Style::default().bg(Color::DarkGray));

            let pop_inner = popup_block.inner(popup_layout[1]);
            let text_block = Block::default().borders(Borders::ALL);

            let textbox_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage((100 - 80) / 2),
                    Constraint::Percentage(80),
                    Constraint::Percentage((100 - 80) / 2),
                ])
                .split(pop_inner);
            let popup_text = Paragraph::new(self.popup_input.as_str())
                .style(Style::default().fg(Color::Black))
                .alignment(Alignment::Center);

            //the damn text in textbox has to be contered
            let rect = textbox_layout[1];

            let inner_rect = Rect {
                x: rect.x + 1,
                y: rect.y + rect.height / 2,
                width: rect.width - 2,
                height: 1,
            };

            popup_block.render(popup_layout[1], buf);
            text_block.render(textbox_layout[1], buf);
            popup_text.render(inner_rect, buf);
        }
    }
}

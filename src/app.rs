use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::Backend, Terminal};
use std::io;

use crate::quiz::Quiz;
use crate::ui;

pub struct App {
    pub quiz: Quiz,
    pub should_quit: bool,
}

impl App {
    pub async fn new() -> App {
        let quiz = Quiz::from_database("sqlite:quiz.db", 1).await.unwrap();

        App {
            quiz,
            should_quit: false,
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|f| ui::draw(f, &self.quiz))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    async fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.should_quit = true;
                }
                KeyCode::Up => {
                    if !self.quiz.show_results {
                        self.quiz.move_selection_up();
                    }
                }
                KeyCode::Down => {
                    if !self.quiz.show_results {
                        self.quiz.move_selection_down();
                    }
                }
                KeyCode::Enter => {
                    if self.quiz.show_results {
                        let _ = self.quiz.reload_srs_queue().await;
                    } else {
                        self.quiz.select_answer();
                        let _ = self.quiz.submit_answer(self.quiz.selected_answer).await;
                        self.quiz.next_question();
                    }
                }
                KeyCode::Left => {
                    if !self.quiz.show_results {
                        self.quiz.prev_question();
                    }
                }
                KeyCode::Right => {
                    if !self.quiz.show_results {
                        self.quiz.select_answer();
                        let _ = self.quiz.submit_answer(self.quiz.selected_answer).await;
                        self.quiz.next_question();
                    }
                }
                KeyCode::Char(' ') => {
                    if !self.quiz.show_results {
                        self.quiz.select_answer();
                    }
                }
                KeyCode::Char('r') => {
                    if self.quiz.show_results {
                        let _ = self.quiz.reload_srs_queue().await;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
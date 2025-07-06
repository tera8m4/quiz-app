use serde::{Deserialize, Serialize};
use crate::database::Database;

#[derive(Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: Option<i64>,
    pub text: String,
    pub answers: [String; 4],
    pub correct: usize,
}

#[derive(Serialize, Deserialize)]
pub struct QuizData {
    pub title: String,
    pub description: String,
    pub questions: Vec<Question>,
}

pub struct Quiz {
    pub quiz_data: QuizData,
    pub current_question: usize,
    pub selected_answer: usize,
    pub user_answers: Vec<Option<usize>>,
    pub show_results: bool,
    pub score: usize,
    pub database: Option<Database>,
}

impl Quiz {
    pub async fn from_database(database_url: &str, quiz_id: i64) -> Result<Quiz, Box<dyn std::error::Error>> {
        let db = Database::new(database_url).await?;
        let quiz_data = db.get_quiz_data(quiz_id).await?;

        let user_answers = vec![None; quiz_data.questions.len()];

        Ok(Quiz {
            quiz_data,
            current_question: 0,
            selected_answer: 0,
            user_answers,
            show_results: false,
            score: 0,
            database: Some(db),
        })
    }

    pub fn next_question(&mut self) {
        if self.current_question < self.quiz_data.questions.len() - 1 {
            self.current_question += 1;
            self.selected_answer = self.get_current_answer().unwrap_or(0);
        } else {
            self.calculate_score();
            self.show_results = true;
        }
    }

    pub fn prev_question(&mut self) {
        if self.current_question > 0 {
            self.current_question -= 1;
            self.selected_answer = self.get_current_answer().unwrap_or(0);
        }
    }

    pub fn select_answer(&mut self) {
        self.user_answers[self.current_question] = Some(self.selected_answer);
    }

    pub async fn select_answer_async(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.user_answers[self.current_question] = Some(self.selected_answer);
        self.submit_answer(self.selected_answer).await?;
        Ok(())
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_answer > 0 {
            self.selected_answer -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_answer < 3 {
            self.selected_answer += 1;
        }
    }

    pub fn get_current_question(&self) -> &Question {
        &self.quiz_data.questions[self.current_question]
    }

    pub fn get_current_answer(&self) -> Option<usize> {
        self.user_answers[self.current_question]
    }

    pub fn calculate_score(&mut self) {
        self.score = 0;
        for (i, user_answer) in self.user_answers.iter().enumerate() {
            if let Some(answer) = user_answer {
                if *answer == self.quiz_data.questions[i].correct {
                    self.score += 1;
                }
            }
        }
    }

    pub fn restart(&mut self) {
        self.current_question = 0;
        self.selected_answer = 0;
        self.user_answers = vec![None; self.quiz_data.questions.len()];
        self.show_results = false;
        self.score = 0;
    }

    pub async fn submit_answer(&mut self, answer: usize) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(db) = &self.database {
            let question = &self.quiz_data.questions[self.current_question];
            if let Some(question_id) = question.id {
                let is_correct = answer == question.correct;
                db.srs.process_review(question_id, 1, is_correct).await?;
            }
        }
        Ok(())
    }

    pub fn get_score_percentage(&self) -> u32 {
        if self.quiz_data.questions.is_empty() {
            0
        } else {
            ((self.score * 100) / self.quiz_data.questions.len()) as u32
        }
    }

    pub fn get_score_message(&self) -> &'static str {
        let percentage = self.get_score_percentage();
        match percentage {
            100 => "Perfect! Excellent work! 🌟",
            75..=99 => "Great job! Well done! 👏",
            50..=74 => "Good effort! Keep practicing! 👍",
            _ => "Don't give up! Try again! 💪",
        }
    }
}
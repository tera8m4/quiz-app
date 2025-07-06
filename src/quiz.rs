use crate::database::Database;
use crate::srs::SrsItem;
use serde::{Deserialize, Serialize};

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
    pub srs_items: Vec<SrsItem>,
    pub user_id: i64,
}

impl Quiz {
    pub async fn from_database(
        database_url: &str,
        _quiz_id: i64,
    ) -> Result<Quiz, Box<dyn std::error::Error>> {
        let db = Database::new(database_url).await?;
        let user_id = 1; // Default user ID

        // Get due SRS items
        let srs_items = db.srs.get_review_queue(user_id, Some(20)).await?;

        // Load questions for the due SRS items
        let mut questions = Vec::new();
        for item in &srs_items {
            let question_row = sqlx::query!(
                "SELECT id, text, answer_1, answer_2, answer_3, answer_4, correct_answer 
                 FROM questions WHERE id = ?",
                item.question_id
            )
            .fetch_one(&db.pool)
            .await?;

            let question = Question {
                id: Some(question_row.id),
                text: question_row.text,
                answers: [
                    question_row.answer_1,
                    question_row.answer_2,
                    question_row.answer_3,
                    question_row.answer_4,
                ],
                correct: question_row.correct_answer as usize,
            };
            questions.push(question);
        }

        // Create quiz data with only due questions
        let quiz_data = QuizData {
            title: if srs_items.is_empty() {
                "No reviews due".to_string()
            } else {
                format!("SRS Review - {} questions", srs_items.len())
            },
            description: "Spaced repetition system review session".to_string(),
            questions,
        };

        let user_answers = vec![None; quiz_data.questions.len()];

        Ok(Quiz {
            quiz_data,
            current_question: 0,
            selected_answer: 0,
            user_answers,
            show_results: false,
            score: 0,
            database: Some(db),
            srs_items,
            user_id,
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

    pub fn has_questions(&self) -> bool {
        !self.quiz_data.questions.is_empty() && self.current_question < self.quiz_data.questions.len()
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

    pub async fn reload_srs_queue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(db) = &self.database {
            // Get updated SRS items
            let srs_items = db.srs.get_review_queue(self.user_id, Some(20)).await?;

            // Load questions for the due SRS items
            let mut questions = Vec::new();
            for item in &srs_items {
                let question_row = sqlx::query!(
                    "SELECT id, text, answer_1, answer_2, answer_3, answer_4, correct_answer 
                     FROM questions WHERE id = ?",
                    item.question_id
                )
                .fetch_one(&db.pool)
                .await?;

                let question = Question {
                    id: Some(question_row.id),
                    text: question_row.text,
                    answers: [
                        question_row.answer_1,
                        question_row.answer_2,
                        question_row.answer_3,
                        question_row.answer_4,
                    ],
                    correct: question_row.correct_answer as usize,
                };
                questions.push(question);
            }

            // Update quiz data
            self.quiz_data = QuizData {
                title: if srs_items.is_empty() {
                    "No reviews due".to_string()
                } else {
                    format!("SRS Review - {} questions", srs_items.len())
                },
                description: "Spaced repetition system review session".to_string(),
                questions,
            };

            self.srs_items = srs_items;
            self.user_answers = vec![None; self.quiz_data.questions.len()];
            self.current_question = 0;
            self.selected_answer = 0;
            self.show_results = false;
            self.score = 0;
        }
        Ok(())
    }

    pub async fn submit_answer(&mut self, answer: usize) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(db) = &self.database {
            let question = &self.quiz_data.questions[self.current_question];
            if let Some(question_id) = question.id {
                let is_correct = answer == question.correct;
                let updated_item = db
                    .srs
                    .process_review(question_id, self.user_id, is_correct)
                    .await?;

                // Update the SRS item in our local list
                if self.current_question < self.srs_items.len() {
                    self.srs_items[self.current_question] = updated_item;
                }
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

    pub fn get_current_srs_item(&self) -> Option<&SrsItem> {
        if self.current_question < self.srs_items.len() {
            Some(&self.srs_items[self.current_question])
        } else {
            None
        }
    }

    pub fn get_srs_stage_name(&self) -> Option<String> {
        use crate::srs::SrsSystem;
        self.get_current_srs_item()
            .map(|item| SrsSystem::get_stage_name(item.srs_stage).to_string())
    }

    pub fn get_srs_progress_info(&self) -> Option<String> {
        use crate::srs::SrsSystem;
        self.get_current_srs_item().map(|item| {
            let stage_name = SrsSystem::get_stage_name(item.srs_stage);
            let incorrect_count = item.incorrect_count;
            if incorrect_count > 0 {
                format!("{} (Failed {} times)", stage_name, incorrect_count)
            } else {
                stage_name.to_string()
            }
        })
    }
}


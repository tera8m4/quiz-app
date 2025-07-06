use crate::database::Database;
use crate::quiz::Question;
use crate::srs::{SrsItem, SrsSystem};
use std::collections::HashMap;

pub struct SrsQuiz {
    pub database: Database,
    pub current_items: Vec<SrsItem>,
    pub current_index: usize,
    pub selected_answer: usize,
    pub session_stats: SessionStats,
    pub user_id: i64,
    pub question_cache: HashMap<i64, Question>,
}

#[derive(Default)]
pub struct SessionStats {
    pub correct_answers: usize,
    pub incorrect_answers: usize,
    pub total_reviews: usize,
    pub items_leveled_up: usize,
    pub items_leveled_down: usize,
}

impl SrsQuiz {
    pub async fn new(database_url: &str, user_id: i64) -> Result<Self, Box<dyn std::error::Error>> {
        let database = Database::new(database_url).await?;
        let current_items = database.srs.get_review_queue(user_id, Some(20)).await?;
        let question_cache = HashMap::new();

        Ok(SrsQuiz {
            database,
            current_items,
            current_index: 0,
            selected_answer: 0,
            session_stats: SessionStats::default(),
            user_id,
            question_cache,
        })
    }

    pub async fn get_current_question(&mut self) -> Result<Option<Question>, Box<dyn std::error::Error>> {
        if self.current_index >= self.current_items.len() {
            return Ok(None);
        }

        let item = &self.current_items[self.current_index];
        
        // Check cache first
        if let Some(question) = self.question_cache.get(&item.question_id) {
            return Ok(Some(question.clone()));
        }

        // Fetch from database
        let question_row = sqlx::query!(
            "SELECT id, text, answer_1, answer_2, answer_3, answer_4, correct_answer 
             FROM questions WHERE id = ?",
            item.question_id
        )
        .fetch_one(&self.database.pool)
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

        // Cache the question
        self.question_cache.insert(item.question_id, question.clone());
        Ok(Some(question))
    }

    pub fn get_current_srs_item(&self) -> Option<&SrsItem> {
        if self.current_index >= self.current_items.len() {
            None
        } else {
            Some(&self.current_items[self.current_index])
        }
    }

    pub async fn submit_answer(&mut self, answer: usize) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_index >= self.current_items.len() {
            return Ok(());
        }

        let item = &self.current_items[self.current_index];
        let question_id = item.question_id;
        let previous_stage = item.srs_stage;
        
        let question = self.get_current_question().await?;
        if let Some(q) = question {
            let is_correct = answer == q.correct;
            
            let updated_item = self.database.srs.process_review(
                question_id,
                self.user_id,
                is_correct,
            ).await?;

            // Update session stats
            self.session_stats.total_reviews += 1;
            if is_correct {
                self.session_stats.correct_answers += 1;
            } else {
                self.session_stats.incorrect_answers += 1;
            }

            if updated_item.srs_stage > previous_stage {
                self.session_stats.items_leveled_up += 1;
            } else if updated_item.srs_stage < previous_stage {
                self.session_stats.items_leveled_down += 1;
            }

            // Update the current item in our list
            self.current_items[self.current_index] = updated_item;
        }
        Ok(())
    }

    pub fn next_question(&mut self) {
        if self.current_index < self.current_items.len() - 1 {
            self.current_index += 1;
            self.selected_answer = 0;
        }
    }

    pub fn prev_question(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.selected_answer = 0;
        }
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

    pub fn is_finished(&self) -> bool {
        self.current_index >= self.current_items.len()
    }

    pub fn get_progress(&self) -> (usize, usize) {
        (self.current_index, self.current_items.len())
    }

    pub fn get_accuracy(&self) -> f32 {
        if self.session_stats.total_reviews == 0 {
            0.0
        } else {
            (self.session_stats.correct_answers as f32 / self.session_stats.total_reviews as f32) * 100.0
        }
    }

    pub async fn reload_review_queue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_items = self.database.srs.get_review_queue(self.user_id, Some(20)).await?;
        self.current_index = 0;
        self.selected_answer = 0;
        self.question_cache.clear();
        Ok(())
    }

    pub fn get_stage_info(&self) -> Option<(String, String)> {
        if let Some(item) = self.get_current_srs_item() {
            let stage_name = SrsSystem::get_stage_name(item.srs_stage).to_string();
            let next_review = if item.srs_stage < 9 {
                let next_stage_name = SrsSystem::get_stage_name(item.srs_stage + 1);
                format!("Next: {}", next_stage_name)
            } else {
                "Burned - No more reviews".to_string()
            };
            Some((stage_name, next_review))
        } else {
            None
        }
    }
}
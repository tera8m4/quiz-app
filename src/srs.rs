use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono::NaiveDateTime, Row, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrsItem {
    pub id: i64,
    pub question_id: i64,
    pub user_id: i64,
    pub srs_stage: i64,
    pub incorrect_count: i64,
    pub next_review_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrsReview {
    pub id: i64,
    pub srs_item_id: i64,
    pub is_correct: bool,
    pub previous_stage: i64,
    pub new_stage: i64,
    pub review_date: NaiveDateTime,
}

pub struct SrsSystem {
    pool: SqlitePool,
}

impl SrsSystem {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn calculate_next_stage(current_stage: i64, is_correct: bool, incorrect_count: i64) -> i64 {
        if is_correct {
            (current_stage + 1).min(9) // Cap at stage 9 (Burned)
        } else {
            let incorrect_adjustment_count = (incorrect_count as f32 / 2.0).ceil() as i64;
            let srs_penalty_factor = if current_stage >= 5 { 2 } else { 1 };
            let penalty = incorrect_adjustment_count * srs_penalty_factor;
            (current_stage - penalty).max(1) // Floor at stage 1
        }
    }

    pub fn calculate_next_review_date(stage: i64) -> NaiveDateTime {
        let now = Utc::now().naive_utc();
        let duration = match stage {
            1 => Duration::hours(4),  // Apprentice 1
            2 => Duration::hours(8),  // Apprentice 2
            3 => Duration::days(1),   // Apprentice 3
            4 => Duration::days(2),   // Apprentice 4
            5 => Duration::weeks(1),  // Guru 1
            6 => Duration::weeks(2),  // Guru 2
            7 => Duration::weeks(4),  // Master
            8 => Duration::weeks(16), // Enlightened
            9 => Duration::weeks(32), // Burned (never)
            _ => Duration::hours(4),  // Default to Apprentice 1
        };

        if stage == 9 {
            // For burned items, return a date far in the future
            now + Duration::weeks(520)
        } else {
            now + duration
        }
    }

    pub fn get_stage_name(stage: i64) -> &'static str {
        match stage {
            1 => "Apprentice 1",
            2 => "Apprentice 2",
            3 => "Apprentice 3",
            4 => "Apprentice 4",
            5 => "Guru 1",
            6 => "Guru 2",
            7 => "Master",
            8 => "Enlightened",
            9 => "Burned",
            _ => "Unknown",
        }
    }

    pub async fn get_or_create_srs_item(
        &self,
        question_id: i64,
        user_id: i64,
    ) -> Result<SrsItem, sqlx::Error> {
        // Try to get existing item
        let existing = sqlx::query(
            "SELECT id, question_id, user_id, srs_stage, incorrect_count, next_review_date, created_at, updated_at 
             FROM srs_items WHERE question_id = ? AND user_id = ?"
        )
        .bind(question_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        let existing = existing.map(|row| SrsItem {
            id: row.get("id"),
            question_id: row.get("question_id"),
            user_id: row.get("user_id"),
            srs_stage: row.get("srs_stage"),
            incorrect_count: row.get("incorrect_count"),
            next_review_date: row.get("next_review_date"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        });

        if let Some(item) = existing {
            Ok(item)
        } else {
            // Create new item
            let next_review_date = Self::calculate_next_review_date(1);
            let now = Utc::now().naive_utc();

            let result = sqlx::query!(
                "INSERT INTO srs_items (question_id, user_id, srs_stage, incorrect_count, next_review_date, created_at, updated_at)
                 VALUES (?, ?, 1, 0, ?, ?, ?)",
                question_id, user_id, next_review_date, now, now
            )
            .execute(&self.pool)
            .await?;

            let id = result.last_insert_rowid();

            Ok(SrsItem {
                id,
                question_id,
                user_id,
                srs_stage: 1,
                incorrect_count: 0,
                next_review_date,
                created_at: now,
                updated_at: now,
            })
        }
    }

    pub async fn process_review(
        &self,
        question_id: i64,
        user_id: i64,
        is_correct: bool,
    ) -> Result<SrsItem, sqlx::Error> {
        let mut item = self.get_or_create_srs_item(question_id, user_id).await?;

        let previous_stage = item.srs_stage;
        let new_incorrect_count = if is_correct {
            item.incorrect_count
        } else {
            item.incorrect_count + 1
        };

        let new_stage = Self::calculate_next_stage(item.srs_stage, is_correct, new_incorrect_count);
        let next_review_date = Self::calculate_next_review_date(new_stage);
        let now = Utc::now().naive_utc();

        // Update the item
        sqlx::query!(
            "UPDATE srs_items SET srs_stage = ?, incorrect_count = ?, next_review_date = ?, updated_at = ?
             WHERE id = ?",
            new_stage, new_incorrect_count, next_review_date, now, item.id
        )
        .execute(&self.pool)
        .await?;

        // Record the review
        sqlx::query!(
            "INSERT INTO srs_reviews (srs_item_id, is_correct, previous_stage, new_stage, review_date)
             VALUES (?, ?, ?, ?, ?)",
            item.id, is_correct, previous_stage, new_stage, now
        )
        .execute(&self.pool)
        .await?;

        // Update the item with new values
        item.srs_stage = new_stage;
        item.incorrect_count = new_incorrect_count;
        item.next_review_date = next_review_date;
        item.updated_at = now;

        Ok(item)
    }

    pub async fn get_review_queue(
        &self,
        user_id: i64,
        limit: Option<i64>,
    ) -> Result<Vec<SrsItem>, sqlx::Error> {
        let now = Utc::now().naive_utc();
        let query_limit = limit.unwrap_or(20);

        let rows = sqlx::query(
            "SELECT id, question_id, user_id, srs_stage, incorrect_count, next_review_date, created_at, updated_at
             FROM srs_items 
             WHERE user_id = ? AND next_review_date <= ? AND srs_stage < 9
             ORDER BY next_review_date ASC
             LIMIT ?"
        )
        .bind(user_id)
        .bind(now)
        .bind(query_limit)
        .fetch_all(&self.pool)
        .await?;

        let items = rows.into_iter().map(|row| SrsItem {
            id: row.get("id"),
            question_id: row.get("question_id"),
            user_id: row.get("user_id"),
            srs_stage: row.get("srs_stage"),
            incorrect_count: row.get("incorrect_count"),
            next_review_date: row.get("next_review_date"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();

        Ok(items)
    }

    pub async fn get_srs_stats(&self, user_id: i64) -> Result<SrsStats, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT 
                COUNT(*) as total_items,
                SUM(CASE WHEN srs_stage = 1 THEN 1 ELSE 0 END) as apprentice_1,
                SUM(CASE WHEN srs_stage = 2 THEN 1 ELSE 0 END) as apprentice_2,
                SUM(CASE WHEN srs_stage = 3 THEN 1 ELSE 0 END) as apprentice_3,
                SUM(CASE WHEN srs_stage = 4 THEN 1 ELSE 0 END) as apprentice_4,
                SUM(CASE WHEN srs_stage = 5 THEN 1 ELSE 0 END) as guru_1,
                SUM(CASE WHEN srs_stage = 6 THEN 1 ELSE 0 END) as guru_2,
                SUM(CASE WHEN srs_stage = 7 THEN 1 ELSE 0 END) as master,
                SUM(CASE WHEN srs_stage = 8 THEN 1 ELSE 0 END) as enlightened,
                SUM(CASE WHEN srs_stage = 9 THEN 1 ELSE 0 END) as burned
             FROM srs_items WHERE user_id = ?",
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(SrsStats {
            total_items: row.total_items as i64,
            apprentice_1: row.apprentice_1.unwrap_or(0) as i64,
            apprentice_2: row.apprentice_2.unwrap_or(0) as i64,
            apprentice_3: row.apprentice_3.unwrap_or(0) as i64,
            apprentice_4: row.apprentice_4.unwrap_or(0) as i64,
            guru_1: row.guru_1.unwrap_or(0) as i64,
            guru_2: row.guru_2.unwrap_or(0) as i64,
            master: row.master.unwrap_or(0) as i64,
            enlightened: row.enlightened.unwrap_or(0) as i64,
            burned: row.burned.unwrap_or(0) as i64,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SrsStats {
    pub total_items: i64,
    pub apprentice_1: i64,
    pub apprentice_2: i64,
    pub apprentice_3: i64,
    pub apprentice_4: i64,
    pub guru_1: i64,
    pub guru_2: i64,
    pub master: i64,
    pub enlightened: i64,
    pub burned: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_next_stage_correct() {
        assert_eq!(SrsSystem::calculate_next_stage(1, true, 0), 2);
        assert_eq!(SrsSystem::calculate_next_stage(8, true, 0), 9);
        assert_eq!(SrsSystem::calculate_next_stage(9, true, 0), 9); // Cap at 9
    }

    #[test]
    fn test_calculate_next_stage_incorrect() {
        // Stage 4, incorrect once
        assert_eq!(SrsSystem::calculate_next_stage(4, false, 1), 3);

        // Stage 6, incorrect 3 times (penalty = ceil(3/2) * 2 = 4)
        assert_eq!(SrsSystem::calculate_next_stage(6, false, 3), 2);

        // Stage 1, incorrect (floor at 1)
        assert_eq!(SrsSystem::calculate_next_stage(1, false, 1), 1);
    }

    #[test]
    fn test_stage_names() {
        assert_eq!(SrsSystem::get_stage_name(1), "Apprentice 1");
        assert_eq!(SrsSystem::get_stage_name(5), "Guru 1");
        assert_eq!(SrsSystem::get_stage_name(9), "Burned");
    }
}


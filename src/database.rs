use sqlx::{Row, SqlitePool, migrate::MigrateDatabase};
use crate::quiz::{Question, QuizData};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        if !sqlx::Sqlite::database_exists(database_url).await.unwrap_or(false) {
            sqlx::Sqlite::create_database(database_url).await?;
        }
        
        let pool = SqlitePool::connect(database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Database { pool })
    }

    pub async fn get_quiz_data(&self, quiz_id: i64) -> Result<QuizData, sqlx::Error> {
        let quiz_row = sqlx::query("SELECT title, description FROM quizzes WHERE id = ?")
            .bind(quiz_id)
            .fetch_one(&self.pool)
            .await?;

        let title: String = quiz_row.get("title");
        let description: String = quiz_row.get("description");

        let question_rows = sqlx::query(
            "SELECT text, answer_1, answer_2, answer_3, answer_4, correct_answer FROM questions WHERE quiz_id = ? ORDER BY id"
        )
        .bind(quiz_id)
        .fetch_all(&self.pool)
        .await?;

        let questions: Vec<Question> = question_rows
            .iter()
            .map(|row| {
                let text: String = row.get("text");
                let answer_1: String = row.get("answer_1");
                let answer_2: String = row.get("answer_2");
                let answer_3: String = row.get("answer_3");
                let answer_4: String = row.get("answer_4");
                let correct: i32 = row.get("correct_answer");

                Question {
                    text,
                    answers: [answer_1, answer_2, answer_3, answer_4],
                    correct: correct as usize,
                }
            })
            .collect();

        Ok(QuizData {
            title,
            description,
            questions,
        })
    }

    pub async fn insert_quiz(&self, quiz_data: &QuizData) -> Result<i64, sqlx::Error> {
        let result = sqlx::query("INSERT INTO quizzes (title, description) VALUES (?, ?)")
            .bind(&quiz_data.title)
            .bind(&quiz_data.description)
            .execute(&self.pool)
            .await?;

        let quiz_id = result.last_insert_rowid();

        for question in &quiz_data.questions {
            sqlx::query(
                "INSERT INTO questions (quiz_id, text, answer_1, answer_2, answer_3, answer_4, correct_answer) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(quiz_id)
            .bind(&question.text)
            .bind(&question.answers[0])
            .bind(&question.answers[1])
            .bind(&question.answers[2])
            .bind(&question.answers[3])
            .bind(question.correct as i32)
            .execute(&self.pool)
            .await?;
        }

        Ok(quiz_id)
    }

    pub async fn list_quizzes(&self) -> Result<Vec<(i64, String, String)>, sqlx::Error> {
        let rows = sqlx::query("SELECT id, title, description FROM quizzes ORDER BY id")
            .fetch_all(&self.pool)
            .await?;

        let quizzes = rows
            .iter()
            .map(|row| {
                let id: i64 = row.get("id");
                let title: String = row.get("title");
                let description: String = row.get("description");
                (id, title, description)
            })
            .collect();

        Ok(quizzes)
    }
}
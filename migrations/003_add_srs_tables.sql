-- Create SRS items table to track individual question progress
CREATE TABLE IF NOT EXISTS srs_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL DEFAULT 1, -- For future multi-user support
    srs_stage INTEGER NOT NULL DEFAULT 1,
    incorrect_count INTEGER NOT NULL DEFAULT 0,
    next_review_date DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (question_id) REFERENCES questions (id),
    UNIQUE(question_id, user_id)
);

-- Create SRS review history table
CREATE TABLE IF NOT EXISTS srs_reviews (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    srs_item_id INTEGER NOT NULL,
    is_correct BOOLEAN NOT NULL,
    previous_stage INTEGER NOT NULL,
    new_stage INTEGER NOT NULL,
    review_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (srs_item_id) REFERENCES srs_items (id)
);

-- Create index for efficient review queue queries
CREATE INDEX IF NOT EXISTS idx_srs_items_next_review ON srs_items (next_review_date);
CREATE INDEX IF NOT EXISTS idx_srs_items_stage ON srs_items (srs_stage);
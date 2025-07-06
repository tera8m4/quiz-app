-- Initialize SRS data for existing questions
-- This migration creates SRS items for all existing questions that don't have them yet

INSERT INTO srs_items (question_id, user_id, srs_stage, incorrect_count, next_review_date, created_at, updated_at)
SELECT 
    q.id,
    1, -- default user_id
    1, -- start at Apprentice 1 (srs_stage = 1)
    0, -- no incorrect answers yet
    datetime('now'), -- next review in 4 hours (Apprentice 1 interval)
    datetime('now'),
    datetime('now')
FROM questions q
LEFT JOIN srs_items s ON q.id = s.question_id AND s.user_id = 1
WHERE s.question_id IS NULL;
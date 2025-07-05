-- Insert sample quiz data from questions.json
INSERT INTO quizzes (title, description) VALUES ('Science Quiz', 'Test your knowledge of basic science concepts');

-- Insert sample questions for the Science Quiz (quiz_id = 1)
INSERT INTO questions (quiz_id, text, answer_1, answer_2, answer_3, answer_4, correct_answer) VALUES
(1, 'What is the chemical formula for water?', 'H2O', 'CO2', 'NaCl', 'CH4', 0),
(1, 'What force keeps planets in orbit around the sun?', 'Magnetism', 'Gravity', 'Friction', 'Inertia', 1),
(1, 'What is the hardest natural substance on Earth?', 'Iron', 'Gold', 'Diamond', 'Quartz', 2),
(1, 'How many bones are in an adult human body?', '186', '206', '226', '246', 1),
(1, 'What gas do plants absorb from the atmosphere during photosynthesis?', 'Oxygen', 'Nitrogen', 'Carbon Dioxide', 'Hydrogen', 2);
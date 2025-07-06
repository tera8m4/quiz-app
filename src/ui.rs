use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::quiz::Quiz;

pub fn draw(f: &mut Frame, quiz: &Quiz) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(f.size());

    draw_title(f, chunks[0]);

    if quiz.show_results {
        draw_results(f, quiz, chunks[1]);
    } else {
        draw_question(f, quiz, chunks[1]);
    }

    draw_instructions(f, quiz, chunks[2]);
}

fn draw_title(f: &mut Frame, area: ratatui::layout::Rect) {
    let title = Paragraph::new("🧠 SRS Quiz Review")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, area);
}

fn draw_question(f: &mut Frame, quiz: &Quiz, area: ratatui::layout::Rect) {
    let question_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(area);

    if quiz.has_questions() {
        draw_question_text(f, quiz, question_chunks[0]);
        draw_answer_options(f, quiz, question_chunks[1]);
        draw_srs_info(f, quiz, question_chunks[2]);
    } else {
        draw_no_questions_message(f, area);
    }
}

fn draw_question_text(f: &mut Frame, quiz: &Quiz, area: ratatui::layout::Rect) {
    let current_q = quiz.get_current_question();
    let question_text = current_q.text.clone();

    let block_title = format!("{}", quiz.quiz_data.title);
    let question = Paragraph::new(question_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title(block_title));
    f.render_widget(question, area);
}

fn draw_answer_options(f: &mut Frame, quiz: &Quiz, area: ratatui::layout::Rect) {
    let current_q = quiz.get_current_question();
    let answers: Vec<ListItem> = current_q
        .answers
        .iter()
        .enumerate()
        .map(|(i, answer)| {
            let prefix = match i {
                0 => "A) ",
                1 => "B) ",
                2 => "C) ",
                3 => "D) ",
                _ => "   ",
            };

            let style = get_answer_style(quiz, i);
            let content = format!("{}{}", prefix, answer);
            ListItem::new(content).style(style)
        })
        .collect();

    let answers_list = List::new(answers)
        .block(Block::default().borders(Borders::ALL).title("Answers"))
        .style(Style::default().fg(Color::White));

    f.render_widget(answers_list, area);
}

fn get_answer_style(quiz: &Quiz, answer_index: usize) -> Style {
    if answer_index == quiz.selected_answer {
        // Currently highlighted answer
        Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else if quiz.get_current_answer() == Some(answer_index) {
        // Previously selected answer
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        // Default answer style
        Style::default().fg(Color::Gray)
    }
}

fn draw_results(f: &mut Frame, quiz: &Quiz, area: ratatui::layout::Rect) {
    let results_text = format!(
        "Quiz Complete! 🎉\n\n{}\n\nYour Score: {} out of {} ({}%)\n\n{}",
        quiz.quiz_data.title,
        quiz.score,
        quiz.quiz_data.questions.len(),
        quiz.get_score_percentage(),
        quiz.get_score_message()
    );

    let results = Paragraph::new(results_text)
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Results"));

    f.render_widget(results, area);
}

fn draw_srs_info(f: &mut Frame, quiz: &Quiz, area: ratatui::layout::Rect) {
    let srs_text = if let Some(progress_info) = quiz.get_srs_progress_info() {
        let progress = format!(
            "Question {} of {}",
            quiz.current_question + 1,
            quiz.quiz_data.questions.len()
        );
        format!("SRS Stage: {} • {}", progress_info, progress)
    } else {
        format!(
            "Question {} of {}",
            quiz.current_question + 1,
            quiz.quiz_data.questions.len()
        )
    };

    let srs_info = Paragraph::new(srs_text)
        .style(Style::default().fg(Color::Magenta))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Progress"));
    f.render_widget(srs_info, area);
}

fn draw_no_questions_message(f: &mut Frame, area: ratatui::layout::Rect) {
    let message = Paragraph::new("📚 No questions available for review\n\nAll your questions are up to date!\nCome back later for more practice.")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("SRS Review"));
    f.render_widget(message, area);
}

fn draw_instructions(f: &mut Frame, quiz: &Quiz, area: ratatui::layout::Rect) {
    let instructions = if quiz.show_results {
        "Enter/r - Reload Queue • Esc/q - Quit"
    } else {
        "↑/↓ Select • Space/Enter/→ Confirm • ← Previous • Esc/q - Quit"
    };

    let help = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, area);
}


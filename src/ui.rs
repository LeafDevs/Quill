use crate::app::{App, Message, ToolCall};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Clear},
    Frame,
};
use tui::text::Text;

const TITLE_ART: [&str; 6] = [
    " ██████╗ ██╗   ██╗██╗██╗     ██╗     ",
    "██╔═══██╗██║   ██║██║██║     ██║     ",
    "██║   ██║██║   ██║██║██║     ██║     ",
    "██║▄▄ ██║██║   ██║██║██║     ██║     ",
    "╚██████╔╝╚██████╔╝██║███████╗███████╗",
    " ╚══▀▀═╝  ╚═════╝ ╚═╝╚══════╝╚══════╝",
];

fn required_height(text: &str, width: u16) -> u16 {
    let mut lines = 0;
    for line in text.lines() {
        let len = line.chars().count() as u16;
        lines += (len / width).max(1);
    }
    lines
}

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();

    // Layout: Top bar (model selector), Title, Chat, Input
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),   // Model selector bar
            Constraint::Length(8),   // Title art
            Constraint::Min(10),     // Chat area
            Constraint::Length(3),   // Input area
        ])
        .split(size);

    draw_model_selector_bar(f, main_chunks[0], app);
    draw_title_art(f, main_chunks[1]);
    draw_chat_area(f, main_chunks[2], app);
    draw_input_area(f, main_chunks[3], app);
}

fn draw_title_art<B: Backend>(f: &mut Frame<B>, area: Rect) {
    // Aqua-pink-aqua gradient colors
    let gradient = [Color::Cyan, Color::LightCyan, Color::Magenta, Color::LightMagenta, Color::Cyan, Color::LightCyan];
    let mut art_spans = Vec::new();
    for (i, line) in TITLE_ART.iter().enumerate() {
        let color = gradient[i % gradient.len()];
        art_spans.push(Spans::from(Span::styled(
            *line,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));
    }
    let art = Paragraph::new(art_spans)
        .alignment(Alignment::Center)
        .block(Block::default());
    f.render_widget(Clear, area); // Clear background for clean look
    f.render_widget(art, area);
}

fn draw_model_selector_bar<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    // Minimal top bar with model selector
    let mut spans = vec![Span::styled("Model:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))];
    for (i, model) in app.models.iter().enumerate() {
        let style = if i == app.selected_model_index {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::raw(" "));
        spans.push(Span::styled(&model.name, style));
    }
    let bar = Paragraph::new(Spans::from(spans))
        .alignment(Alignment::Left)
        .block(Block::default());
    f.render_widget(Clear, area);
    f.render_widget(bar, area);
}

fn draw_chat_area<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    use tui::text::Text;
    let mut text = Text::default();
    // Estimate how many lines each message will take (naive: 2 lines per message)
    let max_lines = area.height as usize;
    let mut lines_used = 0;
    let mut visible_msgs = Vec::new();
    // Walk backwards through messages, collecting as many as fit
    for message in app.messages.iter().rev() {
        // Estimate lines for each message (could be improved with real wrapping logic)
        let lines = 3; // 1 for content, 1 for meta, 1 for spacing
        if lines_used + lines > max_lines {
            break;
        }
        visible_msgs.push(message);
        lines_used += lines;
    }
    visible_msgs.reverse();
    for message in visible_msgs {
        match message {
            Message::User { content, timestamp } => {
                text.extend(vec![
                    Spans::from(vec![
                        Span::styled("> ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::styled(content, Style::default().fg(Color::White)),
                    ]),
                    Spans::from(vec![
                        Span::styled(
                            format!("USER {}", timestamp.format("%H:%M")),
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        ),
                    ]),
                    Spans::from("")
                ]);
            }
            Message::Assistant { content, timestamp } => {
                text.extend(vec![
                    Spans::from(vec![
                        Span::styled("< ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                        Span::styled(content, Style::default().fg(Color::White)),
                    ]),
                    Spans::from(vec![
                        Span::styled(
                            format!("ASSISTANT {}", timestamp.format("%H:%M")),
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        ),
                    ]),
                    Spans::from("")
                ]);
            }
            Message::PendingToolCall { tool_call, original_message, timestamp: _ } => {
                let (desc, color) = match tool_call {
                    ToolCall::ReadFile { path } => (format!("[TOOL CALL] read_file: {}", path), Color::Green),
                    ToolCall::ReadDirectory { path } => (format!("[TOOL CALL] read_directory: {}", path), Color::Blue),
                    ToolCall::EditFile { path, .. } => (format!("[TOOL CALL] edit_file: {}", path), Color::Yellow),
                };
                text.extend(vec![
                    Spans::from(vec![Span::styled(desc, Style::default().fg(color).add_modifier(Modifier::BOLD))]),
                    Spans::from(vec![Span::styled(format!("[tool_call: {}]", original_message), Style::default().fg(Color::DarkGray))]),
                    Spans::from(vec![Span::styled("→ Accept   ← Deny", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
                    Spans::from("")
                ]);
            }
            Message::ToolCallResult { result, timestamp } => {
                text.extend(vec![
                    Spans::from(vec![Span::styled(result, Style::default().fg(Color::White))]),
                    Spans::from(vec![Span::styled(format!("{}", timestamp.format("%H:%M")), Style::default().fg(Color::DarkGray))]),
                    Spans::from("")
                ]);
            }
            Message::ToolCallDenied { tool_call, original_message, timestamp } => {
                let (desc, color) = match tool_call {
                    ToolCall::ReadFile { path } => (format!("[TOOL CALL DENIED] read_file: {}", path), Color::Red),
                    ToolCall::ReadDirectory { path } => (format!("[TOOL CALL DENIED] read_directory: {}", path), Color::Red),
                    ToolCall::EditFile { path, .. } => (format!("[TOOL CALL DENIED] edit_file: {}", path), Color::Red),
                };
                text.extend(vec![
                    Spans::from(vec![Span::styled(desc, Style::default().fg(color).add_modifier(Modifier::BOLD))]),
                    Spans::from(vec![Span::styled(format!("[tool_call: {}]", original_message), Style::default().fg(Color::DarkGray))]),
                    Spans::from(vec![Span::styled(format!("{}", timestamp.format("%H:%M")), Style::default().fg(Color::DarkGray))]),
                    Spans::from("")
                ]);
            }
        }
    }
    let para = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

fn draw_input_area<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    // Modern input box, prominent border, placeholder
    let input_text = if app.input.is_empty() {
        Spans::from(vec![Span::styled("Type your message...", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))])
    } else {
        Spans::from(vec![Span::styled(&app.input, Style::default().fg(Color::White))])
    };
    
    let border_style = if app.is_loading {
        Style::default().fg(Color::Yellow) // Yellow border when loading
    } else {
        Style::default().fg(Color::Cyan) // Normal cyan border
    };
    
    let title = if app.is_loading {
        "Input (processing...) - Enter to send, Ctrl+C to quit"
    } else {
        "Input (Enter to send, Ctrl+C to quit)"
    };
    
    let input = Paragraph::new(input_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title),
        )
        .alignment(Alignment::Left);
    f.render_widget(Clear, area);
    f.render_widget(input, area);
    
    // Show cursor in input only if not loading
    if !app.is_loading {
        let cursor_x = area.x + 1 + app.input_cursor_position as u16;
        let cursor_y = area.y + 1;
        f.set_cursor(cursor_x, cursor_y);
    }
} 
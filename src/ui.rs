use crate::app::{App, Message, ToolCall};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Clear},
    Frame,
};

const TITLE_ART: [&str; 6] = [
    " ██████╗ ██╗   ██╗██╗██╗     ██╗     ",
    "██╔═══██╗██║   ██║██║██║     ██║     ",
    "██║   ██║██║   ██║██║██║     ██║     ",
    "██║▄▄ ██║██║   ██║██║██║     ██║     ",
    "╚██████╔╝╚██████╔╝██║███████╗███████╗",
    " ╚══▀▀═╝  ╚═════╝ ╚═╝╚══════╝╚══════╝",
];

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
    let mut y_offset = area.y;
    let max_width = area.width;
    for message in &app.messages {
        match message {
            Message::User { content, timestamp } => {
                let msg = Paragraph::new(Spans::from(vec![
                    Span::styled(">", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(content, Style::default().fg(Color::White)),
                ]))
                .alignment(Alignment::Right)
                .wrap(Wrap { trim: true });
                let meta = Paragraph::new(Spans::from(vec![
                    Span::styled(
                        format!("USER {}", timestamp.format("%H:%M")),
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                    ),
                ]))
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::DarkGray));
                let meta_area = Rect { x: area.x, y: y_offset, width: area.width, height: 1 };
                let msg_area = Rect { x: area.x, y: y_offset + 1, width: max_width, height: 2 };
                f.render_widget(meta, meta_area);
                f.render_widget(msg, msg_area);
                y_offset += 3;
            }
            Message::Assistant { content, timestamp } => {
                let msg = Paragraph::new(Spans::from(vec![
                    Span::styled("<", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::raw(" "),
                    Span::styled(content, Style::default().fg(Color::White)),
                ]))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
                let meta = Paragraph::new(Spans::from(vec![
                    Span::styled(
                        format!("ASSISTANT {}", timestamp.format("%H:%M")),
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                    ),
                ]))
                .alignment(Alignment::Left)
                .style(Style::default().fg(Color::DarkGray));
                let meta_area = Rect { x: area.x, y: y_offset, width: area.width, height: 1 };
                let msg_area = Rect { x: area.x, y: y_offset + 1, width: max_width, height: 2 };
                f.render_widget(meta, meta_area);
                f.render_widget(msg, msg_area);
                y_offset += 3;
            }
            Message::PendingToolCall { tool_call, original_message, timestamp } => {
                let (desc, color) = match tool_call {
                    ToolCall::ReadFile { path } => (format!("[TOOL CALL] read_file: {}", path), Color::Green),
                    ToolCall::ReadDirectory { path } => (format!("[TOOL CALL] read_directory: {}", path), Color::Blue),
                };
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(color))
                    .title("Pending Tool Call");
                let lines = vec![
                    Spans::from(Span::styled(desc, Style::default().fg(color).add_modifier(Modifier::BOLD))),
                    Spans::from(Span::styled(format!("[tool_call: {}]", original_message), Style::default().fg(Color::DarkGray))),
                    Spans::from(Span::styled("→ Accept   ← Deny", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                ];
                let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
                let area_tool = Rect { x: area.x, y: y_offset, width: area.width, height: 4 };
                f.render_widget(para, area_tool);
                y_offset += 5;
            }
            Message::ToolCallResult { result, timestamp } => {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .title("Tool Result");
                let lines = vec![
                    Spans::from(Span::styled(result, Style::default().fg(Color::White))),
                    Spans::from(Span::styled(format!("{}", timestamp.format("%H:%M")), Style::default().fg(Color::DarkGray))),
                ];
                let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
                let area_tool = Rect { x: area.x, y: y_offset, width: area.width, height: 4 };
                f.render_widget(para, area_tool);
                y_offset += 5;
            }
            Message::ToolCallDenied { tool_call, original_message, timestamp } => {
                let (desc, color) = match tool_call {
                    ToolCall::ReadFile { path } => (format!("[TOOL CALL DENIED] read_file: {}", path), Color::Red),
                    ToolCall::ReadDirectory { path } => (format!("[TOOL CALL DENIED] read_directory: {}", path), Color::Red),
                };
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .title("Tool Call Denied");
                let lines = vec![
                    Spans::from(Span::styled(desc, Style::default().fg(color).add_modifier(Modifier::BOLD))),
                    Spans::from(Span::styled(format!("[tool_call: {}]", original_message), Style::default().fg(Color::DarkGray))),
                    Spans::from(Span::styled(format!("{}", timestamp.format("%H:%M")), Style::default().fg(Color::DarkGray))),
                ];
                let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
                let area_tool = Rect { x: area.x, y: y_offset, width: area.width, height: 4 };
                f.render_widget(para, area_tool);
                y_offset += 5;
            }
        }
    }
    // Show streaming message if present
    if let Some(content) = &app.streaming_message {
        let prefix = "<";
        let prefix_style = Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD);
        let align = Alignment::Left;
        let msg = Paragraph::new(Spans::from(vec![
            Span::styled(prefix, prefix_style),
            Span::raw(" "),
            Span::styled(content, Style::default().fg(Color::White)),
            Span::styled("▋", Style::default().fg(Color::Yellow)), // Blinking cursor effect
        ]))
        .alignment(align)
        .wrap(Wrap { trim: true });
        let meta = Paragraph::new(Spans::from(vec![
            Span::styled("ASSISTANT (typing...)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ]))
        .alignment(align)
        .style(Style::default().fg(Color::DarkGray));
        let meta_area = Rect {
            x: area.x,
            y: y_offset,
            width: area.width,
            height: 1,
        };
        let msg_area = Rect {
            x: area.x,
            y: y_offset + 1,
            width: max_width,
            height: 2,
        };
        f.render_widget(meta, meta_area);
        f.render_widget(msg, msg_area);
    }
    // Show error if present
    if let Some(error) = &app.error_message {
        let error = Paragraph::new(error.clone())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error, area);
    }
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
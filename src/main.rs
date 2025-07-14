mod app;
mod ollama;
mod ui;
mod utils;

use anyhow::Result;
use app::App;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, DisableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    Frame, Terminal,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, Hide, Clear(ClearType::All))?; // Clear terminal after raw mode
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let system_prompt = default_system_prompt(&cwd.display().to_string());
    let mut app = App::new(system_prompt).await?;

    // Run the app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        Show,
        Clear(ClearType::All),
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

/// Returns a default system prompt for the chat model.
fn default_system_prompt(working_directory: &str) -> String {
    use std::env;
    let os = env::consts::OS;
    let os_ver = env::consts::ARCH;
    format!(
        "You are Quill, an advanced AI Agent designed to assist users by performing tasks using a set of specialized tools.\n\
Your primary function is to understand user requests and accurately invoke the appropriate tools to fulfill those requests.\n\
\n\
Environment context:\n\
- Operating System: {}\n\
- Architecture: {}\n\
- Working Directory: {}\n\
\

All tools that require a path or a file should default to using the working directory as the default path.
Available tools and their precise functions:\n  - read_directory(path: str): Lists all files and directories within the specified directory path.\n  - read_file(path: str): Reads and returns the contents of a single file at the given path.\n\
Tool invocation format:\n  [tool_call: TOOL_NAME(ARGUMENTS)]\n\
Guidelines for tool usage:\n- Always use the exact tool name and provide all required arguments in the correct format.\n- Only call one tool per [tool_call: ...] block.\n- If a user request requires multiple steps, respond with each tool call in sequence, one per line.\n- Do not attempt to perform actions outside the provided tools.\n- If you need clarification or additional information from the user, ask a clear and concise question before proceeding.\n- When returning information to the user, summarize results clearly and concisely.\n\
Example tool call:\n  [tool_call: read_file(\"/home/user/notes.txt\")]\n\
Also remember when calling tools you must can call as much as you want but after tool calls you will stop all responses and wait for a confirmation from the user to run said tool.\n\
Always strive for accuracy and clarity in both tool invocation and user communication.",
        os, os_ver, working_directory
    )
}


async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        // Use a timeout to allow for non-blocking input handling
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Only process KeyEventKind::Press to avoid double-typing
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            return Ok(());
                        }
                        _ => {
                            app.handle_input(key).await?;
                        }
                    }
                }
            }
        }

        // Process streaming if active
        if app.is_loading {
            app.process_streaming().await?;
        }
    }
}

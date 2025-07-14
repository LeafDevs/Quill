# Quill - AI Development Assistant

A beautiful terminal-based AI assistant for developers, built with Rust and designed to work seamlessly with Ollama models.

## Features

- 🎨 **Beautiful TUI Interface** - Aqua-pink-aqua gradient title with modern terminal UI
- 🤖 **Ollama Integration** - Connect to your local Ollama instance
- 📝 **Model Selection** - Browse and select from available models using arrow keys
- 💬 **Real-time Chat** - Interactive conversation with AI models
- ⚡ **Fast & Responsive** - Built in Rust for optimal performance
- 🔧 **Developer Friendly** - Perfect for coding assistance and development tasks

## Screenshots

```
┌─────────────────────────────────────────────────────────────┐
│                        Q u i l l                           │
│                  AI Development Assistant                   │
└─────────────────────────────────────────────────────────────┘
┌─────────────────┐ ┌─────────────────────────────────────────┐
│ Select Model    │ │ Model Info                              │
│ (↑/↓)          │ │                                         │
│                 │ │ Model: llama2:7b                        │
│ > llama2:7b     │ │ Size: 3.8 GB                            │
│   codellama:7b  │ │ Modified: 2024-01-15 14:30             │
│   mistral:7b    │ │                                         │
└─────────────────┘ └─────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│ Chat                                                       │
│                                                             │
│ [USER] 14:25                                                │
│   How do I implement a binary search tree in Rust?         │
│                                                             │
│ [ASSISTANT] 14:25                                           │
│   Here's how to implement a binary search tree in Rust:    │
│                                                             │
│   ```rust                                                   │
│   #[derive(Debug)]                                          │
│   struct Node<T> {                                          │
│       value: T,                                             │
│       left: Option<Box<Node<T>>>,                          │
│       right: Option<Box<Node<T>>>,                         │
│   }                                                         │
│   ```                                                       │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│ Input (Enter to send, Ctrl+C to quit)                      │
│ You: _                                                      │
└─────────────────────────────────────────────────────────────┘
```

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Ollama](https://ollama.ai/) installed and running
- At least one Ollama model downloaded

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd quill
```

2. Build the project:
```bash
cargo build --release
```

3. Run Quill:
```bash
cargo run --release
```

## Usage

### Starting Quill

1. Make sure Ollama is running:
```bash
ollama serve
```

2. Launch Quill:
```bash
cargo run --release
```

### Navigation

- **Arrow Keys (↑/↓)**: Navigate through available models
- **Type**: Enter your message in the input area
- **Enter**: Send message to the selected AI model
- **Ctrl+C** or **q**: Quit the application

### Features

- **Model Selection**: Use arrow keys to browse and select from your available Ollama models
- **Real-time Chat**: Have conversations with AI models in real-time
- **Message History**: View your conversation history in the chat area
- **Error Handling**: Clear error messages if something goes wrong
- **Loading States**: Visual feedback when the AI is processing your request

## Configuration

Quill connects to Ollama on the default port `11434`. If you're running Ollama on a different port, you can modify the `base_url` in `src/ollama.rs`.

## Development

### Project Structure

```
src/
├── main.rs      # Application entry point
├── app.rs       # Application state and logic
├── ollama.rs    # Ollama API client
├── ui.rs        # Terminal UI components
└── utils.rs     # Utility functions
```

### Building for Development

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

## Troubleshooting

### Common Issues

1. **"Failed to fetch models"**: Make sure Ollama is running (`ollama serve`)
2. **"Failed to get response"**: Check if the selected model is downloaded
3. **Terminal display issues**: Ensure your terminal supports UTF-8 and colors

### Getting Help

If you encounter any issues:

1. Check that Ollama is running and accessible
2. Verify you have at least one model downloaded (`ollama list`)
3. Ensure your terminal supports the required features
4. Check the error messages displayed in the application

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [tui-rs](https://github.com/fdehau/tui-rs) for the terminal UI
- Powered by [Ollama](https://ollama.ai/) for AI model access
- Inspired by modern terminal applications and AI development tools 
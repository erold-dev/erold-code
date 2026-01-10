# Erold Code

> AI-native development agent with workflow enforcement. Understand, Plan, Execute, Learn.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)

Erold Code is not just another AI coding assistant. It's a **workflow-enforced development methodology** that makes your AI agent smarter over time.

## Why Erold Code?

| Traditional AI CLI | Erold Code |
|-------------------|------------|
| Tools available, use as you wish | Structured workflow enforced |
| Context lost between sessions | Persistent knowledge base |
| No task tracking | Full project management integration |
| Quality is optional | Quality is mandatory |
| Learn nothing, repeat mistakes | Learn everything, compound knowledge |

## The Erold Way

```
1. UNDERSTAND  →  Load context, fetch guidelines, check knowledge base
2. PLAN        →  Decompose into tasks, create in Erold, identify dependencies
3. EXECUTE     →  Work incrementally, track progress, validate against guidelines
4. LEARN       →  Save patterns to knowledge base, update for next time
```

## Features

- **Workflow Enforcement** - Mandatory planning phase, read-before-edit gates
- **Guidelines Integration** - Auto-fetch relevant coding guidelines before writing code
- **Knowledge Base** - Persistent memory that grows with every task
- **Task Management** - Auto-create and track tasks in Erold PM
- **Learning Loop** - Save mistakes, learnings, and patterns for future reference

## Project Structure

```
erold-code/
├── crates/
│   ├── erold-api/        # Erold PM API client
│   ├── erold-config/     # Configuration management
│   ├── erold-llm/        # LLM provider integration
│   ├── erold-tools/      # Tool implementations
│   ├── erold-tui/        # Terminal UI
│   ├── erold-web/        # Web interface
│   └── erold-workflow/   # Workflow state machine
├── src/                  # Main binary
├── docs/                 # Documentation
├── WORKFLOW.md           # Workflow specification
└── ROADMAP.md            # Technical roadmap
```

## Quick Start

```bash
# Clone the repository
git clone https://github.com/erold-dev/erold-code.git
cd erold-code

# Build
cargo build --release

# Run
./target/release/erold
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

## Related Repositories

| Repository | Description |
|------------|-------------|
| [www](https://github.com/erold-dev/www) | Marketing website + 158 coding guidelines |
| [mcp-server](https://github.com/erold-dev/mcp-server) | MCP server for AI assistants |
| [cli](https://github.com/erold-dev/cli) | Simple CLI for Erold PM |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**[erold.dev](https://erold.dev)** - Open source, AI-native project management

# Contributing to Erold Code

Thank you for your interest in contributing to Erold Code! This document provides guidelines for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/erold-code.git`
3. Install Rust toolchain: `rustup install stable`
4. Create a branch: `git checkout -b feature/your-feature-name`

## Project Structure

```
erold-code/
├── crates/
│   ├── erold-api/        # Erold API client
│   ├── erold-config/     # Configuration management
│   ├── erold-llm/        # LLM integration
│   ├── erold-tools/      # Tool implementations
│   ├── erold-tui/        # Terminal UI
│   ├── erold-web/        # Web interface
│   └── erold-workflow/   # Workflow enforcement
├── src/                  # Main binary
├── docs/                 # Documentation
├── WORKFLOW.md           # Workflow specification
└── ROADMAP.md            # Technical roadmap
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Run locally
cargo run
```

## Git Workflow

### Branching Strategy

```
main                    # Production-ready code
├── feature/*           # New features (feature/add-tool)
├── bugfix/*            # Bug fixes (bugfix/fix-crash)
├── hotfix/*            # Urgent fixes (hotfix/security-patch)
└── docs/*              # Documentation (docs/update-readme)
```

### Branch Naming

| Type | Pattern | Example |
|------|---------|---------|
| Feature | `feature/short-description` | `feature/add-guidelines-tool` |
| Bug fix | `bugfix/short-description` | `bugfix/fix-workflow-state` |
| Hotfix | `hotfix/short-description` | `hotfix/security-patch` |
| Docs | `docs/short-description` | `docs/update-workflow-docs` |

### Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add guidelines API tool
fix: resolve workflow state transition bug
docs: update WORKFLOW.md
chore: bump dependencies
refactor: simplify LLM provider interface
```

## Pull Request Process

1. Create a branch from `main`
2. Make your changes
3. Ensure all tests pass: `cargo test`
4. Format code: `cargo fmt`
5. Run lints: `cargo clippy`
6. Submit PR with clear description
7. Address review feedback
8. Squash merge to main

## Code Style

### Rust
- Follow Rust standard conventions
- Use `cargo fmt` for formatting
- Address all `clippy` warnings
- Write tests for new functionality
- Document public APIs

### Architecture
- Keep crates focused and single-purpose
- Use dependency injection where possible
- Follow the workflow enforcement patterns in WORKFLOW.md

## Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p erold-workflow

# Run with output
cargo test -- --nocapture
```

## Reporting Issues

When reporting bugs, include:
- Steps to reproduce
- Expected vs actual behavior
- Environment (OS, Rust version)
- Error messages and logs

## Questions?

- Open a [discussion](https://github.com/erold-dev/erold-code/discussions)
- Check existing issues first

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

Thank you for helping make Erold Code better!

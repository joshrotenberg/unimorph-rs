# Contributing

Thank you for your interest in contributing to unimorph-rs!

## Getting Started

### Prerequisites

- Rust (latest stable)
- Git

### Clone and Build

```bash
git clone https://github.com/joshrotenberg/unimorph-rs
cd unimorph-rs
cargo build
```

### Run Tests

```bash
cargo test --all-features
```

### Run Lints

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Project Structure

```
unimorph-rs/
├── crates/
│   ├── unimorph-core/     # Core library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs   # Core types
│   │   │   ├── store.rs   # SQLite backend
│   │   │   ├── query.rs   # Query builder
│   │   │   ├── repository.rs
│   │   │   └── export.rs
│   │   └── Cargo.toml
│   │
│   └── unimorph-cli/      # CLI application
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/  # Command implementations
│       │   ├── config.rs
│       │   └── colors.rs
│       └── Cargo.toml
│
├── docs/                   # mdBook documentation
│   ├── book.toml
│   └── src/
│
└── Cargo.toml             # Workspace root
```

## Making Changes

### Creating a Branch

```bash
git checkout -b feat/your-feature
# or
git checkout -b fix/your-fix
```

### Commit Messages

Use conventional commits:

```
feat: add new feature
fix: resolve bug in X
docs: update documentation
test: add tests for Y
refactor: restructure Z
```

### Pull Requests

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and lints
5. Submit a pull request

## Development Guidelines

### Code Style

- Follow Rust idioms
- Use `rustfmt` for formatting
- Address all `clippy` warnings
- Document public APIs

### Testing

- Add tests for new features
- Maintain test coverage
- Use meaningful test names

```rust
#[test]
fn inflect_returns_all_forms() {
    // ...
}
```

### Error Handling

- Use `thiserror` for library errors
- Use `anyhow` for CLI errors
- Provide helpful error messages

### Documentation

- Document public items
- Include examples in doc comments
- Update mdBook docs for user-facing changes

## Areas for Contribution

### Good First Issues

Look for issues labeled `good first issue` on GitHub.

### Feature Ideas

- Additional export formats
- Performance optimizations
- New query capabilities
- Language-specific features

### Documentation

- Fix typos
- Improve examples
- Add tutorials
- Translate documentation

### Testing

- Add edge case tests
- Improve test coverage
- Add integration tests

## Code of Conduct

Be respectful and constructive. We welcome contributors of all experience levels.

## Getting Help

- Open a GitHub issue for bugs
- Use discussions for questions
- Check existing issues before creating new ones

## License

Contributions are licensed under the same terms as the project (MIT/Apache-2.0).

# Contributing to Ajen

Thank you for your interest in contributing to Ajen! This guide will help you get started.

## Getting Started

1. **Fork** the repository on GitHub
2. **Clone** your fork locally:
   ```bash
   git clone https://github.com/<your-username>/ajen.git
   cd ajen
   ```
3. **Set up** the development environment:
   ```bash
   cp .env.example .env
   # Configure your API KEYS in .env

   # Start with Docker Compose
   docker compose up

   # Build and run
   cargo run
   ```

## Development Workflow

1. Create a new branch from `main`:
   ```bash
   git checkout -b feat/my-feature
   ```
2. Make your changes
3. Ensure the project builds:
   ```bash
   cargo build
   ```
4. Run tests:
   ```bash
   cargo test
   ```
5. Run clippy:
   ```bash
   cargo clippy -- -D warnings
   ```
6. Format your code:
   ```bash
   cargo fmt
   ```
7. Commit your changes and push to your fork
8. Open a pull request against `main`

## What to Contribute

- **Bug fixes** — Found a bug? Fix it and open a PR.
- **Employee manifests** — Create new AI employee roles with a `manifest.yaml` and `PERSONA.md`.
- **Tools** — Add new tools that employees can use (in `crates/ajen-tools/`).
- **Provider support** — Help add LLM providers.

Check [open issues](https://github.com/ajenhq/ajen/issues) for tasks labeled `good first issue` or `help wanted`.

## Project Structure

```
crates/
  ajen-core/       # Domain types and traits
  ajen-provider/   # LLM provider clients
  ajen-tools/      # Tool registry and implementations
  ajen-engine/     # Employee runtime, director, manifests
  ajen-server/     # HTTP + WebSocket server
web/               # Astro + Preact dashboard
employee-manifests/  # Built-in employee definitions
```

## Code Style

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Run `cargo clippy` and address all warnings
- Keep functions focused and small
- Add doc comments for public APIs

## Commit Messages

Use clear, descriptive commit messages:

- `feat: add openai provider support`
- `fix: handle empty tool response in react loop`
- `docs: update quick start instructions`
- `refactor: extract budget logic into shared module`

## Pull Request Guidelines

- Keep PRs focused — one feature or fix per PR
- Include a clear description of what changed and why
- Link related issues (e.g., `Closes #42`)
- Ensure CI passes before requesting review
- Be responsive to review feedback

## Creating Employee Manifests

To add a new AI employee role:

1. Create a directory under `employee-manifests/`:
   ```
   employee-manifests/my-role/
     manifest.yaml
     PERSONA.md
   ```
2. Define the manifest following the `ajen.dev/v1` schema (see existing manifests for examples)
3. Write a persona document that describes the employee's behavior and expertise

## Reporting Bugs

- Use the [bug report template](https://github.com/ajenhq/ajen/issues/new?template=bug_report.md)
- Include steps to reproduce
- Include your environment (OS, Rust version, etc.)

## Suggesting Features

- Use the [feature request template](https://github.com/ajenhq/ajen/issues/new?template=feature_request.md)
- Explain the use case and expected behavior

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).

# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Ajen, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please email: **security@ajen.dev**

Include the following in your report:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial assessment**: Within 1 week
- **Fix and disclosure**: Coordinated with the reporter

## Scope

Security concerns relevant to this project include:

- **API key exposure** — Leaking LLM provider credentials
- **Prompt injection** — Malicious input that manipulates AI employee behavior
- **File system access** — Tools reading/writing outside intended directories
- **WebSocket hijacking** — Unauthorized access to company event streams
- **Dependency vulnerabilities** — Known CVEs in Rust or npm dependencies

## Best Practices for Contributors

- Never commit API keys, secrets, or credentials
- Validate and sanitize all user inputs
- Use the `.env` file for sensitive configuration (never hardcode)
- Run `cargo audit` periodically to check for dependency vulnerabilities

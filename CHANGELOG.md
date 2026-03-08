# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Core engine with ReAct loop for AI employee execution
- Anthropic provider with Claude model support and cost tracking
- Employee manifest system (YAML + persona files) with 14 built-in roles
- File system tools (read, write, list directory)
- Event bus for real-time streaming of employee actions
- Comms bus for inter-employee messaging
- Budget tracker with per-employee and per-task cost limits
- Director for orchestrating multi-agent workflows
- Axum HTTP server with REST API and WebSocket support
- Human-as-board approval flow
- Docker and Docker Compose setup

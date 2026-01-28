# AI Agent Protocol for lib3mf-rs

## Welcome Agents (Claude, Gemini, et al.)

This repository contains a pure Rust implementation of the 3MF standard.

### Project Context
- **Goal**: Enterprise-grade, spec-compliant 3MF library.
- **Language**: Rust (Safe, Performant).
- **Core Philosophy**: Spec-driven development. Read the specs in `specs/` and the skills in `skills/` before coding.

### Rules of Engagement
1. **Read-First**: Always read relevant `SKILL.md` files before attempting implementation.
2. **Task Tracking**: Update `task.md` when starting/finishing major steps.
3. **Artifacts**: Store major plans in the artifact system (or `docs/plans` if artifacts unavailable).
4. **Testing**: TDD is encouraged. Run tests frequently.
5. **No C++**: Do not introduce C/C++ dependencies unless strictly authorized.
6. **Error Handling**: Use `thiserror` for libs, `anyhow` for bins.

### Skills System
This project uses a unified skills system located in `skills/`.
- Accessible via `.claude/skills` or `.gemini/skills`.
- Format: Markdown with YAML frontmatter.

### Directory Structure
- `crates/`: All Rust code (workspace members).
- `docs/`: Human-readable documentation.
- `specs/`: Official PDF specifications.
- `tests/`: Integration tests.
- `models/`: Test files (e.g. `Benchy.3mf` or `Benchy.stl`).

### Current Phase
Refer to `task.md` for the current active phase.

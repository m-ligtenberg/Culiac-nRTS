---
description: Focused development context loading for Battle of Culiacán RTS
---

# Repository Situation Intelligence (RSI)

Load focused development context for efficient AI-assisted Rust development. Optimized for token efficiency and current development needs.

## Core Context Loading

1. **Essential Project Files**:
   - Read `CLAUDE.md` for development context and best practices
   - Read `README.md` for current project status
   - Check `Cargo.toml` for dependencies and project configuration

2. **Recent Development Activity**:
   - Run `git status` to see current changes
   - Run `git log --oneline -10` for recent commit history
   - Check `git diff --stat` for current modification summary

3. **Current Development State**:
   - List files in `src/` directory structure
   - Check for any compilation issues with `cargo check --quiet`
   - Review any TODO comments in source code

## Focused Analysis Areas

Based on the Battle of Culiacán RTS project:

- **Core Game Systems**: Audio, multiplayer, save system, UI components
- **Recent Modularization**: Check new module structure and organization
- **Performance Monitoring**: Spatial grid, benchmarks, optimization status
- **Authentication System**: New auth modules and database integration

## Smart Context Selection

- Prioritize recently modified files
- Focus on current development phase (modular architecture)
- Load relevant configuration and build files
- Skip verbose documentation unless specifically needed

## Token Efficiency

- Use `cargo check` instead of `cargo build` per CLAUDE.md guidelines
- Load only essential context for current development session
- Compress information using structured summaries
- Avoid loading entire file contents unless necessary for current task

## Output Format

Provide concise development status including:
- Current branch and uncommitted changes
- Recent development focus areas
- Any immediate issues requiring attention
- Next logical development steps based on project state
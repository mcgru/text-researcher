# Story 1.1: Cargo workspace и структура крейтов

Status: ready-for-dev

## Story

As a разработчик,
I want инициализированный Cargo workspace с тремя крейтами (`udpipe-ffi`, `core`, `app`) и `.gitignore`,
so that можно начинать разработку с правильной структурой проекта.

## Acceptance Criteria

1. `cargo build --workspace` компилируется без ошибок
2. workspace `Cargo.toml` содержит `[workspace] members = ["crates/*"]`
3. `.gitignore` исключает `target/` и `Cargo.lock`
4. Все три крейта имеют корректные `Cargo.toml` с зависимостями согласно архитектуре

## Tasks / Subtasks

- [ ] Task 1: Create workspace root Cargo.toml (AC: #2)
  - [ ] Add `[workspace]` section with `members = ["crates/*"]`
  - [ ] Add `[workspace.package]` with version = "0.1.0", edition = "2021"
- [ ] Task 2: Create crate `udpipe-ffi` (AC: #1, #4)
  - [ ] `crates/udpipe-ffi/Cargo.toml` — lib crate, dependencies: `thiserror`, build-dependency: `bindgen = "0.70"`
  - [ ] `crates/udpipe-ffi/build.rs` — placeholder (main function, empty)
  - [ ] `crates/udpipe-ffi/src/lib.rs` — empty lib with `//! FFI bindings to libudpipe`
- [ ] Task 3: Create crate `core` (AC: #1, #4)
  - [ ] `crates/core/Cargo.toml` — lib crate, name = "text-researcher-core", dependencies: `udpipe-ffi` (path), `serde`, `serde_json`, `toml`, `thiserror`, `log`
  - [ ] `crates/core/src/lib.rs` — empty lib with `//! Core analysis engine`
- [ ] Task 4: Create crate `app` (AC: #1, #4)
  - [ ] `crates/app/Cargo.toml` — bin crate, name = "text-researcher", dependencies: `core` (path), `udpipe-ffi` (path), `ratatui = "0.30"`, `crossterm = "0.28"`, `clap = { version = "4.6", features = ["derive"] }`, `anyhow`, `log`, `env_logger`
  - [ ] `crates/app/src/main.rs` — Hello World placeholder
- [ ] Task 5: Create .gitignore (AC: #3)
  - [ ] Exclude: `target/`, `Cargo.lock`, `*.trproj`, `*.trconf`, `models/*.udpipe`
- [ ] Task 6: Verify build (AC: #1)
  - [ ] Run `cargo build --workspace`
  - [ ] All crates compile without errors

## Dev Notes

### Architecture Compliance

- **Project:** Rust edition 2021, version 0.1.0
- **Structure:** Three crates inside `crates/` directory, not at root
- **Naming:** `snake_case` for all file and directory names
- **Dependencies pinned:** ratatui 0.30, crossterm 0.28, clap 4.6, bindgen 0.70
- **This is the FIRST story** — no existing code, pure greenfield setup
- No tests required for this story (structure only, no logic)

### File Structure

```
text-researcher/
├── Cargo.toml          # workspace root
├── .gitignore
├── crates/
│   ├── udpipe-ffi/
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   └── src/
│   │       └── lib.rs
│   ├── core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── app/
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
```

### Dependencies Set

**udpipe-ffi (lib):**
- `thiserror = "1"` — for error types
- `bindgen = "0.70"` — build-dependency for FFI generation

**core (lib, name = "text-researcher-core"):**
- `udpipe-ffi = { path = "../udpipe-ffi" }` — local dependency
- `serde = { version = "1", features = ["derive"] }` — serialization
- `serde_json = "1"` — JSON support
- `toml = "0.8"` — TOML project files
- `thiserror = "1"` — error types
- `log = "0.4"` — logging facade

**app (bin, name = "text-researcher"):**
- `text-researcher-core = { path = "../core" }` — local dependency
- `udpipe-ffi = { path = "../udpipe-ffi" }` — local dependency
- `ratatui = "0.30"` — TUI framework
- `crossterm = "0.28"` — terminal backend
- `clap = { version = "4.6", features = ["derive"] }` — CLI argument parser
- `anyhow = "1"` — error handling
- `log = "0.4"` — logging facade
- `env_logger = "0.11"` — log implementation

### References

- Architecture: `_bmad-output/planning-artifacts/architecture.md` — Sections "Workspace Structure", "Starter Template Evaluation"
- Epics: `_bmad-output/planning-artifacts/epics.md` — Story 1.1

## Dev Agent Record

### Agent Model Used

_To be filled by dev agent_

### Debug Log References

_To be filled by dev agent_

### Completion Notes List

_To be filled by dev agent_

### File List

_To be filled by dev agent_

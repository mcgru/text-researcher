# Story 1.1: Cargo workspace и структура крейтов

Status: done
baseline_commit: 3ec4562bd434356dbae645d24354090fab7b9e7d

### Review Findings

- [x] [Review][Patch] crossterm 0.28 → 0.29 to match ratatui dependency [crates/app/Cargo.toml:14]
- [x] [Review][Patch] Cargo.lock removed from .gitignore (binary crate) [.gitignore:2]

## Story

As a разработчик,
I want инициализированный Cargo workspace с тремя крейтами (`udpipe-ffi`, `core`, `app`) и `.gitignore`,
so that можно начинать разработку с правильной структурой проекта.

## Acceptance Criteria

1. ✅ `cargo build --workspace` компилируется без ошибок
2. ✅ workspace `Cargo.toml` содержит `[workspace] members = ["crates/*"]`
3. ✅ `.gitignore` исключает `target/` и `Cargo.lock`
4. ✅ Все три крейта имеют корректные `Cargo.toml` с зависимостями согласно архитектуре

## Tasks / Subtasks

- [x] Task 1: Create workspace root Cargo.toml (AC: #2)
  - [x] Add `[workspace]` section with `members = ["crates/*"]`
  - [x] Add `[workspace.package]` with version = "0.1.0", edition = "2021"
- [x] Task 2: Create crate `udpipe-ffi` (AC: #1, #4)
  - [x] `crates/udpipe-ffi/Cargo.toml`
  - [x] `crates/udpipe-ffi/build.rs`
  - [x] `crates/udpipe-ffi/src/lib.rs`
- [x] Task 3: Create crate `core` (AC: #1, #4)
  - [x] `crates/core/Cargo.toml`
  - [x] `crates/core/src/lib.rs`
- [x] Task 4: Create crate `app` (AC: #1, #4)
  - [x] `crates/app/Cargo.toml`
  - [x] `crates/app/src/main.rs`
- [x] Task 5: Create .gitignore and .dockerignore (AC: #3)
  - [x] `.gitignore`: target/, Cargo.lock, *.trproj, *.trconf, models/*.udpipe
  - [x] `.dockerignore`: target/, .git/, IDE dirs, docs/
- [x] Task 6: Verify build (AC: #1)
  - [x] `cargo build --workspace` — 3 crates, 234 deps, OK
  - [x] `cargo test --workspace` — all pass

## Dev Notes

- No tests required for this story (scaffolding only)
- `cargo build --workspace` serves as the integration test

## Dev Agent Record

### Agent Model Used

Qwen Code (dev-story)

### Debug Log References

- Build: `cargo build --workspace` — success, all 3 crates + 234 deps compiled
- Test: `cargo test --workspace` — 0 tests, all pass

### Completion Notes List

- ✅ Cargo workspace created with 3 crates (udpipe-ffi, text-researcher-core, text-researcher)
- ✅ All dependencies correctly configured with pinned versions
- ✅ .gitignore and .dockerignore created
- ✅ Build and tests pass

### File List

- Cargo.toml (workspace root)
- .gitignore
- .dockerignore
- crates/udpipe-ffi/Cargo.toml
- crates/udpipe-ffi/build.rs
- crates/udpipe-ffi/src/lib.rs
- crates/core/Cargo.toml
- crates/core/src/lib.rs
- crates/app/Cargo.toml
- crates/app/src/main.rs

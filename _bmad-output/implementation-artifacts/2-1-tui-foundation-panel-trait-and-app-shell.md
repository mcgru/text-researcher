# Story 2.1: TUI foundation — Panel trait и app shell

Status: ready-for-dev

## Story

As a разработчик,
I want trait Panel, TUI event loop на ratatui, и layout с разделением экрана на зоны для панелей,
so that панели можно добавлять и компоновать независимо, а TUI запускается по `-i`.

## Acceptance Criteria

1. `text-researcher -i` открывает полноэкранный TUI
2. Экран разделён: меню (верх), основная зона (текст + свойства), статус (низ)
3. `AppState` хранит `Vec<Box<dyn Panel>>`
4. trait Panel: `render`, `handle_input`, `title`, `focusable`
5. Event loop обрабатывает клавиши, dispatch Action
6. `q` / `Ctrl+C` выходит из TUI
7. Минимальный размер терминала 80×24 проверяется

## Tasks / Subtasks

- [ ] Task 1: Panel trait + Action enum
- [ ] Task 2: AppState with panel registry
- [ ] Task 3: TUI event loop (ratatui + crossterm)
- [ ] Task 4: Layout (menu top, main area, status bottom)
- [ ] Task 5: Terminal size check (80×24 min)
- [ ] Task 6: main.rs integration (replace stub)

## Dev Notes

### Architecture Compliance

- trait Panel per AR-4
- Action enum drives all state changes
- NFR-10: min 80×24 terminal

## Dev Agent Record

_To be filled by dev agent_

# Project History

## 2026-05-26 — Product Inception & Architecture

### Technology Stack Evolution

1. **Initial: V (Vlang)** — TUI-экосистема слишком молодая (bobatea — 19 звёзд, vee — text editor engine). Отказ.
2. **Crystal** — Keimeno (базовый), crt.cr (заброшен). TUI-экосистема слишком слабая. Отказ.
3. **Final: Rust + Ratatui** — зрелый, production-grade TUI. Принято.

### Product Brief Decisions

- **Морфологический движок:** UDPipe 2 через udpipe-rs → позже выяснилось, что готового крейта нет, решено через FFI/bindgen
- **Код:** без изысков, читаемый junior-ом
- **Режимы:** интерактивный TUI + batch CLI
- **MVP:** 2 панели (текст + свойства), меню, статус-строка, RU+EN
- **v2:** панель связей, словари, классификаторы, НКРЯ
- **Инфраструктура:** Make + Docker + CI/CD

### PRD Decisions

- **Формат:** компактный, hobby-уровень, без корпоративной тяжеловесности
- **11 FR** в 3 фичах, **11 NFR** в 3 категориях
- **Batch-режим:** `text-researcher batch <file> -l <lang> [-o <out>]` → JSON (CoNLL-U)
- **Ручное редактирование:** правка свойств слова in-place, сохранение в проекте
- **Сохранение сессии:** `.trproj` — позиция курсора, кеш разбора, правки
- **CLI-аргументы:** clap с `--help`, shell completions, человеческие ошибки
- **User journeys:** UJ-1 (интерактивное исследование), UJ-2 (batch-обработка корпуса)
- **Шорткаты:** богатый набор (Ctrl+S/O/Q, e, Ctrl+L, Ctrl+C, tab, ?)
- **Навигация:** полный vim-набор + мышь

### Architecture Decisions

1. **Workspace:** 3 крейта — `udpipe-ffi` (FFI), `core` (бизнес-логика), `app` (бинарник)
2. **Panel trait:** `trait Panel { render, handle_input, title, focusable }` — расширяемость для v2
3. **FFI:** отдельный крейт `udpipe-ffi`, bindgen в `build.rs`, safe Rust API поверх unsafe
4. **Модели:** комбинированное хранение — Docker-образ включает модели + `download-model` CLI
5. **Проект:** TOML основной, JSON автоопределением, расширение `.trproj`
6. **State:** in-memory `ProjectState`, сериализация при `Ctrl+S`
7. **Vim:** полный набор (h/j/k/l, w/b/e, 0/$, gg/G, Ctrl+←/→)
8. **Мышь:** включена (ratatui + crossterm)

### Implementation Patterns

- **Ошибки:** anyhow (app), thiserror (core, udpipe-ffi)
- **Логирование:** log + env_logger
- **Именование:** Rust-стандарт (snake_case файлы/функции, CamelCase типы)
- **Тесты:** inline `#[cfg(test)]` + `tests/integration/`
- **Lint:** rustfmt + clippy

### Versioning & Git

- semver: major.minor.patch
- Коммит на каждый патч
- Пуш на каждый минор
- Remote: `git@github.com:mcgru/text-researcher.git`

### Raw Conversation Flow

```
User: это будет tui-программа для исследования текста...
  → Бриф: TUI с двумя панелями, морфология на лету

User: vlang
  → Исследование: TUI-экосистема V слабая

User: crystal
  → Исследование: TUI-экосистема Crystal слабая

User: rust + ratatui
  → Принято. Стек финализирован.

User: udpipe-rs
  → Позже выяснено: готового крейта нет → FFI/bindgen

User: batch-режим, сохранение проекта, ручное редактирование
  → Добавлены в PRD

User: clap, красивая обработка аргументов
  → NFR-5

User: workspace, trait Panel, udpipe-ffi отдельный крейт, комбинированные модели, TOML+JSON
  → Все архитектурные решения приняты

User: vim полный набор, мышь включить
  → Зафиксировано

User: semver, commit per patch, push per minor
  → Зафиксировано
```

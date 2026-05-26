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

## 2026-05-26 — Dictionary & UI Refinements

### UDPipe 2 Architecture Pivot
- **Изначально:** FFI через bindgen к C++ libudpipe
- **Реальность:** UDPipe 2 — Python/TensorFlow библиотека, нет C shared library
- **Решение:** subprocess-обёртка `udpipe-client` (вместо `udpipe-ffi`), stdin/stdout CoNLL-U
- **`make deps`:** pip install tensorflow numpy ufal.chu-liu-edmonds
- Без unsafe, без bindgen

### OpenCorpora Dictionary Integration
- БД: `.data/dict.opcorpora.sqlite3.db` (SQLite, symlink, read-only)
- Таблицы: lemmata, forms, grammemes, form_grammemes, lemma_grammemes
- Запрос: форма слова → лемма → граммемы (часть речи, род, число, падеж...)
- `DICT_PATH` env var или `.data/dict.opcorpora.sqlite3.db` по умолчанию

### UI Layout Final
- Меню (верх, 1 строка) → содержание: PropsPane (30% слева) + TextPane (70% справа) → статус (низ, 1 строка)
- Tab: Menu → Props → Text → Menu
- PropsPane: слово (жирным) + лемма + часть речи + граммемы (тип : значение)
- TextPane: перенос слов по ширине панели, подсветка текущего слова

### Dictionary Performance
- **Parallel lookup:** rayon, `lookup_batch()` — каждый поток своё SQLite-соединение (WAL mode)
- **`-j N`:** ограничение потоков (default: все логические ядра)
- **Prefetch:** `--prefetch N` — предвыборка N слов влево + N вправо
- **`--prefetch -1` (default):** фоновый префетч всего текста через `thread::spawn`
- **Negative cache:** `HashMap<String, Option<Vec<DictEntry>>>` — `None` = слово не найдено, не повторяем запрос
- UI мгновенно отзывчив: кеш проверяется первым, потом БД

### Implementation (Stories 1.2–2.7, + extras)
- Story 1.1: Cargo workspace (0.1.1)
- Story 1.2: UDPipe client — subprocess, CoNLL-U parser (0.1.3)
- Story 1.3: Core analyzer — ModelManager, Analyzer, Config (0.1.4)
- Story 1.4: Batch CLI — clap, JSON output (0.1.5)
- Epic 2 (7 stories): TUI foundation, panels, menu, save/load, editing (0.2.1–0.2.3)
- Epic 3 (3 stories): Makefile, Dockerfile, CI/CD (0.3.1)
- UI layout: 30/70 split, type:value format (0.3.2)
- Dictionary: OpenCorpora SQLite lookup (0.4.1)
- Text wrapping + dict lookup fix (0.4.2)
- Parallel dict: rayon, -j flag (0.4.3)
- Prefetch: --prefetch, background all-text (0.4.4–0.4.5)
- Negative cache (0.4.6)

### Raw Conversation Flow (continued)

User: словари, OpenCorpora БД
  → SQLite модуль, lookup по формам/леммам/граммемам

User: текст враппить, скроллить; слова не находятся
  → TextPane: перенос по ширине; фикс: lookup_current_word не вызывался (Action::None всегда)

User: параллельный поиск, -j N
  → rayon, lookup_batch(), per-thread SQLite connections

User: префетч следующих слов, --prefetch
  → prefetch_cache, prefetch_surrounding(), -1 = весь текст в фоне

User: отрицательный кеш
  → HashMap<String, Option<Vec<DictEntry>>>, None для не найденных слов

User: история в history.md, закоммить, собрать, запуш
  → Этот раздел
```

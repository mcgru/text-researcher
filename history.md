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

## 2026-05-26 — Prefetch, Log, Batch Mode Fixes

### Background Prefetch Evolution
- **0.4.4:** prefetch_cache, prefetch_surrounding()
- **0.4.5:** --prefetch -1 (весь текст в фоне), bidirectional
- **0.4.6:** negative cache (None для не найденных)
- **0.4.8:** LogPane — дебаг-панель 6 строк
- **0.4.9:** фикс: background prefetch через thread::spawn, не блокирует UI, LogPane скроллируемый, Tab: Menu→Props→Text→Log
- **0.4.10:** 50ms пауза между батчами, пословное логгирование через канал
- **0.4.11:** батч = 2 слова, лог слова через запятую на одной строке
- **0.4.12:** рамки (borders) на TextPane, PropsPane, LogPane

### Batch Mode Overhaul
- **0.5.0:** batch mode переписан на dictionary lookup (вместо UDPipe), `-f json|txt`
- **0.5.1:** компактный JSON — одна строка на слово
- **0.5.2:** порядок полей JSON: word → lemma → features
- **0.5.3:** фикс resolve_grammemes — lookup-map вместо индексов; prefer entry with real grammemes over @v-only

### Grammeme Resolution Bug
- **Причина:** `resolve_grammemes` сопоставлял SQL-результаты с missing-кодами по индексу. Если код отсутствовал в таблице grammemes (напр. `@v`), индексы сдвигались и все граммемы получали чужие имена.
- **Решение:** строить `HashMap<String, Grammeme>` из SQL-ответа и искать каждый код индивидуально. Отсутствующие коды пропускать.
- **Доп. фикс:** batch-вывод выбирает первую лемму с реальными граммемами (не пустышку с одним `@v`).

### Raw Conversation Flow (continued)

User: похоже не работает фоновый префетч, давай LogPane
  → LogPane, логгирование в реальном времени

User: префетч должен быть В ФОНЕ, приоритет — навигация
  → thread::spawn, poll_prefetch, 50ms паузы

User: батч 2 слова, лог слова через запятую
  → batch_size=2, log_prefetch_word аккумулирует строку

User: окно лога половина ширины, рамки, кэш-первый
  → borders на панелях, cache-first уже работает

User: пакетный режим, -f json|txt
  → batch.rs переписан на dictionary lookup_batch, два формата вывода

User: json компактный, поля word→lemma→features
  → serde_json::to_string вместо pretty, field order через Map

User: почини баг с @v
  → resolve_grammemes: lookup-map, не индексы; batch: best entry

User: сохрани историю, запуш
  → 0.5.4

## 2026-05-26 — JSON Field Order & Word Cleaning

### JSON Field Order Fix
- serde_json::Map = BTreeMap (сортирует ключи алфавитно) → features, lemma, word
- Решение: ручной `format!()` с фиксированным порядком: word, lemma, features
- **0.5.5**

### Word Cleaning
- `trim_matches(|c| !c.is_alphanumeric() && c != '-')` — убирает кавычки, запятые, точки, тире
- Применяется в TextPane (`parse_words`) и batch mode
- **0.5.6**

### Raw Conversation Flow (continued)

User: ошибка — обратный порядок полей в json
  → 0.5.5: ручной format!()

User: удаляй знаки препинания, кавычки
  → 0.5.6: clean_word()

User: сохрани историю, закоммить, запуш
  → Этот раздел

## 2026-05-26 — Multi-backend, Morphology, Batch, UI

### PostgreSQL + SQLite Multi-backend
- **0.6.0:** DictBackend trait, SqliteBackend + PostgresBackend, DICT_BACKEND env/config

### UI Refinements
- **0.6.1:** PropsPane word wrap
- **0.6.2:** English codes in PropsPane (alias, code), morphology.rs reference module
- **0.6.3:** POS line shows English UD code
- **0.6.4:** Compact feature line (@lem:..., @case:...) below separator
- **0.6.5:** Ctrl+C copy to clipboard via arboard
- **0.6.6:** Copy via 'c' key + double-click mouse, mouse capture enabled
- **0.6.7:** Clipboard keep-alive fix, Tab focus: Text→Props→Menu, Log not focusable
- **0.6.8:** PropsPane handles 'c'/'e' even when TextPane focused

### Batch Mode Overhaul
- **0.7.0:** Batch output: word + compact props string, OpenCorpora values in morphology
- **0.7.1:** JSON: individual lemm/props fields alongside compact props string
- **0.7.2:** Chunked batch output: 100→10 words/chunk, flush after each
- **0.7.3:** Configurable batch_chunk_size via config.json (default 10)

### Config & Help
- **0.7.4:** --init: generate documented config.json, backup with timestamp
- **0.7.5:** --init: clean config.json + config.json.txt docs
- **0.7.6:** --help-features: morphology features + POS reference tables
- **0.7.7:** --init: never overwrite, always timestamped copy alongside

### Raw Conversation Flow (continued)

User: PostgreSQL как db-end
  → DictBackend trait + SqliteBackend + PostgresBackend

User: PropsPane wrap, English codes, compact line, clipboard
  → 0.6.1–0.6.6

User: Tab focus, 'c' without focus switch
  → 0.6.7–0.6.8

User: batch output word + compact string
  → 0.7.0–0.7.2

User: config chunk size, --init, --help-features
  → 0.7.3–0.7.7

User: сохрани историю, закоммить, собери, запуш
  → Этот раздел

## 2026-05-28 — Config & Dual-Backend

### Config Refinements
- **0.7.8:** history.md updated
- **0.7.9:** config: dict_path + postgres_url fields
- **0.7.10:** MultiBackend (both), auto-detect dual mode when both paths set
- **0.7.11:** expand ~/ and /home/john/ in dict_path

### Raw Conversation Flow (continued)

User: пропиши dict_path и postgres_url в конфиге
  → 0.7.9

User: без _off, если оба заданы — параллельный поиск
  → 0.7.10: MultiBackend, auto-detect "both"

User: раскрывай ~/ и /home/john/ в пути
  → 0.7.11: expand_tilde()

User: сохрани историю, закоммить, собери, запуш
  → Этот раздел
```

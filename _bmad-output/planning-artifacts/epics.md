---
stepsCompleted: [1, 2, 3, 4]
status: complete
completedAt: '2026-05-26'
configHierarchy: 'CLI args > .trconf (session) > config.json (global) > defaults'
inputDocuments:
  - _bmad-output/planning-artifacts/prds/prd-text-researcher-2026-05-26/prd.md
  - _bmad-output/planning-artifacts/architecture.md
---

# text-researcher - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for text-researcher, decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

- **FR-1:** Запуск и восстановление сессии — авто-восстановление последнего проекта, меню загрузки при отсутствии
- **FR-2:** Отображение текста в TextPane — UTF-8, абзацы, подсветка текущего слова, скроллинг
- **FR-3:** Навигация по словам — стрелки, vim (h/j/k/l/w/b/e/0/$/gg/G), Ctrl+←/→ для предложений
- **FR-4:** Панель PropsPane — морфологические признаки слова под курсором, мгновенное обновление, human-readable
- **FR-5:** Редактирование свойств слова — шорткат `e`, inline/диалог, сохранение правок в проекте, приоритет над авто-разбором
- **FR-6:** Сохранение/загрузка проекта — Ctrl+S/Ctrl+O, `.trproj` (TOML+JSON), предупреждение о несохранённых изменениях
- **FR-7:** Меню и статус-строка — File/Edit/View/Help, Alt+буква/F10, статус: файл/позиция/язык/изменения
- **FR-8:** Клавиатурные шорткаты — Ctrl+S/O/Q, e, Ctrl+L, Ctrl+C, Tab, ?
- **FR-9:** Batch-запуск — `text-researcher batch <file> -l <lang> [-o <out>]`, exit code ≠ 0 при ошибках
- **FR-10:** JSON-вывод batch — массив предложений, каждый токен: word/lemma/upostag/xpostag/feats, формат CoNLL-U
- **FR-11:** Загрузка и переключение моделей — `./models/` или `UDPIPE_MODEL_DIR`, `-l` в batch, `Ctrl+L` в TUI

### NonFunctional Requirements

- **NFR-1:** Обновление PropsPane ≤ 50ms
- **NFR-2:** Загрузка файла ≤ 1MB за ≤ 2 секунды
- **NFR-3:** Потребление памяти ≤ 256MB (без модели)
- **NFR-4:** Код читаемый junior-разработчиком, без продвинутых паттернов
- **NFR-5:** clap с --help, автодополнение shell, понятные ошибки
- **NFR-6:** `make build` — сборка одной командой
- **NFR-7:** `make docker` — Docker-образ одной командой
- **NFR-8:** CI/CD — тесты и сборка при каждом пуше
- **NFR-9:** Терминалы true color + Unicode (xterm-256color+)
- **NFR-10:** Минимальный размер терминала 80×24
- **NFR-11:** Linux (основная), macOS (желательно), Windows (не обязательно)

### Additional Requirements

- **AR-1:** Cargo workspace из 3 крейтов: `udpipe-ffi`, `core`, `app`
- **AR-2:** FFI через bindgen в `build.rs` крейта `udpipe-ffi`
- **AR-3:** Safe Rust API поверх unsafe FFI (Udpipeline, Token, Sentence)
- **AR-4:** Trait `Panel` для всех панелей TUI (render, handle_input, title)
- **AR-5:** In-memory `ProjectState`, сериализация в `.trproj` при Ctrl+S
- **AR-6:** Кеширование UDPipe-результатов в `tokens: Vec<Vec<Token>>`
- **AR-7:** Модели в Docker-образе + `text-researcher download-model <lang>`
- **AR-8:** Error handling: anyhow (app), thiserror (core, udpipe-ffi)
- **AR-9:** Логирование: log + env_logger, RUST_LOG=info
- **AR-10:** Тесты: inline `#[cfg(test)]` + `tests/integration/`
- **AR-11:** CI/CD: GitHub Actions (lint → test → build → docker)
- **AR-12:** Поддержка мыши в TUI (crossterm)
- **AR-13:** `use`-порядок: std → external → crate → super

### UX Design Requirements

_No UX design document found. TUI UX requirements are covered by FR-2, FR-3, FR-4, FR-7, FR-8._

### FR Coverage Map

| FR | Epic | Description |
|---|---|---|
| FR-1 | Epic 2 | Запуск и восстановление сессии |
| FR-2 | Epic 2 | Отображение текста в TextPane |
| FR-3 | Epic 2 | Навигация по словам (стрелки + vim + мышь) |
| FR-4 | Epic 2 | Панель PropsPane с мгновенным обновлением |
| FR-5 | Epic 2 | Ручное редактирование свойств слова |
| FR-6 | Epic 2 | Сохранение/загрузка проекта .trproj |
| FR-7 | Epic 2 | Меню (File/Edit/View/Help) и статус-строка |
| FR-8 | Epic 2 | Клавиатурные шорткаты |
| FR-9 | Epic 1 | Batch-запуск из командной строки |
| FR-10 | Epic 1 | JSON-вывод batch-анализа (CoNLL-U) |
| FR-11 | Epic 1 | Загрузка и переключение языковых моделей |

| NFR | Epic | Description |
|---|---|---|
| NFR-1 | Epic 2 | PropsPane ≤ 50ms |
| NFR-2 | Epic 1 | Загрузка файла ≤ 1MB за ≤ 2s |
| NFR-3 | Epic 2 | Память ≤ 256MB |
| NFR-4 | All | Код читаемый junior-ом |
| NFR-5 | Epic 1 | clap с --help и автодополнением |
| NFR-6 | Epic 3 | make build |
| NFR-7 | Epic 3 | make docker |
| NFR-8 | Epic 3 | CI/CD при каждом пуше |
| NFR-9 | Epic 3 | True color + Unicode терминалы |
| NFR-10 | Epic 2 | Мин. размер терминала 80×24 |
| NFR-11 | Epic 3 | Linux (основная), macOS (желательно) |

### Epic List

### Epic 1: Пакетный анализ текста
Пользователь запускает `text-researcher batch <file> -l <lang>` и получает JSON с полным морфологическим разбором (токенизация, POS, леммы, признаки). Утилита пригодна для использования в исследовательских пайплайнах.
**FRs covered:** FR-9, FR-10, FR-11
**NFRs covered:** NFR-2, NFR-5

### Story 1.1: Cargo workspace и структура крейтов

As a разработчик,
I want инициализированный Cargo workspace с тремя крейтами (`udpipe-ffi`, `core`, `app`) и `.gitignore`,
So that можно начинать разработку с правильной структурой проекта.

**Acceptance Criteria:**

**Given** пустая директория проекта  
**When** выполняю `cargo build --workspace`  
**Then** все три крейта компилируются без ошибок  
**And** workspace `Cargo.toml` содержит `[workspace] members = ["crates/*"]`  
**And** `.gitignore` исключает `target/` и `Cargo.lock` для библиотек  

### Story 1.2: FFI-биндинги к libudpipe (крейт udpipe-ffi)

As a разработчик,
I want safe Rust-обёртку над libudpipe, генерируемую через bindgen в build.rs,
So that остальной код работает с UDPipe через идиоматичный Rust-интерфейс без unsafe.

**Acceptance Criteria:**

**Given** установленный `libudpipe-dev` в системе  
**When** вызываю `cargo build -p udpipe-ffi`  
**Then** крейт компилируется успешно  
**And** `build.rs` генерирует FFI-биндинги к libudpipe  
**And** публичный API: `Udpipeline::new(path) -> Result<Self>`, `.tokenize(text) -> Result<Vec<Sentence>>`, `.tag(&self, sentence) -> Result<Sentence>`  
**And** `Token` содержит: `word`, `lemma`, `upostag`, `xpostag`, `feats`  
**And** все unsafe-блоки изолированы в модуле `ffi.rs`  
**And** ошибки — `UdpipelineError` через thiserror  

### Story 1.3: Ядро анализа (крейт core) — ModelManager и Analyzer

As a разработчик,
I want загружать UDPipe-модели по языку и анализировать текст через удобный API с кешированием,
So that CLI и будущий TUI используют общий слой без дублирования логики.

**Acceptance Criteria:**

**Given** udpipe-ffi скомпилирован  
**When** `ModelManager::load("ru")`  
**Then** модель загружается из `./models/` или `UDPIPE_MODEL_DIR`  
**And** `ModelManager::available_languages()` возвращает список установленных моделей  
**And** `Analyzer::analyze(text)` возвращает `Vec<Sentence>` с полным разбором  
**And** повторный вызов `analyze()` для того же текста читает из кеша `Vec<Vec<Token>>`  
**And** `Analyzer::analyze_at(text, word_index)` возвращает разбор конкретного слова  
**And** ошибки — `CoreError` через thiserror  
**And** модуль `config.rs`: `GlobalConfig` (из `~/.config/text-researcher/config.json`), `SessionConfig` (из `.trconf`), приоритет: session → global → defaults  
**And** NFR-2: текст 1MB анализируется ≤ 2 сек (без учёта загрузки модели)  

### Story 1.4: Batch CLI — парсинг аргументов, конфигурация и JSON-вывод

As a исследователь,
I want запустить `text-researcher input.txt -l ru -o result.json` и получить JSON с полным морфологическим разбором,
So that я могу использовать text-researcher в своих скриптах и пайплайнах.

**Acceptance Criteria:**

**Given** core и udpipe-ffi скомпилированы  
**When** `text-researcher sample.txt -l ru -o out.json`  
**Then** программа работает в batch-режиме (режим по умолчанию)  
**And** `-i` запускает интерактивный TUI (заглушка на этом этапе: сообщение "TUI mode not yet implemented")  
**And** при запуске читается глобальный конфиг `~/.config/text-researcher/config.json`: `default_language`, `model_dir`, `log_level`  
**And** если глобального конфига нет — используются встроенные defaults: `ru`, `./models/`, `info`  
**And** аргументы командной строки переопределяют значения из глобального конфига  
**And** `-l` по умолчанию из конфига (если не указан в конфиге — `ru`)  
**And** `--help` показывает авто-справку clap со всеми флагами  
**And** без `-o` — вывод в stdout  
**And** при отсутствии файла — exit code ≠ 0, сообщение в stderr  
**And** при отсутствии модели — exit code ≠ 0, сообщение в stderr  
**And** выходной JSON: `[{ "text": "...", "tokens": [{ "word": "...", "lemma": "...", "upostag": "...", "xpostag": "...", "feats": {...} }] }]`  

### Epic 2: Интерактивное TUI-исследование
Пользователь открывает текст в терминале, перемещает курсор по словам (vim-клавиши + мышь), мгновенно видит морфологические признаки в панели свойств, исправляет ошибки авторазбора, сохраняет проект и возвращается к нему позже.
**FRs covered:** FR-1, FR-2, FR-3, FR-4, FR-5, FR-6, FR-7, FR-8
**NFRs covered:** NFR-1, NFR-3, NFR-10

### Story 2.1: TUI foundation — Panel trait и app shell

As a разработчик,
I want trait Panel, TUI event loop на ratatui, и layout с разделением экрана на зоны для панелей,
So that панели можно добавлять и компоновать независимо, а TUI запускается по `-i`.

**Acceptance Criteria:**

**Given** крейт app с зависимостями ratatui + crossterm  
**When** запускаю `text-researcher -i`  
**Then** открывается полноэкранный TUI  
**And** экран разделён на зоны: меню (верх), основная область (текст + свойства), статус (низ)  
**And** `AppState` хранит `Vec<Box<dyn Panel>>`  
**And** trait Panel: `render(&self, frame, area)`, `handle_input(&mut self, key) -> Action`, `title() -> &str`  
**And** event loop обрабатывает клавиши и dispatch Action  
**And** `q` / `Ctrl+C` выходит из TUI  
**And** NFR-10: минимальный размер терминала 80×24 проверяется при старте  

### Story 2.2: TextPane — отображение текста и навигация

As a лингвист,
I want видеть загруженный текст с подсветкой текущего слова и перемещать курсор клавиатурой,
So that я быстро ориентируюсь в тексте и выбираю слова для анализа.

**Acceptance Criteria:**

**Given** TUI запущен, `-i` передан с путём к текстовому файлу  
**When** текст загружен  
**Then** текст отображается в TextPane с сохранением абзацев и кодировкой UTF-8  
**And** текущее слово под курсором визуально выделено (инвертированные цвета)  
**And** скроллинг при достижении границ панели  
**And** навигация: стрелки, h/j/k/l, w/b, e, 0/$, gg/G, Ctrl+←/→  
**And** позиция обновляется в статус-строке (строка:колонка)  

### Story 2.3: PropsPane — морфологические свойства слова

As a лингвист,
I want видеть часть речи, лемму, род, число, падеж, время и другие признаки слова под курсором,
So that я мгновенно понимаю морфологическую структуру текста без внешних инструментов.

**Acceptance Criteria:**

**Given** текст загружен и проанализирован через Analyzer (Epic 1)  
**When** курсор перемещается на слово  
**Then** PropsPane обновляется ≤ 50ms (NFR-1)  
**And** отображает: часть речи, лемму, род, число, падеж, время, лицо, наклонение (human-readable, не коды UD)  
**And** если слово не распознано: «Нет данных»  
**And** если модель не загружена: «Модель не загружена»  
**And** признаки читаются из кеша `tokens[line][word_index]`  

### Story 2.4: MenuBar + StatusBar + shortcuts

As a пользователь,
I want строку меню, статус-строку и клавиатурные шорткаты для всех действий,
So that я управляю программой без мыши и знаю текущее состояние.

**Acceptance Criteria:**

**Given** TUI запущен  
**When** нажимаю `Alt+F` / `F10`  
**Then** меню раскрывается, навигация стрелками, Enter для выбора  
**And** File: Open, Save, Save As, Exit. Edit: Undo Edit. View: Switch Focus. Help: Shortcuts, About  
**And** статус-строка: `файл.txt | Ln 42, Col 7 | RU | *` (звёздочка = несохранённые изменения)  
**And** Ctrl+S — сохранить, Ctrl+O — открыть, Ctrl+Q — выйти (с предупреждением)  
**And** Ctrl+L — переключить язык анализа  
**And** `?` — показать/скрыть панель шорткатов (quick help)  
**And** `tab` — переключить фокус между панелями  

### Story 2.5: Сохранение и загрузка проекта (.trproj + .trconf)

As a лингвист,
I want сохранить состояние анализа в `.trproj` с конфигурацией в `.trconf` и загрузить их позже,
So that моя работа и настройки не теряются между сессиями.

**Acceptance Criteria:**

**Given** текст загружен и проанализирован  
**When** Ctrl+S  
**Then** создаётся/обновляется `.trproj` в формате TOML — позиция, кеш, правки  
**And** рядом с `.trproj` создаётся/обновляется `.trconf` в формате JSON (основной) или YAML — сессионная конфигурация: `language`, `last_cursor_position`, `ui_preferences`  
**And** `.trconf` переопределяет значения из глобального `~/.config/text-researcher/config.json` для этого конкретного проекта  
**And** если `.trproj` существует, запись атомарна (write → rename)  
**When** Ctrl+O → выбор файла `.trproj`  
**Then** проект загружается: текст из `source.file_path`, кеш разбора, правки, позиция  
**And** конфигурация читается: `.trconf` → `~/.config/text-researcher/config.json` → defaults (приоритет в этом порядке)  
**And** если `source.file_path` недоступен — сообщение + предложение указать новый путь  
**And** при открытии нового текста с несохранёнными изменениями — диалог «Сохранить перед открытием? (y/n)»  
**And** `.trconf` формат также читает YAML (автоопределение: JSON → YAML)  

### Story 2.6: Восстановление сессии при запуске

As a пользователь,
I want запустить `text-researcher -i project.trproj` или `text-researcher -i file.txt` и получить правильный режим,
So that я трачу ноль времени на ручную настройку окружения.

**Acceptance Criteria:**

**Given** программа установлена  
**When** `text-researcher -i project.trproj`  
**Then** файл определяется как проект → загружается сессия (текст, позиция, правки), конфигурация из `.trconf` (если есть рядом)
**And** `text-researcher -i file.txt`
**Then** файл определяется как текст → запускается свежий TUI с этим текстом, конфигурация из глобального `config.json`; при Ctrl+S создаётся `.trconf` рядом с текстом
**And** `text-researcher -i` (без файла)  
**Then** если есть последний сохранённый проект (путь сохранён в `~/.config/text-researcher/last.trproj`) → открывается он  
**And** если предыдущего проекта нет → меню выбора файла  
**And** определение `.trproj` vs plain text — по расширению и структуре содержимого  

### Story 2.7: Ручное редактирование + поддержка мыши

As a лингвист,
I want исправить неверно определённые признаки слова прямо в TUI и видеть свои правки при следующем открытии,
So that автоматические ошибки не портят мой анализ.

**Acceptance Criteria:**

**Given** слово под курсором  
**When** нажимаю `e`  
**Then** открывается inline-редактор: можно изменить часть речи, род, число, падеж, время и т.д.  
**And** Enter сохраняет правку, Escape отменяет  
**And** слово с ручной правкой получает визуальный индикатор (другой цвет/символ) в TextPane  
**And** правка сохраняется в `project.edits: HashMap<word_index, TokenOverride>`  
**And** при повторном открытии `.trproj` правки имеют приоритет над авторазбором  
**And** мышь (NFR-12): клик по слову в TextPane перемещает курсор на это слово  
**And** колёсико мыши скроллит текст  

### Epic 3: Инфраструктура и поставка
Проект собирается одной командой (`make build`), пакуется в Docker (`make docker`), тестируется и линтуется автоматически при каждом пуше (GitHub Actions: lint → test → build → docker).
**FRs covered:** (none — infrastructure)
**NFRs covered:** NFR-6, NFR-7, NFR-8, NFR-9, NFR-11

### Story 3.1: Makefile — build, test, lint

As a разработчик,
I want единый Makefile с целями build/test/lint/clean,
So that сборка и проверка выполняются одной командой без запоминания флагов cargo.

**Acceptance Criteria:**

**Given** Makefile в корне проекта  
**When** `make build`  
**Then** `cargo build --release --workspace`  
**And** `make test` → `cargo test --workspace`  
**And** `make lint` → `cargo clippy --workspace -- -D warnings && cargo fmt --check`  
**And** `make clean` → `cargo clean`  

### Story 3.2: Docker — multi-stage образ с моделями

As a пользователь,
I want запустить text-researcher в Docker одной командой, с готовыми UDPipe-моделями в образе,
So that мне не нужно устанавливать зависимости и качать модели вручную.

**Acceptance Criteria:**

**Given** Dockerfile в корне проекта  
**When** `make docker`  
**Then** собирается образ `text-researcher:latest`  
**And** multi-stage: builder (сборка Rust) → runner (минимальный, ubuntu:22.04 + libudpipe)  
**And** `COPY models/ /app/models/` — модели внутри образа  
**And** `docker run text-researcher -i` запускает TUI  
**And** `docker run -v $(pwd):/data text-researcher /data/input.txt -o /data/out.json` работает с монтированием  

### Story 3.3: CI/CD — GitHub Actions

As a разработчик,
I want автоматический прогон lint → test → build при каждом push,
So that ошибки выявляются до того, как код попадёт в main.

**Acceptance Criteria:**

**Given** `.github/workflows/ci.yml`  
**When** push в любую ветку  
**Then** workflow: checkout → cargo lint → cargo test → cargo build --release → docker build  
**And** кеширование `~/.cargo` и `target/` между запусками  
**And** ОС: ubuntu-latest  
**And** clippy warnings = failure  
**And** при падении любого шага workflow помечается красным

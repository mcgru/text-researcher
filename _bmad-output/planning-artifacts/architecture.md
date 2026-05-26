---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
inputDocuments:
  - _bmad-output/planning-artifacts/briefs/brief-text-researcher-2026-05-26/brief.md
  - _bmad-output/planning-artifacts/prds/prd-text-researcher-2026-05-26/prd.md
workflowType: 'architecture'
lastStep: 8
status: 'complete'
completedAt: '2026-05-26'
project_name: 'text-researcher'
user_name: 'mcgru'
date: '2026-05-26'
---

# Architecture Decision Document

## Project Context Analysis

### Requirements Overview

**Functional Requirements (11 FR, 3 features):**

- **Интерактивный TUI (FR-1–FR-8):** запуск/восстановление сессии, отображение текста в TextPane, навигация по словам (стрелки + vim-клавиши), панель морфологических свойств (PropsPane) с мгновенным обновлением, ручное редактирование свойств слова, сохранение/загрузка проекта, меню и статус-строка, клавиатурные шорткаты. Реализует UJ-1.
- **Batch CLI (FR-9–FR-10):** запуск `text-researcher batch <file> -l <lang> [-o <out>]`, вывод JSON в формате CoNLL-U. Реализует UJ-2.
- **Управление моделями (FR-11):** загрузка UDPipe-моделей из `./models/` или `UDPIPE_MODEL_DIR`, переключение языков (ru ↔ en).

**Non-Functional Requirements (11 NFR):**

- **Производительность:** ≤50ms обновление панели свойств, ≤2s загрузка файла 1MB, ≤256MB RAM
- **Код:** Rust, читаемый junior-ом, без продвинутых паттернов
- **Инструменты:** `make build`, `make docker`, CI/CD на каждом пуше
- **CLI:** clap с `--help`, автодополнение shell, человеческие сообщения об ошибках
- **Совместимость:** true color + Unicode терминалы (xterm-256color+), минимум 80×24, Linux (основная), macOS (желательно)

**Scale & Complexity:**

- Primary domain: CLI/TUI приложение + batch-утилита
- Complexity level: средняя (для hobby-проекта)
- Estimated architectural components: 5–7 (TUI engine, CLI handler, UDPipe FFI, project/session state, panels, shortcuts, batch processor)

### Technical Constraints & Dependencies

- **UDPipe 2:** C++ библиотека, вызывается из Rust через FFI (`bindgen` → `libudpipe`). Готового крейта `udpipe-rs` не существует — потребуется собственная FFI-обёртка.
- **Ratatui:** зрелый TUI-фреймворк (v0.30.0), поддерживает multi-pane layout, кроссплатформенный.
- **clap:** v4.6.1, стандарт для CLI-парсинга в Rust.
- **Модели UDPipe:** внешние `.udpipe` файлы, не вкомпиливаются в бинарник. Нужна стратегия их размещения и загрузки.

### Cross-Cutting Concerns Identified

- **Кеширование UDPipe-результатов:** при навигации по тексту UDPipe-анализ не должен перезапускаться — результаты кешируются в памяти проекта
- **Dual-mode архитектура:** один бинарник, два code path (TUI + CLI), общий слой анализа
- **Управление состоянием сессии:** сохранение/восстановление позиции курсора, кеша разбора, пользовательских правок
- **Расширяемость панелей:** архитектура должна позволять добавлять новые панели (v2: связи, словари) без переписывания TUI-движка
- **FFI-безопасность:** вызов C++ кода из Rust через unsafe-блоки, управление памятью на границе языков

---

## Starter Template Evaluation

### Primary Technology Domain

CLI/TUI приложение на Rust — двойной режим (интерактивный TUI + batch CLI).

### Starter Options Considered

Для Rust-проектов стандартный стартер — `cargo init`. Аналогов create-react-app в экосистеме Rust нет. Проект инициализируется командой:

```bash
cargo init text-researcher --name text-researcher
```

### Selected Starter: cargo init + ручная конфигурация

**Rationale:** Rust-экосистема не имеет scaffolding-инструментов для TUI-приложений. `cargo init` создаёт минимальную структуру, после чего вручную добавляются зависимости и настраивается проект.

**Инициализация:**

```bash
cargo init text-researcher --name text-researcher
```

**Ключевые зависимости (Cargo.toml):**

```toml
[dependencies]
ratatui = "0.30"
clap = { version = "4.6", features = ["derive"] }
crossterm = "0.28"        # бэкенд терминала для ratatui
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"               # [ASSUMPTION: формат проекта будет TOML]

[build-dependencies]
bindgen = "0.70"           # генерация FFI-биндингов к libudpipe
```

**Архитектурные решения, заложенные стартером:**

- **Язык и рантайм:** Rust (stable), без async — проект не требует конкурентности на уровне рантайма. TUI однопоточный, batch — синхронный.
- **Билд-система:** Cargo + Makefile для Docker/CI.
- **Тестирование:** встроенный `cargo test`.
- **Структура проекта:** стандартная — `src/main.rs` как точка входа, далее разбиение на модули.
- **FFI:** `bindgen` в `build.rs` генерирует Rust-биндинги к `libudpipe` на этапе сборки.

**Замечание:** инициализация проекта через `cargo init` должна быть первым implementation story.

---

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- Workspace-структура: 3 крейта
- Панельная архитектура: trait `Panel`
- FFI-стратегия: отдельный крейт `udpipe-ffi`
- Хранение моделей: комбинированное (образ + загрузка)
- Формат проекта: TOML + JSON-совместимость
- State management: in-memory struct → `.trproj`

**Deferred to v2:**
- Dependency parsing panel integration
- Word dictionary / classifier dialogs
- Russian National Corpus integration

### 1. Workspace Structure

**Decision:** Три крейта в Cargo workspace.

```
text-researcher/
├── Cargo.toml          # workspace root
├── Makefile
├── Dockerfile
├── models/             # UDPipe-модели (.udpipe)
├── crates/
│   ├── udpipe-ffi/     # FFI-биндинги к libudpipe
│   │   ├── Cargo.toml
│   │   ├── build.rs    # bindgen
│   │   └── src/
│   │       ├── lib.rs
│   │       └── ...     # safe Rust-обёртки
│   ├── core/           # text-researcher-core
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── project.rs   # ProjectState, save/load
│   │       ├── analyzer.rs  # вызов udpipe-ffi, кеш
│   │       └── model.rs     # ModelManager
│   └── app/            # text-researcher (бинарник)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs      # точка входа, dispatch TUI/CLI
│           ├── tui/
│           │   ├── app.rs       # TUI-движок, event loop
│           │   ├── panels/
│           │   │   ├── mod.rs       # trait Panel
│           │   │   ├── text.rs      # TextPane
│           │   │   ├── props.rs     # PropsPane
│           │   │   ├── menu.rs      # MenuBar
│           │   │   └── status.rs    # StatusBar
│           │   └── shortcuts.rs
│           └── cli/
│               ├── batch.rs    # batch-режим
│               └── args.rs     # clap-аргументы
```

**Rationale:** Изоляция unsafe-кода (udpipe-ffi), переиспользование общей логики (core), чистая граница между TUI и CLI.

### 2. Panel Architecture

**Decision:** Trait `Panel` для всех панелей TUI.

```rust
// crates/app/src/tui/panels/mod.rs

pub enum Action {
    None,
    Quit,
    Save,
    OpenFile,
    SwitchLanguage(String),
    EditWord { word_index: usize, new_feats: Feats },
}

pub trait Panel {
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_input(&mut self, key: KeyEvent) -> Action;
    fn title(&self) -> &str;
    fn focusable(&self) -> bool { true }
}
```

**Registered panels (v1):**
- `TextPane` — основной текст, навигация, хайлайт текущего слова
- `PropsPane` — морфологические признаки
- `MenuBar` — File/Edit/View/Help
- `StatusBar` — имя файла, позиция, язык

**Rationale:** v2-панели (связи, словари) добавляются реализацией trait без изменения TUI-движка.

### 3. FFI Strategy

**Decision:** Отдельный крейт `udpipe-ffi` с `bindgen` в `build.rs`.

```toml
# crates/udpipe-ffi/Cargo.toml
[build-dependencies]
bindgen = "0.70"

# build.rs генерирует биндинги к libudpipe
```

Крейт предоставляет safe Rust-интерфейс:
- `Udpipeline::new(model_path) -> Result<Self>`
- `pipeline.tokenize(text) -> Vec<Sentence>`
- `pipeline.tag(sentence) -> Vec<Token>` (POS + морфология + лемма)

Все unsafe-блоки внутри крейта. `core` зависит от `udpipe-ffi` только через safe API.

**Rationale:** Единственная точка unsafe. Junior читает `core` и `app` без столкновения с FFI.

### 4. Model Storage Strategy

**Decision:** Комбинированный подход.

- **Docker:** `COPY models/ /app/models/` — модели в образе
- **CLI:** `text-researcher download-model <lang>` — качает модель при отсутствии
- **Env:** `UDPIPE_MODEL_DIR` переопределяет путь к моделям
- **Поиск:** программа проверяет (по порядку): `UDPIPE_MODEL_DIR` → `./models/` → предлагает скачать

**Rationale:** Быстрый старт в Docker, гибкость для bare-metal, отсутствие интернета — не блокер (если модели уже есть).

### 5. Project Format

**Decision:** TOML основной, JSON поддерживается автоопределением.

Расширение: `.trproj`.

```toml
# пример .trproj файла
[meta]
version = "0.1.0"
language = "ru"

[source]
file_path = "/home/user/text/chapter1.txt"

[cursor]
line = 42
word_index = 7

[edits]
# пользовательские правки: индекс_слова -> переопределённые признаки
"142" = { upostag = "VERB", feats = { Tense = "Past", Gender = "Masc" } }
```

При открытии: пробуем TOML, если ошибка — пробуем JSON. При сохранении — всегда TOML.

**Rationale:** TOML читаем в git diff, нативен для Rust. JSON — совместимость с внешними инструментами.

### 6. State Management

**Decision:** In-memory `ProjectState` → сериализация в `.trproj`.

```rust
// crates/core/src/project.rs

pub struct ProjectState {
    pub meta: ProjectMeta,
    pub source: SourceRef,          // путь к тексту
    pub text: String,               // загруженный текст
    pub tokens: Vec<Vec<Token>>,    // кеш UDPipe-разбора (предложение → слова)
    pub cursor: CursorPos,
    pub edits: HashMap<usize, TokenOverride>,  // пользовательские правки
    pub language: String,
}
```

- `Ctrl+S`: сериализация `ProjectState` в TOML → запись в `.trproj`
- Запуск: десериализация `.trproj` → загрузка текста из `source.file_path` → восстановление кеша и правок
- Кеш UDPipe не пересчитывается при навигации — только чтение из `tokens`

**Rationale:** Без внешней БД. Один файл = один проект. Синхронно — async не нужен.

### 7. Navigation & Input

**Decision:** Полный vim-набор + мышь.

**Клавиши:**
- Стрелки + `h/j/k/l` — перемещение по словам/строкам
- `w/b` — слово вперёд/назад
- `e` — конец слова
- `0/$` — начало/конец строки
- `gg/G` — начало/конец текста
- `Ctrl+←/Ctrl+→` — предыдущее/следующее предложение

**Мышь:** Ratatui + crossterm — клик по слову перемещает курсор.

**Rationale:** vim-пользователи оценят полный набор. Мышь снижает порог входа для новичков.

### Decision Impact Analysis

**Implementation Sequence:**
1. `cargo init` workspace + крейты → skeleton
2. `udpipe-ffi` — FFI-биндинги
3. `core` — ProjectState, analyzer, model manager
4. `app/tui/panels` — trait Panel + реализации
5. `app/tui/app.rs` — TUI event loop
6. `app/cli` — batch-режим
7. `app/main.rs` — dispatch + clap
8. `Makefile` + `Dockerfile` + CI/CD

**Cross-Component Dependencies:**
- `udpipe-ffi` ← (не зависит от других крейтов)
- `core` ← `udpipe-ffi`
- `app` ← `core`, `udpipe-ffi`

---

## Implementation Patterns & Consistency Rules

### Critical Conflict Points

6 областей, где разные AI-агенты могут принять разные решения.

### Naming Patterns

| Категория | Правило | Пример |
|---|---|---|
| Файлы/модули | `snake_case` | `project_state.rs`, `text_pane.rs` |
| Структуры/перечисления | `CamelCase` | `ProjectState`, `TextPane`, `Action` |
| Функции/методы | `snake_case` | `fn save_project()`, `fn handle_input()` |
| Константы | `SCREAMING_SNAKE_CASE` | `MAX_TEXT_SIZE`, `DEFAULT_LANG` |
| Тестовые функции | `test_<что>_<ожидание>` | `test_save_project_writes_toml` |

### Code Organization

- `#[cfg(test)] mod tests { ... }` — юнит-тесты внутри файла рядом с кодом
- `tests/` на уровне крейта — интеграционные тесты
- `use`-порядок: `std` → external crates → `crate::` → `super::`

### Error Handling

- **`app` (бинарник):** `anyhow::Result<T>` + `.context("...")` для человеческих сообщений
- **`core` (библиотека):** `thiserror` — типизированные enum-ошибки
- **`udpipe-ffi` (библиотека):** `thiserror` — ошибки FFI + загрузки моделей
- `unwrap()` запрещён в production-коде; только `?` и `.context()`

### Logging

- Крейт: `log` + `env_logger` (без async)
- Уровни: `error!` / `warn!` / `info!` / `debug!` / `trace!`
- По умолчанию: `RUST_LOG=info`
- В TUI: `error!`/`warn!` пишутся в status bar, остальные — в stderr

### State Update Patterns

- `ProjectState` — неизменяемый в публичном API; изменения через методы `&mut self`
- Кеш UDPipe — append-only: новые токены добавляются, старые не переписываются
- Пользовательские правки — `HashMap<usize, TokenOverride>`, слияние при сохранении

### Enforcement

**Все агенты должны:**
- Следовать `rustfmt` (стандартные настройки)
- Проходить `cargo clippy` без warnings
- Проходить `cargo test` без failures
- Именовать тесты по шаблону `test_<что>_<ожидание>`
- Не использовать `unwrap()` / `expect()` вне тестов

---

## Project Structure & Boundaries

### Complete Project Directory Structure

```
text-researcher/
├── Cargo.toml                  # workspace root
├── Cargo.lock
├── Makefile
├── Dockerfile
├── .dockerignore
├── .gitignore
├── README.md
├── CHANGELOG.md
├── .github/
│   └── workflows/
│       └── ci.yml              # CI/CD: build, test, clippy, docker
├── models/                     # UDPipe-модели (.udpipe)
│   ├── .gitkeep
│   └── README.md               # инструкция по загрузке моделей
├── crates/
│   ├── udpipe-ffi/
│   │   ├── Cargo.toml
│   │   ├── build.rs            # bindgen → libudpipe
│   │   └── src/
│   │       ├── lib.rs          # pub API: Udpipeline, Token, Sentence
│   │       ├── ffi.rs          # unsafe FFI-обёртки
│   │       ├── error.rs        # UdpipelineError (thiserror)
│   │       └── token.rs        # Token, Sentence structs
│   ├── core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          # pub API крейта
│   │       ├── project.rs      # ProjectState, save/load .trproj
│   │       ├── analyzer.rs     # вызов udpipe-ffi, кеширование
│   │       ├── model.rs        # ModelManager: загрузка, скачивание
│   │       ├── language.rs     # Language enum, коды языков
│   │       └── error.rs        # CoreError (thiserror)
│   └── app/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs         # точка входа: clap dispatch (tui | batch | download-model)
│           ├── tui/
│           │   ├── mod.rs
│           │   ├── app.rs      # TUI-движок: event loop, layout, dispatch
│           │   ├── panels/
│           │   │   ├── mod.rs  # trait Panel, enum Action
│           │   │   ├── text.rs     # TextPane
│           │   │   ├── props.rs    # PropsPane
│           │   │   ├── menu.rs     # MenuBar
│           │   │   └── status.rs   # StatusBar
│           │   └── shortcuts.rs# обработка клавиатурных шорткатов
│           └── cli/
│               ├── mod.rs
│               ├── args.rs     # clap derive: Cli { Batch {..}, DownloadModel {..} }
│               └── batch.rs    # batch-режим: загрузка → анализ → JSON-вывод
└── tests/
    ├── integration/
    │   ├── tui_flow.rs         # сценарные тесты TUI (ratatui backend mock)
    │   └── batch_output.rs     # проверка JSON-вывода batch
    └── fixtures/
        ├── sample_ru.txt       # тестовый русский текст
        └── sample_en.txt       # тестовый английский текст
```

### Architectural Boundaries

```
┌──────────────────────────────────────────────┐
│                  app (binary)                 │
│  ┌──────────────┐  ┌──────────────────────┐  │
│  │   TUI engine │  │   CLI (batch)        │  │
│  │  ratatui+cros │  │   clap + serde_json  │  │
│  └──────┬───────┘  └──────────┬───────────┘  │
│         │                     │               │
│         └──────────┬──────────┘               │
│                    │                          │
│              ┌─────▼─────┐                    │
│              │   core    │                    │
│              │ project,  │                    │
│              │ analyzer, │                    │
│              │ model mgr │                    │
│              └─────┬─────┘                    │
│                    │                          │
│              ┌─────▼──────┐                   │
│              │ udpipe-ffi │                   │
│              │ unsafe FFI │                   │
│              └─────┬──────┘                   │
└────────────────────┼──────────────────────────┘
                     │ C ABI
              ┌──────▼──────┐
              │  libudpipe  │  (C++ shared lib)
              └─────────────┘
```

- **app → core:** только через pub API `core::`. Не лезет в FFI напрямую.
- **core → udpipe-ffi:** только через safe API (Udpipeline, Token, Sentence).
- **udpipe-ffi → libudpipe:** unsafe FFI, изолирован в `ffi.rs`. Safe-обёртки в `lib.rs`.
- **app → udpipe-ffi:** запрещено. Все вызовы через `core`.

### Requirements to Structure Mapping

| FR / Feature | Файлы |
|---|---|
| FR-1: запуск/восстановление | `main.rs`, `project.rs` |
| FR-2: отображение текста | `tui/panels/text.rs` |
| FR-3: навигация | `tui/panels/text.rs`, `shortcuts.rs` |
| FR-4: свойства слова | `tui/panels/props.rs`, `analyzer.rs` |
| FR-5: редактирование свойств | `tui/panels/props.rs`, `project.rs` |
| FR-6: сохранение/загрузка | `project.rs` |
| FR-7: меню/статус | `tui/panels/menu.rs`, `tui/panels/status.rs` |
| FR-8: шорткаты | `shortcuts.rs` |
| FR-9: batch-запуск | `cli/batch.rs`, `cli/args.rs` |
| FR-10: JSON-вывод | `cli/batch.rs` |
| FR-11: загрузка моделей | `model.rs`, `cli/args.rs` (download-model) |
| NFR-5 (clap) | `cli/args.rs` |
| CI/CD | `.github/workflows/ci.yml`, `Makefile` |

### Development Workflow

- **Сборка:** `make build` → `cargo build --release`
- **Тесты:** `make test` → `cargo test --workspace`
- **Линтинг:** `make lint` → `cargo clippy -- -D warnings && cargo fmt --check`
- **Docker:** `make docker` → `docker build -t text-researcher .`
- **CI:** push → GitHub Actions: lint → test → build → docker

---

## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:** Все технологические решения совместимы. Rust stable + ratatui v0.30 + crossterm v0.28 + clap v4.6 работают без конфликтов. Workspace из трёх крейтов с чистыми границами (app → core → udpipe-ffi) предотвращает циклические зависимости.

**Pattern Consistency:** Именование (snake_case/CamelCase), обработка ошибок (anywhere/thiserror), логирование (log/env_logger) — всё по стандартным Rust-конвенциям. Конфликтов нет.

**Structure Alignment:** Структура проекта соответствует workspace-решению. FR/NFR привязаны к конкретным файлам.

### Requirements Coverage Validation ✅

**Functional Requirements (11/11 покрыто):**
- FR-1–FR-8 (TUI): `tui/panels/*.rs`, `shortcuts.rs`
- FR-9–FR-10 (batch): `cli/batch.rs`, `cli/args.rs`
- FR-11 (модели): `model.rs`, `cli/args.rs`

**Non-Functional Requirements (11/11 покрыто):**
- NFR-1–NFR-3 (производительность): UDPipe-кеш в `analyzer.rs`, in-memory state
- NFR-4 (читаемый код): workspace + trait Panel — понятная структура
- NFR-5 (clap): `cli/args.rs`
- NFR-6–NFR-8 (Make/Docker/CI): `Makefile`, `Dockerfile`, `.github/workflows/ci.yml`
- NFR-9–NFR-11 (совместимость): ratatui + crossterm кроссплатформенны

### Implementation Readiness Validation ✅

**Decision Completeness:** 7 критических решений задокументированы с версиями и обоснованием.

**Structure Completeness:** Полное дерево файлов (42+ файла), границы крейтов, карта FR→файлы.

**Pattern Completeness:** 6 категорий паттернов (naming, error handling, logging, state, tests, enforcement).

### Gap Analysis

**Мелкие пробелы (не блокируют имплементацию):**
- Dockerfile — будет создан на этапе имплементации (шаблон стандартный: multi-stage Rust build)
- CI/CD — предполагается GitHub Actions (бесплатно для публичных репозиториев)
- Установка `libudpipe` — необходимо документировать в README (`apt install libudpipe-dev` или сборка из исходников)
- Shell completions — clap генерирует автоматически, не требует архитектурного решения

### Architecture Completeness Checklist

**Requirements Analysis:**
- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped

**Architectural Decisions:**
- [x] Critical decisions documented with versions
- [x] Technology stack fully specified
- [x] Integration patterns defined
- [x] Performance considerations addressed

**Implementation Patterns:**
- [x] Naming conventions established
- [x] Structure patterns defined
- [x] Communication patterns specified
- [x] Process patterns documented

**Project Structure:**
- [x] Complete directory structure defined
- [x] Component boundaries established
- [x] Integration points mapped
- [x] Requirements to structure mapping complete

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION

**Confidence Level:** High

**Key Strengths:**
- Чистое разделение на крейты с изоляцией unsafe
- Trait-based панели для расширяемости в v2
- Полное покрытие FR/NFR архитектурными решениями
- Rust-идиоматичные паттерны без переусложнения
- Двойной режим (TUI + CLI) без дублирования логики

**Areas for Future Enhancement:**
- Добавление панели связей (v2) — trait Panel уже готов
- Интеграция НКРЯ (v2) — отдельный крейт `nkrja-ffi` по аналогии с `udpipe-ffi`
- Поддержка дополнительных языков — Language enum расширяем

### Implementation Handoff

**AI Agent Guidelines:**
- Следовать архитектурным решениям как документировано
- Использовать implementation patterns единообразно
- Соблюдать границы крейтов: app → core → udpipe-ffi
- Не использовать unwrap() в production-коде
- Обращаться к этому документу при архитектурных вопросах

**First Implementation Priority:**
```bash
cargo init text-researcher --name text-researcher
```
Далее: workspace setup → `udpipe-ffi` → `core` → `app` → Makefile/Docker/CI

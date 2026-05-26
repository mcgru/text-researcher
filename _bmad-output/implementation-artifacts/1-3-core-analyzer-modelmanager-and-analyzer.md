# Story 1.3: Core analyzer — ModelManager и Analyzer

Status: ready-for-dev

## Story

As a разработчик,
I want загружать UDPipe-модели по языку и анализировать текст через удобный API с кешированием,
so that CLI и TUI используют общий слой без дублирования логики.

## Acceptance Criteria

1. `ModelManager::load("ru")` загружает модель из `./models/` или `UDPIPE_MODEL_DIR`
2. `ModelManager::available_languages()` возвращает список установленных моделей
3. `Analyzer::analyze(text)` возвращает `Vec<Sentence>` с полным разбором
4. Повторный вызов `analyze()` для того же текста читает из кеша
5. `Analyzer::analyze_at(text, word_index)` возвращает разбор конкретного слова
6. Ошибки — `CoreError` через thiserror
7. Модуль `config.rs`: GlobalConfig, SessionConfig, приоритет: session → global → defaults

## Tasks / Subtasks

- [ ] Task 1: Config module
  - [ ] `GlobalConfig` из `~/.config/text-researcher/config.json`
  - [ ] `SessionConfig` из `.trconf` (JSON/YAML)
  - [ ] Приоритет: session → global → defaults
- [ ] Task 2: ModelManager
  - [ ] Скан `./models/` и `UDPIPE_MODEL_DIR` на `.udpipe` файлы
  - [ ] `load(language)` — возвращает Udpipeline
  - [ ] `available_languages()` — Vec<String>
- [ ] Task 3: Analyzer
  - [ ] `analyze(text)` — вызывает Udpipeline::tokenize, кеширует результат
  - [ ] `analyze_at(text, word_index)` — возвращает конкретный токен
  - [ ] Кеш: `HashMap<String, Vec<Sentence>>` (текст → разбор)
- [ ] Task 4: CoreError
  - [ ] Enum с вариантами: ModelNotFound, AnalysisFailed, ConfigError
  - [ ] From impl для UdpipelineError
- [ ] Task 5: Tests
  - [ ] Config tests (defaults, override priority)
  - [ ] ModelManager tests (scan test fixtures dir)
  - [ ] Analyzer cache tests (повторный вызов без subprocess)

## Dev Notes

### Architecture Compliance

- Все unsafe изолированы в udpipe-client — core полностью safe
- Config priority: session (.trconf) → global (config.json) → defaults
- Кеш in-memory, не персистентный (персистентность в project.rs, Story 1.4)

### NFR-2

- Текст 1MB анализируется ≤ 2 сек (без учёта загрузки модели)
- Кеш исключает повторные вызовы subprocess для одного текста

## Dev Agent Record

_To be filled by dev agent_

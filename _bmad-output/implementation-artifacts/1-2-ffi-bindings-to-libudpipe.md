# Story 1.2: UDPipe client — обёртка над UDPipe 2 (Python subprocess)

Status: ready-for-dev

## Story

As a разработчик,
I want safe Rust-обёртку над UDPipe 2 (вызов Python через subprocess, stdin/stdout в CoNLL-U),
so that остальной код работает с UDPipe через идиоматичный Rust-интерфейс.

## Acceptance Criteria

1. `cargo build -p udpipe-client` компилируется
2. Запуск UDPipe 2 через subprocess: `python3 -m udpipe2 --model <path>`
3. Публичный API: `Udpipeline::new(model_path) -> Result<Self>`, `.tokenize(text) -> Result<Vec<Sentence>>`
4. `Token` содержит: `word`, `lemma`, `upostag`, `xpostag`, `feats` (распарсено из CoNLL-U)
5. `UdpipelineError` через thiserror
6. `make deps` устанавливает Python-зависимости (tensorflow, numpy, ufal.chu-liu-edmonds)

## Tasks / Subtasks

- [ ] Task 1: Переименовать крейт udpipe-ffi → udpipe-client
  - [ ] Обновить Cargo.toml, удалить build.rs и bindgen
  - [ ] Обновить зависимости в core/Cargo.toml (path)
- [ ] Task 2: Subprocess manager
  - [ ] Запуск `python3 -m udpipe2 --model <path>` как дочерний процесс
  - [ ] Передача текста через stdin, чтение CoNLL-U из stdout
  - [ ] Кеширование процесса (один запуск на модель)
- [ ] Task 3: CoNLL-U парсер
  - [ ] Парсинг строк CoNLL-U: ID, FORM, LEMMA, UPOS, XPOS, FEATS
  - [ ] Сборка в Vec<Sentence>, Vec<Token>
- [ ] Task 4: Safe Rust API (lib.rs)
  - [ ] `Udpipeline` struct с `new(model_path)` и `tokenize(text)`
  - [ ] `Sentence` и `Token` structs (Serialize, Deserialize, Debug)
- [ ] Task 5: Error handling
  - [ ] `UdpipelineError` enum via thiserror
  - [ ] Ошибки: ModelNotFound, SubprocessFailed, ParseError, Timeout
- [ ] Task 6: `make deps`
  - [ ] Makefile цель: установка Python-зависимостей через pip
- [ ] Task 7: Tests
  - [ ] Юнит-тесты парсера CoNLL-U (статическая строка)
  - [ ] Интеграционный тест (если модель доступна, иначе #[ignore])

## Dev Notes

### Architecture Change (2026-05-26)

UDPipe 2 — Python-библиотека (TensorFlow), нет C shared library.
Подход: subprocess вместо FFI. Крейт `udpipe-client` вместо `udpipe-ffi`.
Без unsafe, без bindgen. Коммуникация через stdin/stdout в формате CoNLL-U.

### Dependencies

- Python 3.8+, tensorflow, numpy, ufal.chu-liu-edmonds
- UDPipe 2 скрипты: `pip install udpipe2` или локальный `udpipe2.py`
- `serde` для Token/Sentence сериализации

### CoNLL-U Format (reference)

```
# sent_id = 1
# text = Это пример.
1	Это	это	DET	DT	Animacy=Inan|Case=Nom
2	пример	пример	NOUN	NN	Animacy=Inan|Case=Nom|Gender=Masc|Number=Sing
```

### References

- Architecture: `_bmad-output/planning-artifacts/architecture.md`
- Epics: `_bmad-output/planning-artifacts/epics.md` — Story 1.2 (пересмотр)

## Dev Agent Record

### Agent Model Used

_To be filled by dev agent_

### Debug Log References

_To be filled by dev agent_

### Completion Notes List

_To be filled by dev agent_

### File List

_To be filled by dev agent_

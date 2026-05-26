# Story 1.4: Batch CLI — парсинг аргументов, конфигурация и JSON-вывод

Status: ready-for-dev

## Story

As a исследователь,
I want запустить `text-researcher input.txt -l ru -o result.json` и получить JSON с полным морфологическим разбором,
so that я могу использовать text-researcher в своих скриптах и пайплайнах.

## Acceptance Criteria

1. `text-researcher sample.txt -l ru -o out.json` — batch-режим (по умолчанию), JSON на выходе
2. `-i` запускает TUI (пока заглушка)
3. `-l` по умолчанию из конфига (default: ru)
4. `--help` показывает авто-справку clap
5. Без `-o` — вывод в stdout
6. При отсутствии файла — exit code ≠ 0 + stderr
7. При отсутствии модели — exit code ≠ 0 + stderr
8. JSON: `[{text, tokens: [{word, lemma, upostag, xpostag, feats}]}]`

## Tasks / Subtasks

- [ ] Task 1: clap CLI definition (args.rs)
- [ ] Task 2: batch mode handler (batch.rs)
- [ ] Task 3: main.rs dispatch (batch | tui-stub | download-model)
- [ ] Task 4: Integration test (fixture text file → JSON output)

## Dev Notes

### CLI Design

```
text-researcher [INPUT] [-l LANG] [-o OUTPUT]   # batch (default)
text-researcher -i [FILE]                         # interactive TUI
text-researcher download-model LANG               # download model
```

### NFR-5

clap derive, --help auto-generated, shell completions.

## Dev Agent Record

_To be filled by dev agent_

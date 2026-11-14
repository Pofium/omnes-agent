# kag_sync.md — регламент синхронизации `omnesagent-kag` с ob2h

> **Статус:** действующий регламент (план `PLAN_OB2H_INTEGRATION_2026-11-06.md`, фаза Ф1).
> **Владелец-гейт:** изменения AST-движка `omnesagent-kag` принимаются только при
> сохранении паритета по golden-набору (см. §3) и зелёном bench-гейте (§5).
> **Прецедент:** `2ea87333a` (`feat(kag): синхронизация AST-сканера с ob2h — 10 новых языков`,
> зеркальный порт из `C:\Projects\ob2h` ревизии `93c3f64`).

## 1. Назначение и границы

`omnesagent-kag` — встроенный (embedded) AST/KAG-движок OmnesAgent. ob2h —
внешний MCP-сервер памяти «семейства Omnes» (`C:\Projects\ob2h`). Движки — близнецы:
один и тот же алгоритм извлечения графа поддерживается в двух местах.

**ADR-O2 — алгоритмический, а не дата-паритет.** Переносятся только *алгоритмы и
семантика* (правила извлечения узлов/рёбер, резолва, нормализации). Записи памяти и
узлы графа **не** синхронизируются: у каждого движка своя БД и свои данные.

**Направление синка:** ob2h → `omnesagent-kag` (реверс не выполняется). Изменения
переносятся из ревизии ob2h в коммит форка, а не наоборот.

## 2. Чеклист переноса алгоритма

Для каждой переносимой фичи ob2h:

1. **Найти upstream-ревизию.** `git -C C:\Projects\ob2h log --oneline` → hash фичи
   (например `2ad9027`, `678afe0`, `a646d38`). Зафиксировать hash в описании задачи
   плана и в теле коммита.
2. **Сверить исходники построчно.** ob2h: `c:\Projects\ob2h\src\project\ast.rs`,
   `src\project\mod.rs`. Форк: `backend/crates/omnesagent-kag/src/ast/mod.rs`,
   `src/project/mod.rs`. Порт — «дословный» по алгоритму; расхождения допускаются
   только осознанные (см. §4) и должны быть закомментированы в коде.
3. **Портировать в правильный слой.** Парсеры/резолверы — в `src/ast/`; шаги
   `scan_project` (type-pass, авто-создание узлов, pkg-слияние) — в `src/project/mod.rs`.
   **Порядок шагов в `scan_project` обязан совпадать с ob2h:** `resolve_imports` →
   `resolve_calls` → `consolidate_external_imports` → вставка узлов/рёбер.
4. **Добавить/обновить тесты.** Минимум — юнит-тест в `tests/test_ast_extractor.rs` или
   `tests/test_phase6_parsers.rs`; для поведения БД — интеграционный тест в
   `tests/` по образцу `test_scan_full.rs` (реальный `migrate` + `ProjectService`).
5. **Прогнать golden dual-scan (§3).** Если перенос меняет извлечённые символы/рёбра,
   перегенерировать baseline **нельзя без причины** — сначала объяснить расхождение.
   Golden меняется только при осознанном изменении эталона ob2h (новая версия движка).
6. **Один пункт плана = один коммит.** Conventional commit, без AI-атрибуции. В теле —
   ссылка на upstream-hash и результат метрик.
7. **Проверки перед коммитом:**
   ```bash
   cd backend
   cargo test -p omnesagent-kag
   cargo clippy -p omnesagent-kag --all-targets
   ```

## 3. Golden dual-scan и перегонка baseline

Паритет проверяется тестом `backend/crates/omnesagent-kag/tests/test_golden_dual_scan.rs`.

- **Корпус:** `tests/golden/corpus/` — маленький неизменяемый набор (Rust/Python/TS/Go/SQL
  + skip-проба `*_pb2.py`). Baseline фиксируется на корпусе, **а не на всём репозитории**:
  только так F1 детерминирован (полный репо-скан шумит от меняющихся исходников).
- **Эталон:** `tests/golden/ob2h-<version>.expected.json` — вывод ob2h-сканера по корпусу.
- **Метрики:** symbol F1 (узлы с `provenance='ast'`) и call F1 (рёбра `CALLS`).
- **Пороги DoD Ф1:** `symbol F1 ≥ 0.95`, `call F1 ≥ 0.90`. Текущее состояние: `1.000 / 1.000`.

**Перегонка эталона** (только при смене версии ob2h или осознанном изменении корпуса):

```powershell
cd backend/crates/omnesagent-kag/tests/golden
.\regenerate.ps1 -Ob2h C:\Projects\ob2h\target\release\ob2h.exe `
                 -Python "C:\Program Files\Python313\python.exe" `
                 -Ob2hVersion 1.9.2
```

`regenerate.ps1` сканирует корпус в **изолированном** `OB2H_DATA_DIR` (временный каталог),
поэтому реальные данные ob2h пользователя не затрагиваются. `export_ob2h.py` выгружает
`graph_nodes`/`graph_edges` эталона в нормализованный JSON (символы — узлы
`provenance='ast'`; рёбра — всё, кроме `provenance='codegraph'`; формат ребра
`src\tLABEL\tdst`).

Ожидается ровно один файл `ob2h-*.expected.json` — тест падает, если файлов ноль или
больше одного.

## 4. Осознанные расхождения форка с ob2h

Документируются и сохраняются намеренно (не «баги паритета»):

1. **Scoped `node_id`.** ob2h хранит глобально-уникальный `node_id`; форк неймспейсит
   его префиксом `<project_id>:` (`golden:fn:…`, `proj:cg:…`). Мультипроектность.
   Нормализация в golden-тесте снимает префикс перед сравнением.
2. **Авто-синк CodeGraph.** ob2h импортирует граф CodeGraph вручную CLI-командой
   `import-codegraph <path>`; форк вызывает `codegraph_bridge::sync_codegraph_db`
   автоматически из `scan_project`. **Направление совпадает** (см. §6).
3. **Прочее по мере появления** — фиксировать здесь при каждом новом отклонении.

## 5. Bench-гейт (не деградировал)

`omnesagent-kag/src/bench.rs`: `GateVerdict` сверяет recall/MRR golden-case'ов
относительно baseline (`RECALL_DROP_LIMIT = 0.10`, `MRR_DROP_LIMIT = 0.15`).
Изменения AST-движка не должны ронять метрики за эти лимиты. Владелец-гейт проверяет
`GateVerdict` перед приёмкой фазы.

## 6. Ревизия направления CodeGraph↔граф (Ф1.8)

Сверка `omnesagent-kag/src/project/codegraph_bridge.rs` с ob2h `678afe0`
(`feat(project): добавить команду import-codegraph`):

- **Направление совпадает:** внешняя БД `.codegraph/codegraph.db` → встроенный граф
  (`graph_nodes`/`graph_edges`, `provenance='codegraph'`). Форк не «экспортирует» KAG
  наружу — реверс отсутствует в обоих движках.
- **Отличия (не нарушают паритет алгоритма):**
  - источник `node_id`: ob2h `cg:{id}` vs форк `{project_id}:cg:{id}` (см. §4.1);
  - `description`: ob2h `signature.or(docstring)` vs форк слияние `signature\ndocstring`;
  - контекст ребра: ob2h `''` vs форк JSON `metadata` или `{"line":N}` (богаче);
  - форк открывает БД `READ_ONLY` и fail-open (отсутствие/блокировка БД не роняет скан);
    ob2h-команда возвращает ошибку.
- **Вывод:** направление подтверждено, реализация форка — расширенная (auto + fail-open).
  Отдельных доработок Ф1.8 не требует.

## 7. Паритет-матрица v1.9.9 — Memory Lifecycle Pack (Приложение C)

Дополнение к Приложению B базовой спеки (`PLAN_OB2H_INTEGRATION_2026-11-06.md`).
План-источник: `PLAN_OB2H_INTEGRATION_V1.9.9_2026-11-14.md`.

| Возможность ob2h | Версия | omnesagent | Статус / фаза |
|---|---|---|---|
| Lessons (confidence/reinforce/decay) | 1.9.9 | потребление через MCP (ADR-O7); близнец не дублирует | Ф8 |
| Slots (budget/pinned/scope, `<agent_slots>`) | 1.9.9 | потребление через MCP; инъекция в `memory_context` | Ф9 |
| Timeline / session_recap (инкрементальный курсор) | 1.9.9 | recap-путь моста при архивации run | Ф10.1 |
| Провенанс `memory_verify` | 1.9.9 | правило verify-before-use; recap-путь фактов | Ф10.2 |
| Аудит-леджер + `reason` у forget | 1.9.9 | правило reason; `memory_audit` отчёты | Ф10.3 |
| Скрабинг паттернов при захвате (Ф69.1) | 1.9.9 | `omnesagent-memory::redact` — паритет паттернов | Ф11.1 |
| Bench recall-all@k / any-hit@k (Ф69.2) | 1.9.9 | `omnesagent-kag::bench` — порт метрик (наблюдение) | Ф11.3 |
| Viewer egui + `viewer_open` | 1.9.9 | privileged-классификация + правила скилла | Ф12 |
| Dream slot-рефлекс (`OB2H_DREAM_SLOTS`) | 1.9.9 | n/a — сторона ob2h, default off | не-цель |
| Синк lessons/slots в дельта-бандлах | 1.9.9 | n/a — ADR-K6, форк не участвует | не-цель |

**Метрики близнеца (Ф11.3):** `recall_all_at_k`/`any_hit_at_k` добавлены в
`omnesagent-kag::bench` и выводятся в `BenchSummary` (`recall_all5`/`any_hit5`,
LongMemEval strict = все ключи в top-k). Пороги `GateVerdict`
(`RECALL_DROP_LIMIT`/`MRR_DROP_LIMIT`) не меняются — метрики наблюдательные (ADR-O11).

## 9. Паритет с ob2h v2.0 (Polyglot AST & Vector Pipeline)

План-источник: `PLAN_OB2H_V2_AND_REBRAND_2026-11-14.md`.

| Возможность ob2h v2.0 | omnesagent | Статус реализации |
|---|---|---|
| Схема БД v10 (`file_blobs`, `valid_until`, `bq_mask`, `subtokens`) | `omnesagent-kag::schema`, `omnesagent-memory::sqlite` | Реализовано (версия схемы 10, PRAGMA busy_timeout=5000, BEGIN IMMEDIATE) |
| TreeSitter LRU-64 + Chained calls & Macro token_tree | `omnesagent-kag::ast::treesitter` | Реализовано (LRU-64, цепочки вызовов, macro heuristics) |
| Generated File Stubbing | `omnesagent-kag::ast` | Реализовано (FileStub для *.g.dart, *.freezed.dart, *.pb.rs) |
| AST-aware Chunker | `omnesagent-kag::embedding::chunker` | Реализовано (`// File: ... \| Scope: ...`, blake3) |
| Mean-Centered Binary Quantization (BQ) & Two-stage search | `omnesagent-kag::vector::bq` | Реализовано (1-бит центрированное квантование, 48 байт/384 dim, popcnt Hamming) |
| Multilingual E5 Small & Асимметричные префиксы | `omnesagent-kag::embedding` | Реализовано (`query: ` / `passage: `, micro-batch 32) |
| EmbeddingFingerprint (5 полей) | `omnesagent-kag::embedding::fingerprint` | Реализовано (`model_id`, `backend`, `quant`, `dim`, `prefix_version`) |
| Git Co-Change / Temporal Coupling | `omnesagent-kag::project::cochange` | Реализовано (`CO_CHANGES_WITH`, `provenance='GIT_HISTORY'`, min 3 коммита) |
| AffectedTests & Blast Radius | `omnesagent-kag::graph::analytics` | Реализовано (decay $0.85^{\text{depth}}$, `affected_tests`) |
| U-PPR v2.0 & In-degree penalty | `omnesagent-kag::graph::pagerank` | Реализовано (In-degree penalty, $\lambda_{\text{code}}=0$, `mute_node`) |
| Elbow Cutoff для Repo-map | `omnesagent-kag::graph::repomap` | Реализовано ($S_i < 0.15 S_{\text{max}}$ или спад $> 50\%$) |
| Dream 2.0 Conflict Taxonomy | `omnesagent-memory::conflict` | Реализовано (Temporal, Correction, Coexistence, cycle prevention) |
| Ralph Git-HEAD freshness | `omnesagent-ralph::journal` | Реализовано (`git_head_sha`, `is_fresh`) |
| Tool Intent Grouping ob2h v2 | `omnesagent-runtime::agent::orchestrator::tool_group` | Реализовано (`knowledge_reconcile`, `memory_feedback` как mutating) |


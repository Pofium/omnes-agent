# Спецификация: доработки ob2h в OmnesAgent (Ф0–Ф6)

> **Версия:** 1.0.0 (черновик на утверждение владельцем)
> **Дата:** 2026-11-06
> **Базовое состояние ob2h:** v1.9.2 (`C:\Projects\ob2h`, main @ `2728a1c`, контракт 41 MCP-инструмент)
> **Базовое состояние форка:** ветка `ob2h_updates` @ `bc4352d27` + незакоммиченный 30-й срез декомпозиции `schema.rs` (`schema/multimodal.rs`, `schema/pacing.rs`, `schema/pipeline.rs`)
> **Объём:** интеграция ob2h как внешнего слоя знаний семейства Omnes в харнес OmnesAgent и паритет встроенной памяти с алгоритмами ob2h
> **Связанные документы:** `PLAN.md`, `PLAN_AFTER_AUDIT.md`, `PLAN_FORK_SYNC_2026-10-24.md`, `C:\Projects\ob2h\PLAN_v2.0.md`

---

## 1. Контекст и модель семейства Omnes

**Omnes — семейство продуктов владельца.** ob2h — общий сервер долговременной памяти и знаний,
подключаемый через MCP ко всем харнес-системам семейства. OmnesAgent — самостоятельная
харнес-система (форк ZeroClaw) со **собственной встроенной памятью**, построенной по архитектуре
ob2h (`omnesagent-memory` + `omnesagent-kag`). Встроенная память — независимый близнец ob2h,
развиваемый в форке; она не является легаси и не подлежит замене на ob2h.

```
                    ┌──────────────────────────────────┐
                    │  ob2h — сервер памяти/знаний     │  общая для семейства:
                    │  41 MCP-инструмент, дриминг,     │  пользователь, проекты,
                    │  E2EE-синк PC<->VPS, Secrets     │  архитектурные решения
                    └───────┬────────┬─────────┬───────┘
                            │ MCP    │ MCP     │ MCP (stdio/http/sse)
                 ┌──────────┘        │         └───────────┐
           ┌─────┴─────┐      ┌──────┴─────┐      ┌────────┴─────────┐
           │  Hermes   │      │   ZCode    │      │   OmnesAgent     │
           └───────────┘      └────────────┘      │  (форк ZeroClaw) │
                                                  └──────────────────┘
                                                    omnesagent-memory  = операционная память
                                                    omnesagent-kag     = встроенный KAG/AST-движок
                                                    omnesagent-ralph   = автономный цикл разработки
                                                    omnesagent-s1      = System One (Laya)
```

**Разделение ролей памяти (основание всех фаз):**

| Слой | Носитель | Что хранит | Жизненный цикл |
|---|---|---|---|
| Операционная / агентская память | `omnesagent-memory` внутри форка | сессии, agent-scoped записи, prefetch-блоки turn-цикла, аудио-аудит | управляется харнесом (retention, budget, agent_scoped) |
| Кросс-продуктовая долговременная память | ob2h | пользователь (USER/SOUL), проекты, архитектурные решения, граф знаний, дриминг | консолидация дримом, синк PC<->VPS |
| Кодовый граф | `omnesagent-kag` (встроенный) и ob2h (внешний, проект `[omnes-agent]`) | AST-узлы, рёбра, метрики, PPR | инкрементальный скан по SHA256 |

Стороны уже симметрично «видят» друг друга:
- ob2h v1.9.2 harness-детект (`src/llm/harness.rs`) по `clientInfo.name` распознаёт
  `zcode/omnesagent` и читает `~/.omnesagent/config.toml` (deepseek/openrouter; anthropic
  пропускается — другой протокол) для снабжения дриминга LLM (эшелон 2 из трёх).
- Обратного направления (форк -> ob2h) как MCP-сервера нет: ob2h не прописан в
  `~/.omnesagent/config.toml` секции `[mcp]`; доступ форка к ob2h ограничен конфигом
  `[ob2h_bridge]`, чей потребитель ещё не реализован (см. §4.3).

## 2. Термины

| Термин | Значение |
|---|---|
| **ob2h** | `C:\Projects\ob2h` — Rust MCP-сервер памяти/знаний семейства Omnes |
| **Близнец** | встроенная реализация подсистемы ob2h в форке (`omnesagent-memory`/`omnesagent-kag`) |
| **Паритет алгоритмов** | синхронизация *алгоритмов/семантики* между ob2h и близнецом; без синхронизации *данных* |
| **Интероп** | обмен *данными* (записями памяти, findings) между форком и ob2h по явной политике |
| **Golden-набор** | зафиксированный корпус репозитория + эталонные результаты скана для dual-scan сравнения |
| **OB2H_DATA_DIR** | `C:\Projects\ob2h\data` — единственное хранилище ob2h этой машины (единая память всех агентов) |
| **Ф60.3 / Ф62.x / Ф69+** | нумерация фаз планов ob2h (см. `C:\Projects\ob2h\PLAN.md`, `PLAN_v2.0.md`) |

## 3. Инвентаризация ob2h v1.9.2 (что появилось после точки последнего синка)

### 3.1 Движок и производительность

| Изменение | Коммит ob2h | Значение для форка |
|---|---|---|
| Типизированные рёбра AST (`ast_edges` с типами CALLS/IMPORTS/IMPLEMENTS/DEPENDS_ON) | `259ae9a` | паритет `omnesagent-kag::graph` |
| FTS5-индекс AST-графа (Ф62.1) | `259ae9a` | ускорение гибридного поиска по графу |
| Резолв Rust `crate::/super::` импортов, точные границы тел функций | `a646d38` | точность call-path/blast radius |
| Фикс: сохранение межфайловых CALLS/IMPORTS рёбер при инкрементальном скане | `0c79b95` | без него инкрементальный скан рвёт граф |
| Perf: вставка узлов через `RETURNING id` + `prepare_cached` | `208fd94` | портируемый паттерн |
| LRU-кэш эмбеддингов (Ф62.2); `superseded_among` (Ф62.4) | `c6e0c21` | `omnesagent-memory::prefetch`/`dedup` |
| `import-codegraph` — прямой импорт графа из CodeGraph | `678afe0` | сверить направление с `omnesagent-kag::project::codegraph_bridge` |

### 3.2 Знания, синк, безопасность

- **Secrets Vault (v1.9, фазы 56–59, контракт 36->41):** XChaCha20-Poly1305 + Argon2id
  (m=64MiB, t=3), мастер-ключ только в памяти (zeroize), TTL 30 мин, IPC-unlock по loopback,
  миграция M9, 5 инструментов `secrets_*`, GUI PIN ровно 4 цифры (ADR-23), полная изоляция
  от памяти/RAG/графа/дрима/синка (ADR-18/21/22).
- **E2EE-синк PC<->VPS** (AES-GCM) поверх gzip-дельта-бандлов v2 (коммит `651eb82`).
- **Ralph Knowledge Layer:** debt-леджер Ф60.3 (`ob2h ralph debt`, `22ada5b`), вердикты
  только по объективным тестовым сигналам (ADR-K4), `findings-to-memory`.
- **Три эшелона LLM (v1.9.2, `f9a2eb5`):** (1) явный `OB2H_LLM_*` (литерал или имя env,
  фолбэк `<data_dir>/.env`), (2) harness-детект — для `zcode/omnesagent` читает
  `~/.omnesagent/config.toml`, `SwappableLLM` на MCP `initialize`, форс `OB2H_LLM_HARNESS`,
  гейт `OB2H_LLM_HARNESS_DETECT=0`, (3) локальная `LocalMiniLLM` Qwen3-0.6B (candle 0.9,
  ~1.4 GB, ленивое автоскачивание, `OB2H_LOCAL_LLM=0`).
- **Диагностика:** громкий отчёт `knowledge_extract` при неудачных LLM-вызовах
  (`entities=0`), резолв `OB2H_LLM_API_KEY` (имя->env->`<data_dir>/.env`) (`2e4db9b`).

### 3.3 План ob2h v2.0 (`PLAN_v2.0.md`, черновик 25.10.2026) — ориентир паритета

Семь столпов, релевантных форку: tree-sitter hybrid AST (Rust/TS/JS/Py/Go, provenance
`SYNTAX/RESOLVED/HEURISTIC`, fallback при >10% `ERROR`-узлов, бюджет грамматик <= 8 МБ);
ONNX/FastEmbed + `embedding_fingerprint` (запрет смешивания векторных пространств);
Usage-Weighted PPR (`W = W0 * min(1 + ln(1 + access_count), CAP 3.0) * e^(-lambda*Dt) *
(0.1 + 0.9*trust)`, CATD-lite, grace-period >= 4 обращений, сабграф 5–10k узлов);
Dream-консолидация (Temporal Supersession / User Correction / Contextual Coexistence,
рёбра `SUPERSEDES`, tombstone, HITL для SOUL/USER/SYSTEM, rollback-гейт ADR-13);
vec0-инкремент + SIMD-rescore; Agent Loop Integrity (Access Telemetry Parity,
авто-`stale` findings Ralph при изменении AST); Enterprise-инсталлятор (фазы 69–72b+).

## 4. Инвентаризация форка (точки сопряжения и проблемы)

### 4.1 Что уже встроено

| Компонент | Путь | Состояние |
|---|---|---|
| AST/KAG-движок | `backend/crates/omnesagent-kag/` (ast/ 18 языковых групп + Фаза 6: HTML/CSS/манифесты; graph/ pagerank/PPR/blast/callpath/repomap; bench.rs с `GateVerdict`) | работает; `2ea87333a` — синк 10 языков с ob2h |
| Векторный слой | `omnesagent-kag/src/vector/` (rrf, similarity), `embedding/` (fake, local_bert под фичей `candle`) | работает |
| Операционная память | `backend/crates/omnesagent-memory/` (sqlite/lucid/postgres/markdown + `agent_scoped*`, `prefetch`, `dedup`, `conflict`, `consolidation`, `decay`, `audit`, `budget`) | работает; Qdrant удалён `9309a1541` «in favor of ob2h» |
| Gateway REST памяти | `backend/crates/omnesagent-gateway/src/api_memory.rs` (`/api/memory` query/store/delete, per-agent routing) | работает |
| Ralph-инструменты | `backend/crates/omnesagent-tools/src/ralph_tool.rs` (Ralph Knowledge Layer, об2h v1.3 Ф26–28, sqlite-backed) | работает |
| Ralph-крейт | `backend/crates/omnesagent-ralph/` (`loop_driver`, `knowledge`, `debt`, `journal`, ...) | без прямых вызовов ob2h |
| System One | `backend/crates/omnesagent-s1/` (`laya.rs`, `provider.rs`, `contract.rs`) | скелет Laya (Choice/Score/Noul) |
| MCP-клиент | `backend/crates/omnesagent-tools/src/mcp_client.rs` (stdio/http/sse, префикс имени `format!("{}__{}", server, tool)` — `mcp_client.rs:940`) | работает |

### 4.2 Конфигурация

- `[mcp]` (`backend/crates/omnesagent-config/src/schema/mcp.rs`): `McpConfig { enabled: true,
  deferred_loading: false, servers: Vec<McpServerConfig> }`; `McpServerConfig { name,
  transport (stdio/http/sse), command, args, env[secret], url, headers[secret],
  tool_timeout_secs, max_response_bytes, pinned_resources, tls_ca_cert_path }`;
  TOML-синтаксис — `[[mcp.servers]]` array-of-tables. **ob2h в `~/.omnesagent/config.toml`
  не зарегистрирован** (проверено grep 06.11.2026).
- `[ob2h_bridge]` (`backend/crates/omnesagent-config/src/ralph.rs:176`):
  `Ob2hBridgeConfig { enabled: false, ob2h_command: "ob2h", ob2h_args: ["serve"], projects,
  push_scope: verified|all, auto_push_after_archive: false }`. **Потребитель моста в
  `omnesagent-ralph` не реализован** — конфиг объявлен, сквозной публикации findings нет.
- Правила обязательности ob2h для агентов: `AGENTS.md` (монорепо), `.agents/rules/ob2h.md`,
  скилл `.agents/skills/ob2h/SKILL.md`.

### 4.3 Выявленные проблемы (основание Ф0)

| # | Проблема | Evidence | Приоритет |
|---|---|---|---|
| P1 | ob2h недоступен встроенным агентам как MCP-сервер: нет записи в `[[mcp.servers]]` | `~/.omnesagent/config.toml` (grep ob2h пуст) | P0 |
| P2 | **Латентный баг классификации read-only**: `tool_group.rs:175-179` проверяет префиксы `ob2h:project_scan` и т.д. (двоеточие), а реальные имена MCP-инструментов — `ob2h__project_scan` (`mcp_client.rs:940`, двойное подчёркивание). Записи read-only-группы никогда не совпадают; ob2h-инструменты попадают в CodeExploration только за счёт общего правила `name.contains("__")` (`tool_group.rs:57`), теряя специализированный отбор | `tool_group.rs:175`, `mcp_client.rs:940` | P0 |
| P3 | Дубли проектов в БД ob2h: `[omnes-agent]` (скан 06.11) и `[omnesagent]` (23.10) — один путь `C:/Projects/Omnes-agent`; мёртвые `[ob2h]` (старый путь `omnesbot_for_hermes`) | `ob2h project list` (06.11) | P1 |
| P4 | AST-движок близнеца отстал от ob2h v1.9.2 (typed edges, FTS5, crate::/super, межфайловые рёбра при инкрементале, LRU, superseded_among) | §3.1 | P0 |
| P5 | `[ob2h_bridge]` без потребителя; Ralph не публикует findings в ob2h | §4.2 | P1 |
| P6 | Незакоммиченный 30-й срез декомпозиции `schema.rs` (multimodal/pacing/pipeline.rs untracked) | `git status` 06.11 | P1 |
| P7 | Read-only-группа не покрывает новые read-only инструменты ob2h (`memory_context`, `graph_reason`, `project_impact`, `secrets_status`, `graph_stats`, `omnes_stats`) | `tool_group.rs` | P1 |

## 5. Цели и не-цели

**Цели:**
1. Дать встроенным агентам OmnesAgent прямой доступ к ob2h (41 MCP-инструмент) как к
   внешнему слою кросс-продуктовых знаний, при сохранении встроенной памяти харнеса.
2. Довести алгоритмы близнеца (`omnesagent-kag`/`omnesagent-memory`) до паритета с
   ob2h v1.9.2 и закрепить регламент синка алгоритмов.
3. Ввести управляемый интероп: публикация ключевых решений и Ralph-findings в ob2h
   по явной политике, без автосинков.
4. Подготовить форк к столпам ob2h v2.0 (PPR-телеметрия, stale-инвалидация, tree-sitter)
   так, чтобы близнецы не расходились.

**Не-цели:**
- Замена `omnesagent-memory`/`omnesagent-kag` на ob2h; единая БД двух памятей; синхронизация
  данных памятей по умолчанию (см. ADR-O1/O2).
- Портирование в форк Enterprise-инсталлятора ob2h (столп VII), sync-механики PC<->VPS
  и Secrets Vault как замены форковым config-secrets (`PropKind::SecretMap`, #11419).
- Развитие ob2h (сторона ob2h ведётся в `C:\Projects\ob2h` по `PLAN_v2.0.md`).
- Реализация System One (Laya) — отдельный трек (`omnesagent-s1`); здесь только переиспользование
  опыта `LocalMiniLLM` (см. Ф6).

## 6. Архитектурные решения (форковые ADR)

### ADR-O1. Две памятни, две роли
Встроенная память форка — операционный слой харнеса (сессии, agent-scoped, prefetch);
ob2h — долговременный кросс-продуктовый слой (пользователь, проекты, решения).
Ни один из слоёв не является «источником истины» для другого. Замена/слияние запрещены.

### ADR-O2. Паритет алгоритмов — да, паритет данных — нет
Из ob2h в близнеца переносятся алгоритмы и семантика (типы рёбер, формулы ранжирования,
схема фидбека); записи памяти и узлы графа НЕ синхронизируются автоматически.
Основание: split-brain двух живых памятей хуже паритета возможностей. Совместимо с
ADR-K6 ob2h (`ralph_*` вне sync-бандлов) — обмен только по явным политикам (ADR-O3).

### ADR-O3. Интероп — только по политике владельца
Публикация данных форка в ob2h (`memory_save`, `findings-to-memory`) выполняется:
- явно (агент вызывает `ob2h__memory_save` сам), либо
- конфигурируемо (`[ob2h_bridge]`, `push_scope`, явный список `projects`),
- никогда — молчаливым автосинком всех записей.

### ADR-O4. ob2h подключается штатным MCP-механизмом форка
`[[mcp.servers]]` + `deferred_loading` + read-only-классификация оркестратора. Никаких
отдельных MemoryProvider-плагинов или in-process встраивания ob2h.

### ADR-O5. Секреты: ob2h Vault — межагентный сейф, форк — конфиг-секреты
Доступ агентов к `secrets_*` ob2h разрешён; PIN вводится только GUI/CLI ob2h и никогда
не проходит через чат/LLM/MCP-аргументы (ADR-18/21/22 ob2h). Форковые конфиг-секреты
(`#[secret]` поля `McpServerConfig.env` и др.) остаются в форке; изоляция проверяется
тестами на непопадание секретов в память/RAG/граф/логи.

### ADR-O6. Телеметрия доступа — пакетная
Все касания памяти/графа инкрементируют `access_count` в ob2h пакетно (аккумулятор в
памяти, сброс раз в 10 с или на shutdown) — по образцу столпа III/VI ob2h v2.0, чтобы
не блокировать `Mutex<Connection>`.

## 7. Фазы

### Ф0 — Гигиена и подключение ob2h как MCP-сервера (0.5–1 день, P0)

**Цель:** встроенные агенты форка получают 41 инструмент ob2h; исправить классификацию
read-only; прибрать БД ob2h и рабочее дерево.

Задачи:

- [x] **0.1** Завершить и закоммитить 30-й срез декомпозиции `schema.rs`
      (`schema/multimodal.rs`, `schema/pacing.rs`, `schema/pipeline.rs` уже в рабочем
      дереве): `cargo check -p omnesagent-config`, `-p omnesagent-providers`, `--tests`.
      **Done:** модули `schema/{multimodal,pacing,pipeline}.rs` присутствуют; рабочее дерево
      чистое (единственный untracked — этот план).
- [x] **0.2** Зарегистрировать ob2h в `~/.omnesagent/config.toml` (см. пример в Приложении A.1):
      `transport = "stdio"`, `command` -> `C:\Projects\ob2h\target\release\ob2h.exe`,
      `args = ["serve"]`, `env.OB2H_DATA_DIR = "C:\\Projects\\ob2h\\data"`,
      `tool_timeout_secs` достаточный для AST-сканов (рекомендация: 300).
      **Done:** `[[mcp.servers]] name="ob2h"` добавлен (stdio, args=["serve"], timeout=300,
      env.OB2H_DATA_DIR); TOML провалидирован через `tomllib`.
- [x] **0.3** Включить `deferred_loading = true` (41 схема инструмента не должны жить в
      системном промпте; загрузка через `tool_search`).
      **Done:** `[mcp] enabled = true`, `deferred_loading = true`.
- [x] **0.4** Исправить префиксы read-only-группы `tool_group.rs:175-179`:
      `ob2h:` -> `ob2h__` (реальный формат имён из `mcp_client.rs:940`) и расширить список:
      `ob2h__memory_context`, `ob2h__graph_reason`, `ob2h__project_impact`, `ob2h__graph_stats`,
      `ob2h__omnes_stats`, `ob2h__secrets_status` (P2, P7). Обновить юнит-тесты
      `test_code_exploration_read_only` и соседние.
      **Done** (коммит `7dfde4be9`, ветка `feat/fork-sync-v0.8.7`): allowlist = 11 read-only
      префиксов; добавлены `test_ob2h_read_only_uses_real_double_underscore_prefix` и
      `test_code_exploration_excludes_ob2h_write_tools`; `cargo test -p omnesagent-runtime
      --lib tool_group::tests` → 6 passed.
- [x] **0.5** Дедуп проектов в БД ob2h: удалить дубликат `omnesagent` (тот же
      `root_path //?/C:/Projects/Omnes-agent`, что и канонический `omnes-agent`).
      **Done** (мутация внешней БД `C:\Projects\ob2h\data\ob2h.db`): бэкап
      `ob2h backup --scope full` -> `data\backups\2026-11-07_185849` (integrity_check ok);
      остановлены 2 процесса ob2h; удалены строки `project_id='omnesagent'` —
      graph_edges 3496, graph_nodes 253, project_files 1438, memories 15, projects 1
      (+ 0 в прочих child-таблицах); `PRAGMA wal_checkpoint(TRUNCATE)`; `integrity_check`
      ok; FTS-индексы (`memories_fts`, `chunks_fts`, `graph_nodes_fts`) консистентны;
      канонический `omnes-agent` не затронут (230984 ast_refs / 46765 nodes / 102433 edges /
      23 memories). Guard-скрипт: `%TEMP%\ob2h_dedup_omnesagent.py`.
      **Решение владельца:** данные дубля удалены вместе с проектом (не мигрированы);
      мёртвые `[ob2h]`/`[ob2m]` (путь `omnesbot_for_hermes`) оставлены как есть.
- [x] **0.6** Smoke-тест сквозного пути: рестарт сессии/демона, в EngineeringTask-интенции
      вызвать `ob2h__omnes_stats` и `ob2h__memory_search`; в CodeExploration убедиться, что
      ob2h-инструменты попадают в read-only-набор.
      **Done** (клиент-скрипт `%TEMP%\ob2h_mcp_smoke.py`): спавн `ob2h.exe serve` с боевым
      `env.OB2H_DATA_DIR`, MCP-хендшейк -> `serverInfo = {name: ob2h, version: 1.9.2}`,
      `tools/list` = **41 инструмент**; все 11 read-only имён из allowlist присутствуют
      (`missing: none`); реальные read-only вызовы: `omnes_stats` -> `memories=430 ... db=2011108KB`,
      `memory_search` -> результаты. Runtime-гейтинг (CodeExploration/EngineeringTask) покрыт
      юнит-тестами `tool_group::tests` (см. 0.4).
      **Наблюдение (follow-up):** на сервере есть ещё read-only инструменты вне allowlist
      Ф0.4 — `project_scan_status`, `project_call_path`, `project_graph_search`, `ast_diff`,
      `ast_history`, `dream_log`, `dream_status`, `ralph_context`, `ralph_report`, `session_log`,
      `secrets_get`. Расширение allowlist — отдельное решение (не входит в объём Ф0.4).

**DoD Ф0:** `[[mcp.servers]]` с ob2h в конфиге; тесты `tool_group` зелёные с реальным
префиксом `ob2h__`; `ob2h project list` не содержит дублей пути форка; WIP schema-срез
закоммичен; smoke-сценарий пройден вручную и зафиксирован в журнале плана.

### Ф1 — Паритет AST-движка: omnesagent-kag <- ob2h v1.9.2 (2–4 дня, P0)

**Цель:** перенести алгоритмические улучшения ob2h в `omnesagent-kag`, зафиксировать
регламент синка. Данные не переносятся (ADR-O2).

Задачи:

- [x] **1.1** Верификационный dual-scan golden-набор: скан `C:\Projects\Omnes-agent`
      внешним ob2h (`ob2h project scan --id omnes-agent`) и встроенным kag-сканером;
      дифф по символам/рёбрам; зафиксировать baseline в `backend/crates/omnesagent-kag/tests/golden/`.
      **Done:** harness `tests/test_golden_dual_scan.rs` (symbol F1 / call F1 против
      baseline), эталон `tests/golden/ob2h-1.9.2.expected.json`, перегонка
      `tests/golden/regenerate.ps1` + `export_ob2h.py` (изолированный `OB2H_DATA_DIR`,
      реальные данные ob2h не затрагиваются).
      **Решение по дизайну (отличие от буквы плана):** baseline фиксируется на **маленьком
      неизменяемом корпусе** `tests/golden/corpus/` (Rust/Python/TS/Go/SQL + skip-проба
      `*_pb2.py`), а не на всём репозитории: только так F1 детерминирован и осмыслен
      (полный репо-скан даёт шум от постоянно меняющихся исходников). Зафиксировать в
      `docs/kag_sync.md` (§1.9).
      **Порты паритета (3 расхождения, выявленные difff'ом), все в `omnesagent-kag`:**
      (а) `ast::module_to_paths` — абсолютный Python-модуль (`app.models`) больше не
      трактуется как относительный; отдельная ветка `sys.path`-корней (порт `2ad9027`);
      (б) `ast::resolve_calls` — Python-специфичное приписывание вызывающего
      (`is_python` → только `Function` и в пределах `line_end`); (в)
      `ast::NODE_TYPE_BY_PREFIX`/`node_type_for_prefix` + авто-создание висячих целей
      рёбер в `ProjectService::scan_project` (route:/pkg:/table: заглушки, ADR-AST-6) и
      `ast::package_of_module`/`consolidate_external_imports` (IMPORTS `module:` → `pkg:<eco>:`).
      **Результат:** `symbol P=R=F1=1.000`, `call P=R=F1=1.000` (оба выше DoD-порогов);
      `cargo test -p omnesagent-kag` — все наборы зелёные.
- [x] **1.2** Резолв Rust `crate::/super::` импортов + точные границы тел функций
      (порт `a646d38` из ob2h) в `omnesagent-kag/src/ast/`.
      **Done:** `ast::resolve_imports` + `resolve_module_file` + `first_existing` портированы
      дословно (сверено с `c:\Projects\ob2h\src\project\ast.rs:7395-7586`); вызывается в
      `ProjectService::scan_project` шагом 1 type-pass **до** `resolve_calls` (порядок ob2h).
      IMPORTS-рёбра `file: → module:<raw>` переписываются в `file:<rel_path>` для Rust
      `crate::/super::/self::`, TS `./`/`../`/`@/`, Python `.mod`/`pkg.mod`/типовых корней
      `backend/app/…`; внешние модули остаются `module:`. Коммит `c08b3e1b7`
      (`feat(kag): resolve AST import edges to project files (ob2h parity)`).
      Вторая половина 1.2 — «точные границы тел функций»: портирован `ts_body_end`
      (баланс фигурных скобок со строками/комментариями, expression-body стрелки)
      и применён к TS/JS-функциям в `parse_ts_js`; коммит `a311be3c5`
      (`feat(kag): compute precise TS/JS function body bounds (ob2h parity)`).
      5 новых тестов в `tests/test_ast_extractor.rs`; `cargo test -p omnesagent-kag` —
      52+21+12 passed, `cargo check` чист.
      **Остаток Ф43 (2ad9027) не портирован:** `AST_EXTRACTOR_VERSION` + `kv`-гейт
      «версия экстрактора → полный рескан», многострочные def/route-декораторы,
      резолв `self/cls`-вызовов, флаг `--full` — см. новый пункт 1.2b.
- [x] **1.2b** Остаток Ф43 (`2ad9027`) в `omnesagent-kag`: константа
      `AST_EXTRACTOR_VERSION` + `kv`-гейт «версия не совпала → полный рескан»
      (нужен, чтобы инкрементальные узлы/`ast_refs` инвалидировались при правках
      экстрактора), многострочные `def`/route-декораторы, резолв `self/cls`-вызовов,
      флаг `--full` у CLI-скана.
      **Done:** `ast/mod.rs` — `py_fn_re` без требования `)`/`:` на строке `def`;
      `decorator_block_end`/`py_body_end`/`py_signature_end` (границы тела по
      отступам); склейка многострочных route-декораторов; `collect_files` скипает
      `*_pb2.py`/`*_pb2_grpc.py`; `collect_call_sites` резолвит `self.`/`cls.`.
      `project/mod.rs` — `pub const AST_EXTRACTOR_VERSION = "1"` + kv-гейт в
      `scan_project` + атомарная запись версии в tx. Tool `project_code`:
      параметр `full` (эквивалент ob2h `project scan --full`). Тесты:
      `test_python_multiline_signature_and_route`, `test_self_method_call_resolved`,
      `tests/test_scan_full.rs` (3 теста). `cargo test -p omnesagent-kag` — 23+1+12+3
      passed; `cargo check -p omnesagent-tools` чист.
- [x] **1.3** Сохранение межфайловых CALLS/IMPORTS рёбер при инкрементальном скане
      (порт `0c79b95`): таблица `ast_refs` (M8) + загрузка узлов/рефов неизменённых
      файлов в `scan_res` перед type-pass + сохранение `ast_refs`.
      **Done** (ветка `feat/fork-sync-v0.8.7`): `schema.rs` — `SCHEMA_VERSION` 6→7,
      таблица `ast_refs` + 2 индекса (`MIGRATION_BASE`); в `ProjectService::scan_project`
      добавлена загрузка неизменённых узлов (`graph_nodes`, с `strip_prefix` scoped-`node_id`)
      и `ast_refs` в `scan_res` перед type-pass + сохранение `ast_refs` (DELETE по проекту/
      `deleted_files`+`file_meta`, INSERT call/import с фильтром по инкременту); dev-deps
      `rusqlite`/`parking_lot`; новый интеграционный тест
      `tests/test_incremental_crossfile.rs`. Коммиты `a2f362423` (fix: снятие tx-guard)
      и `acaf1e217` (feat). `cargo test -p omnesagent-kag` — 52+21+1+12 passed.
      Побочно устранён унаследованный самодеадлок `parking_lot::Mutex` в `scan_project`
      (guard транзакции не снимался перед повторным `self.conn.lock()` на codegraph-синке) —
      ранее путь не покрывался тестами.
- [x] **1.4** Типизированные рёбра AST (CALLS/IMPORTS/IMPLEMENTS/DEPENDS_ON) в схеме
      близнеца (`omnesagent-kag/src/schema.rs`, миграция по образцу ob2h M10).
      **Done:** паритет уже достигнут — тип ребра хранится в `graph_edges.label TEXT`
      (та же модель, что у ob2h «простой» `graph_edges`), UNIQUE(source_id,target_id,label).
      Все четыре типа генерируются AST-сканером: `CALLS` — `resolve_calls`
      (`ast/mod.rs:3639`, provenance=RESOLVED); `IMPORTS` — `languages.rs`/`iac.rs`/`schemas.rs`;
      `IMPLEMENTS` — `ast/mod.rs` (1248/1453/…, интерфейсы/трейты, включая TS);
      `DEPENDS_ON` — `manifests.rs`/`iac.rs`. Константа `USAGE_EDGE_LABELS`
      (`graph/callpath.rs:12`) фиксирует канонический набор. Golden dual-scan
      (Ф1.1) подтверждает совпадение по CALLS/IMPORTS с ob2h. Доработок не требует.
- [x] **1.5** FTS5-индекс графа (порт Ф62.1) в `omnesagent-kag/src/graph/service.rs`
      + гибридная выдача в `project_graph_search`-эквиваленте.
      **Done:** в `omnesagent-kag/src/schema.rs` поднята `SCHEMA_VERSION` 7→8 и добавлена
      внешне-контентная FTS5-таблица `graph_nodes_fts(label, description, content='graph_nodes',
      tokenize='trigram')` с триггерами `ai/ad/au` и backfill существующих узлов
      (порт ob2h `MIGRATION_V10`, креейт обновляет индекс автоматически — триггеры
      срабатывают и на вставках из `scan_project`/`codegraph_bridge`). В
      `GraphService::search` лексический скоринг переведён с полного перебора
      `SELECT … FROM graph_nodes` на кандидатов из `graph_nodes_fts MATCH` (OR по словам),
      за скорингом label/desc прежняя формула; при недоступности/битом FTS5 — fallback
      на прежний полный перебор (без регресса). Гибридная выдача (лексика FTS5 +
      векторный RRF) сохранена. Тесты: миграция/идемпотентность/триггеры
      `graph_nodes_fts` + `cargo test -p omnesagent-kag` все наборы зелёные.
- [x] **1.6** Perf-паттерн `RETURNING id` + `prepare_cached` (порт `208fd94`) в горячих
      вставках узлов/рёбер.
      **Done:** `RETURNING id` уже был (заменяет двухшаговый INSERT+SELECT вставкой узла);
      горячие per-loop statements переведены `tx.prepare` → `tx.prepare_cached`
      (кэш компиляции SQL живёт на `Connection`, переиспользуется между сканами):
      `scan_project` — вставка узлов, вставка рёбер, авто-создание узлов-целей,
      `SELECT id …node_id`, вставка `ast_refs`; `codegraph_bridge::sync_codegraph_db` —
      вставка узлов и рёбер. `cargo test -p omnesagent-kag` — все наборы зелёные.
- [x] **1.7** LRU-кэш эмбеддингов (порт Ф62.2) в `omnesagent-kag/src/embedding/` и
      семантика `superseded_among` (Ф62.4) в `omnesagent-memory/src/dedup.rs`/`conflict.rs`.
      **Done:** (Ф62.2) в `omnesagent-kag/src/embedding/mod.rs` добавлен
      `CachedEmbeddingProvider` — потокобезопасный `LruCache` (`parking_lot::Mutex`,
      ёмкость по умолчанию 1000) поверх любого бэкенда, ключ = `trim()+lowercase`,
      батч с частичными попаданиями эмбеддит только промахи, есть `stats()`/`cache_len()`;
      `default_embedding_provider()` оборачивает бэкенд в кэш; зависимость `lru = "0.18.2"`.
      (Ф62.4) в `omnesagent-memory/src/conflict.rs` добавлен `superseded_among(&[MemoryEntry])` —
      запись подавляется как устаревшая ТОЛЬКО если её замена (`superseded_by`) присутствует
      в той же выборке (иначе знание терялось бы); подключён в `dedup::core_candidates`
      (аналог consolidation-сайта ob2h `service.rs:1492`). Тесты: кэш (hit/miss/нормализация/
      частичный батч/эвикция LRU/dim/пустой вход) + `superseded_among`/`core_candidates`.
      Расхождение с ob2h (осознанное): модель форка хранит связь в колонке
      `memories.superseded_by` (а не в рёбрах `memory_links.kind='supersedes'`), а подавление
      на чтении выполняется SQL-фильтром `superseded_by IS NULL`; хелпер централизует
      семантику Ф62.4 на слое чистой логики. `cargo test -p omnesagent-memory -p omnesagent-kag` зелёные.
- [x] **1.8** Сверить направление `import-codegraph` (ob2h `678afe0`) с
      `omnesagent-kag/src/project/codegraph_bridge.rs`; задокументировать в `docs/kag_sync.md`.
      **Done:** направление **совпадает** — внешняя БД `.codegraph/codegraph.db` →
      встроенный граф (`provenance='codegraph'`), реверс отсутствует в обоих движках.
      Отличия форка (не нарушают паритет): scoped `node_id` `{project}:cg:{id}`,
      слияние `signature\ndocstring`, контекст ребра = JSON `metadata`/`{"line":N}`,
      `READ_ONLY` + fail-open (у ob2h — ручная CLI-команда `import-codegraph`, ошибка наружу).
      Вывод: доработок не требует. Секция «Ревизия направления» в `docs/kag_sync.md` (§6).
- [x] **1.9** Регламент синка: `docs/kag_sync.md` — чеклист переноса алгоритмов из ob2h
      (прецедент: `2ea87333a`), правила golden-перегонки, владелец-гейт.
      **Done:** создан `docs/kag_sync.md`: §1 границы (ADR-O2 алгоритм-паритет),
      §2 чеклист переноса (upstream-hash → порт → тесты → golden → 1 пункт = 1 коммит),
      §3 правила golden-перегонки (`regenerate.ps1`, изолированный `OB2H_DATA_DIR`,
      пороги DoD), §4 осознанные расхождения (scoped `node_id`, авто-синк CodeGraph),
      §5 bench-гейт, §6 ревизия CodeGraph↔граф (Ф1.8). Закрывает пункт DoD Ф1
      «`docs/kag_sync.md` существует».

**DoD Ф1:** dual-scan: symbol F1 >= 95%, call F1 >= 90% (пороги фазы 71.7 ob2h v2.0);
`cargo test -p omnesagent-kag` (включая `test_ast_extractor` 16 + `test_phase6_parsers` 12)
зелёные; `GateVerdict` bench не деградировал (`recall/MRR` в лимитах `bench.rs`);
`docs/kag_sync.md` существует.

### Ф2 — Разделение ролей и управляемый интероп (2–3 дня, P1)

**Цель:** зафиксировать границы памятей; включить публикацию ключевых решений в ob2h
по политике (ADR-O1/O3).

Задачи:

- [x] **2.1** ADR-документ в `backend/docs/adr/` (или соглашённое место): роли двух
      памятей, запрет автосинка данных, границы секретов (текст ADR-O1/O3).
      **Done:** создан `docs/adr/0001-ob2h-interop.md` (соглашённое место — рядом с
      `docs/kag_sync.md`; каталога `backend/docs/` не существует). Зафиксированы:
      роли двух памятей (ADR-O1), алгоритм-паритет без дата-паритета (ADR-O2),
      интероп только по политике владельца / без автосинка (ADR-O3), штатный MCP-механизм
      (ADR-O4), границы секретов и запрет публикации `#[credential_class]`-контента (ADR-O5),
      fail-open семантика моста.
- [x] **2.2** Реализовать потребителя `[ob2h_bridge]` в `omnesagent-ralph` (сейчас конфиг
      без потребителя): публикация архивированных runs/verified findings в ob2h через
      CLI-вызов (`ob2h_command`/`ob2h_args` + `projects`), `push_scope = verified|all`,
      `auto_push_after_archive`; fail-open с журналом (не ронять Ralph-цикл при
      недоступности ob2h).
      **Done:** новый модуль `omnesagent-ralph/src/bridge.rs`: `Ob2hBridge` +
      `McpStdioClient` — MCP-stdio клиент по контракту `ob2h serve`
      (newline-delimited JSON-RPC 2.0: `initialize` → `notifications/initialized` →
      `tools/call memory_save` с `content/key/category/importance/source/project_id`).
      Публикация в `loop_driver.rs` после `state.transition_to(RalphPhase::Archived)`
      при `enabled && auto_push_after_archive`: чтение findings из `journal.jsonl`,
      фильтр по `push_scope` (`verified|all`), `memory_save` на каждую находку × каждый
      `projects`-таргет, стабильный `key = finding_id` (идемпотентность). Транспорт
      абстрагирован трейтом `Ob2hTransport`; таймаут на MCP-раунд — `round_trip`.
      **Fail-open:** ошибка спавна/таймаут/ошибка инструмента логируется (`eprintln!`) и
      не прерывает цикл. Проводка: `RalphOrchestrator::new(..., ob2h_bridge)` →
      `drive_loop(..., bridge)`; `backend/src/main.rs` передаёт `config.ob2h_bridge.clone()`.
- [x] **2.3** Политика публикации памяти: агентские методы (системный промпт/скилл форка)
      предписывают: решения владельца и архитектурные факты -> `ob2h__memory_save`
      (`project_id = "omnes-agent"`); операционный контекст сессии -> встроенная память.
      **Done:** секция «Политика публикации памяти (ADR-O1/O3)» в
      `.agents/skills/ob2h/SKILL.md` и расширенный п.2 «Long-Term Memory» в
      `.agents/rules/ob2h.md`: решения владельца/архитектурные факты → `memory_save`
      с `project_id = "omnes-agent"` и стабильным `key`; операционный контекст сессии →
      встроенная память харнеса; запрет автосинка.
- [x] **2.4** Дедупликация на границе: перед `memory_save` в ob2h — обязательный
      `memory_search` (правило уже в скилле); повторить в системном промпте форка.
      **Done:** в `.agents/skills/ob2h/SKILL.md` (п.2 политики) и `.agents/rules/ob2h.md`
      закреплено: перед каждым `memory_save` — `memory_search` по тому же `key`/смыслу,
      при совпадении `memory_update` вместо дубля. Мост со своей стороны использует
      стабильный `key = finding_id` (`memory_save` идемпотентен по ключу).
- [x] **2.5** Тесты: mock-обёртка CLI-моста (успех/таймаут/отсутствие бинарника);
      не публиковать записи с `#[credential_class]`-контентом.
      **Done:** в `bridge.rs` (mod tests) — mock-транспорт `Ob2hTransport`:
      успех (`publishes_only_verified_findings_under_verified_scope`,
      `all_scope_publishes_every_finding`), таймаут (`times_out_when_ob2h_never_answers`),
      отсутствие бинарника (`reports_missing_ob2h_binary`, реальный спавн), no-op при
      disabled (`disabled_bridge_is_a_noop`), credential-guard
      (`credential_class_content_is_never_published`, `plain_content_is_publishable`).
      `cargo test -p omnesagent-ralph` — 20 unit + 3 e2e зелёные.

**DoD Ф2:** `[ob2h_bridge]` при `enabled = true` публикует verified-findings выбранного
проекта; при `enabled = false` и при падении ob2h Ralph-цикл жив; ADR-документ принят
владельцем.

### Ф3 — Ralph Knowledge Layer <-> ob2h (2–3 дня, P1)

**Цель:** связать автономный цикл форка с Knowledge Layer ob2h v1.9 (debt-леджер, вердикты,
AST-дельты, stale-инвалидация).

Задачи:

- [x] **3.1** Debt-леджер: экспорт форкового `omnesagent-ralph::debt` в `ob2h ralph debt`
      (Ф60.3) при архивации run (через мост Ф2).
      **Done:** `Ob2hBridge::publish_debt` (`bridge.rs`) открывает run (`ralph_start`,
      feature_slug = `<slug>@<run_id>` — обход «one active run per (project,feature)») и
      записывает debt-пункты одной `deferred`-итерацией (`ralph_iteration`); credential-guard
      (Ф2.5) фильтрует секрет-класс. `loop_driver` накапливает `DebtLedger` по каждой
      закоммиченной итерации (до `commit_iteration`) и экспортирует его при архивации,
      fail-open. Тесты: `publishes_debt_as_deferred_findings`,
      `debt_with_credentials_is_skipped`, `disabled_bridge_skips_debt_export`.
- [x] **3.2** AST-дельты в валидацию: `loop_driver` использует `ast_diff`/`ast_history`
      (MCP ob2h или kag-эквиваленты Ф1) для символьного диффа итераций вместо сырого git-диффа.
      **Done:** kag-эквивалент Ф1 в `ast.rs` (`snapshot_paths` → `diff_snapshots` через
      `omnesagent_kag::AstCodeExtractor`); `loop_driver` снимает снапшот до/после executor'а,
      пишет `ast_changes` в `IterationRecord` (7 unit-тестов ast).
- [x] **3.3** Stale-инвалидация (столп VI v2.0): при изменении сигнатур/рёбер AST помечать
      связанные findings форка `stale`; объединить с `omnesagent-ralph::journal`.
      **Done:** `journal::stale_findings` + `Journal::append_finding`/`read_findings`
      (latest-per-id); `loop_driver::apply_staleness_pass` помечает findings `stale` до записи
      собственного finding итерации (идемпотентно; added-символы не инвалидируют). Тесты в
      `journal.rs` и `loop_driver.rs` (`staleness_pass_marks_findings_for_modified_signature`,
      `staleness_pass_ignores_added_symbols`).
- [x] **3.4** Вердикты: автоматические вердикты итераций форка — только по объективным
      тестовым сигналам (соответствие ADR-K4 ob2h); человек/дрим могут переопределить.
      **Done:** модуль `verdict.rs` (`auto_verdict` по `TestsSummary`, `self_assessment` не
      читается, `verdict_source = "auto_tests"`); `loop_driver` заполняет `verdict`/
      `verdict_source` во всех ветках записи `IterationRecord`. 4 unit-теста.
- [ ] **3.5** Опционально (отдельным решением): интеграция `omnesagent-ralph` в Triage Router
      (Tier 2 из `PLAN.md`) — вне обязательного объёма этой фазы.

**DoD Ф3:** `[x] archive run форка создаёт debt-записи в ob2h; после правки сигнатуры
зависимые findings помечены stale в журнале; вердикт-логика покрыта тестами.`
Проверено: `cargo test -p omnesagent-ralph` — 39 unit + 3 e2e зелёные;
`cargo clippy -p omnesagent-ralph --all-targets` и `cargo check --workspace` без ошибок.

### Ф4 — Secrets Vault мост (1–2 дня, P2)

**Цель:** агентский доступ к межагентному сейфу ob2h без дублирования с конфиг-секретами форка.

Задачи:

- [x] **4.1** Разрешить read-класс `ob2h__secrets_status` в read-only-группе (сделано в Ф0.4);
      `secrets_get/set/delete` — только EngineeringTask/SystemAdmin-интенции (не CodeExploration).
      **Done:** allowlist `secrets_status` в `is_read_only_tool` (Ф0.4); `secrets_get/lock`
      добавлены в список write-класса; новый тест `test_ob2h_vault_tools_are_engineering_only`
      (`omnesagent-runtime::orchestrator::tool_group`) — в CodeExploration остаётся только
      `secrets_status`, Engineering/SystemAdmin сохраняют все.
- [x] **4.2** Правило в системном промпте/скилле: PIN и разблокировка — только GUI/CLI ob2h;
      инструмент `secrets_get` не вызывать для значений, вводимых пользователем в чат (ADR-18).
      **Done:** раздел «Vault & Secrets (ADR-O5)» в `.agents/rules/ob2h.md` (системное правило
      монорепо) и раздел «🔐 Сейф (Vault) и секреты (ADR-O5, ADR-18)» в
      `.agents/skills/ob2h/SKILL.md`.
- [x] **4.3** Тесты изоляции: содержимое `secrets_get` не попадает в записи памяти (оба слоя),
      RAG-чанки, граф, tool-логи форка; секреты `McpServerConfig.env` не видны ob2h.
      **Done:** покрытие негативными тестами по четырём границам — (а) скраб текста MCP-ответов
      (`mcp_context::resource_wrapper_redacts_secrets`); (б) `McpServerConfig.env`/`headers` =
      `EncryptedSecret` без раскрытия при интроспекции (`schema::empty_secret_map_emits_container_row`,
      `secret_map_entries_add_list_and_remove_without_leaking_values`), per-server `env` в
      `mcp_transport::spawn`; (в) write-сейф-инструменты вне CodeExploration (тест 4.1);
      (г) `#[credential_class]`-контент не публикуется мостом (`bridge::credential_class_content_is_never_published`,
      `debt_with_credentials_is_skipped`).
- [x] **4.4** Документ: когда использовать сейф ob2h (межагентные токены LLM-провайдеров),
      когда форковые config-secrets (конфиг серверов) — в ADR Ф2.
      **Done:** ADR-O5 в `docs/adr/0001-ob2h-interop.md` расширен правилом выбора
      (таблица «критерий → ob2h Vault vs форковый конфиг-секрет») и картой покрытия изоляции.

**DoD Ф4:** [x] негативные тесты изоляции зелёные; правила задокументированы; утечки в
`omnes_stats`/логах не обнаружены прогоном.
Проверено: `cargo test -p omnesagent-runtime tool_group` — 7/7 зелёные; существующие тесты
изоляции конфига/моста/скраба зелёные (`cargo test -p omnesagent-config`,
`-p omnesagent-ralph`, `-p omnesagent-tools`).

### Ф5 — Access Telemetry Parity (2 дня, P2)

**Цель:** реальные обращения форка к знаниям учитываются в ob2h — фундамент Usage-Weighted
PPR v2.0 (столп III/VI).

Задачи:

- [x] **5.1** Инструментировать точки касания: чтения форковой памяти (`recall`/`get`
      через `SqliteMemory` — `recall_scoped`, `get`, `get_for_agent`) пишут в аккумулятор
      доступа; `/api/memory` покрыт транзитивно (читает через `recall`). Вызовы `ob2h__*`
      из оркестратора уже учитывались на стороне ob2h (собственный `touch_access`).
      **Отложено отдельным пунктом:** `omnesagent-kag::graph::service` (нужна миграция
      `graph_nodes.access_count`) и `omnesagent-memory::prefetch` (не подключён к боевому
      пути, `saturation_access` там хардкод).
- [x] **5.2** Пакетный репортер: аккумулятор `AccessTelemetry` (`omnesagent-memory::telemetry`),
      flush раз в 10 с (фон через `omnesagent_spawn::spawn!`) или на shutdown (`Drop` для
      `SqliteMemory`); единый `apply_access_batch` для обоих путей; при ошибке batch
      возвращается в аккумулятор (`requeue`) — fail-open. **Done:** тесты
      `telemetry::*` (4), `sqlite::touch_access_is_batched_until_flush`,
      `sqlite::recall_records_access_telemetry_until_flush`, `sqlite::get_records_access_telemetry`,
      `sqlite::drop_flushes_pending_access_telemetry` — `cargo test -p omnesagent-memory`
      521+4 зелёные.
- [ ] **5.3** Гейт качества: телеметрия не добавляетBlocking вызовов в turn-цикл
      (бенч-замер p99 turn latency до/после).

**DoD Ф5:** форковая сторона закрыта — `access_count` растёт от реальных обращений форка
(чтения памяти через `recall`/`get` + `touch_access` из инжекта), без блокирующих записей в
turn-цикле (пакетный flush). Замер в `ob2h`-графе и p99-бенч — отдельными пунктами: ob2h v2.0
ещё не выпущен, а `graph_nodes` в форке не имеет колонки `access_count`.

### Ф6 — Паритет со столпами ob2h v2.0 (перспектива, P3, по мере релизов upstream)

Привязать к фазам `PLAN_v2.0.md`; каждый пункт — отдельным решением после выхода
соответствующего релиза ob2h:

- [ ] **6.1** (Ф70–71 ob2h) tree-sitter в `omnesagent-kag` под фичей `treesitter`:
      Rust/TS/JS/Py/Go, provenance-поля рёбер, fallback на regex при >10% `ERROR`-узлов,
      бюджет грамматик <= 8 МБ.
- [ ] **6.2** (Ф72) ONNX/FastEmbed + `embedding_fingerprint` в `omnesagent-kag/src/embedding/`:
      запрет смешивания векторных пространств, `reindex`-процедура; для форка оценить
      reuse `local_bert.rs` (candle).
- [ ] **6.3** (72b) vec0-инкремент + SIMD-rescore в векторном слое близнеца.
- [ ] **6.4** (столп IV) Dream-консолидация: сверить `omnesagent-memory::consolidation/
      conflict/decay` с семантикой SUPERSEDES/HITL ob2h; для форка — операционный аналог
      (ночная консолидация agent-scoped памяти), данные не сливаются (ADR-O2).
- [ ] **6.5** (столп III) Usage-Weighted PPR в `omnesagent-kag/src/graph/pagerank.rs`
      (формула ob2h v2.0, CAP_access, CATD-lite, grace-period) — после Ф5.
- [ ] **6.6** (Laya/System One) Переиспользовать `LocalMiniLLM`-опыт ob2h (Qwen3-0.6B,
      candle 0.9) в `omnesagent-s1` (`laya.rs`): локальный инференс Choice/Score/Noul
      421M ModernBERT-модели; связь с утверждённым ТЗ System One. Отдельный трек — здесь
      только фиксация переиспользования стека candle.

**DoD Ф6 (по каждому пункту):** golden-перегонка Ф1 проходит на новых алгоритмах;
паритет-матрица (Приложение B) обновляется после каждого релиза ob2h.

## 8. Стратегия верификации

1. **Golden-наборы (Ф1):** репозиторий форка как фиксированный корпус; эталон = вывод
   ob2h-сканера; метрики symbol F1 / call F1; хранение в
   `backend/crates/omnesagent-kag/tests/golden/`; перегонка — скриптом, коммит эталона
   с версией ob2h в имени файла.
2. **Bench-гейт:** `omnesagent-kag/src/bench.rs` (`GateVerdict`, `RECALL_DROP_LIMIT`,
   `MRR_DROP_LIMIT`) — обязательный прогон после каждой фазы Ф1/Ф6; деградация = откат
   фазы (соответствие ADR-13 ob2h).
3. **Регресс-правила:** после каждой фазы — `cargo check --workspace`, покрейтовые тесты
   затронутых крейтов; для Ф0 — дополнительно ручной smoke §7-Ф0.0.6.
4. **Изоляция секретов (Ф4):** негативные тесты на всех путях утечки (память, RAG, граф,
   логи, tool-события).
5. **Латентность (Ф5):** замер p99 turn latency до/после телеметрии.
6. **Read-only-матрица (Ф0):** юнит-тест `filter_tools_for_intent` на реальных префиксах
   `ob2h__*` — ни один write-инструмент ob2h не попадает в CodeExploration.

## 9. Риски и митигации

| Риск | Вероятность | Митигация |
|---|---|---|
| Расхождение близнецов при ускорении ob2h v2.0 | высокая | регламент `docs/kag_sync.md` (Ф1.9) + паритет-матрица (Прил. B) в цикле релизов |
| Split-brain двух памятей при интеропе | средняя | ADR-O2/O3: без автосинка; дедуп-правило `search-before-save`; публикация только verified |
| Блокировка `ob2h.exe` на Windows при пересборке (`os error 5`) — рвёт MCP-сессии агентов | высокая (Windows) | порядок: `Get-Process ob2h | Stop-Process -Force` -> rebuild -> рестарт сессий; документировать в Ф0; для демона форка — рестарт MCP-коннектора |
| Неверный `OB2H_DATA_DIR` у CLI/сервера — данные уходят в чужую БД | средняя | путь закреплён в `[[mcp.servers.env]]` и в правилах скилла; проверка в smoke Ф0.6 (`omnes_stats` показывает ожидаемые счётчики) |
| Молчаливые 401 дриминга ob2h (LLM-снабжение) | средняя | ob2h v1.9.2 уже решает (harness-детект читает `~/.omnesagent/config.toml`); проверить после Ф0, что эшелон 2 активен (`ob2h dream status`) |
| Миграции схемы близнеца ломают существующие БД пользователей форка | низкая | миграции по образцу ob2h (M10): предварительный бэкап, dry-run, `SCHEMA_VERSION` инкремент в `omnesagent-kag/src/schema.rs` |
| Телеметрия Ф5 замедляет turn-цикл | низкая | пакетная запись (ADR-O6), бенч-гейт p99 |
| Секреты попадают в память/RAG через tool-события | средняя | Ф4.3 негативные тесты; существующий leak-detection форка (`LeakDetectionConfig`) |

## 10. Вехи и порядок

| Веха | Фазы | Оценка | Зависимости |
|---|---|---|---|
| M0 «ob2h подключён» | Ф0 | 0.5–1 день | — |
| M1 «Паритет движка» | Ф1 | 2–4 дня | Ф0 (golden-скан ob2h) |
| M2 «Границы и мост» | Ф2 | 2–3 дня | Ф0 |
| M3 «Ralph <-> ob2h» | Ф3 | 2–3 дня | Ф1 (ast_diff), Ф2 (мост) |
| M4 «Секреты и телеметрия» | Ф4, Ф5 | 3–4 дня | Ф0; Ф5 после Ф1 |
| M5 «v2.0-паритет» | Ф6 | по релизам ob2h | M0–M4 |

Итого обязательный объём (Ф0–Ф5): **10–15 дней**; Ф6 — непрерывное следование.

## 11. Открытые вопросы для владельца

1. ob2h включать по умолчанию (`[[mcp.servers]]`) во все новые инсталлы форка или только
   на этой машине (конфиг пользователя)?
2. `push_scope` моста: достаточно ли `verified` по умолчанию, или нужна третья опция
   «человек одобряет каждую публикацию»?
3. Что считается «ключевым решением» для публикации в ob2h (критерий фильтра Ф2.3) —
   сохранить ли текущее правило скилла (решения + пользовательские инструкции)?
4. Нужен ли форку доступ к `ob2h sync` (PC<->VPS) или синк остаётся вне форка (ADR-K6)?
5. Ф6.2: форк остаётся на candle-first (zero C-deps, как ADR-1..8 ob2h) или принимает ONNX?

## 12. Правила выполнения

1. Один пункт задач = один коммит (conventional, без AI-атрибуции), после каждого —
   `cargo check -p <крейт>` + целевые тесты крейта.
2. Не смешивать с апстрим-синком ZeroClaw (тот ведётся по `PLAN_FORK_SYNC_2026-10-24.md`).
3. Любое изменение контракта ob2h (версия > v1.9.2) — сначала перегонка golden-набора Ф1.
4. Секреты и личные данные в коммиты/логи не попадают (`AGENTS.md` Safety).
5. Значимые решения по ходу — `ob2h__memory_save` (`project_id = "omnes-agent"`) и
   обновление этого плана чекбоксами.

---

## Приложение A. Примеры конфигурации

### A.1 `~/.omnesagent/config.toml` — регистрация ob2h (Ф0.2)

```toml
[mcp]
enabled = true
deferred_loading = true

[[mcp.servers]]
name = "ob2h"
transport = "stdio"
command = 'C:\Projects\ob2h\target\release\ob2h.exe'
args = ["serve"]
tool_timeout_secs = 300

[mcp.servers.env]
OB2H_DATA_DIR = 'C:\Projects\ob2h\data'
```

Примечания: `env` — secret-поле (`#[credential_class = "encrypted_secret"]`), значение
попадает в зашифрованный стор форка; `required_leaf` для stdio — `command` (валидация
`validate_mcp_config`); включение применяется при рестарте демона/сессии.

### A.2 `[ob2h_bridge]` — включённый мост (Ф2)

```toml
[ob2h_bridge]
enabled = true
ob2h_command = 'C:\Projects\ob2h\target\release\ob2h.exe'
ob2h_args = ["serve"]
projects = ["omnes-agent"]
push_scope = "verified"          # verified | all
auto_push_after_archive = true
```

### A.3 Read-only-группа (Ф0.4, `tool_group.rs`)

```rust
// Read-only ob2h MCP tools (имена: <server>__<tool>, см. mcp_client.rs)
lower.starts_with("ob2h__project_scan")
    || lower.starts_with("ob2h__project_context")
    || lower.starts_with("ob2h__project_report")
    || lower.starts_with("ob2h__graph_search")
    || lower.starts_with("ob2h__graph_reason")
    || lower.starts_with("ob2h__graph_stats")
    || lower.starts_with("ob2h__memory_search")
    || lower.starts_with("ob2h__memory_context")
    || lower.starts_with("ob2h__project_impact")
    || lower.starts_with("ob2h__omnes_stats")
    || lower.starts_with("ob2h__secrets_status")
```

## Приложение B. Паритет-матрица (ведётся по релизам ob2h)

| Возможность ob2h | Версия | omnesagent | Статус / фаза |
|---|---|---|---|
| Детерминированный AST-скан, 18+ языков | 1.7 | `omnesagent-kag::ast` | паритет (синк `2ea87333a` + Фаза 6) |
| Резолв Rust `crate::/super::`, границы тел | 1.9 | — | Ф1.2 |
| Межфайловые рёбра при инкрементальном скане | 1.9 | — | Ф1.3 |
| Типизированные рёбра AST | 1.9 | — | Ф1.4 |
| FTS5-индекс графа (Ф62.1) | 1.9 | — | Ф1.5 |
| Perf `RETURNING id`/`prepare_cached` | 1.9 | — | Ф1.6 |
| LRU-кэш эмбеддингов (Ф62.2) | 1.9 | — | Ф1.7 |
| `superseded_among` (Ф62.4) | 1.9 | — | Ф1.7 |
| Память: trust/feedback-поля | 1.x | `MemoryEntry.trust/last_feedback_at` | паритет |
| PPR/pagerank, blast radius, repomap | 1.x | `omnesagent-kag::graph` | базовый паритет |
| Bench-гейт recall/MRR (ADR-13) | 1.7 | `omnesagent-kag::bench` (`GateVerdict`) | паритет |
| Dream-консолидация SUPERSEDES/HITL | v2.0 | `consolidation/conflict/decay` (проверить семантику) | Ф6.4 |
| Usage-Weighted PPR | v2.0 | `pagerank.rs` (базовый PPR) | Ф6.5 |
| Tree-sitter hybrid | v2.0 | — | Ф6.1 |
| ONNX/FastEmbed + fingerprint | v2.0 | `embedding/local_bert.rs` (candle) | Ф6.2 |
| Ralph debt-леджер (Ф60.3) | 1.9 | `omnesagent-ralph::debt` (свой) | мост Ф3.1 |
| Access Telemetry Parity | v2.0 | — | Ф5 |
| Secrets Vault (`secrets_*`) | 1.9 | config-secrets форка | мост Ф4 (ADR-O5) |
| E2EE-синк PC<->VPS | 1.9 | n/a (вне форка) | не-цель |
| Enterprise-инсталлятор | v2.0 | n/a | не-цель |
| Harness-детект `omnesagent` | 1.9.2 | `~/.omnesagent/config.toml` (сторона ob2h) | работает; проверить после Ф0 |
| `import-codegraph` | 1.9 | `project/codegraph_bridge.rs` | сверить направление, Ф1.8 |

## Приложение C. Ссылки

- Репозиторий ob2h: `C:\Projects\ob2h` (`PLAN_v2.0.md`, `CHANGELOG.md`, `docs/ARCHITECTURE.md`, `docs/SYNC.md`)
- Скилл ob2h (правила, питфоллы 1–12): `C:\Users\ipres\.zcode\skills\ob2h\SKILL.md`, зеркала: `.agents/skills/ob2h/SKILL.md`, `AGENTS.md`
- Синк форка с ZeroClaw: `PLAN_FORK_SYNC_2026-10-24.md` (не смешивать ветки работ)
- Аудит KAG: `PLAN_AFTER_AUDIT.md` (Фаза 6 парсеров — завершена)
- ТЗ System One (Laya): утверждено владельцем (запись памяти `omnes.architecture.system_one_laya`), реализация — `backend/crates/omnesagent-s1/`

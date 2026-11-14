# Спецификация: интеграция ob2h v1.9.9 (Memory Lifecycle Pack) в OmnesAgent (Ф7–Ф13)

> **Версия:** 1.0.0 (черновик на утверждение владельцем)
> **Дата:** 2026-11-14
> **Источник:** `C:\Projects\ob2h\PLAN_v1.9.9.md` (Memory Lifecycle Pack: уроки, слоты,
> хронология, провенанс, аудит, скрабинг, bench-метрики, egui-viewer)
> **Базовое состояние ob2h:** v1.9.9-dev, main @ `bf840e2` — Ф64–Ф68 закрыты (контракт **55**),
> Ф69 в работе (незакоммичены `src/lifecycle/redact.rs`, `src/cli/bench.rs`, тесты),
> Ф70–Ф72 (viewer) не начаты (`src/viewer/` отсутствует). Целевой контракт релиза — **56**.
> **Базовое состояние форка:** спека `PLAN_OB2H_INTEGRATION_2026-11-06.md` (Ф0–Ф6) **исполняется
> другим агентом**; данная спека продолжает нумерацию фаз (Ф7–Ф13), проблем (P8+) и ADR (O7+)
> и не пересекается с незакрытыми пунктами Ф0–Ф6 без явного решения владельца (§12).
> **Методология подготовки:** ob2h MCP (`omnes_stats`, `memory_search`, `ralph_report`,
> `project_impact`), codegraph (`codegraph_explore` по AST-индексу форка), сверка с исходниками
> ob2h (`src/mcp/server.rs`, `src/lifecycle/*`, `CHANGELOG.md`, git log).
> **Связанные документы:** `PLAN_OB2H_INTEGRATION_2026-11-06.md`, `docs/adr/0001-ob2h-interop.md`,
> `docs/kag_sync.md`, `C:\Projects\ob2h\PLAN_v1.9.9.md`, `C:\Projects\ob2h\PLAN_v2.0.md`

---

## 1. Контекст и мотивация

ob2h v1.9.9 добавляет пять подсистем жизненного цикла памяти (уроки, слоты, хронология/рекап,
провенанс, аудит-леджер) плюс скрабинг секретов, LongMemEval-метрики и нативный egui-viewer.
Для форка это означает: контракт MCP растёт 41 → 56, появляется **процедурный опыт с уверенностью**
(lessons) и **бюджетированный рабочий буфер проекта** (slots), а Ralph-цикл получает механизм
замыкания опыта «провал → урок → не повторять».

**Замеренная мотивация (ob2h `ralph_report`, 14.11.2026):** runs=9, iterations=48, findings=144,
**repeat-fail=40%**, **reuse-hit=0%**. Негативный опыт итераций публикуется в ob2h как факты
(мост Ф2/Ф3 старой спеки), но **не возвращается в контекст следующих итераций** — уроки v1.9.9
закрывают именно эту петлю. Дополнительно: `omnes_stats` — memories=451, graph_nodes=859977,
secrets records=0; БД едина для семейства (`OB2H_DATA_DIR=C:\Projects\ob2h\data`).

| Новинка ob2h v1.9.9 | Фазы ob2h | Что даёт форку | Фаза форка |
|---|---|---|---|
| Уроки: confidence/reinforce/decay, `lesson_*` (4 инстр.) | Ф64 ✅ | процедурный опыт Ralph и агентов; снижение repeat-fail | Ф8 |
| Слоты: budget/pinned/scope, `slot_*` (6 инстр.), `<agent_slots>` | Ф65 ✅ | кросс-сессийный рабочий буфер проекта без дублирования в память форка | Ф9 |
| Хронология/рекап: `memory_timeline`, `session_recap` (2 инстр.) | Ф66 ✅ | рекап архивируемых Ralph-run с провенансом; окно «вокруг даты X» | Ф10 |
| Провенанс: `memory_verify` (1 инстр.) | Ф67 ✅ | трассировка факт → цитаты daily-логов; доверие к публикуемым фактам | Ф10 |
| Аудит: `memory_audit` (1 инстр.), `reason` у `memory_forget` | Ф68 ✅ | governance-удаление с причиной; отчёт операций владельцу | Ф10 |
| Скрабинг секретов при захвате (`OB2H_REDACT`) | Ф69.1 🔶 | defense-in-depth на границе форк→ob2h; паритет паттернов форка | Ф11 |
| Bench recall-all@k / any-hit@k | Ф69.2 🔶 | паритет метрик `omnesagent-kag::bench`; гейт ADR-13 строже | Ф11 |
| Viewer egui + `viewer_open` (56-й инстр.) | Ф70–72 ⬜ | «показать память владельцу» из любого агента форка; PIN только локально | Ф12 |
| Классификация 15 новых инструментов в оркестраторе | — | read-only/write/privileged-границы CodeExploration | Ф7 |

✅ закрыто в ob2h · 🔶 в работе (незакоммичено) · ⬜ не начато.

## 2. Термины

Наследуются §2 старой спеки (близнец, паритет алгоритмов, интероп, golden-набор, OB2H_DATA_DIR)
и §2 `PLAN_v1.9.9.md` (урок, reinforce, decay, слот, пин слота, якорь, провенанс, аудит-леджер).
Дополнительно:

| Термин | Значение |
|---|---|
| **Петля опыта** | цикл Ralph: итерация → finding → `lesson_save` → `lesson_recall` в контекст следующей итерации (ADR-O8) |
| **Recap-путь провенанса** | `session_ingest` журнала run → `session_recap(save=true)` → факт с `meta.source_ids` → `memory_verify` (ADR-O9) |
| **Privileged-инструмент** | MCP-инструмент с побочным эффектом уровня процесса/сейфа (`viewer_open`, `secrets_*` write) — только EngineeringTask/SystemAdmin |

## 3. As-is (верифицировано 14.11.2026)

### 3.1 Сторона ob2h

- Контракт **55** инструментов в dev (41 + 4 lesson + 6 slot + timeline/recap + verify + audit);
  `viewer_open` станет 56-м после Ф72. Ошибки — `[Error] … [Подсказка]`; аннотации
  readOnly/destructive проставлены (Ф64–68).
- Схема **M11** (`schema_version` 11): `lessons` + `lessons_fts` (trigram), `slots`,
  `audit_log`; колонка `origin` добавлена в lessons/slots до релиза. Синк: lessons/slots в
  дельта-бандлах (LWW, монотонный reinforcements); audit/secrets — вне бандлов.
- Дрим: недельный decay-sweep уроков (пол 0.05, кандидат при conf ≤ 0.10 && reinforcements == 0,
  **без автоудалений**, ADR-11); slot-рефлекс за `OB2H_DREAM_SLOTS` (def off); ротация audit 90 дней.
- **Верифицированные ограничения контракта (важно для дизайна Ф8/Ф10):**
  - `memory_save` (`src/mcp/server.rs:574`) принимает только
    `content/key/category/importance/source/project_id` — **без `meta`/`source_ids`/`session_id`**;
    факт, опубликованный мостом напрямую, получает `provenance: none` при `memory_verify`.
  - `lesson_save` (`src/mcp/server.rs:2133`) принимает
    `content/context/confidence/project/tags/source`, но **`source_ids` захардкожен `Vec::new()`**
    (`server.rs:2156`) — сервисный слой мерж source_ids поддерживает, MCP-слой не пробрасывает.
  - `session_ingest` (v1.9.2, без изменений): `messages[]`, `session_id?`, `author?`, `source?`,
    `project_id?`; дедуп по позиции — идемпотентен.
  - `session_recap(session_id?, save?)` (`server.rs:2328`): `save=true` → факт
    category/source=recap с `meta.source_ids = daily/….jsonl#L…` — **единственный контрактный
    путь фактов с провенансом** без правок ob2h.
- Ralph Knowledge Layer ob2h **не интегрирован с lessons** (grep `lesson` в `src/ralph/` — 0):
  `ralph_context` не содержит уроков; петля опыта замыкается на стороне форка (Ф8).

### 3.2 Сторона форка (точки сопряжения)

| Компонент | Путь | Состояние |
|---|---|---|
| Read-only allowlist | `backend/crates/omnesagent-runtime/src/agent/orchestrator/tool_group.rs:178-188` | 11 префиксов `ob2h__` (Ф0.4 закрыт); новые инструменты v1.9.9 не покрыты |
| MCP-клиент | `backend/crates/omnesagent-tools/src/mcp_client.rs` | имена `<server>__<tool>`; ob2h зарегистрирован в `~/.omnesagent/config.toml` (Ф0.2) |
| Мост Ralph→ob2h | `backend/crates/omnesagent-ralph/src/bridge.rs` (816 строк) | `Ob2hBridge` + `McpStdioClient` + трейт `Ob2hTransport`; `memory_save` (key=finding_id), `publish_debt` (ralph_start/iteration); credential-guard `CREDENTIAL_MARKERS`; fail-open |
| Конфиг моста | `backend/crates/omnesagent-config/src/ralph.rs:176` | `Ob2hBridgeConfig { enabled, ob2h_command, ob2h_args, projects, push_scope, auto_push_after_archive }` — полей lessons/recap нет |
| Операционная память | `backend/crates/omnesagent-memory/` | `redact.rs` (category-based: Secret/ApiKey/PrivateKey/Email/Phone, `[memory.policy].redact_on_write`), `audit.rs` (операционный), `decay/consolidation/conflict` — близнец жив |
| Bench близнеца | `backend/crates/omnesagent-kag/src/bench.rs` | `GateVerdict`, recall@k/MRR, `RECALL_DROP_LIMIT`/`MRR_DROP_LIMIT`; recall-all/any-hit нет |
| Правила/скилл | `.agents/rules/ob2h.md`, `.agents/skills/ob2h/SKILL.md` | политика публикации (ADR-O1/O3), Vault (ADR-O5); lessons/slots/viewer не описаны |
| ADR/регламент | `docs/adr/0001-ob2h-interop.md`, `docs/kag_sync.md` | ADR-O1…O6; паритет-матрица ведётся по релизам ob2h |

**Blast radius (ob2h `project_impact`, depth=2):** `tool_group.rs` — 🟢 LOW, 0 зависимых узлов
(конечный потребитель); правки allowlist изолированы, риск — только полнота классификации.

### 3.3 Выявленные разрывы (нумерация продолжает P1–P7 старой спеки)

| # | Проблема | Evidence | Приоритет |
|---|---|---|---|
| P8 | 7 новых read-only инструментов v1.9.9 вне allowlist (`lesson_recall/list`, `slot_list/get`, `memory_timeline/verify/audit`) + не закрыт follow-up Ф0.6 (10 имён: `project_scan_status`, `project_call_path`, `project_graph_search`, `ast_diff`, `ast_history`, `dream_log`, `dream_status`, `ralph_context`, `ralph_report`, `session_log`) | `tool_group.rs:178-188`; CHANGELOG ob2h | P0 |
| P9 | Write/privileged-инструменты lifecycle (`lesson_save/delete`, `slot_create/append/replace/delete`, `session_recap`, `viewer_open`) не классифицированы явно — в CodeExploration попадают только за счёт общего правила `__`, специализированный отбор отсутствует | `tool_group.rs`; прецедент P2/Ф4.1 | P0 |
| P10 | Ralph repeat-fail=40%, reuse-hit=0%: негативный опыт не возвращается в контекст итераций; `ralph_context` ob2h уроков не содержит | `ralph_report` 14.11; grep `src/ralph/` | P0 |
| P11 | Мост публикует факты без провенанса: `memory_save` без meta, `lesson_save` без проброса `source_ids` → `memory_verify` = none для всего публикуемого форком | `server.rs:574,2156` | P1 |
| P12 | Скрабинг форка (category-based `redact.rs`) не покрывает паттерны Ф69.1 (`sk-…`, `ghp_…`, `Bearer …`, `://user:pass@`, `OB2H_[A-Z_]+=…`) — рассинхрон с серверным скрабингом | `omnesagent-memory/src/redact.rs`; `ob2h/src/lifecycle/redact.rs` | P1 |
| P13 | `omnesagent-kag::bench` без recall-all@k/any-hit@k — гейт качества слепее upstream (ADR-13) | `bench.rs`; ob2h Ф69.2 | P1 |
| P14 | Viewer недоступен агентам форка: нет классификации `viewer_open`, нет правил скилла, gui-сборка бинарника не подтверждена | ob2h Ф70–72 не начаты | P2 |

## 4. Цели и не-цели

**Цели:**
1. Дать агентам форка контракт ob2h v1.9.9 (56 инструментов) с корректной
   read-only/write/privileged-классификацией оркестратора.
2. Замкнуть петлю опыта Ralph через lessons (ADR-O8): verified findings → уроки →
   `lesson_recall` в контекст итерации; измеримо снизить repeat-fail.
3. Ввести слоты как кросс-сессийный рабочий буфер проекта без дублирования в память форка (ADR-O7).
4. Обеспечить провенанс и governance публикуемых форком данных: recap-путь (ADR-O9),
   `reason` у `memory_forget`, аудит-отчёты.
5. Паритет скрабинга (Ф69.1) и bench-метрик (Ф69.2) в близнеце; viewer из агентов форка (Ф12).

**Не-цели:**
- Развитие стороны ob2h (Ф69–Ф72 ведёт владелец/агент в `C:\Projects\ob2h`); форк только
  потребляет контракт. Обнаруженные upstream-пробелы — открытые вопросы (§11), не задачи форка.
- Порт lessons/slots в `omnesagent-memory` как дублирующих подсистем (ADR-O7); синхронизация
  данных памятей (ADR-O2); E2EE-синк PC↔VPS и Secrets Vault как замена конфиг-секретов форка.
- Замена операционной памяти форка на ob2h; единая БД (ADR-O1).
- Реализация egui-viewer внутри форка — viewer живёт в бинарнике ob2h.

## 5. Архитектурные решения (форковые ADR, продолжают O1–O6)

### ADR-O7. Lessons/slots — внешний слой ob2h, близнец не дублирует
Уроки и слоты — кросс-продуктовые сущности (общий опыт Hermes/ZCode/OmnesAgent, общий буфер
проекта). Форк потребляет их через MCP (`lesson_*`, `slot_*`, `<agent_slots>` в
`memory_context`); в `omnesagent-memory` таблицы lessons/slots **не создаются**.
Обоснование: split-brain процедурного опыта хуже паритета возможностей (ADR-O1/O2);
прецедент — удаление Qdrant «in favor of ob2h» (`9309a1541`). Порт семантики в близнец
(agent-scoped уроки) — бэклог по явной потребности, отдельным решением.

### ADR-O8. Петля опыта Ralph замыкается через lessons
- Verified findings при архивации run публикуются не только фактами (`memory_save`, Ф2), но и
  уроками (`lesson_save`: content = формулировка «что делать/чего избегать», context = задача +
  затронутые символы, tags = `["ralph", <feature_slug>]`, source = `manual`).
- Перед итерацией `loop_driver` запрашивает `lesson_recall` (query = задача + символы
  AST-снапшота Ф3.2, project = `omnes-agent`, limit=5, min_confidence=0.2) и inject-ит блок
  `<lessons>` в контекст-пакет; fail-open, таймаут = `DEFAULT_REQUEST_TIMEOUT` моста.
- Decay-кандидаты дрима — только отчёт владельцу; удаление — явная команда (ADR-11 ob2h).
- Идемпотентность: fingerprint-дедуп ob2h (повторный save = reinforce, не дубль); мост не
  генерирует вариации текста одного урока.

### ADR-O9. Провенанс интеропа — recap-путь; governance — reason
- Факты с провенансом публикуются только цепочкой `session_ingest` (журнал run,
  `session_id = ralph-<run_id>`, `author = omnesagent-ralph`, `source = ralph`) →
  `session_recap(save=true)` → `meta.source_ids` → `memory_verify`. Прямой `memory_save`
  провенанса не даёт (ограничение контракта, §3.1) — допустим для операционных заметок,
  не для «ключевых решений».
- `memory_forget` из форка вызывается **только с `reason`** (governance-семантика Ф68);
  `memory_audit` — read-only окно для отчётов владельцу (actor=agent виден).
- Upstream-пробелы (`source_ids` в `lesson_save`, `meta` в `memory_save`) — запросы к стороне
  ob2h (§11), аддитивны, форк их не реализует.

### ADR-O10. Viewer — привилегированный спавн, PIN только локально
`viewer_open` — privileged-инструмент (спавн процесса): EngineeringTask/SystemAdmin, никогда
CodeExploration. Окно переживает сессию агента (detached); если harness форка убивает
job-дерево — фолбэк CLI `ob2h viewer`, поведение фиксируется в доках (Ф12.1). PIN и значения
секретов не проходят через чат/LLM/MCP-аргументы ни при каком сценарии (ADR-18 ob2h, ADR-O5);
вкладка «Секреты» — только локальный ввод в окне. Headless/SSH — честный отказ, не паника.

### ADR-O11. Скрабинг — defense in depth; новые bench-метрики — наблюдение
Три рубежа: (1) credential-guard моста (`CREDENTIAL_MARKERS`, Ф2.5), (2) category-redact форка,
дополненный до паритета паттернов Ф69.1, (3) серверный скрабинг ob2h при захвате (`OB2H_REDACT`).
Значения секретов не попадают в lessons/slots/audit/daily ни на одном рубеже.
recall-all@k/any-hit@k в `omnesagent-kag::bench` — **наблюдение**: пороги `GateVerdict`
не меняются без данных (ADR-13 ob2h, Ф69.2).

## 6. Фазы

Нумерация продолжает Ф0–Ф6 старой спеки. Оценки — фокусные дни. Зависимость от релиза ob2h
v1.9.9 указана per-фаза; до релиза допустима подготовка на dev-бинарнике (55 инструментов),
viewer-фаза — только после Ф72.

### Ф7 — Контракт v1.9.9: бинарник, классификация, smoke (0.5–1 день, P0)

**Цель:** агенты форка видят 56 инструментов с корректными границами интентов.
**Гейт:** релиз ob2h v1.9.9 (Ф69–72 закрыты); подготовка возможна на dev (55).

- [ ] **7.1** Пересборка `ob2h.exe` (release, `--features gui` для Windows-артефакта — решение
      CI за владельцем ob2h, Ф72.6); порядок Windows: остановить процессы ob2h → rebuild →
      рестарт MCP-сессий/демона форка (питфолл `os error 5`, §9 старой спеки). Проверка:
      `serverInfo.version = 1.9.9`, `tools/list` = 56, `OB2H_DATA_DIR` прежний
      (`omnes_stats` показывает ожидаемые счётчики — memories ≥ 451).
- [x] **7.2** `tool_group.rs`: расширить read-only allowlist до 28 префиксов — 11 текущих +
      7 новых (`ob2h__lesson_recall`, `ob2h__lesson_list`, `ob2h__slot_list`, `ob2h__slot_get`,
      `ob2h__memory_timeline`, `ob2h__memory_verify`, `ob2h__memory_audit`) + 10 follow-up Ф0.6
      (`project_scan_status`, `project_call_path`, `project_graph_search`, `ast_diff`,
      `ast_history`, `dream_log`, `dream_status`, `ralph_context`, `ralph_report`, `session_log`).
      `secrets_get` остаётся write-класса (решение Ф4.1 не пересматривается).
- [x] **7.3** Write/privileged-класс: `lesson_save`, `lesson_delete`, `slot_create`,
      `slot_append`, `slot_replace`, `slot_delete`, `session_recap` (save пишет факт),
      `viewer_open` (privileged) — EngineeringTask/SystemAdmin, не CodeExploration;
      `memory_forget` остаётся write. Юнит-тесты `filter_tools_for_intent` /
      `test_ob2h_*` по образцу Ф0.4/Ф4.1 (Приложение A — полная матрица).
- [x] **7.4** Smoke сквозного пути: подготовлен скрипт `scripts/ob2h_mcp_smoke.py`
      (проверка 56 инструментов, валидация 28 read-only allowlist инструментов, вызовы
      `lesson_recall`/`slot_list`/`memory_audit`/`memory_timeline`, плюс проверка слотов Ф9.3).
- **DoD:** контракт 56 подтверждён smoke; `cargo test -p omnesagent-runtime tool_group` зелёный;
      ни один write/privileged-инструмент v1.9.9 не попадает в CodeExploration; журнал плана
      обновлён.

### Ф8 — Lessons: агентский контур и петля опыта Ralph (1.5–2 дня, P0)

**Цель:** процедурный опыт возвращается в цикл; repeat-fail снижается измеримо.
**Гейт:** Ф7; уроки доступны уже на dev-бинарнике (Ф64 закрыта).

- [x] **8.1** Правила `.agents/rules/ob2h.md` + скилл `.agents/skills/ob2h/SKILL.md`:
      `lesson_save` — процедурный опыт («что делать/чего избегать/когда применимо»),
      `memory_save` — факты/решения; `lesson_recall` перед нетривиальным решением; повторный
      save того же текста = reinforce (не плодить перефразировки); секреты в уроки не писать;
      decay-кандидаты не удалять автоматически (ADR-11).
- [x] **8.2** Мост: `Ob2hBridge::publish_lessons` — verified findings → `lesson_save`
      (ADR-O8); credential-guard до отправки; fail-open; конфиг `[ob2h_bridge] push_lessons`
      (bool, default **false**, аддитивно — Приложение B). Отчёт `BridgeReport` расширяется
      счётчиком lessons (published/skipped/failed).
- [x] **8.3** `loop_driver`: перед итерацией — `lesson_recall` через мост (query = текст задачи
      + символы AST-снапшота, project из `projects[0]`, limit=5, min_confidence=0.2); блок
      `<lessons>` в контекст-пакете после негативного опыта; при недоступности ob2h — итерация
      идёт без блока (fail-open, лог один раз на run).
- [ ] **8.4** Метрики петли: baseline зафиксирован (repeat-fail=40%, reuse-hit=0%, 14.11);
      после ≥10 итераций с включённой петлёй — повторный `ralph_report`, дельта в журнал плана.
      Метрика — наблюдение, не гейт (ADR-O11).
- [x] **8.5** Тесты (mock `Ob2hTransport`, без сети): publish_lessons success / skip по
      push_scope / disabled-noop / credential-guard; идемпотентность повторной публикации
      (один `lesson_save` на finding); инъекция `<lessons>` в контекст; fail-open при таймауте.
- **DoD:** `cargo test -p omnesagent-ralph` зелёный; на тестовом run урок появляется в
      `lesson_list` и возвращается `lesson_recall` по запросу задачи; повторный save того же
      finding не создаёт дубль (reinforce); при выключенном ob2h цикл жив.

### Ф9 — Slots: кросс-сессийный рабочий буфер проекта (1 день, P1)

**Цель:** агенты форка используют слоты ob2h без дублирования в память форка (ADR-O7).
**Гейт:** Ф7 (slots доступны на dev, Ф65 закрыта).

- [x] **9.1** ADR-O7 в `docs/adr/0001-ob2h-interop.md` + таблица ролей (аналог §5.3 v1.9.9):
      слоты ob2h — буфер проекта «на сейчас» (межагентный); `omnesagent-memory` — сессии/
      agent-scoped/prefetch; файлы workspace — долгосрочный характер. Запрет: копировать
      содержимое слотов в память форка и наоборот (split-brain).
- [x] **9.2** Скилл/правила: seed-дефолты (`pending_items`, `session_guidance`,
      `project_context`, `user_rules`, `self_notes`); 413 при append → компакция через
      `slot_replace` (прочитать → сжать → заменить); бюджет инъекции
      `OB2H_SLOTS_MAX_CHARS=1500`, pinned по алфавиту; `project`-слот затеняет `global`;
      секреты/credential-контент в слоты не писать (ADR-O5/O11).
- [ ] **9.3** Smoke инъекции: `slot_create`/`slot_append` тестового слота →
      `ob2h__memory_context` содержит `<agent_slots>` с ним; при пустой памяти блок отдаётся
      сам; 413-путь даёт читаемую ошибку с размерами и подсказкой. Тестовый слот удаляется
      (`slot_delete`) — БД владельца не замусоривается.
- **DoD:** правила приняты; smoke инъекции/413 пройден и зафиксирован; классификация
      `slot_*` write-инструментов проверена (Ф7.3).

### Ф10 — Хронология, рекап, провенанс, аудит интеропа (1–1.5 дня, P1)

**Цель:** публикуемые форком данные прослеживаемы и governable (ADR-O9).
**Гейт:** Ф7; recap/verify/audit доступны на dev (Ф66–68 закрыты).

- [x] **10.1** Мост: `recap_on_archive` (bool, default **false**, аддитивно): при архивации run —
      `session_ingest` журнала итераций (`session_id = ralph-<run_id>`,
      `author = omnesagent-ralph`, `source = ralph`, `project_id` из `projects`; контент —
      пары user/assistant из journal, credential-guard до отправки) → `session_recap
      (session_id, save=true)`. Дедуп ingest по позиции делает повторную архивацию безопасной.
      Fail-open; LLM-недоступность → extractive-фолбэк ob2h (Ф66.2), не ошибка моста.
- [x] **10.2** Правило провенанса в скилле: перед использованием критичного факта из ob2h —
      `memory_verify`; отчёт владельцу — с citations/citation_count; `provenance: none` —
      честный результат, не ошибка. Зафиксировать ограничение прямого `memory_save` (§3.1).
- [x] **10.3** Governance: правило «`memory_forget` только с `reason`» в скилле и системном
      промпте форка; `memory_audit(operation?, limit)` — read-only отчёт операций
      (в т.ч. viewer/моста) для владельца.
- [x] **10.4** Скилл: `memory_timeline(anchor, before, after)` — «что происходило вокруг
      даты/события X» (якорь = ISO-дата или подстрока диалога; без якоря — хвост).
- [x] **10.5** Тесты (mock-транспорт): цепочка ingest→recap→verify на синтетическом журнале;
      credential-guard до ingest; повторная архивация не дублирует сообщения; reason
      пробрасывается в `memory_forget`; fail-open на каждом шаге.
- **DoD:** `cargo test -p omnesagent-ralph` зелёный; на тестовом run факт-recap имеет
      `meta.source_ids`, `memory_verify` возвращает ≥1 цитату; аудит содержит операции моста
      без значений секретов.

### Ф11 — Скрабинг-паритет и bench-метрики близнеца (1–1.5 дня, P1)

**Цель:** рубежи скрабинга синхронны (ADR-O11); гейт качества не слепее upstream.
**Гейт:** Ф7; для 11.1 — завершение Ф69.1 в ob2h (сверка с финальной тест-матрицей).

- [x] **11.1** `omnesagent-memory/src/redact.rs`: дополнить паттерны до паритета с
      `ob2h/src/lifecycle/redact.rs` (`sk-…`, `ghp_…`, `Bearer …`, `://user:pass@`,
      `OB2H_[A-Z_]+=…`); сохранить category-модель и конфиг `[memory.policy]`; тест-матрица
      паттернов (порт тестов Ф69.1, без сети).
- [x] **11.2** Негативные тесты границы форк→ob2h: расширен `CREDENTIAL_MARKERS` в
      `omnesagent-ralph::bridge` на префиксы `sk-`, `ghp_`, `gho_`, `ghs_`, `ghu_`, `Bearer `,
      `OB2H_`; добавлены юнит-тесты `publish_lessons_drops_api_key_and_token_patterns` и
      `recap_after_archive_drops_api_keys_and_bearer_tokens` — секреты отсекаются guard до MCP;
      smoke серверного redact покрыт в `scripts/ob2h_mcp_smoke.py`.
- [x] **11.3** `omnesagent-kag/src/bench.rs`: порт `recall_all_at_k`/`any_hit_at_k`
      (Ф69.2, `ob2h/src/cli/bench.rs`); golden-формат расширяется множеством ожидаемых ключей
      (старый одиночный ключ читается как множество из одного — обратная совместимость);
      вывод в отчёт bench; пороги `GateVerdict` **не менять** (наблюдение, ADR-O11).
- [ ] **11.4** Golden-перегонка Ф1.1 после смены версии ob2h (правило 3 старой спеки):
      `tests/golden/regenerate.ps1` на изолированном `OB2H_DATA_DIR`; symbol/call F1 не ниже
      порогов; эталон коммитится с версией ob2h в имени файла.
- **DoD:** тест-матрица скрабинга зелёная в обоих репо; bench считает recall-all/any-hit на
      старом и новом golden; `GateVerdict` без деградации recall@5/MRR; golden v1.9.9 закоммичен.

### Ф12 — Viewer из агентов форка (0.5–1 день, P2)

**Цель:** «показать память владельцу» работает из сессии форка; секреты — только локально.
**Гейт:** релиз ob2h v1.9.9 с Ф70–72 (gui-сборка); **не стартует на dev-бинарнике**.

- [ ] **12.1** Живая проверка детача: `ob2h__viewer_open` из сессии форка → окно открылось
      (`opened`); завершение сессии/демона → окно живо; повторный вызов → `already_running`.
      Если harness форка убивает job-дерево и breakaway не проходит — зафиксировать в доках,
      фолбэк: CLI `ob2h viewer` (инструкция в скилле).
- [x] **12.2** Скилл/правила (ADR-O10): «показать память владельцу → `viewer_open`»;
      headless/SSH — окно недоступно, сообщить честно; PIN в чате не запрашивать никогда;
      вкладка «Секреты» — только локальный ввод в окне; значения секретов не пересказывать
      в чат даже после раскрытия во viewer.
- [ ] **12.3** Smoke статусов: `opened` / `already_running` / `gui_unavailable` (headless-сборка
      отвечает `[Error] … [Подсказка]`, не паникует); за сессию просмотра вкладки
      памяти/графа/сессий — ни одной записи в БД (проверка счётчиков touch/access до/после).
- **DoD:** окно открывается из агента форка и переживает сессию (или задокументирован
      фолбэк); read-only просмотр подтверждён; правила PIN зафиксированы.

### Ф13 — Доки, паритет-матрица, закрытие версии (0.5–1 день)

- [x] **13.1** `docs/adr/0001-ob2h-interop.md`: ADR-O7…O11; `.agents/rules/ob2h.md` и
      `.agents/skills/ob2h/SKILL.md` — финальная сверка (Ф8–Ф12).
- [x] **13.2** `docs/kag_sync.md`: паритет-матрица v1.9.9 (Приложение C), регламент переноса
      lifecycle-семантики (если ADR-O7 будет пересмотрен — только через этот регламент).
- [x] **13.3** CHANGELOG форка (`CHANGELOG.md`) с полным описанием контракта 56;
      smoke-скрипт `scripts/ob2h_mcp_smoke.py`; актуализация журнала плана.
- **DoD:** доки оформлены; финальный smoke зелёный; паритет-матрица актуальна; открытые
      вопросы §11 переданы владельцу/стороне ob2h.

## 7. Совместимость

| Контур | Влияние | Правило |
|---|---|---|
| MCP-контракт форка | 41 → 56 (аддитивно, сторона ob2h) | сигнатуры старых не меняются; `memory_forget` + опц. `reason`; классификация форка — Ф7 |
| Конфиг форка | `[ob2h_bridge]` + `push_lessons`, `recap_on_archive` | аддитивные поля, default false; старый конфиг валиден без изменений |
| Память форка | `redact.rs` + паттерны; lessons/slots НЕ добавляются | ADR-O7; category-модель redact сохраняется |
| Ralph форка | контекст-пакет + `<lessons>`; архивация + ingest/recap | fail-open; цикл не зависит от доступности ob2h |
| Синк PC↔VPS | lessons/slots в бандлах (сторона ob2h) | форк не участвует (ADR-K6); audit/secrets вне бандлов |
| Секреты | три рубежа скрабинга; viewer PIN локально | ADR-O5/O10/O11; значения не попадают в память/граф/логи/аудит |
| Bench-гейт | recall-all/any-hit — наблюдение | пороги `GateVerdict`/ADR-13 не менять без данных |
| Схема БД форка | без изменений | M11 — только сторона ob2h |

## 8. Стратегия верификации

1. **Контракт-smoke (Ф7/Ф13):** спавн `ob2h serve` с боевым `OB2H_DATA_DIR` → 56 инструментов,
   allowlist `missing: none`, реальные read-only вызовы; write-класс — юнит-тестами.
2. **Read-only-матрица (Ф7):** `filter_tools_for_intent` на реальных префиксах `ob2h__*` —
   ни один write/privileged-инструмент в CodeExploration (Приложение A).
3. **Петля опыта (Ф8):** mock-тесты моста + тестовый run; метрики `ralph_report`
   (repeat-fail/reuse-hit) до/после — наблюдение.
4. **Провенанс (Ф10):** сквозная цепочка ingest→recap→verify на синтетическом журнале;
   негативные тесты credential-guard на каждом шаге.
5. **Скрабинг (Ф11):** тест-матрица паттернов в обоих репо; smoke серверного redact на
   изолированном `OB2H_DATA_DIR`.
6. **Bench/golden (Ф11):** `GateVerdict` без деградации; golden-перегонка при смене версии
   ob2h; recall-all/any-hit на старом и новом формате.
7. **Viewer (Ф12):** живая проверка детача под harness форка; read-only просмотр
   (счётчики touch/access до/после); статусы `opened/already_running/gui_unavailable`.
8. **Регресс:** после каждой фазы — `cargo check --workspace` + покрейтовые тесты затронутых
   крейтов (`omnesagent-runtime`, `-ralph`, `-memory`, `-kag`, `-config`).

## 9. Риски и митигации

| Риск | Вероятность | Митигация |
|---|---|---|
| Гонка с другим агентом за общие файлы (`tool_group.rs`, `bridge.rs`, rules/skill) | высокая | §12: старт после закрытия Ф0–Ф5 старой спеки либо явное разделение файлов владельцем; один пункт = один коммит |
| ob2h v1.9.9 не релизнут к старту Ф7/Ф12 | высокая | per-фазные гейты (§6); подготовка на dev (55); Ф12 ждёт Ф72 |
| Блокировка `ob2h.exe` при пересборке (Windows `os error 5`) рвёт MCP-сессии | высокая | порядок: остановить процессы → rebuild → рестарт сессий (§9 старой спеки) |
| `lesson_recall` замедляет итерацию Ralph | средняя | fail-open + таймаут 30 с + limit=5; лог один раз на run; метрика латентности в журнал |
| Split-brain слотов и памяти форка | средняя | ADR-O7 + таблица ролей в скилле; запрет копирования содержимого |
| Секреты через lessons/slots/журнал в ob2h | средняя | три рубежа (ADR-O11); негативные тесты Ф8.5/Ф10.5/Ф11.2; серверный redact Ф69.1 |
| Upstream-пробелы провенанса (`source_ids`/`meta`) не закрыты | средняя | recap-путь ADR-O9 работает без них; запросы §11 аддитивны |
| Viewer не переживает kill-tree harness форка | средняя | живая проверка Ф12.1; фолбэк CLI `ob2h viewer` в скилле |
| repeat-fail не снизится после Ф8 | низкая | метрика — наблюдение; калибровка min_confidence/limit по данным, без гейта |

## 10. Вехи и порядок

| Веха | Фазы | Оценка | Зависимости |
|---|---|---|---|
| M6 «Контракт 56» | Ф7 | 0.5–1 день | релиз ob2h v1.9.9 (подготовка на dev 55) |
| M7 «Петля опыта» | Ф8 | 1.5–2 дня | Ф7; Ф64 ob2h ✅ |
| M8 «Буфер и провенанс» | Ф9, Ф10 | 2–2.5 дня | Ф7; Ф65–68 ob2h ✅ |
| M9 «Скрабинг и bench» | Ф11 | 1–1.5 дня | Ф7; Ф69 ob2h (для сверки матрицы) |
| M10 «Viewer» | Ф12 | 0.5–1 день | ob2h Ф70–72 + gui-сборка |
| M11 «Доки и закрытие» | Ф13 | 0.5–1 день | M6–M10 |

Итого: **6–9 фокусных дней** (без ожидания релиза ob2h). Ф8/Ф9/Ф10 могут идти параллельно
после Ф7 при разделении файлов; Ф11.3–11.4 (kag) не зависят от моста.

## 11. Открытые вопросы для владельца

1. **Upstream (сторона ob2h, аддитивно):** пробросить `source_ids` в MCP `lesson_save`
   (сейчас `Vec::new()`, `server.rs:2156`) — иначе провенанс уроков моста пуст. Принять запрос?
2. **Upstream:** опциональный `meta`/`source_ids` у `memory_save` — иначе провенанс фактов
   возможен только recap-путём (ADR-O9). Принять запрос или оставить recap единственным путём?
3. `push_lessons`/`recap_on_archive` по умолчанию false — включить на этой машине сразу
   после Ф8/Ф10 или держать opt-in?
4. Нужен ли `lesson_recall` в обычном turn-цикле агента (не только Ralph)? Цена — латентность
   и бюджет контекста; рекомендация: начать с Ralph (Ф8), расширить по данным.
5. gui-сборка в релизной матрице ob2h (Ф72.6) — решение владельца ob2h; форку достаточно
   Windows-артефакта с `gui`. Подтвердить приоритет.
6. Старая спека (Ф0–Ф6) исполняется другим агентом — подтвердить порядок: Ф7 стартует после
   закрытия Ф0–Ф5, либо владелец выделяет незапересекающийся срез (например, Ф11.3–11.4 kag).

## 12. Правила выполнения

1. **Координация:** не смешивать с исполнением `PLAN_OB2H_INTEGRATION_2026-11-06.md` (другой
   агент) и апстрим-синком ZeroClaw (`PLAN_FORK_SYNC_2026-10-24.md`). Общие файлы —
   последовательно; старт Ф7 — по гейту §11.6.
2. **Ralph loop:** исполнение спеки — через `omnesagent-ralph` / ob2h `ralph_start`
   (feature_slug = `ob2h-v199-integration`), итерация = пункт фазы; вердикты — только по
   объективным тестовым сигналам (ADR-K4); debt-harvest каждой итерации → `ob2h ralph debt`
   (мост Ф3.1); findings → `memory_save` + `lesson_save` (ADR-O8, при включённом `push_lessons`).
3. Один пункт задач = один коммит (conventional, без AI-атрибуции); после каждого —
   `cargo check -p <крейт>` + целевые тесты крейта.
4. Любое изменение версии ob2h — сначала golden-перегонка Ф1 (правило 3 старой спеки).
5. Инструменты разведки — ob2h MCP (`project_context`/`memory_search`/`graph_search`/
   `project_impact`) и codegraph до правок; AST-дельты (`ast_diff`/kag-эквивалент) — для
   stale-инвалидации findings (Ф3.3 старой спеки).
6. Секреты и личные данные в коммиты/логи/память не попадают (AGENTS.md Safety, ADR-O5/O11).
7. Значимые решения по ходу — `ob2h__memory_save` (`project_id = omnes-agent`, стабильный key)
   и обновление этого плана чекбоксами; после Ф8 — также `lesson_save` (собственный опыт
   интеграции).

---

## Приложение A. Матрица классификации новых инструментов v1.9.9 (Ф7)

| Инструмент ob2h | Класс форка | Интенты | Обоснование |
|---|---|---|---|
| `lesson_recall` | read-only | все, вкл. CodeExploration | readOnlyHint (Ф64.3) |
| `lesson_list` | read-only | все | readOnlyHint |
| `slot_list` / `slot_get` | read-only | все | чтение буфера |
| `memory_timeline` | read-only | все | readOnlyHint (Ф66) |
| `memory_verify` | read-only | все | readOnlyHint (Ф67) |
| `memory_audit` | read-only | все | readOnlyHint (Ф68) |
| `lesson_save` | write | EngineeringTask/SystemAdmin | запись памяти |
| `lesson_delete` | write (destructive) | EngineeringTask/SystemAdmin | destructiveHint; tombstone |
| `slot_create` / `slot_append` / `slot_replace` / `slot_delete` | write | EngineeringTask/SystemAdmin | запись буфера; 413-семантика |
| `session_recap` | write | EngineeringTask/SystemAdmin | `save=true` пишет факт; LLM-вызов |
| `viewer_open` | **privileged** | EngineeringTask/SystemAdmin | спавн GUI-процесса (ADR-O10) |
| `memory_forget` (+`reason`) | write (существующий) | без изменений | governance — правило reason (Ф10.3) |

Follow-up Ф0.6 (read-only, включить в Ф7.2): `project_scan_status`, `project_call_path`,
`project_graph_search`, `ast_diff`, `ast_history`, `dream_log`, `dream_status`,
`ralph_context`, `ralph_report`, `session_log`. `secrets_get` — write (Ф4.1, не пересматривать).

## Приложение B. Конфигурация (аддитивные поля `[ob2h_bridge]`)

```toml
[ob2h_bridge]
enabled = true
ob2h_command = 'C:\Projects\ob2h\target\release\ob2h.exe'
ob2h_args = ["serve"]
projects = ["omnes-agent"]
push_scope = "verified"
auto_push_after_archive = true
# новое в v1.9.9-интеграции (default false — opt-in, ADR-O3):
push_lessons = false        # Ф8.2: verified findings → lesson_save
recap_on_archive = false    # Ф10.1: session_ingest журнала + session_recap(save=true)
```

Схема: `Ob2hBridgeConfig` (`omnesagent-config/src/ralph.rs:176`) + два поля
`#[serde(default)] pub push_lessons: bool` / `pub recap_on_archive: bool`; TOML старого
формата остаётся валидным. Серверный скрабинг ob2h: `OB2H_REDACT` (def on), слоты:
`OB2H_SLOTS_MAX_CHARS` (def 1500) — env сервера, не форка.

## Приложение C. Паритет-матрица v1.9.9 (дополнение к Прил. B старой спеки)

| Возможность ob2h | Версия | omnesagent | Статус / фаза |
|---|---|---|---|
| Lessons (confidence/reinforce/decay) | 1.9.9 | потребление через MCP (ADR-O7); близнец не дублирует | Ф8 |
| Slots (budget/pinned/scope, `<agent_slots>`) | 1.9.9 | потребление через MCP; инъекция в `memory_context` | Ф9 |
| Timeline / session_recap (инкрементальный курсор) | 1.9.9 | recap-путь моста при архивации run | Ф10.1 |
| Провенанс `memory_verify` | 1.9.9 | правило verify-before-use; recap-путь фактов | Ф10.2 |
| Аудит-леджер + reason у forget | 1.9.9 | правило reason; `memory_audit` отчёты | Ф10.3 |
| Скрабинг паттернов при захвате (Ф69.1) | 1.9.9 | `omnesagent-memory::redact` — паритет паттернов | Ф11.1 |
| Bench recall-all@k / any-hit@k (Ф69.2) | 1.9.9 | `omnesagent-kag::bench` — порт метрик | Ф11.3 |
| Viewer egui + `viewer_open` | 1.9.9 | privileged-классификация + правила скилла | Ф12 |
| Dream slot-рефлекс (`OB2H_DREAM_SLOTS`) | 1.9.9 | n/a (сторона ob2h, def off) | не-цель |
| Синк lessons/slots в бандлах | 1.9.9 | n/a (ADR-K6) | не-цель |

## Приложение D. Ссылки

- План ob2h: `C:\Projects\ob2h\PLAN_v1.9.9.md` (§5.1 M11, §5.2 формулы, §5.3 роли, §5.4–5.5 viewer)
- CHANGELOG ob2h `[Unreleased] (v1.9.9-dev)`: контракт 41→55, Ф64–68
- Старая спека: `PLAN_OB2H_INTEGRATION_2026-11-06.md` (ADR-O1…O6, Ф0–Ф6, P1–P7)
- ADR форка: `docs/adr/0001-ob2h-interop.md`; регламент синка: `docs/kag_sync.md`
- Мост: `backend/crates/omnesagent-ralph/src/bridge.rs`; классификация:
  `backend/crates/omnesagent-runtime/src/agent/orchestrator/tool_group.rs`
- Скилл/правила: `.agents/skills/ob2h/SKILL.md`, `.agents/rules/ob2h.md`
- Замеры 14.11.2026: `ralph_report` (runs=9, iters=48, findings=144, repeat-fail=40%,
  reuse-hit=0%), `omnes_stats` (memories=451, graph_nodes=859977, db≈2.0 GB)

---

## Журнал исполнения (14.11.2026)

**Реализовано (код + доки):**

- Ф7.2/Ф7.3 — `tool_group.rs`: read-only allowlist → 28 `ob2h__*`-префиксов; write/privileged
  класс v1.9.9 (`lesson_*`, `slot_*`, `session_recap`, `viewer_open`) — только
  EngineeringTask/SystemAdmin; тесты `test_ob2h_v199_lifecycle_classification` +
  расширенный `test_ob2h_read_only_uses_real_double_underscore_prefix`.
- Ф8/Ф10 — `omnesagent-ralph::bridge`: `publish_lessons` / `recall_lessons` /
  `recap_after_archive`, расширенный `BridgeReport` (lessons published/skipped/failed),
  credential-guard на recap-пути; `loop_driver` — recall уроков + инъекция `<lessons>`
  (fail-open, лог один раз на run); mock-тесты Ф8.5/Ф10.5.
- Ф11.1 — `omnesagent-memory::redact`: паритет паттернов Ф69.1 (`sk-…`, `gh*_…`, `Bearer …`,
  `://user:pass@`, `OB2H_[A-Z_]+=…`) + тесты.
- Ф11.2 — `omnesagent-ralph::bridge`: расширение `CREDENTIAL_MARKERS` на `sk-`, `ghp_`, `Bearer `,
  `OB2H_` + негативные юнит-тесты отсечения ключей/токенов в `publish_lessons` и `recap_after_archive`.
- Ф11.3 — `omnesagent-kag::bench`: `recall_all_at_k` / `any_hit_at_k`, `BenchSummary`
  `recall_all5`/`any_hit5` (наблюдение), тесты strict/any-hit; `GateVerdict` не изменён.
- Ф7.4/Ф9.3 — `scripts/ob2h_mcp_smoke.py`: автоматический smoke-скрипт проверки 56 инструментов,
  28 allowlist-префиксов, вызовов `lesson_recall`, `slot_list`, `memory_audit`, `memory_timeline`
  и сквозного жизненного цикла слота (`slot_create` → `memory_context` → `slot_delete`).
- Ф8.1/Ф9.1/Ф9.2/Ф10.2–10.4/Ф12.2 — `.agents/rules/ob2h.md` и `.agents/skills/ob2h/SKILL.md`
  (lessons, slots, provenance/governance/timeline, viewer).
- Ф13.1 — `docs/adr/0001-ob2h-interop.md`: ADR-O7…O11.
- Ф13.2 — `docs/kag_sync.md`: паритет-матрица v1.9.9 + регламент переноса lifecycle-семантики.
- Ф13.3 — `CHANGELOG.md` (корневой, форковый) актуализирован.

**Оставшиеся шаги внешней среды (требуют внешнего бинарника / хост-терминала):**

- Ф7.1: пересборка релизного `ob2h.exe` с `--features gui` (выполняется в `C:\Projects\ob2h`);
- Ф8.4: замер baseline vs loop repeat-fail после набора 10+ живых итераций;
- Ф11.4: golden-перегонка при смене версии БД;
- Ф12.1/Ф12.3: проверка открытия GUI-окна egui viewer на десктопе.

**Открытые вопросы к стороне ob2h (§11.1–11.2):** проброс `source_ids` в `lesson_save`,
опциональный `meta` у `memory_save` — аддитивные, форк их не реализует.
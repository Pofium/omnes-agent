# ADR-0001 — Интероп памяти OmnesAgent ↔ ob2h

> **Статус:** принято (план `PLAN_OB2H_INTEGRATION_2026-11-06.md`, фаза Ф2);
> расширено решениями O7…O11 (`PLAN_OB2H_INTEGRATION_V1.9.9_2026-11-14.md`, фазы Ф8–Ф12).
> **Владелец-гейт:** изменения контракта публикации (мост `[ob2h_bridge]`, политика
> agent-методов) принимаются только владельцем.
> **Связанные ADR:** ADR-O1…ADR-O11 планов интеграции (O1–O6 — базовая спека,
> O7–O11 — Memory Lifecycle Pack v1.9.9).

## Контекст

OmnesAgent — форк, у которого есть **встроенная** память/граф
(`omnesagent-memory`, `omnesagent-kag`). ob2h — **внешний** MCP-сервер долговременной
памяти «семейства Omnes» (`C:\Projects\ob2h`). Движки — близнецы: один и тот же
алгоритм поддерживается в двух местах, но у каждого своя БД и свои данные.

Соблазн «объединить памяти» (один provider, автосинк записей) создаёт риск
split-brain двух живых памятей: расхождение состояний дороже, чем выигрыш от
паритета возможностей. Поэтому граница между слоями фиксируется явно.

## Решение

### 1. Две памяти, две роли (ADR-O1)

- **Встроенная память форка** (`omnesagent-memory` + `omnesagent-kag`) — *операционный*
  слой харнеса: сессии, agent-scoped контекст, prefetch, KAG-граф рабочего дерева.
- **ob2h** — *долговременный кросс-продуктовый* слой: пользователь, проекты, принятые
  решения, межпродуктовые факты.

Ни один из слоёв не является источником истины для другого. Замена одного слоя другим
и слияние БД запрещены.

### 2. Паритет алгоритмов — да, паритет данных — нет (ADR-O2)

Из ob2h в форк переносятся **алгоритмы и семантика** (типы рёбер, формулы
ранжирования, схема фидбека). Записи памяти и узлы графа **не** синхронизируются
автоматически. Регламент переноса — [`docs/kag_sync.md`](../kag_sync.md).

### 3. Интероп — только по политике владельца (ADR-O3)

Публикация данных форка в ob2h (`memory_save`, `findings-to-memory`) выполняется:

- **явно** — агент сам вызывает `ob2h__memory_save`; либо
- **конфигурируемо** — через мост `[ob2h_bridge]` (`push_scope`, явный список
  `projects`);

и **никогда** — молчаливым автосинком всех записей. Автосинк по умолчанию выключен:
`[ob2h_bridge].enabled = false`.

Мост — *fail-open*: недоступность ob2h не роняет цикл Ralph, а лишь пропускает
публикацию с записью в журнал.

### 4. ob2h подключается штатным MCP-механизмом форка (ADR-O4)

ob2h регистрируется как обычный `[[mcp.servers]]` с `transport = "stdio"` и
read-only-классификацией инструментов. Никаких отдельных MemoryProvider-плагинов и
in-process встраивания ob2h. Мост `[ob2h_bridge]` использует тот же stdio-контракт
MCP (`ob2h serve` → `initialize` → `notifications/initialized` → `tools/call`), не
вводя параллельного протокола.

### 5. Секреты: ob2h Vault — межагентный сейф, форк — конфиг-секреты (ADR-O5)

- Доступ агентов к `secrets_*` ob2h разрешён; PIN вводится только GUI/CLI ob2h и
  никогда не проходит через чат/LLM/MCP-аргументы (ADR-18).
- Форковые конфиг-секреты (`#[credential_class]`-поля: `encrypted_secret`,
  `external_auth_store` и т. п.) остаются в форке.
- **Граница публикации:** контент, несущий `#[credential_class]`-классификацию, в ob2h
  не публикуется. Мост `[ob2h_bridge]` отбрасывает такие находки до `memory_save`
  (см. `omnesagent-ralph::bridge`). Изоляция проверяется негативными тестами
  (непопадание секретов в память/RAG/граф/логи).

**Когда сейф ob2h, а когда форковый конфиг (правило выбора):**

| Критерий | ob2h Vault (`secrets_*`) | Форковый конфиг-секрет (`#[credential_class]`) |
|---|---|---|
| Область | Межагентная, кросс-продуктовая | Конфигурация конкретного сервера/канала форка |
| Примеры | Токены LLM-провайдеров, общие API-ключи | `McpServerConfig.env`, `access_token` каналов, `api_key` провайдеров |
| Запись | GUI/CLI ob2h (PIN); `secrets_set` из EngineeringTask/SystemAdmin | `#[secret]`-поле в TOML/схеме форка |
| Чтение агентом | `secrets_get` — только под Engineering/SystemAdmin; `secrets_status` — read-класс | Значение распаковывается харнесом при инициализации подсистемы, агенту не отдаётся |
| В чате/MCP-аргументах | Никогда | Никогда |
| В памяти/RAG/графе/логах | Никогда | Никогда |

Правило выбора: **межагентное и переиспользуемое — в сейф ob2h; привязанное к
конфигурации конкретного сервера форка — в конфиг форка.** Копирование секретов между
слоями (в любую сторону) запрещено.

**Изоляция (Ф4.3):** гарантии и их покрытие тестами:

- (а) значения сейфа, попадающие в текст MCP-ответов, скрабируются
  (`omnesagent-tools::mcp_context`, тест `resource_wrapper_redacts_secrets`);
- (б) `McpServerConfig.env`/`headers` классифицированы как `EncryptedSecret` и
  шифруются на сохранении; значения не раскрываются при интроспекции конфига
  (`omnesagent-config::schema`, тесты `empty_secret_map_emits_container_row`,
  `secret_map_entries_add_list_and_remove_without_leaking_values`) — каждый
  stdio-сервер получает только собственный `env` (`mcp_transport::spawn`);
- (в) write-инструменты сейфа недоступны в CodeExploration — контроль на стороне
  агента `filter_tools_for_intent` (только `secrets_status` в read-only-группе,
  тест `test_ob2h_vault_tools_are_engineering_only`);
- (г) `#[credential_class]`-контент не публикуется мостом
  (`omnesagent-ralph::bridge`, тесты `credential_class_content_is_never_published`,
  `debt_with_credentials_is_skipped`).

### 6. Lessons/slots — внешний слой ob2h, близнец не дублирует (ADR-O7)

Уроки (`lesson_*`) и слоты (`slot_*`) ob2h — **кросс-продуктовые** сущности: общий
процедурный опыт семейства (Hermes/ZCode/OmnesAgent) и общий рабочий буфер проекта
«на сейчас». Форк потребляет их через MCP (`lesson_*`, `slot_*`, блок `<agent_slots>`
в `memory_context`); в `omnesagent-memory` таблицы lessons/slots **не создаются**.

Обоснование: split-brain процедурного опыта и буфера дороже выигрыша от локальной
копии (тот же принцип, что ADR-O1/O2); прецедент — удаление Qdrant «in favor of ob2h».
Порт семантики в близнец (agent-scoped уроки) — бэклог по явной потребности, отдельным
решением.

**Таблица ролей (аналог §5.3 ob2h v1.9.9):**

| Слой | Что хранит | Жизненный цикл |
|---|---|---|
| Слоты ob2h (`slot_*`) | рабочий буфер проекта «на сейчас» (межагентный) | бюджет/пины/scope ob2h; инъекция `OB2H_SLOTS_MAX_CHARS` |
| `omnesagent-memory` | сессии, agent-scoped записи, prefetch | управляется харнесом (retention, budget) |
| Файлы workspace | долгосрочный характер проекта | VCS владельца |

Запрет: копировать содержимое слотов в память форка и наоборот (split-brain).

### 7. Петля опыта Ralph замыкается через lessons (ADR-O8)

- Verified findings при архивации run публикуются не только фактами (`memory_save`),
  но и уроками: `lesson_save` — content = формулировка «что делать / чего избегать»,
  context = задача + затронутые символы AST-снапшота, tags = `["ralph", <feature_slug>]`,
  source = `manual`; `push_lessons` (default **false**, opt-in) управляет публикацией.
- Перед итерацией `loop_driver` запрашивает `lesson_recall` (query = текст задачи +
  символы AST-снапшота, project = `omnes-agent`, limit = 5, min_confidence = 0.2) и
  inject-ит блок `<lessons>` в контекст-пакет; **fail-open** — недоступность ob2h не
  роняет итерацию (лог один раз на run).
- Decay-кандидаты дрима — только отчёт владельцу; удаление — явная команда (ADR-11 ob2h).
- Идемпотентность: fingerprint-дедуп ob2h (повторный `lesson_save` = reinforce, не дубль);
  мост не генерирует вариации текста одного урока.
- Метрика петли (repeat-fail/reuse-hit из `ralph_report`) — **наблюдение**, не гейт (ADR-O11).

### 8. Провенанс интеропа — recap-путь; governance — reason (ADR-O9)

- Факты с провенансом публикуются только цепочкой `session_ingest` (журнал run,
  `session_id = ralph-<run_id>`, `author = omnesagent-ralph`, `source = ralph`) →
  `session_recap(save=true)` → `meta.source_ids` → `memory_verify`. Прямой `memory_save`
  провенанса не даёт (ограничение контракта) — допустим для операционных заметок, не для
  «ключевых решений». `recap_on_archive` (default **false**, opt-in) управляет recap-путём;
  дедуп ingest по позиции делает повторную архивацию безопасной.
- `memory_forget` из форка вызывается **только с `reason`** (governance-семантика Ф68);
  `memory_audit` — read-only окно для отчётов владельцу (actor = agent виден).
- Upstream-пробелы (`source_ids` в `lesson_save`, `meta` в `memory_save`) — аддитивные
  запросы к стороне ob2h, форк их не реализует.

### 9. Viewer — привилегированный спавн, PIN только локально (ADR-O10)

`viewer_open` — **privileged**-инструмент (спавн GUI-процесса): доступен в
EngineeringTask/SystemAdmin, **никогда** в CodeExploration. Окно переживает сессию агента
(detached); если harness форка убивает job-дерево — фолбэк CLI `ob2h viewer`, поведение
фиксируется в скилле. PIN и значения секретов не проходят через чат/LLM/MCP-аргументы ни
при каком сценарии (ADR-18 ob2h, ADR-O5); вкладка «Секреты» — только локальный ввод в окне.
Headless/SSH — честный отказ, не паника.

### 10. Скрабинг — defense in depth; новые bench-метрики — наблюдение (ADR-O11)

Три рубежа скрабинга секретов:

1. credential-guard моста (`CREDENTIAL_MARKERS`, Ф2/ADR-O5) — отбрасывание находок до публикации;
2. category-redact форка (`omnesagent-memory::redact`), дополненный до паритета паттернов
   Ф69.1 (`sk-…`, `ghp_…`/`gho_…`/`ghs_…`, `Bearer …`, `://user:pass@`, `OB2H_[A-Z_]+=…`);
3. серверный скрабинг ob2h при захвате (`OB2H_REDACT`, default on).

Значения секретов не попадают в lessons/slots/audit/daily ни на одном рубеже.
`recall_all@k` / `any_hit@k` в `omnesagent-kag::bench` — **наблюдение**: пороги
`GateVerdict` (`RECALL_DROP_LIMIT`/`MRR_DROP_LIMIT`, ADR-13 ob2h) не меняются без данных.

### 11. Векторный и графовый конвейер v2.0 (ADR-O12..ADR-O16)

- **ADR-O12 (Mean-Centered BQ & Two-Stage Search):**
  Двухступенчатый поиск: ступень 1 — 1-битовое центрированное квантование (`val[i] > mean`), 48 байт на 384 dim (`[u64; 6]`), отбор кандидатов по Hamming distance (`popcnt`); ступень 2 — точный SIMD FP32 rescore.
- **ADR-O13 (AST-aware Chunking):**
  Нарезка исходного кода по границам AST-узлов (функции, структуры, классы) с контекстным префиксом `// File: {path} | Scope: {scope}` и blake3-хешированием.
- **ADR-O14 (Co-Change & Temporal Coupling):**
  Анализ истории git commit log (окно 100 коммитов, исключение коммитов > 10 файлов, порог $\ge 3$ общих правок) формирует рёбра `CO_CHANGES_WITH` с `provenance = 'GIT_HISTORY'` и `confidence = min(1.0, count/10.0)`.
- **ADR-O15 (AffectedTests & Blast Radius):**
  В `ImpactReport` включаются затронутые тесты (`affected_tests`) с расчетом уверенности по экспоненциальному затуханию `confidence = 0.85^depth` при обратном обходе вызовов.
- **ADR-O16 (U-PPR v2.0 & Stale-While-Revalidate):**
  Usage-weighted PPR с дифференциацией затухания ($\lambda_{\text{code}} = 0$, $\lambda_{\text{memory}} > 0$), In-degree penalty для структурных хабов, поддержкой `mute_node` (`trust *= 0.3`) и флагом `degraded_mode: "warming_up_embeddings"` при холодном старте.

## Последствия

- Дедупликация на границе обязательна: перед `memory_save` агент выполняет
  `memory_search` (правило в скилле `.agents/skills/ob2h/SKILL.md` и системном
  промпте форка). Мост использует стабильный `key` находки, что делает повторную
  публикацию идемпотентной.
- Публикация находок Ralph включается только при `enabled = true` **и**
  `auto_push_after_archive = true`; `push_scope` по умолчанию `verified`.
- Каждое расширение контракта интеропа (новый канал публикации, автосинк) требует
  отдельного решения владельца.

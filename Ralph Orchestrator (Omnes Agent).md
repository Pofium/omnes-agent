# ПРОЕКТ A: «Ralph Orchestrator» — автономный цикл разработки в Omnes Agent

| | |
|---|---|
| Версия | 2.1 (полностью самодостаточный документ; ревью 09.09.2026) |
| Дата | 2026-09-09 |
| Репозиторий | `C:\Projects\Omnes-agent` (backend — Rust workspace, frontend — Flutter) |
| Статус | Независимый проект. Работает БЕЗ Проекта B (ob2h); при наличии ob2h — обогащается (см. §4) |
| Поглощено | Концепция **Ponytail** (DietrichGebert/ponytail): лестница минимальности, review/delete-list, debt-леджер, честный агentic-бенчмарк — §2 |

---

## 1. Общие сведения

### 1.1. Цель
Оркестратор автономных циклов разработки (Ralph Loop × OpenSpec) как полноправная
подсистема Omnes Agent: конечный автомат фичи, запуск исполнителей на задачи с
чистым контекстом, тесты, git-коммиты, человеческие гейты, UI.

### 1.2. Границы независимости
- Проект **не требует** ob2h: все данные цикла живут в репозитории проекта
  (`openspec/changes/<slug>/`), цикл самодостаточен (§4).
- Знаниевый слой — **только собственная память omnes-agent** (embedded-движок
  `omnesagent-memory` + `omnesagent-kag`); **низовой интеграции с ob2h нет**:
  обмен знаниями — опционально и точечно, через ob2h-bridge (скилл+тулза+плагин, §4.3).
- Ничего из ob2h не нужно для сборки/тестов этого проекта.

### 1.3. Словарь
Ralph Loop (цикл «перечитал → сделал → протестировал → коммит → повтор»),
дельта-спека OpenSpec, итерация, вердикт (verified/failed/unconfirmed/stale),
лестница Ponytail (§2.1), debt-леджер, плато.

---

## 2. Анализ Ponytail: что взято и почему

Ponytail — скилл-пакет «ленивого сеньора» для 20+ AI-агентов: правила минимальности
+ инструменты ревизии. Наш контекст другой (автономный цикл, а не разовая сессия),
но именно поэтому часть идей становится у нас **сильнее**:

| # | Идея Ponytail | Как интегрируем в Ralph |
|---|---|---|
| 1 | **Лестница 7 ступеней** (YAGNI → reuse → stdlib → platform → dep → one line → minimum), ступени после понимания задачи | Обязательная часть промпт-контракта исполнителя; каждая итерация фиксирует `ladder_rung` (на какой ступени решение). Появляется метрика минимальности цикла |
| 2 | **Root cause, not symptom** (чинить общую функцию, не все вызовы) | Правило в контракте: фиксы только в общей точке; AST/поиск по коду — на стороне исполнителя |
| 3 | **«Не-отсекаемое»** (валидация на границах, data-loss, безопасность, a11y, явно запрошенное) | Carve-outs в minimality review: delete-list не имеет права трогать это |
| 4 | **ONE runnable check** на нетривиальную логику (минимальный тест без фреймворков) | В tasks.md критерий `done_when` допускает минимальный чек; оркестратор не требует тяжёлых тестов там, где спека не требует |
| 5 | **`ponytail:` маркеры** осознанных упрощений (ceiling + upgrade path) и **debt-леджер** («later ≠ never») | Сбор маркеров из диффа каждой итерации → journal; на Verify/Archive — harvest в ledger, оттуда — задачи/находки (§7) |
| 6 | **`/ponytail-review` — delete-list из диффа** | **Minimality review** перед коммитом: лёгкий LLM-проход возвращает список лишнего; применяется только к безопасным категориям (§7.2) |
| 7 | **`/ponytail-gain` — честный scoreboard** (LOC/tokens/cost/time против baseline, с контрольными ARM'ами и safety-тиром) | **Бенчмарк-культура**: E2E-сравнение «Ralph с памятью / Ralph без памяти / наивный цикл», публикация методики и сырых цифр (§9) |
| 8 | **Мульти-harness дистрибуция**: один канонический текст + тонкие адаптеры, режимы lite/full/off, дефолт через env/config | Режимы цикла `lite/full` (§5.4); правила исполнителя — один канонический блок в промпт-контракте |
| 9 | **Subagent injection** (правила пробрасываются в подагентов, matcher) | В native-исполнителе контракт распространяется на все подзадачи/подагентов |

**Не берём:** terse-prose стиль (не наше), plugin/marketplace-механику под 20 хостов
(у нас один транспорт — MCP + gateway), lifecycle-хуки npm (не наш стек).

---

## 3. Архитектура

```
[Flutter UI «Ralph»] ──REST/WS──► [omnesagent-gateway]
                                        │
                                        ▼
              ┌────────────────────────────────────────────┐
              │        omnesagent-ralph (новый крейт)      │
              │ StateMachine · LoopDriver(async) · Plateau │
              │ ExecutorRouter · MinimalityReview          │
              │ DebtHarvester · Git · Knowledge (своя память)│
              └───────┬──────────────────────┬─────────────┘
                      │ (всё в репо проекта) │ своя embedded-память (async)
                      ▼                      ▼
        openspec/changes/<slug>/   omnesagent-memory (MemoryBackendKind enum)
        ├ state.json (автомат)     + omnesagent-kag (ProjectService:
        ├ journal.jsonl (итерации)   search_symbols[async], analyze_impact,
        └ код, тесты, git            build_context → reuse, blast radius)
                                             │
                                             ▼ точечный обмен знаниями (опция)
                      [ob2h-bridge: скилл + ob2h_bridge_* + конфиг [ob2h_bridge]]
                                             │ MCP
                                             ▼
                                      ob2h serve (Проект B)
                      ▲
                      │ задача+контекст → JSON-ответ
              [Исполнитель: native | omnescode | claude -p | hermes -z | custom]
```

---

## 4. Хранилище цикла (встроенное, без внешних зависимостей)

### 4.1. state.json — машина состояний (source of truth, ADR-A4)
См. Приложение A. Атомарная запись (tmp+rename) при каждом переходе.

### 4.2. journal.jsonl — журнал итераций (append-only, git-версионируемый)
Каждая строка — итерация (структура в Приложении B): задача, n, hypothesis,
plan, result, tests_summary, verdict, ladder_rung, markers (ponytail:),
git_before/after, tokens, duration. Плюс findings (размышления) с вердиктами.
Версонируется git'ом вместе со спекой — полная история цикла в репозитории.

### 4.3. Знаниевый слой: собственная память omnes-agent; ob2h — внешний пир через ob2h-bridge

Решение (утверждено 09.09.2026): **никакой низовой интеграции omnes-agent ↔ ob2h** —
ни общих БД, ни прямого доступа к чужому хранилищу, ни «знаниевого бекенда-свитча».
Системы из одного семейства, но живут раздельно; обмен — только через специальный
**ob2h-bridge** (скилл + тулза + плагин, §4.3.1) и только точечно, по проектам.

**Собственное знаниевое хранилище цикла** (внутреннее дело omnes-agent):
- findings → embedded-память агента (`omnesagent-memory`, `MemoryBackendKind` enum +
  factory `build_memory_backend()`):
  записи `category='ralph:*'`, вердикт в meta → FTS5+вектора, decay/dedup бесплатно;
- runs/итерации → репозиторий (state.json + journal.jsonl, §4.1–4.2);
- архитектурный контекст → `omnesagent-kag` (`ProjectService.search_symbols` [**async**],
  `analyze_impact` [sync], `build_context` [sync]) — reuse-кандидаты, blast radius, god nodes.

**ob2h в цикле не участвует.** Он подключается опционально как внешний пир того же
семейства — для обмена знаниями по конкретным проектам (§4.3.1–4.3.3).

#### 4.3.1. ob2h-bridge: три формы одного компонента

| Форма | Где живёт | Что делает |
|---|---|---|
| **Скилл** `ob2h-bridge` | skills/ omnes-agent | LLM-инструкция: когда предлагать обмен (push выжимки/findings после Archive; pull перед новой фичей по тому же проекту), что запрещено (фоновая двусторонняя синхронизация) |
| **Тулза** `ob2h_bridge_status / push / pull` | omnesagent-tools | Исполняющие вызовы: скоуп (findings/summary/debt), project_id; внутри — единственный в omnes-agent MCP-клиент к ob2h (JSON-RPC/stdio, подпроцесс `ob2h serve`) |
| **Плагин/конфиг** `[ob2h_bridge]` | omnesagent-config + plugin-реестр | Автоподключение скилла+тулз агенту; параметры подключения и скоупов (§5.2) |

#### 4.3.2. Контракт обмена (envelope v1)

Обе системы обмениваются нейтральным JSON-конвертом:
`{schema_version, origin: "omnesagent"|"ob2h", project_id, kind: finding|summary|debt,
key, content, verdict, symbols[], meta, updated_at}`.
Каждая сторона маппит конверт в свою схему — родство почти 1:1, дивергентные
колонки покрыты явной таблицей соответствия (omnesagent `agent_id/namespace/
session_id` ↔ ob2h `importance/source/origin/deleted_at`), поэтому изменение
схемы одной из сторон не ломает мост.

#### 4.3.3. Правила без конфликтов

1. Только публичные интерфейсы: ob2h — свои MCP-инструменты; omnes-agent —
   MemoryBackend/gateway. Прямые записи в чужую SQLite запрещены.
2. Namespace-изоляция ключей: у ob2h записи моста получают ключи
   `omnesagent:<project_id>:…` и `source='bridge:omnesagent'`; у omnes-agent —
   `ob2h:<project_id>:…`. Записи с чужим префиксом **не реэкспортируются**
   (эхо-защита от циклического дублирования).
3. Scoping: обмен только по явно указанному project_id и kind; по умолчанию
   уезжает только `verified`-контент и summary (fail-знания остаются локальными).
4. Конфликт key → LWW по `updated_at`; удаление передаётся tombstone-пометкой,
   не молчаливым удалением.
5. Идемпотентность: дедуп по content-hash; повторный push не создаёт дубль.
6. v1 — обмен только по инициативе (человек/скилл): push после Archive, pull перед
   фичей; фоновых автосинков нет. Обе стороны полностью автономны при
   выключенном или недоступном мосте.

Ralph использует мост опционально: в конце Archive — предложить push; перед
стартом фичи по уже знакомому проекту — предложить pull. По умолчанию мост выключен.

---

## 5. Интеграция в бекенд Omnes Agent

### 5.1. Новый крейт `backend/crates/omnesagent-ralph`
```
src/
├── lib.rs            // RalphOrchestrator, RunHandle
├── state.rs          // state.json (serde, атомарная запись)
├── journal.rs        // journal.jsonl (append-only, serde-строки)
├── tasks.rs          // парсер tasks.md (T-XXX, depends, done_when, scenarios)
├── openspec.rs       // каркас/архив дельта-спеки (Fission-AI-совместимо)
├── loop_driver.rs    // цикл Apply
├── executor.rs       // трейт Executor + адаптеры (§6)
├── knowledge.rs      // свой знаниевый слой: findings → MemoryBackend (category='ralph:*'), контекст-пакет (§4.3)
├── minimality.rs     // Ponytail: delete-list review (§7.2)
├── debt.rs           // Ponytail: harvest ponytail:-маркеров (§7.3)
├── plateau.rs        // fingerprint падений
├── git.rs            // коммиты/реверты ralph(...)
├── approval.rs       // человеческие гейты через gateway
└── events.rs         // события для gateway/WS
```
MCP-клиент к ob2h в крейт **не входит**: соединение с ob2h живёт только в
тулзах ob2h-bridge (omnesagent-tools, §4.3.1).
Workspace-правила: все фоновые задачи через `spawn!` (`omnesagent-spawn`),
зависимости — tokio/serde/`omnesagent-log`/`omnesagent-config`/`omnesagent-spawn`,
без внешних SDK.

### 5.2. `omnesagent-config` — секция `[ralph]` (строго типизированная)
```toml
[ralph]
enabled = true
executor = "native"            # native | omnescode | claude | hermes | custom
executor_command = ""          # шаблон для custom с {prompt_file}
default_autonomy = "L1"        # L0 | L1 | L2
max_iterations_per_task = 5
max_total_iterations = 60
plateau_window = 3
context_max_tokens = 6000
test_command_fallback = "cargo test"
test_output_format = "exit_code"  # exit_code | junit_xml — парсинг результатов тестов
executor_timeout_secs = 300    # таймаут одного вызова исполнителя (M1: CLI subprocesses)
mode = "full"                  # Ponytail-режимы: lite | full  (off = цикл выключен)
minimality_review = true       # delete-list перед коммитом (§7.2)
debt_harvest = true            # сбор ponytail:-маркеров на Verify/Archive (§7.3)
notify_channel = ""            # "telegram:<chat_id>" через omnesagent-channels
commit_format = "cc"           # cc (Conventional Commits) | legacy (ralph(T-XXX, iN): ...)
[ralph.limits]
total_tokens = 0

[ob2h_bridge]                  # внешний пир знаний; по умолчанию выключен (§4.3)
enabled = false
ob2h_command = "ob2h"          # подпроцесс ob2h serve (MCP stdio)
ob2h_args = ["serve"]
projects = []                  # разрешённые project_id/slug; пусто — обмен только по явному запросу
push_scope = "verified"        # verified | all — что уезжает при push
auto_push_after_archive = false
```

### 5.3. Gateway (REST + WS), approval, channels
- REST: `POST/GET /api/ralph/runs`, `GET /api/ralph/runs/{id}` (+`/iterations`, `/ast-changes`, `/debt`), `POST /api/ralph/runs/{id}/stop`, `POST /api/ralph/verdicts`, гейты `GET /admin/ralph/pending`, `POST /admin/ralph/approve|deny` (по образцу существующего SOP-паттерна).
- WS `/ws/ralph`: `run.started`, `gate.pending`, `iteration.completed{task,n,verdict,ladder_rung}`, `iteration.failed{fingerprint}`, `review.delete_list`, `debt.harvested{count}`, `run.stopped{reason}`.
- `omnesagent-channels`: уведомления только на гейты/финал/плато.

### 5.4. Формат `tasks.md` (нормативный)

Файл `openspec/changes/<slug>/tasks.md` — список задач для цикла Apply.
Парсится `tasks.rs` из крейта `omnesagent-ralph`.

```markdown
## T-001: <название задачи>
- **depends:** (пусто | T-000, T-002, ...)
- **done_when:** <критерий завершения; ONE-check допускается — assert/маленький тест>
- **scenarios:**
  - Сценарий 1: <описание входов и ожидаемого результата>
  - Сценарий 2: <описание>
- **paths_scope:** src/foo.rs, src/bar/ (ограничение правок; пусто = весь репозиторий)
- **test_command:** cargo test --test config_test (пусто → test_command_fallback из конфига)
```

Правила парсинга:
- Заголовок `## T-XXX:` — обязательный ID + название;
- `depends` — DAG; циклические зависимости → ошибка парсинга;
- `done_when` — человеко-читаемый критерий, используется как часть промпта исполнителю;
- `paths_scope` — whitelist файлов/папок; git diff проверяется на соответствие;
- Порядок задач — по ID, не по позиции в файле.

### 5.5. Concurrency: один run на проект (NFR-A1)

Механизм: при старте run на `openspec/changes/<slug>/` оркестратор создаёт
`state.json` с атомарной записью (tmp+rename). Перед созданием проверяет:
1. Существует ли `state.json` с `phase ≠ archived | stopped`;
2. Если да — отклоняет новый run с ошибкой `ERR_RUN_ACTIVE`;
3. Если нет — создаёт новый `state.json`.

Дополнительно: `RwLock<HashMap<String, RunHandle>>` в `RalphOrchestrator` для
in-memory трекинга; при gateway-перезапуске — восстановление из `state.json`.

### 5.6. Gateway: интеграция в `AppState`

По аналогии с `sop_engine: Option<Arc<Mutex<SopEngine>>>` в `AppState`:
```rust
pub ralph: Option<Arc<RalphOrchestrator>>,
```
Гейтвей при старте проверяет `[ralph].enabled` → создаёт `RalphOrchestrator` → кладёт
в `AppState`. REST/WS-хэндлеры делегируют вызовы через `state.ralph`.

### 5.7. Flutter — фича `features/ralph`
Таймлайн итераций (задача, гипотеза, результат, бейдж вердикта, ступень лестницы),
дерево изменений, delete-list ревью с кнопками принять/отклонить, debt-леджер
(таблица маркеров с no-trigger подсветкой), кнопки ✓/✗ вердиктов, индикатор режима
(lite/full). i18n — `locales.toml` (ru/en), компоненты существующей дизайн-системы.

---

## 6. Исполнители (Executor)

### 6.1. Трейт
```rust
#[async_trait]
pub trait Executor {
    async fn run_task(&self, ctx: &TaskContext) -> Result<ExecutorAnswer, ExecutorError>;
}
```
`ExecutorAnswer` — строго типизированный (Приложение C): hypothesis, plan, result,
self_assessment, ladder_rung, markers. Парсинг из финального ```ralph-блока```.

### 6.2. Адаптеры и приоритет
| Вариант | Механика | Этап |
|---|---|---|
| `claude` / `hermes` / `custom` | `CLI -p <prompt_file>` | M2 (первый релиз) |
| `native` | In-process turn через публичный API omnesagent-runtime + omnesagent-tools (ADR-A3: без правок agent-loop) | M3 |
| `omnescode` | Неинтерактивный режим (доработка omnescode — отдельной задачей) | опц. |

Контракт един для всех; правила Ponytail (лестница, root-cause, carve-outs,
ONE-check) — часть канонического промпта (Приложение C). Контракт распространяется
на подагентов исполнителя (native — явно; CLI — инструкцией в промпте).

---

## 7. Инструкции для оркестратора (норматив)

### 7.1. Машина состояний
```
explore → proposed → (gate L1/L2?) → applying → verifying → archived
                        ▲                │           │
                        └── правки спеки ┘           └→ провал verify → applying
любое состояние → stopped (плато/лимит/бюджет/human)
```

### 7.2. Цикл Apply — псевдокод `loop_driver.rs` (async fn)
```
async fn drive_loop(...):
  loop:
    state = read state.json
    match state.phase:
      applying:
        task = next_ready(tasks.md, state.done)      # парсер §5.4
        if task is None: phase = verifying; continue
        n = iter_count(task.id) + 1
        if n > max_iterations_per_task: mark blocked; continue
        ctx = knowledge.build_context(task).await    # async: search_symbols + sync analyze_impact/build_context
                                                     # своя память: MemoryBackendKind + kag API
        git_before = git rev-parse HEAD
        answer = executor.run_task(TaskContext{task, ctx, spec}).await  # timeout: executor_timeout_secs
        tests = run(task.test_command ?? config.test_command_fallback)
        fp = fingerprint(sorted(failed_test_names))  # sha256(sorted(failed_test_names + exit_code))
        verdict = tests.exit_code == 0 ? verified : failed
        write_journal(iteration{task,n,answer,tests,verdict,answer.ladder_rung,answer.markers})
        knowledge.record(&it).await                  # findings → memories (category='ralph:*')
        if tests.ok:
          if minimality_review:                      # Ponytail delete-list (§7.2.1)
            dl = executor.review_delete_list(git_diff).await   # carve-outs enforced
            if dl.applicable: apply(dl); tests = rerun(test_command)
          # CC-совместимый формат (по умолчанию; legacy = ralph(T-XXX, iN): ...)
          git commit -m "feat(ralph/T-{task.id}): i{n} — {answer.summary}"
          state.done += task.id
        else:
          if plateau(fp, window): stop(plateau); break
          continue
      verifying: harvest_debt(); opsx_verify(); ...
```

#### 7.2.1. Minimality review (Ponytail, перед коммитом, если включён)
- Вход: git diff зелёной итерации. Выход: delete-list (что убрать/упростить).
- **Carve-outs (неприкасаемые):** валидация на границах доверия, обработка потери
  данных, безопасность, доступность, явно запрошенное в спеке.
- Применение: оркестратор принимает delete-list только правками удаления/упрощения;
  после правок — обязательный повторный прогон тестов; красный тест → откат правок review.
- Cost control: один LLM-проход, бюджет по токенам; на `lite`-режиме — выключен.

### 7.3. Debt harvest (Ponytail)
На `verifying`/`archived` (если включён): сбор `ponytail:`-маркеров из диффа цикла
→ debt-леджер (в journal + UI). Каждая строка: `file:line, что упрощено,
ceiling, upgrade-trigger`. Маркеры без trigger → тег `no-trigger` (подсветка в UI).
Осознанные упрощения с реальным upgrade-планом → предложения задач (в tasks.md
следующей фичи или findings). Леджер read-only, ничего не меняет в коде.

### 7.4. Контекст-пакет
Состав (своя память omnes-agent): спека задачи + её сценарии; failed-findings
(запрос по `memories` `category='ralph:*'` через `MemoryBackendKind` enum +
factory `build_memory_backend()`); reuse-кандидаты
`search_symbols` [**async**]; blast radius `analyze_impact` [sync]; god nodes `build_context` [sync];
инварианты `openspec/specs/`; лимит `context_max_tokens` (спека → негативный
опыт → reuse → зона → инварианты). История чата не входит.

Сборка контекст-пакета (`knowledge.rs`) — **async fn**, т.к. `search_symbols` async.
Остальные вызовы (sync) оборачиваются в `spawn_blocking` при необходимости.

При недоступности памяти (standalone-CLI без runtime) — degraded: задача +
сценарии + failed-findings из journal.jsonl. ob2h-bridge в контекст-пакет не
вмешивается: его данные попадают в память заранее, через pull (§4.3.3).

### 7.5. Остановки и git
- Плато: `plateau_window=3` одинаковых fingerprint → stop; лимиты задач/цикла/токенов.
- Fingerprint: `sha256(sorted(failed_test_names + exit_code_string))` — детерминированный;
  сравнение по sliding window последних `plateau_window` итераций одной задачи.
- Git: коммит на зелёную итерацию `feat(ralph/T-003): i2 — <summary>` (Conventional Commits;
  legacy-формат `ralph(T-003, i2): …` доступен через `commit_format = "legacy"` в конфиге);
  push запрещён; revert → journal остаётся (история правдива), state.json — откат статусом задачи.

### 7.5.1. Resume после краша
При старте оркестратор проверяет `state.json` в `openspec/changes/<slug>/`:
1. Если `phase ∈ {applying, verifying}` — resume: продолжить с `next_ready` или `harvest_debt`;
2. Если последняя запись journal.jsonl не имеет `git_after` — считать итерацию failed
   (не дублировать);
3. Проверка: `git rev-parse HEAD` vs `state.last_known_head` (если поле задано) —
   при расхождении предупредить о ручных коммитах (warning в логах + WS-событие
   `run.resumed{manual_commits_detected: true}`);
4. При `phase = stopped | archived` — новый run, не resume.

### 7.6. Degraded-режим
Память агента недоступна (standalone-CLI без runtime, нестандартная сборка) →
флаг `degraded=true`, контекст-пакет из journal.jsonl (§7.4), предупреждение в
логах и UI. В обычной установке omnes-agent память встроена — degraded практически
не возникает. ob2h-bridge на цикл не влияет: мост выключен/недоступен → обмен
просто не происходит, цикл работает автономно.

---

## 8. Функциональные и нефункциональные требования

| ID | Требование |
|---|---|
| FR-A1 | Машина состояний + state.json, resume после краша, атомарные записи |
| FR-A2 | Парсер tasks.md (depends/done_when/scenarios; done_when допускает ONE-check) |
| FR-A3 | Цикл Apply §7.2: контекст → исполнитель → тесты → journal → ветвление |
| FR-A4 | Трейт Executor: native + claude/hermes/custom (+omnescode опц.), единый контракт с лестницей Ponytail, фиксация ladder_rung |
| FR-A5 | Minimality review с carve-outs и повторным прогоном тестов (config-gated) |
| FR-A6 | Debt harvest + леджер + no-trigger теги (config-gated) |
| FR-A7 | Остановки: плато/лимиты/бюджет/human |
| FR-A8 | Гейты L0/L1/L2 через gateway-approval + канальные уведомления |
| FR-A9 | REST/WS §5.3; Flutter §5.4 (таймлайн, delete-list, debt, режим) |
| FR-A10 | Знаниевый слой на собственной памяти omnes-agent: findings → `memories` (`category='ralph:*'`, `MemoryBackendKind` enum), контекст из kag API (`search_symbols` [async] / `analyze_impact` / `build_context`); **ob2h в цикле не участвует** |
| FR-A10a | ob2h-bridge (§4.3): скилл + тулзы `ob2h_bridge_{status,push,pull}` + конфиг `[ob2h_bridge]`; envelope v1, namespace-изоляция, LWW, эхо-защита, идемпотентность; по умолчанию выключен |
| FR-A12 | Парсер `tasks.md` (§5.4): T-XXX ID, depends DAG, done_when, scenarios, paths_scope, per-task test_command |
| FR-A13 | Resume после краша (§7.5.1): проверка HEAD, не-дублирование итераций, warning при ручных коммитах |
| FR-A14 | Executor timeout (`executor_timeout_secs`, §5.2) — kill subprocess при превышении |
| FR-A11 | Конфиг [ralph] §5.2; режимы lite/full; spawn! для фоновых задач |
| NFR-A1 | Один активный run на проект; цикл не блокирует gateway |
| NFR-A2 | Полный аудит: raw-ответ исполнителя на диск; journal git-версионируется |
| NFR-A3 | Windows/кириллические пути — обязательный тест-кейс |
| NFR-A4 | Токены: фиксация на итерацию; бюджет review отдельно |

## 9. Бенчмарк-культура (заимствовано у Ponytail)
Поставляется фикстура-репо + сценарий сравнения по git-diff (LOC/tokens/cost/time):
- ARM 1: Ralph полный (контекст-пакеты + minimality review);
- ARM 2 (контроль): тот же цикл без контекст-пакетов и review;
- ARM 3 (наивный): обычная агентская сессия без цикла.
Плюс safety-тир (adversarial-задачи: цикл не имеет права ломать безопасность).
Метрики цикла: repeat-failure rate (fingerprint), доля итераций с переиспользованием
(rung 2), рост debt-леджера. Методика и сырые цифры публикуются рядом с кодом.

---

## 10. ADR (собственные, нумерация проекта A)

| ADR | Решение | Альтернатива | Обоснование |
|---|---|---|---|
| ADR-A1 | Журнал цикла — файлы в репозитории (state.json + journal.jsonl) | БД агента/ob2h | Самодостаточность, git-история, читаемость любым инструментом |
| ADR-A2 | Отдельный крейт, CLI-режим standalone + режим под gateway | Внутри runtime | Работает до UI; runtime не разбухает |
| ADR-A3 | Native executor поверх публичного API runtime, без правок agent-loop | Правки runtime | Изоляция; runtime — отдельными PR |
| ADR-A4 | state.json — источник истины; journal — факты | Только в БД | Краш-устойчивость, resume, история в git |
| ADR-A5 | Minimality review — отдельный LLM-проход до коммита, не часть исполнителя | Просить исполнителя сразу писать минимум | Разделение обязанностей: исполнитель строит, review урезает; carve-outs проверяемы |
| ADR-A6 | MCP-клиент к ob2h — свой минимальный JSON-RPC/stdio | Внешний SDK | Единый стиль workspace; ob2h уже stdio-сервер |
| ADR-A7 | Никакой низовой интеграции omnes-agent ↔ ob2h: у каждой системы своя память; обмен — только через ob2h-bridge (скилл+тулза+плагин) поверх публичных интерфейсов, с namespace-изоляцией ключей и эхо-защитой | Общая БД / прямой доступ к SQLite / знаниевый бекенд-свитч | Системы одного семейства с разъехавшимися схемами: прямая связка сцепила бы их жизненные циклы и создала конфликты записи; мост даёт точечный обмен по проектам без сцепления |

## 11. Этапы и приёмка

### A-M1 — крейт + CLI (≈ 1–2 недели)
- [ ] `cargo test -p omnesagent-ralph` + clippy зелёные;
- [ ] Парсер `tasks.md` (§5.4) — unit-тесты: depends DAG, циклические зависимости → ошибка, paths_scope;
- [ ] E2E на fixture-репо: 2 красные → 1 зелёная итерация → delete-list → коммит → verify → archive → debt-леджер;
- [ ] плато → stop + отчёт; kill -9 → resume без дублей (§7.5.1); проверка HEAD-расхождения;
- [ ] executor: `claude -p` обязателен; `hermes -z` — второй; timeout = `executor_timeout_secs`;
- [ ] Concurrency: попытка запустить второй run на тот же slug → `ERR_RUN_ACTIVE` (§5.5).

### A-M2 — gateway + ob2h-bridge (≈ 3–5 дней)
- [ ] REST/WS по §5.3 (curl + ws-клиент), approval по паттерну SOP, Telegram-уведомление на gate.pending;
- [ ] ob2h-bridge: тулзы `ob2h_bridge_{status,push,pull}` идемпотентны; эхо-защита (запись с чужим namespace-префиксом не реэкспортируется) покрыта тестом; LWW-конфликт разрешается предсказуемо; при выключенном мосте цикл полностью автономен.

### A-M3 — Flutter (≈ 1 неделя)
- [ ] Экран «Ralph»: таймлайн, delete-list, debt, вердикты (обязательная визуальная/device-проверка);
- [ ] i18n ru/en.

### A-M4 — бенчмарк (≈ 3 дня)
- [ ] Фикстура + 3 ARM'а §9; методика опубликована; цифры приложены к релизу.

## 12. Риски

| Риск | Мера |
|---|---|
| Native executor тянет правки runtime | ADR-A3: только публичный API; первый релиз — CLI-исполнители |
| omnescode без неинтерактивного режима | claude/hermes на M1; доработка omnescode отдельной задачей |
| Конфликты мержа с upstream ZeroClaw | Крейт изолирован; gateway-эндпоинты в отдельных файлах |
| Minimality review режет нужное | Carve-outs + повторные тесты + откат правок при красном прогоне |
| Долгие итерации блокируют UI | Фоновая задача + WS-события |
| Расхождение state.json ↔ знаниевый бекенд | state.json — истина; записи бекендов идемпотентны (UNIQUE) |
| Дивергенция схем семейства (omnesagent ↔ ob2h) при обмене | envelope v1 + явная таблица соответствия полей в мосте (§4.3.2); namespace-изоляция и эхо-защита не дают записям гулять между системами |
| Коллизия записи одного key из обеих систем | LWW по updated_at + tombstones (§4.3.3); обмен инициируется одной стороной за раз |

## 13. Открытые вопросы
1. ~~Под-команда `omnes ralph` vs отдельный бинарь?~~ → **Решение**: подкоманда `omnes ralph`
   (крейт-библиотека + CLI-обёртка через `clap` в основном бинарнике; отдельный бинарь
   создаёт проблему дистрибуции).
2. ~~Авто-PR после Archive?~~ → **Решение**: M1 — ручной push; M2+ — опциональный
   `auto_pr = true` в конфиге (через `gh` CLI или `octocrab`).
3. ~~Публиковать ли крейт отдельно?~~ → **Решение**: нет (workspace-internal, зависит от
   `omnesagent-config/memory/kag`).
4. Test output parsing: exit code only (M1) или структурированный JUnit XML (M2)?
   → Предварительно: exit code на M1, опциональный `test_output_format = "junit_xml"` позже.

---

## Приложение A. state.json
```json
{
  "run_id": "01J8ZQ4V9C8X3K2M",
  "feature_slug": "add-config-hot-reload",
  "phase": "applying",
  "autonomy": "L1",
  "mode": "full",
  "done": ["T-001"], "blocked": [],
  "iterations": {"T-002": 1},
  "fingerprints": [{"task": "T-002", "n": 1, "fp": "sha256:…"}],
  "gates": [{"phase": "proposed", "decision": "approved", "by": "human", "at": "…"}],
  "degraded": false,
  "last_known_head": "e4f5a6b",
  "updated_at": "2026-09-09T12:00:00Z"
}
```

## Приложение B. Строка journal.jsonl (итерация)
```json
{"type":"iteration","task":"T-002","n":2,
 "hypothesis":"i1: гонка mtime-vs-read; чиним общую точку под lock",
 "plan":["stat+read под lock","тест на гонку"],
 "result":{"what_done":"lock вокруг stat+read","errors":[],"self_assessment":"ok"},
 "tests_summary":{"passed":14,"failed":0,"fingerprint":"sha256:9f2c…"},
 "verdict":"verified","ladder_rung":"reuse: std::sync::RwLock",
 "markers":[],"git_before":"a1b2c3d","git_after":"e4f5a6b",
 "tokens_used":8421,"duration_ms":96000,"at":"2026-09-09T12:03:00Z"}
```

## Приложение C. Промпт-контракт исполнителя (канонический блок Ponytail)
```
[Правила минимальности — лестница, останавливайся на первой верной ступени
ПОСЛЕ того, как понял задачу и прочитал код, который меняешь:]
1. Это вообще нужно строить? нет → не строй (YAGNI)
2. Уже есть в этой кодовой базе? переиспользуй, не переписывай
3. Стандартная библиотека умеет? используй её
4. Нативная возможность платформы покрывает? используй
5. Уже установленная зависимость решает? используй
6. Можно одной строкой? одной строкой
7. Только иначе: минимальный работающий код.

[Правила:]
- Root cause, не симптом: фикс делай в общей точке, проверь всех вызывающих.
- Никаких абстракций/зависимостей/boilerplate, которых не просили.
- Удаление лучше добавления; скучное лучше хитрого; меньше файлов.
- Осознанное упрощение с потолком помечай комментарием:
  `ponytail: <потолок>, <триггер пересмотра>`.
[Не отсекаемое никогда:] понимание задачи (трассируй поток целиком), валидация
на границах доверия, обработка потери данных, безопасность, доступность,
калибровка под реальное железо, всё явно запрошенное спекой.
[Проверка:] нетривиальная логика оставляет ОДНУ запускаемую проверку
(самопроверка/assert или маленький тест-файл; без фреймворков и фикстур).

[Формат ответа — последним блоком: ```ralph {"task_id","hypothesis","plan",
"result":{"what_done","errors","self_assessment"},"ladder_rung","markers"}]
[Запреты:] git commit/push, правки openspec/*, выход за paths_scope.
[Правила действуют и для любых подагентов, которых ты создашь.]
```

## Приложение D. Тулзы ob2h-bridge и MCP-контракт ob2h

### D.1. Тулзы ob2h-bridge (живут в `omnesagent-tools`, §4.3.1)
Обмен знаниями с ob2h; **сам цикл Ralph к ob2h не обращается.**

| Тулза | Аргументы | Возвращает |
|---|---|---|
| `ob2h_bridge_status` | project_id? | {connected, last_sync_at, pending_count} |
| `ob2h_bridge_push` | project_id, scope: verified\|all, kind: finding\|summary\|debt | {pushed_count, skipped_echo, conflicts_lww} |
| `ob2h_bridge_pull` | project_id, kind: finding\|summary\|debt, since? | {pulled_count, skipped_echo, conflicts_lww} |

### D.2. MCP-контракт ob2h (нормативный, v1; внешний API ob2h для оркестрации)
Если ob2h сам оркестрирует цикл Ralph (отдельный сценарий — не M1/M2):

| Инструмент | Аргументы | Возвращает |
|---|---|---|
| `ralph_start` | project_id, feature_slug, goal, autonomy?, limits? | run_id |
| `ralph_iteration` | run_id, task_id, n, hypothesis, plan, result, tests_summary, ladder_rung?, git_before?, git_after? | iteration_id, verdict, ast_changes_count, stale_marked |
| `ralph_verdict` | iteration_id\|finding_id, verdict, verdict_source, note? | статус |
| `ralph_context` | run_id, task_id, max_tokens? | контекст-пакет (+reuse-кандидаты) |
| `ralph_report` | run_id? | сводка (+gain-метрики, debt) |
| `ast_diff` | project_id, from, to? | symbol-level дифф |
| `ast_history` | project_id, symbol | хронология символа |

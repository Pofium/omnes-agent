# 🚀 План заимствования и архитектурной адаптации OB2H в OmnesAgent

> **Статус документа:** На утверждении владельцем  
> **Версия плана:** 2.0.0 (Комплексный перенос возможностей OB2H v1.3 и v1.4)  
> **Источники:** Кодовая база `C:\Projects\omnesbot_for_hermes` (`PLAN_v1.3.md`, `PLAN_v1.4.md`, `src/`)  
> **Целевая кодовая база:** `C:\Projects\Omnes-agent` (`backend/crates/omnesagent-*`)  
> **Принцип совместимости:** 100% обратная совместимость (Zero Breaking Changes) со всеми существующими контрактами API, шлюза и инструментов.

---

## 1. Архитектурный контекст и статус-кво

В предыдущей итерации в `omnesagent-kag` и `omnesagent-tools` был успешно перенесён **Трек C («Coding Graph» из v1.4)** и **Generic PPR**:
- Generic Pure-Rust **Personalized PageRank (PPR)** (`omnesagent-kag/src/graph/pagerank.rs`).
- **Structural queries**: `project_call_path`, `project_dead_code`.
- **Repo-map** под токен-бюджет (2k/4k/8k) на базе PPR по дереву зависимостей.
- **Edit-time blast radius hint** с TTL-кэшированием в SQLite KV (`project_blast_hint`).
- **AST Type-Resolve Lite** (`resolve_calls`) и модель происхождения рёбер `provenance` (`EXTRACTED`, `RESOLVED`, `INFERRED`).
- **Детекция сообществ** (Label Propagation + модулярность $Q$) и эвристики роутов (Axum/Actix/FastAPI) и таблиц ORM.
- **MCP/Runtime инструмент `memory_merge`** (каноническое слияние дублей с поглощением).

Настоящий план описывает портирование оставшихся ключевых возможностей из планов **OB2H v1.3 и v1.4**, сгруппированных по 5 функциональным пакетам.

---

## 2. Детализация пакетов заимствований и адаптации

```
                     ┌─────────────────────────────────────────────────────────┐
                     │          ПАКЕТ 1: ОБУЧАЕМАЯ ПАМЯТЬ & TRUST             │
                     │  Save-time cos>=0.98 дедуп · trust · memory_feedback    │
                     │  Typed edges (contradicts/supersedes) · [conflict] блок │
                     └────────────────────────────┬────────────────────────────┘
                                                  │
                     ┌────────────────────────────▼────────────────────────────┐
                     │          ПАКЕТ 2: ЛЁГКАЯ БАЗА (INT8 КВАНТОВАНИЕ)        │
                     │  BLOB v2 [magic:0x01][scale][i8 x dim] · сжатие 75%     │
                     │  Dual-read в deserialize · batch quantize utility       │
                     └────────────────────────────┬────────────────────────────┘
                                                  │
                     ┌────────────────────────────▼────────────────────────────┐
                     │          ПАКЕТ 3: ЧЕСТНЫЙ PREFETCH & MMR                │
                     │  Формула 5 факторов (hybrid + imp + trust + age + acc)  │
                     │  MMR-диверсификация (lambda=0.7) · жесткий бюджет 8К    │
                     └────────────────────────────┬────────────────────────────┘
                                                  │
                     ┌────────────────────────────▼────────────────────────────┐
                     │          ПАКЕТ 4: RALPH KNOWLEDGE LAYER                 │
                     │  Схема M6 (ralph_runs/iterations/findings/ast_changes)  │
                     │  Инструменты ralph_start, iteration, verdict, context   │
                     └────────────────────────────┬────────────────────────────┘
                                                  │
                     ┌────────────────────────────▼────────────────────────────┐
                     │          ПАКЕТ 5: НОЧНОЙ BENCH-ГЕЙТ & ВАЛИДАЦИЯ         │
                     │  Golden set bench (recall@k, MRR)                       │
                     │  Авто-откат dream_restore при падении recall>10%/MRR>15%│
                     └─────────────────────────────────────────────────────────┘
```

---

### 📦 Пакет 1: Обучаемая память, Trust-петля и защита от дублей (v1.3 Ф23 + v1.4 Ф31.1 + v1.4 Ф32)

#### 1.1. Мотивация и заимствуемый код из `ob2h`
- В `C:\Projects\omnesbot_for_hermes\src\memory\service.rs`:
  - Константа `pub const COS_IDENTITY: f32 = 0.98;`
  - Функция `top_cosine_neighbor(vec, key)`
  - Функция `record_feedback(key, verdict, note)`
  - Функция `link_after_save` и `conflicts_among`
- В `C:\Projects\omnesbot_for_hermes\src\mcp\tools.rs`:
  - Описание инструмента `memory_feedback`

#### 1.2. План адаптации в архитектуру OmnesAgent
1. **Миграция схемы SQLite** (`omnesagent-kag/src/schema.rs` / `omnesagent-memory/src/sqlite.rs`):
   - Добавить колонки в `memories`:
     ```sql
     ALTER TABLE memories ADD COLUMN trust REAL NOT NULL DEFAULT 0.5;
     ALTER TABLE memories ADD COLUMN last_feedback_at TEXT;
     ```
   - Добавить в `memory_relations`:
     ```sql
     ALTER TABLE memory_relations ADD COLUMN deleted_at TEXT;
     ```
     (soft-delete связей при забывании записи, чтобы история отношений не терялась бесследно).
2. **Save-time косинусный дедуп (v1.4 Ф31.1)** в `omnesagent-memory/src/sqlite.rs`:
   - При вызове `store_row_with_metadata` перед выполнением `INSERT`:
     - Выполняем поиск топ-1 ближайшего соседа по эмбеддингу среди существующих записей того же пространства/агента.
     - Если `cosine >= 0.98` (`COS_IDENTITY` — идентичный факт в иной формулировке):
       - Вместо дублирующей строки делаем тихий `UPDATE`: обновляем `content` на более свежий, мержим JSON `meta`, сохраняем `max(importance)`, инкрементируем `access_count`, обновляем дату `updated_at`.
       - Возвращаем ключ существующей записи. База защищена от разбухания идентичными правилами.
     - Если `cosine` в диапазоне `0.75..0.98`:
       - Добавляем в метаданные новой записи маркер `meta.merge_candidate = "<existing_key>"` для последующей ревизии в дриминге.
3. **Новый MCP/Runtime инструмент `memory_feedback` (v1.3 Ф23.4)**:
   - Файл: `backend/crates/omnesagent-tools/src/memory_feedback.rs`.
   - Входные параметры:
     ```json
     {
       "key": "имя_воспоминания",
       "verdict": "helpful | unhelpful | outdated",
       "note": "опциональный комментарий агента"
     }
     ```
   - Логика изменения `trust`:
     - `helpful`: `trust = min(1.0, trust + 0.15)`
     - `unhelpful`: `trust = max(0.0, trust - 0.20)`
     - `outdated`: `trust = max(0.0, trust - 0.30)`
   - Сохранение истории фидбека в `meta.feedback` (до 20 последних вердиктов).
   - Инвариант **«Никаких автоудалений»**: если `trust < 0.15`, запись автоматически получает флаг `meta.candidate_for_forget = true`, но **НЕ удаляется** физически.
4. **Typed edges и conflict-разметка (v1.4 Ф32)**:
   - Поддержка связей `contradicts`, `supersedes`, `causes` в `memory_relations`.
   - В `omnesagent-tools/src/memory_recall.rs`: если в выдаче есть пара фактов с ребром `contradicts`, к ответу добавляется блок `[conflict]` с указанием `trust` обеих сторон. Агент видит обе точки зрения и принимает взвешенное решение.

---

### 📦 Пакет 2: Лёгкая база — int8 квантование эмбеддингов (v1.3 Ф24)

#### 2.1. Мотивация и заимствуемый код из `ob2h`
- В `C:\Projects\omnesbot_for_hermes\src\vector\similarity.rs`:
  - `const Q_MAGIC: u8 = 0x01;`
  - `serialize_q(vec: &[f32]) -> Vec<u8>`: `scale = max|v| / 127.0`, `i8 = round(v / scale).clamp(-127, 127)`.
  - `deserialize_q(blob: &[u8]) -> Option<Vec<f32>>`
  - `deserialize(blob: &[u8]) -> Option<Vec<f32>>` (dual-read: если `blob[0] == 0x01` — int8, иначе legacy f32).

#### 2.2. План адаптации в архитектуру OmnesAgent
1. **Обновление векторного слоя**:
   - Файлы: `omnesagent-kag/src/vector/similarity.rs` и `omnesagent-memory/src/vector.rs`.
   - Внедрить `serialize_q` и dual-read `deserialize`.
   - Размер блоба для 384-мерного вектора снижается с **1536 байт** до **389 байт** (1 байт magic + 4 байта scale + 384 байта int8).
   - Погрешность косинусного сходства после квантования $\le 0.005$ (экспериментально подтверждено в тестах ob2h).
2. **Пакетная конвертация существующих данных**:
   - Добавить в `omnesagent-kag/src/vector/mod.rs` метод `quantize_existing_embeddings(&Connection)` для миграции существующих таблиц `graph_nodes` и `memories` в фоне без блокировки работы.

---

### 📦 Пакет 3: Честный prefetch и MMR-диверсификация (v1.3 Ф22)

#### 3.1. Мотивация и заимствуемый код из `ob2h`
- В `C:\Projects\omnesbot_for_hermes\src\memory\service.rs`:
  - `build_context`
  - Формула скоринга: `0.35*rel + 0.25*importance + 0.2*trust + 0.1*recency + 0.1*sat_access`
  - Функция `mmr_select(rels, vecs, limit, lambda)`
  - Отсечение по границам записей под `max_chars` / `max_tokens`.

#### 3.2. План адаптации в архитектуру OmnesAgent
1. **Инжекция в `omnesagent-runtime` и `omnesagent-memory`**:
   - В метод сборки контекста перед передачей в системный промпт агента:
     - Рассчитываем экспоненциальный `recency` с периодом полураспада 90 дней: $\exp(-\text{age\_days} / 90)$.
     - Рассчитываем насыщение использования: $\text{sat\_access} = \text{clamp}(\ln(1 + \text{access\_count}) / \ln(50), 0.0, 1.0)$.
     - Добавляем компонент `trust` (из Пакета 1).
2. **MMR-диверсификация (`Maximal Marginal Relevance`)**:
   - Алгоритм жадного выбора кандидатов с $\lambda = 0.7$:
     $$\text{MMR} = \operatorname*{argmax}_{D_i \in R \setminus S} \left[ \lambda \cdot \text{Score}(D_i) - (1 - \lambda) \max_{D_j \in S} \text{Cosine}(D_i, D_j) \right]$$
   - Исключает дублирование похожих записей в системном промпте.
3. **Бюджет блока**:
   - Лимит 8000 символов с обрезкой строго по границам карточек воспоминаний.
   - Вызов `touch_access` только для тех записей, которые фактически вошли в сформированный блок.

---

### 📦 Пакет 4: Ralph Knowledge Layer (v1.3 Ф26–28)

#### 4.1. Мотивация и заимствуемый код из `ob2h`
- В `C:\Projects\omnesbot_for_hermes\src\ralph\mod.rs`:
  - Структуры и хранилище для `ralph_runs`, `ralph_iterations`, `ralph_findings`, `ast_changes`.
  - Авто-вердикт по exit-code тестов (`tests_summary`).
  - `ralph_context` со сбором reuse-кандидатов из KAG.
  - Staleness-pass (пометка `verdict = 'stale'` при изменении связанных AST-символов).
  - Debt-леджер (`kind = deferred`).

#### 4.2. План адаптации в архитектуру OmnesAgent
1. **Схема БД M6 в `omnesagent-kag/src/schema.rs`**:
   - Создание таблиц:
     - `ralph_runs` (id, project_id, feature_slug, goal, status, created_at, updated_at).
     - `ralph_iterations` (id, run_id, n, hypothesis, plan, result, tests_summary, verdict, created_at).
     - `ralph_findings` (id, run_id, iteration_id, kind, symbol, claim, verdict, meta, created_at).
     - `ast_changes` (id, run_id, iteration_id, project_id, symbol, change_type, old_sig, new_sig, created_at).
2. **Интеграция с существующим крейтом `omnesagent-ralph`**:
   - Связать `LoopDriver` и `RalphKnowledgeService` с таблицами SQLite.
   - Внедрить вычисление AST-дельты при каждом шаге итерации через `ProjectService`.
3. **Регистрация MCP/Runtime инструментов**:
   - `ralph_start` — старт сессии разработки фичи.
   - `ralph_iteration` — фиксация шага с кодом тестов.
   - `ralph_context` — контекстный пакет задачи (спека + негативный опыт + reuse-кандидаты).
   - `ralph_report` — аналитический отчет по итерациям и техническому долгу.

---

### 📦 Пакет 5: Ночной bench-гейт и автоматизация регрессий (v1.4 Ф30 + v1.3 Ф21)

#### 5.1. Мотивация и заимствуемый код из `ob2h`
- В `C:\Projects\omnesbot_for_hermes\src\dream\bench_gate.rs` и `src/cli/bench.rs`:
  - Замер метрик `recall@k` и `MRR` по фиксированному набору `golden.jsonl`.
  - Правило `is_degradation`: падение `recall@5 > 10%` ИЛИ `MRR > 15%`.
  - Автоматический откат дриминга `dream_restore`.

#### 5.2. План адаптации в архитектуру OmnesAgent
1. **Golden-set контур в `omnesagent-kag`**:
   - Создать модуль `omnesagent-kag/src/bench/mod.rs` с набором эталонных запросов и проверочных ключей.
2. **Бенчмарк-гейт в планировщике фоновых задач / Dreaming**:
   - После выполнения фазы дриминга запускается быстрый замер (15 запросов).
   - Если зафиксирована деградация — база восстанавливается из снапшота до дрима, в лог пишется предупреждение. Таймаут бенчмарка не считается деградацией (fail-safe).

---

## 3. План реализации по шагам (Execution Order)

Рекомендуемый порядок выполнения:

1. **Этап 1 (Пакет 1: Trust & Save-time Deduplication)**:
   - Миграция схемы (`trust`, `last_feedback_at`, `deleted_at`).
   - Реализация `COS_IDENTITY >= 0.98` в `SqliteMemory::store`.
   - Инструмент `memory_feedback` + регистрация в Runtime.
   - Conflict-разметка при поиске.
   - *Закрытое тестирование: юнит-тесты дедупликации, фидбека и конфликтов.*

2. **Этап 2 (Пакет 2: Int8 Квантование)**:
   - Реализация `serialize_q` / `deserialize_q` и dual-read.
   - Фоновая утилита квантования.
   - *Закрытое тестирование: roundtrip f32 -> int8 -> f32 с проверкой косинусной погрешности $\le 0.005$.*

3. **Этап 3 (Пакет 3: Честный Prefetch & MMR)**:
   - Расчет формулы скоринга с `recency` и `sat_access`.
   - Реализация MMR-диверсификации ($\lambda = 0.7$).
   - Лимит 8000 символов с отсечением по границам карточек.
   - *Закрытое тестирование: тесты MMR-диверсификации и бюджета символов.*

4. **Этап 4 (Пакет 4: Ralph Knowledge Layer)**:
   - Добавление схемы M6 (`ralph_runs`, `ralph_iterations`, `ralph_findings`, `ast_changes`).
   - Интеграция `RalphKnowledgeService` с KAG AST-сканером.
   - Добавление инструментов `ralph_start`, `ralph_iteration`, `ralph_context`, `ralph_report`.
   - *Закрытое тестирование: цикл 3 итераций (fail -> fail -> green) с генерацией вердиктов.*

5. **Этап 5 (Пакет 5: Ночной Bench-гейт)**:
   - Golden-set harness и замер `recall@k` / `MRR`.
   - Автооткат при деградации в дриминге.
   - *Закрытое тестирование: искусственная деградация памяти и проверка автоотката.*

---

## 4. Контрольный лист верификации (Verification & Closed-Loop Testing)

После каждого этапа выполняются:
- Проверка компиляции: `cargo check --workspace --lib`
- Юнит-тесты затронутых крейтов:
  - `cargo test -p omnesagent-memory --lib`
  - `cargo test -p omnesagent-kag --lib`
  - `cargo test -p omnesagent-tools --lib`
  - `cargo test -p omnesagent-runtime --lib`
  - `cargo test -p omnesagent-ralph --lib`
- Валидация сохранения обратной совместимости (ни один существующий инструмент или контракт WebSocket/HTTP шлюза не ломается).

# Спецификация: Доработки блока памяти OmnesAgent по ob2h v2.0.3 и ребрендинг по брендбуку OmnesProducts v1.1 (Ф14–Ф20)

> **Версия:** 1.0.1 (спецификация и дорожная карта утверждены владельцем)  
> **Дата составления:** 2026-11-14  
> **Статус:** Утверждено владельцем (2026-11-14; продолжает серии `PLAN_OB2H_INTEGRATION_2026-11-06.md` [Ф0–Ф6] и `PLAN_OB2H_INTEGRATION_V1.9.9_2026-11-14.md` [Ф7–Ф13])  
> **Базовые источники:**
> - Спецификация `ob2h` v2.0.3: `C:\Projects\ob2h\PLAN_v2.0.md` (ревизия 2026-11-14, Столпы I–VII, Фазы 69–77)
> - Брендбук экосистемы `OmnesProducts` v1.1: `C:\Projects\OmnesProducts\README.md`, `tools/brandbook_template.html`, `brand/omnes-agent.svg`
> - Контракты OmnesAgent: `docs/adr/0001-ob2h-interop.md`, `docs/kag_sync.md`, `DESIGN_SYSTEM.md`, `.gemini/style-guide.md`
> **Целевые подсистемы:**
> - **Блок памяти / KAG / Ralph (Backend):** `omnesagent-kag`, `omnesagent-memory`, `omnesagent-ralph`, `omnesagent-runtime`, `omnesagent-tools`
> - **Визуальная система и UI (Frontend):** `frontend/shared`, `frontend/desktop`, `frontend/web`, `DESIGN_SYSTEM.md`, `.gemini/style-guide.md`

---

## 1. Контекст, аудит As-Is и дельта `ob2h v2.0.3` + `OmnesProducts v1.1`

### 1.1. Состояние блока памяти и KAG OmnesAgent (As-Is после Ф0–Ф6 и старта Ф7)

Как зафиксировано в [`PLAN_OB2H_INTEGRATION_2026-11-06.md`](PLAN_OB2H_INTEGRATION_2026-11-06.md) и [`PLAN_OB2H_INTEGRATION_V1.9.9_2026-11-14.md`](PLAN_OB2H_INTEGRATION_V1.9.9_2026-11-14.md), в OmnesAgent функционируют два взаимодополняющих SQLite-хранилища (под единым названием «близнец»):
1. **`kag.db`** (`backend/crates/omnesagent-kag/src/schema.rs`, `SCHEMA_VERSION = 9`) — кодовый и графовый KAG-движок (`projects`, `project_files`, `ast_refs`, `graph_nodes`, `graph_edges`, `graph_nodes_fts`, `documents`, `chunks`, `chunks_fts`, `memories`, `memory_relations`, `dream_runs`, `ralph_runs`, `ralph_iterations`, `ralph_findings`, `ast_changes`, опциональный `vec_graph_nodes`).
2. **`brain.db`** (`backend/crates/omnesagent-memory/src/sqlite.rs`) — операционная память агентов (`memories`, `memories_fts`, `embedding_cache`, `memory_meta`), поверх которой работают `conflict.rs`, `dream.rs`, `redact.rs`, `retrieval.rs` и `prefetch.rs`.

В ходе Ф6 (по ранней редакции `ob2h v2.0`) в блок памяти OA уже перенесены:
- В `omnesagent-kag/src/schema.rs`: колонки `access_count` и `last_accessed` в `graph_nodes`, `access_count`, `last_accessed`, `trust`, `last_feedback_at` в `memories`, поля `provenance` и `confidence` в `graph_edges`, таблица `memory_relations`, FTS5 `graph_nodes_fts` (триггер `AFTER UPDATE OF label, description` не реагирует на сброс телеметрии);
- В `omnesagent-kag/src/ast/treesitter.rs`: 2-проходный `TreeSitterEngine` поверх грамматик `inkjet` (18 языков в `TsLanguage`, включая `Dart`), fallback на regex при `FALLBACK_ERROR_RATIO > 0.10`, провенанс `SYNTAX` / `RESOLVED` / `HEURISTIC`;
- В `omnesagent-kag/src/embedding/`: Candle-first `LocalBertEmbedding`, `CachedEmbeddingProvider` (LRU 1000) и базовый `EmbeddingFingerprint { model_id, dim }`;
- В `omnesagent-kag/src/vector/`: статически слинкованный `sqlite-vec` (`vec0.rs`, `vec_graph_nodes`) и `simd.rs` (AVX2+FMA `dot`/`norm_sq`/`cosine`);
- В `omnesagent-kag/src/graph/pagerank.rs`: `NodeUsage`, `usage_weight` (`PPR_ACCESS_CAP = 3.0`, `PPR_GRACE_ACCESSES = 4`, `PPR_RECENCY_HALF_LIFE_SECS = 30d`), `personalized_pagerank_weighted`, батчевый сброс телеметрии `AccessTelemetry` (ADR-O6);
- В `omnesagent-memory/src/conflict.rs`: `ConflictScope`, `supersede_candidate` и `superseded_among`;
- В `omnesagent-runtime/src/agent/orchestrator/tool_group.rs:178-209`: read-only allowlist расширен инструментами `ob2h` v1.9.9 (Ф7.2).

Однако в ревизиях **`ob2h` v2.0.1 – v2.0.3 от 2026-11-14** (`C:\Projects\ob2h\PLAN_v2.0.md`) спецификация v2.0 была существенно углублена новыми математическими инвариантами, алгоритмами индексации и защитными механизмами. Эти улучшения необходимо перенести в близнеца и поддержать в MCP-интеропе OA.

### 1.2. Состояние визуального стиля OmnesAgent (As-Is) vs брендбук `OmnesProducts` v1.1

Верификация фронтенда и бренд-материалов выявила расхождение текущего UI OA с единым брендбуком экосистемы `OmnesProducts` (`C:\Projects\OmnesProducts\tools\brandbook_template.html` и `README.md`):
- **Логотип и иконки:** в [`desktop_titlebar.dart`](frontend/desktop/lib/widgets/desktop_titlebar.dart#L34-L55) (аналогично в `frontend/web`) и [`login_screen.dart`](frontend/web/lib/features/auth/login_screen.dart#L128-L163) используется растровый `assets/Logo/app_launcher.png` со свечением `#00D2FF`, а файл `assets/Logo/OA_icon.svg` в `frontend/desktop` и `frontend/web` содержит **устаревший циановый знак** (`viewBox="0 0 923.8 836.3"`, `fill="#46d0e7"`, щит с крыльями/микросхемами). В брендбуке `OmnesProducts` v1.1 продуктовый знак **Omnes Agent** (`brand/omnes-agent.svg`, `viewBox="0 0 818 815"`) — это плоский 3-цветный вектор: круг **Dark Charcoal (`#111827`)**, слон с хоботом-S (S-Trunk) в цвете **Off-White (`#F4F5F7`)** и единый смысловой акцент **Brand Orange (`#FF6B00`)** — **AR-визор на глазах** («автономное агентное зрение»).
- **Цветовая палитра:** [`ShadcnColors`](frontend/shared/lib/design_system/shadcn_colors.dart), [`DesktopTheme`](frontend/desktop/lib/theme/desktop_theme.dart) и документы [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) / [`.gemini/style-guide.md`](.gemini/style-guide.md) используют Electric Cyan (`#00D2FF` / `#00E5FF` / `#0284C7`) как главный брендовый акцент, тогда как палитра `OmnesProducts` строится на пропорции **60% Dark Charcoal (`#111827`) / 30% Off-White (`#F4F5F7`, границы `#D8DCE3`, muted `#5B6472`) / ≤10% Brand Orange (`#FF6B00`)**.
- **Типографика:** в `DesktopTheme` и `ShadcnTheme` сейчас задан только `fontFamily: 'Segoe UI'` и точечный `Consolas`, без подключения бренд-пары `OmnesProducts` (§06 брендбука): **Montserrat** (SemiBold/Bold) для заголовков и навигационных акцентов, **Inter / Segoe UI** (Regular/Medium) для основного текста и **JetBrains Mono / Consolas** для кода/терминала/диффов.

### 1.3. Сводная таблица выявленных разрывов (P15–P24)

*(Нумерация продолжает P1–P7 старой спеки и P8–P14 спеки v1.9.9.)*

| # | Проблема | Evidence (код / спека) | Приоритет |
|---|---|---|---|
| **P15** | **AST v2.0 (парсер и идентификаторы):** `treesitter.rs` не поддерживает инкрементальное обновление `Tree::edit` с LRU-кэшем деревьев (ob2h 70.8), не извлекает эвристики из `token_tree` макросов (`tokio::select!`, `axum`, декораторы Python, 70.13), не использует канонический SCIP-подобный `node_id` с префиксом `<project_id>:` (70.14), не делает **Generated File Stubbing** для `*.g.dart`, `*.freezed.dart`, `*.pb.dart`, `*.pb.rs`, `*.generated.ts` (70.15) и не разбирает `<script>` в `.vue`/`.svelte` через `inkjet` TS (71.5 — сейчас SFC парсятся только regex в `ast/mod.rs:293`). | `omnesagent-kag/src/ast/treesitter.rs`; `ast/mod.rs:293`; ob2h 70.8, 70.13–70.15, 71.5 | **P0** |
| **P16** | **Call Graph, `TESTED_BY` и Co-Change (`CO_CHANGES_WITH`):** в `treesitter.rs` не извлекаются цепочки вызовов `a.b().c()` (71.1); тестовые узлы (`#[test]`, `#[tokio::test]`, `test()`, `describe()`) не маркируются `kind_detail = "test"`, а `GraphAnalytics::analyze_impact` (`analytics.rs:47`) не возвращает `affected_tests` по `TESTED_BY` и фильтр `min_confidence` (71.2, 71.6); отсутствует анализ `git log` для межфайловых/межъязыковых рёбер `CO_CHANGES_WITH` (`GIT_HISTORY`, Rust ↔ Dart ↔ SQL, 71.8); схема БД в `schema.rs` версии 9 отстаёт от M12 (71.3). | `omnesagent-kag/src/ast/treesitter.rs`; `graph/analytics.rs:47`; `schema.rs:5`; ob2h 71.1–71.3, 71.6, 71.8 | **P0** |
| **P17** | **Конвейер эмбеддингов v2.0:** в `LocalBertEmbedding::embed` (`local_bert.rs:79`) CPU-инференс не вынесен в `spawn_blocking` пул с семафором и батчингом 32/64 (72.1); нет AST-aware нарезки с заголовком `// File: ... \| Scope: ...` (72.2a); `EmbeddingProvider` (`embedding/mod.rs:20`) не несёт асимметричных `query_prefix()`/`document_prefix()` (72.2b); `EmbeddingFingerprint` (`fingerprint.rs:21`) хранит лишь `{ model_id, dim }` без `backend`, `quant`, `prefix_version` (72.3); нет Stale-While-Revalidate (`degraded_mode: "warming_up_embeddings"`, 72.5) и пред-эмбеддинговой маскировки секретов (72.6). | `omnesagent-kag/src/embedding/{mod.rs, local_bert.rs, fingerprint.rs}`; ob2h 72.1–72.6 | **P0** |
| **P18** | **Векторный поиск (Mean-Centered BQ `popcnt`) и `file_blobs` (`blake3`):** в `vector/{simd.rs, vec0.rs}` отсутствует двухступенчатый поиск Stage-1 Mean-Centered Binary Quantization (`[u64; 6]`, 48 байт на 384 dim, Hamming `popcnt` top-400–800 за `<0.5 мс`) → Stage-2 точный SIMD float32 rescore до top-200 (72b.2), что критично при выключенном `vec0` и при анизотропии E5; нет строгого разделения пространств `memories` vs `graph_nodes` по `project_id` (72b.3) и таблицы `file_blobs` (`blake3`) для переключения веток git без ре-эмбеддинга (72b.5). | `omnesagent-kag/src/vector/{simd.rs, vec0.rs}`; `schema.rs`; ob2h 72b.2–72b.5 | **P1** |
| **P19** | **Graph RAG 2.0 (полный U-PPR в продакшне):** `personalized_pagerank_weighted` реализован в `pagerank.rs:170`, но `repomap.rs:16,115` вызывает обычный `personalized_pagerank`, а `GraphService::search` и `repomap` не используют полную формулу v2.0.3: $\lambda_{\text{code}} = 0$ vs $\lambda_{\text{memory}} > 0$, CATD-lite, **Inverse In-Degree Penalty** $\frac{W_0}{\ln(1 + \text{in\_degree}(v))}$ (73.2), **Mute Node** (`trust *= 0.3`, 73.2/73.6), $k$-hop сабграф (5–10k) со стартовым вектором $p_0 = \text{IDF} \times \text{trust}$ и **Active Working Set Boost** (`git diff HEAD`, 73.4), декомпозицию `CamelCase`/`snake_case` в `graph_nodes_fts` (`subtokens`, 73.9), **Elbow Cutoff** ($S_i < 0.15 S_{\text{max}}$ или скачок $>50\%$, 73.8) и `BEGIN IMMEDIATE` + `wal_checkpoint(TRUNCATE)` + `PRAGMA optimize` (73.3). | `omnesagent-kag/src/graph/{pagerank.rs, service.rs, repomap.rs}`; `schema.rs:433`; ob2h 73.2–73.9 | **P0** |
| **P20** | **Dream 2.0 и Ralph Свежесть:** `omnesagent-memory/src/conflict.rs` и `dream.rs` выполняют попарное сравнение без обязательного предфильтра **Entity/Subject Overlap Blocking** (74.2) и без классификации конфликтов на *Temporal Supersession* / *User Correction* / *Contextual Coexistence* (74.1); нет неразрушающих томбстоунов с `valid_until` и проверкой циклов `SUPERSEDES`/`CONTRADICTS` (74.3); записи `memories` и `ralph_findings` не хранят `git_head_sha` для доказательной свежести через `git diff <sha>..HEAD -- <file>` (74.4, 75.1); отсутствуют HITL-предпросмотр `knowledge_reconcile` (74.5), бюджет токенов/времени Dream (74.6) и Rollback Gate по ADR-13 (74.7). | `omnesagent-memory/src/{conflict.rs, dream.rs, sqlite.rs}`; `omnesagent-ralph/src/journal.rs`; ob2h 74.1–74.7, 75.1 | **P1** |
| **P21** | **MCP-интероп v2.0:** в `tool_group.rs` и KAG-клиенте не учтены новые инструменты и поля ответа `ob2h` v2.0 (`ob2h__knowledge_reconcile` как write/HITL, `mute_node` / negative feedback в `ob2h__memory_feedback`, обработка `degraded_mode: "warming_up_embeddings"` без ошибки). | `omnesagent-runtime/src/agent/orchestrator/tool_group.rs:178-209`; ob2h 72.5, 73.2, 74.5 | **P1** |
| **P22** | **Ребрендинг — иконки и логотип:** `frontend/{desktop,web}/assets/Logo/OA_icon.svg` содержит старый циановый логотип (`#46d0e7`), titlebar (`desktop_titlebar.dart:34-55`) и `login_screen.dart:128-163` рисуют растровый `app_launcher.png` со свечением `0xFF00D2FF` вместо векторного слона `omnes-agent.svg` (`#111827` / `#F4F5F7` / `#FF6B00` AR-визор); растровые `app_icon.png`, `app_launcher.png`, `app_icon.ico` и web favicons не пересобраны из нового знака. | `C:\Projects\OmnesProducts\brand\omnes-agent.svg`; `frontend/{desktop,web}/assets/Logo/*`; `desktop_titlebar.dart`; `login_screen.dart`; `app_icon.ico` | **P0** |
| **P23** | **Ребрендинг — цветовая гамма:** `ShadcnColors` (`primary = 0xFF00E5FF`), `DesktopTheme` (`accentCyan = 0xFF00D2FF`, `borderFocusDark = 0xFF00D2FF`), `DESIGN_SYSTEM.md` и `.gemini/style-guide.md` не выровнены под три константы брендбука `OmnesProducts` v1.1 (Dark Charcoal `#111827` ~60%, Off-White `#F4F5F7` ~30%, сигнальный Brand Orange `#FF6B00` ≤10%). | `frontend/shared/lib/design_system/shadcn_colors.dart`; `frontend/{desktop,web}/lib/theme/desktop_theme.dart`; `DESIGN_SYSTEM.md`; `.gemini/style-guide.md` | **P0** |
| **P24** | **Ребрендинг — типографика:** в `pubspec.yaml`, `DesktopTheme` и `ShadcnTheme` отсутствуют бренд-шрифты `OmnesProducts` (§06): **Montserrat** (`SemiBold 600` / `Bold 700`) для заголовков `H1–H3`, топбара и бренд-бейджей, **Inter / Segoe UI** для тела UI и **JetBrains Mono / Consolas** для кода/терминала/таймингов с оранжевым промптом `>_`. | `frontend/{desktop,web,shared}/pubspec.yaml`; `DesktopTheme`; `ShadcnTheme`; brandbook §06 | **P1** |

---

## 2. Цели и Не-цели (Goals & Non-Goals)

### Цели (Goals)
1. **Полный алгоритмический паритет блока памяти с `ob2h` v2.0.3 (Столпы I–VI):** довести `omnesagent-kag`, `omnesagent-memory` и `omnesagent-ralph` до уровня ревизии `PLAN_v2.0.md` v2.0.3 — инкрементальный AST с каноническим `node_id` и stubbing сгенерированных файлов, `CO_CHANGES_WITH` + `TESTED_BY`, изолированный конвейер эмбеддингов с асимметричными префиксами и BQ+SIMD rescore, полная формула U-PPR + Sub-token FTS5 + Elbow Cutoff в продакшн-поиске, Dream 2.0 с 3-уровневой таксономией конфликтов, Entity Overlap и `git_head_sha`.
2. **Согласованность схемы и MCP v2.0 интеропа:** мигрировать `kag.db` на `SCHEMA_VERSION = 10` (эквивалент M12 `ob2h`), обновить аддитивные поля в `brain.db` и адаптировать оркестратор/мост к новым тулам и `degraded_mode` внешнего `ob2h` v2.0.
3. **Системный ребрендинг по `OmnesProducts` v1.1:** перевести логотипы/иконки на векторный знак `omnes-agent.svg` (S-Trunk слон с оранжевым AR-визором), мягко скорректировать цветовую палитру тёмной и светлой тем к триаде **Dark Charcoal `#111827` / Off-White `#F4F5F7` / Brand Orange `#FF6B00`** и подключить шрифтовую систему **Montserrat + Inter/Segoe UI + JetBrains Mono/Consolas** во всех Flutter-пакетах и дизайн-документации.

### Не-цели (Non-Goals)
1. **ONNX C-FFI (`fastembed`/`ort`) по умолчанию в OA:** сохраняется решение владельца из Ф6.2 (#5 [`PLAN_OB2H_INTEGRATION_2026-11-06.md`](PLAN_OB2H_INTEGRATION_2026-11-06.md)) — встроенный `omnesagent-kag` остаётся **Candle-first (zero C-deps)**; при этом `EmbeddingFingerprint` v2 включает поле `backend` (`"candle"` / `"onnx"`) для совместимости форматов с внешним `ob2h`.
2. **Упаковка `ob2h` через `cargo-dist` (Столп VII, Ф76 `ob2h`):** дистрибуция `ob2h` и его команда `self uninstall` относятся исключительно к репозиторию `C:\Projects\ob2h`; инсталляторы OmnesAgent покрываются отдельными планами OA.
3. **Изменение компонентной геометрии и правил UX:** сохраняются 4 уровня высоты (`L0–L3`), таблица отступов `4px/8px`, единая геометрия поповеров/дропдаунов (`radius: 8px`, `offset: (0, 6)`) из [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) и нормативный запрет на эмодзи в UI из [`AGENTS.md`](AGENTS.md) (только SVG через `flutter_svg` / `UiIcon`).

---

## 3. Архитектурные решения (ADR-O12 .. ADR-O18)

*(Продолжают `ADR-O1..O6` из `PLAN_OB2H_INTEGRATION_2026-11-06.md` и `ADR-O7..O11` из `PLAN_OB2H_INTEGRATION_V1.9.9_2026-11-14.md`.)*

### ADR-O12: Гибридный AST v2.0 — канонический SCIP-подобный `node_id` с префиксом `<project_id>:`, LRU `Tree::edit` и Generated File Stubbing
- **Решение:** Формировать `node_id` по детерминированной схеме `<project_id>:<lang>:<module_or_relpath>#<Type>.<member>()` вместо позиционного хэша со строками. Префикс `<project_id>:` сохраняет изоляцию проектов в общем `kag.db` (требование `docs/kag_sync.md` §4.1), а отсутствие номеров строк гарантирует, что переформатирование и `git checkout` не разрывают ссылки в `memories` и `ralph_findings`.
- **Кэш деревьев:** В `ProjectWatcher` и `TreeSitterEngine` вводится LRU-кэш на 64 последних изменённых файла (`tree_sitter::Tree` + исходный буфер для `Tree::edit`), удерживающий расход RAM ниже 15 МБ при `<3 мс` на инкрементальный ре-парсинг остальных файлов.
- **Stubbing и SFC:** Маски `*.g.dart`, `*.freezed.dart`, `*.pb.dart`, `*.pb.rs`, `*_pb2.py`, `*.generated.ts` парсятся в режиме **Generated File Stubbing** (только верхнеуровневые публичные объявления с `provenance = "SYNTAX"`, без извлечения `CallSite` и без нарезки векторных чанков). Для `.vue` и `.svelte` в `treesitter.rs` добавляется экстракция блоков `<script>`/`<script setup>` с прогоном через `TsLanguage::TypeScript`/`JavaScript`.

### ADR-O13: Темпоральная связность (`CO_CHANGES_WITH`) и тестовое покрытие (`TESTED_BY`) в Blast Radius
- **Решение:** В `omnesagent-kag::project::cochange` при `scan_project` (при наличии `.git`) анализируются последние 200 коммитов (`git log -n 200 --name-only --pretty=format:%H`). Коммиты, затрагивающие $>10$ файлов, отбрасываются как шум (массовый реформат/мерж). Пары файлов с $\ge 3$ совместными коммитами получают ненаправленное (или парное) ребро `CO_CHANGES_WITH` в `graph_edges` с `provenance = "GIT_HISTORY"` и `confidence = min(1.0, co_commits / 10.0)`.
- **Тестовый след:** Функции с атрибутами/именами тестов (`#[test]`, `#[tokio::test]`, `#[rstest]`, `test(...)`, `testWidgets(...)`, `describe(...)`, `test_*`) получают `graph_nodes.kind_detail = "test"`. При вызове `GraphAnalytics::analyze_impact` (`analytics.rs`) обратные обходы от изменённого узла по `CALLS`/`IMPORTS` к узлам с `kind_detail = "test"` формируют раздел `affected_tests` (`TESTED_BY`), а параметр `min_confidence: Option<f64>` позволяет отсеивать слабые эвристики.

### ADR-O14: Асинхронный конвейер эмбеддингов v2.0, асимметричные префиксы и `EmbeddingFingerprint` v2
- **Решение:**
  1. Инференс `LocalBertEmbedding` изолируется в `tokio::task::spawn_blocking` под ограниченным `Semaphore` (по числу физических ядер / max 2–4 воркера) и с батчингом 32/64 чанка.
  2. Трейт `EmbeddingProvider` (`embedding/mod.rs`) получает методы `query_prefix(&self) -> &'static str` и `document_prefix(&self) -> &'static str`, а также высокоуровневые методы `embed_query` / `embed_documents` (по умолчанию вызывают `embed` с автоподстановкой префикса; для E5 — `"query: "` / `"passage: "`; для Nomic — `"search_query: "` / `"search_document: "`; для MiniLM/jina — `""`).
  3. Планируется переключение дефолтной модели `LocalBertEmbedding` (`local_bert.rs:39`) с `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` на `intfloat/multilingual-e5-small` (с префиксами `"query: "` / `"passage: "`); переключение фиксируется по результатам триплетного RU/EN-бенчмарка.
  4. `EmbeddingFingerprint` (`embedding/fingerprint.rs`) расширяется до пяти полей: `{ model_id, backend, quant, dim, prefix_version }` (строковый ключ `model_id:backend:quant:dim:prefix_vN`), где смена модели, схемы AST-чанкинга или префикса инвалидирует векторное пространство и требует переиндексации.
  5. На холодном старте модель прогревается неблокирующе; запросы во время загрузки отвечают по FTS5 + U-PPR с признаком `degraded_mode: Some("warming_up_embeddings")`. Перед вызовом любого эмбеддера текст проходит через санитайзер секретов (переиспользование `omnesagent_memory::redact`).

### ADR-O15: Двухступенчатый векторный индекс (Mean-Centered BQ `popcnt` → SIMD Rescore) и `file_blobs` (`blake3`)
- **Решение:**
  - **Ступень 1 (BQ Oversampling):** Для каждого 384-dim вектора вычисляется mean-centered бинарная маска `bit[i] = val[i] > mean[i]` (покоординатное среднее выборки, fallback — `val[i] > vec_mean`) и пакуется в `[u64; 6]` (48 байт; колонка `bq_mask BLOB`). Аппаратный Hamming-поиск `u64::count_ones(a ^ b)` отбирает топ-400–800 кандидатов (фильтрованных строго по пространству имён `memories` vs `graph_nodes` и `project_id`) за `<0.5 мс` на 100k векторов без зависимости от C-FFI.
  - **Ступень 2 (Exact SIMD Rescore):** Выборка топ-400–800 переранжируется точным `simd::cosine` (`f32`) в топ-200 и передаётся в RRF / U-PPR. При включённой фиче `vec0` (`sqlite-vec`) инкрементальный upsert выполняется в `BEGIN IMMEDIATE`, но SIMD/BQ-конвейер остаётся обязательным быстрым путём.
  - **Кэш ветвления `file_blobs`:** Хэш содержимого `blake3` (заменяет/дополняет per-project `sha256` из `project_files`) служит первичным ключом таблицы `file_blobs(blake3_hash, parser_version, embedding_fingerprint, ast_json, vector_ids)`. При `git checkout` переиспользуются готовые AST-узлы и векторы без ре-инференса.

### ADR-O16: Полная формула U-PPR в `GraphService` и `repomap`, Sub-token FTS5 и Elbow Cutoff
- **Решение:** Заменить упрощённые вызовы `personalized_pagerank` в `GraphService::search` и `repomap::build_repo_map` на единый конвейер U-PPR v2.0.3:
  1. **Вес ребра и защита от хабов (`pagerank.rs`):**
     $$W(e) = \frac{W_{\text{base}}(\text{kind}) \cdot \text{conf}(e)}{\Phi_{\text{hub}}(v_{\text{target}})} \cdot \min\Big(1 + \ln(1 + \text{access\_count}(v_{\text{target}})),\ 3.0\Big) \cdot e^{-\lambda_{\text{eff}} \cdot \Delta t} \cdot \left(0.1 + 0.9 \cdot \text{trust}(v_{\text{target}})\right)$$
     - Для узлов кода: $\lambda_{\text{code}} = 0$ (код не стареет по часам), $\Phi_{\text{hub}}(v) = \ln(e + \text{in\_degree}(v))$ (HippoRAG In-Degree Penalty против хабов `Error`, `Result`, `Serialize`);
     - Для узлов памяти: $\lambda_{\text{memory}} = \frac{\lambda_0}{1 + \beta \ln(1 + \text{deg}(v))}$ (CATD-lite), $\Phi_{\text{hub}} = 1$;
     - Cold-start grace period: при `access_count < 4` множитель популярности фиксируется равным `1.0`.
     - **Mute Node:** разовый негативный фидбек умножает `trust` на `0.3`, мгновенно опуская узел через $(0.1 + 0.9 \cdot \text{trust})$.
  2. **Сабграф и персонализация $p_0$:** Извлекается $k$-hop сабграф кандидатов (5k–10k узлов) вокруг лексических (FTS5) и векторных seeds; узлы со статусом `SUPERSEDED` исключаются из $p_0$. Стартовый вес $p_0(v) = \text{IDF}(v) \cdot \text{trust}(v) \cdot B_{\text{git}}(v)$, где $B_{\text{git}}(v) = 1.2$ для узлов из файлов текущего `git diff HEAD` (Active Working Set Boost).
  3. **Sub-token FTS5:** Колонка `subtokens TEXT` в `graph_nodes` заполняется декомпозицией `CamelCase`/`snake_case` (например, `"EmbeddingWorkerPool embedding worker pool"`) и индексируется в `graph_nodes_fts`.
  4. **Elbow Cutoff в `repomap.rs`:** При упаковке AST-скелета в `max_tokens` итерация прерывается досрочно, если вес очередного узла $S_i < 0.15 \cdot S_{\text{max}}$ или падение относительно предыдущего $> 50\%$ ($S_i < 0.5 \cdot S_{i-1}$).
  5. **WAL-обслуживание:** Пакетный сброс `AccessTelemetry` выполняется в `BEGIN IMMEDIATE` с `busy_timeout = 5000`; при завершении цикла Dream и graceful shutdown Gateway вызываются `PRAGMA wal_checkpoint(TRUNCATE); PRAGMA optimize;`.

### ADR-O17: Трёхуровневая таксономия конфликтов памяти, Entity Overlap блокинг и Git-Anchored Свежесть
- **Решение:**
  1. Перед оценкой семантической схожести в `omnesagent-memory/src/conflict.rs` вводится **обязательный Entity/Subject Overlap Blocking**: два факта внутри одного `ConflictScope` (`namespace`, `tenant_id`, `project_id`) сравниваются на конфликт только при непустом пересечении нормализованных сущностей (имён символов, путей к файлам, идентификаторов модулей). Это исключает ложные коллизии между однотипными фактами разных подсистем («таймаут Auth 5с» vs «таймаут Billing 5с»).
  2. Классификатор (Offline эвристики предикатов по умолчанию + опциональный Online LLM micro-prompt) относит пару к одному из трёх типов: `TemporalSupersession` (замещение во времени), `UserCorrection` (явная правка пользователя — высший приоритет) или `ContextualCoexistence` (разные контексты — оба факта остаются активными).
  3. Разрешение не удаляет записи физически: старой записи проставляется `valid_until` / `superseded_by` и создаётся ребро `SUPERSEDES` или `CONTRADICTS` в `memory_relations` с проверкой отсутствия циклов ($A \to B \to A$). Для `pinned = true` записей авто-замещение запрещено — формируется HITL-отчёт для `knowledge_reconcile`.
  4. Код-связанные факты и `ralph_findings` сохраняют `git_head_sha`; при проверке свежести пустой `git diff <sha>..HEAD -- <file>` доказывает актуальность безотносительно возраста записи. При падении golden-метрик на $>10\%$ после Dream срабатывает **Rollback Gate (ADR-13)**.

### ADR-O18: Визуальная и шрифтовая гармонизация OmnesAgent с брендбуком `OmnesProducts` v1.1
- **Решение:** Провести эволюционный ребрендинг UI и дизайн-документации при полном сохранении компонентной архитектуры (`L0–L3`, поповеры `4–6px` gap, `ui_icons.dart` без эмодзи):
  1. **Единый векторный знак:** Во всех клиентах (`frontend/desktop`, `frontend/web`, `frontend/assets`) старый циановый щит (`OA_icon.svg` `#46d0e7` и растровые `app_icon.png`/`app_launcher.png`/`app_icon.ico`) заменяется на эталон `C:\Projects\OmnesProducts\brand\omnes-agent.svg` (слон S-Trunk в круге `#111827`, тело `#F4F5F7`, оранжевый AR-визор `#FF6B00`), отображаемый в UI через `SvgPicture.asset`. По брендбуку §07 минимальный размер в UI/трее — `16×16 px` (`24 px` для уверенного чтения AR-визора), охранное поле — $\frac{1}{4} D$. На тёмных фонах допускается светлая подложка-плашка (`#F4F5F7`) при необходимости отделения угольного круга от фона.
  2. **Палитра 60 / 30 / 10:**
     - **60% Base (Dark Charcoal):** тёмная тема перебалансируется вокруг `#111827` (`L0 #0D1117`, `L1 #111827`, `L2 #1B2434`, `L3 #161F2E` с бордером `#283244`).
     - **30% Paper / Neutral (Off-White):** светлая тема перебалансируется вокруг `#F4F5F7` (`L0 #F4F5F7`, `L1 #FAFBFC`, `L2/L3 #FFFFFF`, бордер `#D8DCE3` `--line`, muted `#5B6472` `--muted`, primary text `#111827`).
     - **≤10% Signal Accent (Brand Orange `#FF6B00`):** заменяет Electric Cyan (`#00D2FF`) во всех брендовых и фокусных слотах (`borderFocus`, активная вкладка/режим агента, промпт `>_`, бейдж `DESKTOP`/`WEB ADE`, индикаторы выделения); в светлой теме для контрастных акцентов используется `#E05E00` / `#FF6B00`. В тёмной теме `DesktopTheme.btnPrimaryBg` остаётся графитовым (`#1F2937` с текстом `#F4F5F7` и фокусом/ховером `#FF6B00`), тогда как сплошная заливка `#FF6B00` резервируется строго для главной акцентированной CTA-кнопки («Отправить»/«Запустить»). Семантические цвета статусов (`#10B981`, `#F59E0B`, `#EF4444`) и информационный синий кода (`#38BDF8` / `#0284C7`) сохраняются.
  3. **Типографическая триада:** `Montserrat` (`SemiBold 600` / `Bold 700`, трекинг `0…+2%`) — заголовки экранов/секций, топбар `OmnesAgent`; `Inter` / `Segoe UI` (`Regular 400` / `Medium 500`) — основной текст, меню, формы; `JetBrains Mono` / `Consolas` (`Regular 400` / `Medium 500`) — код, терминал, git-ветки, тайминги, шорткаты (с акцентом промпта `>_` в `#FF6B00`).

---

## 4. Детальный план работ по фазам (Ф14–Ф20)

### Фаза 14 (Ф14): Гибридный полиглот-AST v2.0 и канонический `node_id` (Столп I, аналог Ф70 ob2h)
*Целевые крейты/файлы:* `backend/crates/omnesagent-kag/src/ast/{mod.rs, treesitter.rs}`, `backend/crates/omnesagent-kag/src/project/{mod.rs, watcher.rs}`

- [x] **Ф14.1. Инкрементальный парсинг `Tree::edit` и LRU-кэш деревьев (64 файла):** в `treesitter.rs` и `project/watcher.rs` кэшировать пары `(Tree, prev_source)` для 64 последних изменённых файлов в `LruCache`. При изменении файла вычислять `InputEdit` и вызывать `parser.parse(new_src, Some(&old_tree))`; для остальных файлов — полный ре-парсинг без раздувания RAM.
- [x] **Ф14.2. Эвристика макросов и декораторов (`token_tree`):** расширить `treesitter.rs` рекурсивным сканированием `macro_invocation` в Rust (`tokio::select!`, `axum` роутеры, `sqlx::query!`, `tracing::*`) и аргументов декораторов в Python/TS; найденные вызовы/связи фиксируются как рёбра с `provenance = PROVENANCE_HEURISTIC` и `confidence = 0.65`.
- [x] **Ф14.3. Канонический SCIP-подобный `node_id`:** перевести генерацию `node_id` в `treesitter.rs` и `ast/mod.rs` на детерминированный формат `<project_id>:<lang>:<module_or_relpath>#<Type>.<member>()` (без зависимости от `line_start`), добавить миграционный алиас-маппер для сохранения старых ссылок в `ralph_findings`.
- [x] **Ф14.4. Generated File Stubbing:** добавить фильтр расширений `*.g.dart`, `*.freezed.dart`, `*.pb.dart`, `*.pb.rs`, `*_pb2.py`, `*.generated.ts`, `.d.ts` (опционально) в `treesitter.rs` и `ast/mod.rs` — извлекаются только публичные верхнеуровневые сигнатуры (`class`, `struct`, `fn`, `enum`) с `provenance = "SYNTAX"`, пропуская парсинг тел, `CallSite` и чанкинг для векторизации.
- [x] **Ф14.5. Инжекция `<script>` в SFC (`.vue` / `.svelte`):** добавить расширения `vue` и `svelte` в `TsLanguage::from_path` (`treesitter.rs:57`) с извлечением содержимого `<script>` / `<script setup>` (с учётом `lang="ts"`) и парсингом через `TsLanguage::TypeScript` / `JavaScript` с коррекцией смещения строк.

*DoD Ф14:* `cargo test -p omnesagent-kag --features treesitter --lib ast` проходит; тест подтверждает стабильность `node_id` после вставки пустых строк/комментариев; скан `frontend/` не создаёт `CallSite` и чанки внутри `*.g.dart`/`*.freezed.dart`.

---

### Фаза 15 (Ф15): Глубокий Call Graph, `TESTED_BY`, `CO_CHANGES_WITH` и схема KAG v10 (Столп I, аналог Ф71 ob2h + M12)
*Целевые крейты/файлы:* `backend/crates/omnesagent-kag/src/{schema.rs, ast/treesitter.rs, graph/analytics.rs, project/cochange.rs, project/mod.rs}`

- [x] **Ф15.1. Цепочки вызовов (`obj.method().next()`):** расширить `treesitter.rs` разбором вложенных CST `call_expression` / `method_call_expression` с эмиссией звеньев цепочки (`provenance = PROVENANCE_HEURISTIC`, `confidence = 0.75`).
- [x] **Ф15.2. Маркировка тестовых узлов (`kind_detail = "test"`) и `TESTED_BY`:** распознавать атрибуты/декораторы и паттерны тестов Rust (`#[test]`, `#[tokio::test]`, `#[rstest]`), Dart (`test(`, `testWidgets(`, `group(`), TS/JS (`describe(`, `it(`, `test(`), Python (`def test_*`), Go (`func Test*`); проставлять `graph_nodes.kind_detail = "test"`.
- [x] **Ф15.3. Расширение `GraphAnalytics::analyze_impact` (`analytics.rs:47`):** добавить в `ImpactReport` поле `affected_tests: Vec<AffectedTest>` (обратные пути по `CALLS`/`IMPORTS` до узлов `kind_detail == "test"`, концептуально `TESTED_BY`) и параметр фильтрации `min_confidence: Option<f64>` для отсечения слабых эвристических рёбер.
- [x] **Ф15.4. Граф совместных изменений (`CO_CHANGES_WITH`):** создать `backend/crates/omnesagent-kag/src/project/cochange.rs`, запускающий `git log -n 200 --name-only --pretty=format:%H` (таймаут 3 с, кэш последнего обработанного `HEAD` SHA в таблице `kv`); коммиты с `> 10` файлами отбрасываются; пары файлов с `co_commits >= 3` получают рёбра `CO_CHANGES_WITH` между узлами `File` с `provenance = "GIT_HISTORY"` и `confidence = (co_commits as f64 / 10.0).min(1.0)`.
- [x] **Ф15.5. Миграция KAG `SCHEMA_VERSION` 9 → 10 (аналог M12 ob2h, Приложение A):** обновить `omnesagent-kag/src/schema.rs` в единой транзакции с идемпотентным проверками `table_has_column` / `CREATE TABLE IF NOT EXISTS`:
  - `graph_nodes`: `kind_detail TEXT`, `bq_mask BLOB`, `subtokens TEXT`;
  - `memories` (в `kag.db` и аддитивно в `brain.db`): `git_head_sha TEXT`, `valid_until TEXT`, `bq_mask BLOB`;
  - `ralph_findings`: `git_head_sha TEXT`;
  - Новая таблица `file_blobs (blake3_hash TEXT PRIMARY KEY, parser_version TEXT NOT NULL, embedding_fingerprint TEXT NOT NULL, ast_json TEXT NOT NULL, vector_ids TEXT NOT NULL, updated_at TEXT NOT NULL)`.

*DoD Ф15:* миграция существующей БД v9 → v10 идемпотентна и сохраняет 100% строк; `analyze_impact` возвращает релевантные тесты в `affected_tests`; на тестовом git-репозитории строится `CO_CHANGES_WITH` между разными языками (напр., `.rs` ↔ `.dart`).

---

### Фаза 16 (Ф16): Векторный конвейер v2.0, Mean-Centered BQ `popcnt` и кэш `file_blobs` (Столпы II и V, аналог Ф72 + Ф72b ob2h)
*Целевые крейты/файлы:* `backend/crates/omnesagent-kag/{Cargo.toml, src/embedding/*, src/vector/*, src/project/mod.rs}`

- [x] **Ф16.1. Worker pool для `LocalBertEmbedding`:** перевести инференс в `embedding/local_bert.rs` на `tokio::task::spawn_blocking` с ограничивающим `Arc<tokio::sync::Semaphore>` и микро-батчами по 32/64 чанка, чтобы CPU-расчёт не блокировал Tokio-потоки шлюза и MCP.
- [x] **Ф16.2. AST-aware чанкинг (`embedding/chunker.rs`):** нарезать код строго по границам AST-узлов (`function_item`, `impl_item`, `class_definition`, `struct_item`) вместо скользящего окна строк, с шапкой `// File: <rel_path> | Scope: <canonical_scope>` и пропуском stub-файлов (Ф14.4).
- [x] **Ф16.3. Асимметричные префиксы, `EmbeddingFingerprint` v2 и переход на `intfloat/multilingual-e5-small`:**
  - В трейте `EmbeddingProvider` (`embedding/mod.rs`) добавить `query_prefix(&self) -> &'static str`, `document_prefix(&self) -> &'static str`, а также `embed_query` / `embed_documents`;
  - Расширить `EmbeddingFingerprint` (`embedding/fingerprint.rs`) полями `backend: String`, `quant: String`, `prefix_version: String` (`format!("{}:{}:{}:{}:{}", model_id, backend, quant, dim, prefix_version)`); при несовпадении отпечатка запускать фоновый `reindex_project_embeddings`;
  - Подготовить и выполнить переключение дефолтной модели `LocalBertEmbedding` (`local_bert.rs:39`) с `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` на `intfloat/multilingual-e5-small` (с префиксами `"query: "` / `"passage: "`) по результатам триплетного RU/EN-бенчмарка.
- [x] **Ф16.4. Stale-While-Revalidate и санитайзер секретов:**
  - Если `LocalBertEmbedding` загружает веса с диска (cold start), первый поисковый запрос не ждёт загрузки, а немедленно возвращает результаты FTS5 + U-PPR с флагом `degraded_mode: Some("warming_up_embeddings")`;
  - Перед любым `embed_documents`/`embed` прогонять тексты через пред-эмбеддинговый санитайзер секретов (синхронизированный с расширенными паттернами `omnesagent_memory::redact`).
- [x] **Ф16.5. Двухступенчатый поиск Mean-Centered BQ `popcnt` → SIMD Rescore (`vector/bq.rs`, `vector/simd.rs`):**
  - Реализовать расчёт и упаковку `bq_mask`: бит `i = (val[i] > mean[i])` (или `val[i] > vec_mean` как fallback) в `[u64; 6]` (48 байт для 384 dim; для 768 dim `[u64; 12]`);
  - Stage-1: сканирование `bq_mask` через `(a[w] ^ b[w]).count_ones()` для отбора top-400–800 кандидатов в рамках одного пространства имён (`memories` vs `graph_nodes`, фильтр по `project_id`);
  - Stage-2: точный `simd::cosine` по `f32`-блобам шортлиста до итогового top-200;
  - При активной фиче `vec0` сохранять инкрементальный upsert в `BEGIN IMMEDIATE`, используя BQ+SIMD как основной/ускоренный путь без зависимости от C-FFI.
- [x] **Ф16.6. Content-addressed кэш `file_blobs` (`blake3`):** добавить зависимость `blake3` в `omnesagent-kag/Cargo.toml` и интегрировать чтение/запись `file_blobs` в `ProjectService::scan_project` (`project/mod.rs`) — при совпадении `blake3_hash + parser_version + embedding_fingerprint` переключение git-веток переиспользует AST и векторы без ре-инференса.

*DoD Ф16:* Unit-тесты `EmbeddingFingerprint` v2 и BQ-упаковки (`50%±15%` плотность единиц на E5/MiniLM); Stage-1 BQ-поиск по 100k синтетических векторов в тесте/бенче укладывается в `< 0.5 мс`; повторное сканирование после переключения ветки переиспользует кэш `file_blobs`.

---

### Фаза 17 (Ф17): Graph RAG 2.0, полная формула U-PPR, Sub-token FTS5 и Elbow Cutoff (Столп III, аналог Ф73 ob2h)
*Целевые крейты/файлы:* `backend/crates/omnesagent-kag/src/{graph/pagerank.rs, graph/service.rs, graph/repomap.rs, schema.rs}`, `backend/crates/omnesagent-memory/src/sqlite.rs`

- [x] **Ф17.1. Полная формула U-PPR и Mute Node (`graph/pagerank.rs`):**
  - Добавить в `NodeUsage` поле `is_code: bool` и `in_degree: usize` (или `EdgeUsageContext`);
  - Для `is_code == true` применить $\lambda_{\text{code}} = 0.0$ (нулевое временное затухание) и **Inverse In-Degree Penalty** $\Phi_{\text{hub}}(v) = \ln(e + \text{in\_degree}(v))$;
  - Для `is_code == false` (память) реализовать CATD-lite затухание $\lambda_{\text{eff}} = \frac{\lambda_0}{1 + \beta \ln(1 + \text{degree}(v))}$;
  - Реализовать helper/метод `mute_node(conn, node_or_memory_id)` с мгновенным штрафом `trust *= 0.3` (и/или `confidence *= 0.3` с полом `0.05`) и обновлением `last_feedback_at`.
- [x] **Ф17.2. Сабграф кандидатов (5k–10k) и взвешенная персонализация $p_0$ (`graph/service.rs`):**
  - Перевести `GraphService::search` и `GraphService::reason` на извлечение $k$-hop сабграфа вокруг FTS5 + BQ/SIMD seeds (жёсткий потолок `10_000` узлов, исключая `SUPERSEDED`);
  - Формировать стартовый вектор $p_0(v) = \text{IDF}(v) \cdot \text{trust}(v) \cdot B_{\text{git}}(v)$, где $B_{\text{git}}(v) = 1.2$ (+20%) для файлов из текущего `git diff HEAD --name-only` (**Active Working Set Boost**);
  - Передавать `node_weights` (через обновлённый `usage_weight`) в `personalized_pagerank_weighted`.
- [x] **Ф17.3. Sub-token декомпозиция в FTS5 (`schema.rs`, `project/mod.rs`):**
  - Реализовать декомпозицию идентификаторов `split_subtokens("EmbeddingWorkerPool") -> "EmbeddingWorkerPool embedding worker pool"`, `split_subtokens("parse_source_file") -> "parse_source_file parse source file"`;
  - Писать результат в `graph_nodes.subtokens` и включить `subtokens` в виртуальную таблицу `graph_nodes_fts` (и её триггеры `INSERT`/`UPDATE OF label, description, subtokens`).
- [x] **Ф17.4. Токен-бюджетный AST-скелет и Elbow Cutoff (`graph/repomap.rs`):**
  - Перевести `build_repo_map` с базового `personalized_pagerank` (`repomap.rs:16`) на `personalized_pagerank_weighted`;
  - Обогатить рендер узла компактной строкой вызываемых зависимостей с их `confidence` (`file -> signature -> [calls]`);
  - Добавить **Elbow / Drop-off Cutoff**: досрочно завершать упаковку блоков в `max_tokens`, когда скор узла $S_i < 0.15 \cdot S_{\text{max}}$ или соседний скачок $S_i < 0.5 \cdot S_{i-1}$.
- [x] **Ф17.5. WAL-гигиена и `BEGIN IMMEDIATE` (`service.rs`, `memory/sqlite.rs`):**
  - Задать единый контракт PRAGMA на всех открытиях БД (`journal_mode=WAL; synchronous=NORMAL; busy_timeout=5000;`);
  - Перевести `apply_graph_access_batch` (`service.rs:94`) и `apply_access_batch` (`memory/sqlite.rs:66`) на транзакции `BEGIN IMMEDIATE` (через `rusqlite::TransactionBehavior::Immediate`);
  - В конце цикла Dream и при graceful shutdown шлюза выполнять `PRAGMA wal_checkpoint(TRUNCATE); PRAGMA optimize;`.

*DoD Ф17:* запрос `"worker pool"` находит `EmbeddingWorkerPool` через `graph_nodes_fts`; узлы с высокой степенью входа (`Error`, `Result`) подавляются In-Degree Penalty и не вытесняют доменные символы; `mute_node` моментально опускает шумный узел на следующем запросе; `repomap` останавливается по Elbow Cutoff на узком запросе.

---

### Фаза 18 (Ф18): Dream 2.0, разрешение противоречий, Git-Anchored Свежесть и MCP v2.0 интероп (Столпы IV и VI, аналог Ф74–Ф75 ob2h)
*Целевые крейты/файлы:* `backend/crates/omnesagent-memory/src/{conflict.rs, dream.rs, sqlite.rs}`, `backend/crates/omnesagent-ralph/src/{journal.rs, loop_driver.rs}`, `backend/crates/omnesagent-runtime/src/agent/orchestrator/tool_group.rs`

- [x] **Ф18.1. Трёхуровневая таксономия конфликтов и Entity Overlap Blocking (`memory/conflict.rs`):**
  - Ввести enum `ConflictKind { TemporalSupersession, UserCorrection, ContextualCoexistence }`;
  - Добавить экстрактор нормализованных сущностей/путей `extract_entities(text)` и предфильтр **Entity/Subject Overlap Blocking**: проверка на противоречие и расчёт схожести (cosine $> 0.82$ / Jaccard) запускаются *только* если в пределах одного `ConflictScope` пары имеют $\ge 1$ общую сущность/файл/символ;
  - Двухрежимный детектор: Offline эвристический классификатор предикатов и маркеров правки пользователя (default) + опциональный Online LLM micro-prompt.
- [x] **Ф18.2. Неразрушающие томбстоуны и защита от циклов (`memory/dream.rs`, `kag::schema`):**
  - Вместо перезаписи старого факта проставлять `valid_until` / `superseded_by` и создавать направленное ребро `SUPERSEDES` или `CONTRADICTS` в `memory_relations`;
  - Перед записью ребра $A \to B$ проверять достижимость $B \rightsquigarrow A$ для исключения циклов; исключать `SUPERSEDED` записи из стартового $p_0$ при поиске.
- [x] **Ф18.3. Git-Anchored Facts & Findings (`git_head_sha`):**
  - Добавить опциональное поле `git_head_sha: Option<String>` в `FindingRecord` (`omnesagent-ralph/src/journal.rs:75`), таблицу `ralph_findings` и метаданные код-связанных записей `memories`;
  - В `apply_staleness_pass` (Ralph) и в Dream проверять `git diff <sha>..HEAD -- <file>`: если diff пуст — факт/finding подтверждён свежим независимо от возраста; если файл существенно изменён — приоритет перепроверки (в Ralph перевод в `stale`).
- [x] **Ф18.4. HITL `knowledge_reconcile`, бюджет Dream и Rollback Gate (ADR-13):**
  - Для записей с `pinned = true` (и эквивалентов SOUL/USER/SYSTEM) Dream 2.0 формирует только предложение `ReconcileProposal` в отчёте `DreamReport` / инспекции без авто-изменения;
  - Ограничить цикл Dream жёстким лимитом времени (`max_duration_secs`) и токенов (`max_tokens_budget`);
  - Перед фиксацией изменений Dream прогонять quick-check `omnesagent-kag::bench::evaluate_gate_decision`: при падении `recall@5 > 10%` или `MRR > 15%` откатывать изменения транзакции.
- [x] **Ф18.5. Обновление MCP-интеропа с внешним `ob2h` v2.0 (`tool_group.rs`, `ob2h_bridge`):**
  - Классифицировать в `tool_group.rs` потенциальный новый инструмент `ob2h__knowledge_reconcile` (read/preview → allowlist только для dry-run или write/HITL класс в `EngineeringTask`); убедиться, что `ob2h__memory_feedback` корректно прокидывает негативный фидбек/mute;
  - В адаптере ответов `ob2h` MCP обрабатывать поле `degraded_mode: "warming_up_embeddings"` без генерации ошибки агента (логировать диагностикой и возвращать частичную выдачу).

*DoD Ф18:* Unit-тесты доказывают, что факты разных модулей («таймаут Auth 5с» vs «таймаут Billing 5с») без Entity Overlap не конфликтуют; `pinned`-записи не перезаписываются; циклы $A \to B \to A$ отклоняются; `git diff <sha>..HEAD` корректно сохраняет валидность старых подтверждённых findings на неизменных файлах.

---

### Фаза 19 (Ф19): Ребрендинг OmnesAgent по брендбуку `OmnesProducts` v1.1 (Логотип, Цветовая гамма, Типографика)
*Целевые пакеты и файлы:* `frontend/shared/`, `frontend/desktop/`, `frontend/web/`, `DESIGN_SYSTEM.md`, `.gemini/style-guide.md`

- [x] **Ф19.1. Замена значка и бренд-ассетов на эталон `omnes-agent.svg` (S-Trunk + AR-визор `#FF6B00`):**
  - Скопировать `C:\Projects\OmnesProducts\brand\omnes-agent.svg` (`viewBox="0 0 818 815"`, круг `#111827`, слон `#F4F5F7`, AR-визор `#FF6B00`) в качестве нового векторного знака в:
    - `frontend/desktop/assets/Logo/OA_icon.svg`
    - `frontend/web/assets/Logo/OA_icon.svg`
    - `frontend/assets/Logo/OA_icon.svg`
  - Перегенерировать производные растровые иконки (без градиентов и свечений, прозрачный фон / скругление по платформе):
    - `app_icon.png`, `app_launcher.png`, `basic_logo_dark.png`, `basic_logo_light.png` в `frontend/desktop/assets/Logo/`, `frontend/web/assets/Logo/`, `frontend/assets/Logo/`;
    - Windows-иконка окна/exe: `frontend/desktop/windows/runner/resources/app_icon.ico`;
    - Web-иконки: `frontend/web/web/favicon.png`, `frontend/web/web/icons/Icon-192.png`, `Icon-512.png`, `Icon-maskable-*`.
  - В [`frontend/desktop/lib/widgets/desktop_titlebar.dart`](frontend/desktop/lib/widgets/desktop_titlebar.dart#L34-L55), [`frontend/web/lib/widgets/desktop_titlebar.dart`](frontend/web/lib/widgets/desktop_titlebar.dart#L34-L55) и [`frontend/web/lib/features/auth/login_screen.dart`](frontend/web/lib/features/auth/login_screen.dart#L128-L163) заменить `Image.asset('assets/Logo/app_launcher.png')` на `SvgPicture.asset('assets/Logo/OA_icon.svg')` (`flutter_svg` уже подключён в `pubspec.yaml`), убрать циановое свечение `0xFF00D2FF` и в тёмном контексте учитывать светлую плашку / контрастный контур по §03 брендбука.
- [x] **Ф19.2. Корректировка цветовой гаммы (Dark Charcoal `#111827` + Off-White `#F4F5F7` + Brand Orange `#FF6B00`):**
  - Обновить [`frontend/shared/lib/design_system/shadcn_colors.dart`](frontend/shared/lib/design_system/shadcn_colors.dart):
    - Сместить фоны к Dark Charcoal (`background = Color(0xFF0D1117)`, `surface = Color(0xFF111827)`, `card = Color(0xFF18202F)`, `cardElevated = Color(0xFF1F2937)`, `border = Color(0xFF283244)`, `foreground = Color(0xFFF4F5F7)`, `foregroundMuted = Color(0xFF94A3B8)`, `foregroundSubtle = Color(0xFF5B6472)`);
    - Перевести брендовый акцент с цианового на Brand Orange (`primary = Color(0xFFFF6B00)`, `borderActive = Color(0xFFFF6B00)`, `primaryMuted = Color(0xFFE05E00)`, `primaryGlow = Color(0x26FF6B00)`), сохранив `info = Color(0xFF38BDF8)` и статусы `success` (`#10B981`), `warning` (`#F59E0B`), `destructive` (`#EF4444`).
  - Обновить [`frontend/desktop/lib/theme/desktop_theme.dart`](frontend/desktop/lib/theme/desktop_theme.dart) и [`frontend/web/lib/theme/desktop_theme.dart`](frontend/web/lib/theme/desktop_theme.dart) (унифицировав расхождения светлой темы между desktop и web!):
    - **Dark Theme (вокруг `#111827`):** `bgSidebarDark = Color(0xFF0D1117)` (L0), `bgCanvasDark = Color(0xFF111827)` (L0/L1), `bgSurfaceDark = Color(0xFF161E2E)` (L1), `bgSurfaceElevatedDark = Color(0xFF1F2937)` (L2), `borderSubtleDark = Color(0xFF232C3D)`, `borderMediumDark = Color(0xFF334155)`, `borderFocusDark = Color(0xFFFF6B00)`, `textPrimaryDark = Color(0xFFF4F5F7)`, `btnPrimaryBg = Color(0xFF1F2937)` (графитовый с текстом `#F4F5F7` и фокусом/ховером `#FF6B00`; сплошной `#FF6B00` — только для главной акцентированной CTA-кнопки «Отправить»/«Запустить»);
    - **Light Theme (вокруг `#F4F5F7`):** `bgSidebarLight = Color(0xFFF4F5F7)` (brand `--paper`), `bgSurfaceLight = Color(0xFFFAFBFC)`, `bgCanvasLight = Color(0xFFFFFFFF)`, `bgSurfaceElevatedLight = Color(0xFFFFFFFF)`, `borderSubtleLight = Color(0xFFD8DCE3)` (brand `--line`), `borderMediumLight = Color(0xFF9AA3B2)`, `borderFocusLight = Color(0xFFE05E00)`, `textPrimaryLight = Color(0xFF111827)` (brand Dark Charcoal), `textSecondaryLight = Color(0xFF1E293B)`, `textMutedLight = Color(0xFF5B6472)` (brand `--muted`), `btnPrimaryBg` в светлой теме `#111827` (Charcoal) с текстом `#F4F5F7`;
    - **Брендовые акценты:** ввести `accentBrand = Color(0xFFFF6B00)` и `accentBrandDark = Color(0xFFE05E00)`, перенаправить/депрекейтнуть прямые брендовые вхождения `0xFF00D2FF` и `accentCyan` в виджетах (сохраняя `accentSky = Color(0xFF38BDF8)` / `accentSapphire = Color(0xFF0284C7)` только для вторичного моноширинного кода/ссылок). Контролировать долю оранжевого на экране по правилу брендбука ($\le 10\%$ площади).
- [x] **Ф19.3. Внедрение типографики брендбука (`Montserrat` + `Inter`/`Segoe UI` + `JetBrains Mono`/`Consolas`):**
  - Добавить локальные файлы шрифтов в `frontend/shared/assets/fonts/` (или `frontend/{desktop,web}/assets/fonts/`) и объявить `fonts:` в `pubspec.yaml`:
    - `Montserrat`: SemiBold (w600), Bold (w700) — офлайн, без зависимости от Google Fonts по сети;
    - `JetBrains Mono`: Regular (w400), Medium (w500), SemiBold (w600).
  - В `DesktopTheme` и `ShadcnTheme` определить константы `fontHeading = 'Montserrat'`, `fontBody = 'Segoe UI'` (fallback `['Inter', 'Roboto', 'sans-serif']`), `fontMono = 'JetBrains Mono'` (fallback `['Consolas', 'monospace']`) и настроить `textTheme` (`display*`, `headline*`, `title*` → `Montserrat`; `body*`, `label*` → `Inter`/`Segoe UI`), а также применить `fontHeading` к заголовку `OmnesAgent` в `desktop_titlebar.dart` и `login_screen.dart` и `fontMono` к путям, веткам, диффам и промптам `>_`.
- [x] **Ф19.4. Синхронизация документации дизайн-системы:**
  - Обновить [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) до версии 2.1.0 (*Unified Charcoal-Orange Standard*, токены `L0–L3`, 60/30/10, замена `#00D2FF` на `#FF6B00`, типографика §6 на `Montserrat` / `Inter` / `JetBrains Mono`);
  - Обновить [`.gemini/style-guide.md`](.gemini/style-guide.md) (убрать упоминание «титанового щита с неоновыми крыльями и цианом `#00D2FF`», зафиксировать новую палитру, типографику и знак `Omnes Agent`).

*DoD Ф19:* `flutter analyze` в `frontend/shared`, `frontend/desktop` и `frontend/web` завершается с `No issues found!`; греп по `00D2FF` / `00E5FF` / `46d0e7` во `frontend/` не находит оставшихся брендовых акцентов старой палитры; во всех экранах и на панели задач отображается новый слон с оранжевым AR-визором; эмодзи в UI отсутствуют.

---

### Фаза 20 (Ф20): Golden-верификация v2.0, сквозной `bench all` гейт и закрытие документации
*Целевые файлы:* `docs/adr/0001-ob2h-interop.md`, `docs/kag_sync.md`, `backend/crates/omnesagent-kag/src/bench.rs`

- [x] **Ф20.1. Актуализация документации:** обновить [`docs/adr/0001-ob2h-interop.md`](docs/adr/0001-ob2h-interop.md) (добавить ADR-O12..O18 и контракт `ob2h` v2.0) и [`docs/kag_sync.md`](docs/kag_sync.md) (описание `SCHEMA_VERSION = 10`, SCIP `node_id` с `<project_id>:`, `file_blobs`, `CO_CHANGES_WITH`, `TESTED_BY`, `bq_mask`, 3 типов конфликтов Dream 2.0).
- [ ] **Ф20.2. Сквозной bench & golden-гейт (ADR-13):** зафиксировать и пройти целевые пороги на golden-корпусе (`omnes-agent`):
  - Stage-1 BQ `popcnt` (100k векторов, 384 dim): `< 0.5 мс`;
  - U-PPR на $k$-hop сабграфе (10k узлов, 20 итераций): `< 15 мс`;
  - Гибридный поиск (p95 на 50k memories): `< 50 мс`;
  - Качество поиска: падение `recall@5 <= 10%` и `MRR <= 15%` против базового эталона (гейт [`bench.rs`](backend/crates/omnesagent-kag/src/bench.rs)).
- [ ] **Ф20.3. Финальная проверка воркспейса:** `cargo test --workspace` (по измененным крейтам: `omnesagent-kag`, `omnesagent-memory`, `omnesagent-ralph`, `omnesagent-runtime`), `flutter analyze` и `flutter test` во `frontend/{shared,desktop,web}`.
- [ ] **Ф20.2. Сквозной bench & golden-гейт (ADR-13):** зафиксировать и пройти целевые пороги на golden-корпусе (`omnes-agent`):
  - Stage-1 BQ `popcnt` (100k векторов, 384 dim): `< 0.5 мс`;
  - U-PPR на $k$-hop сабграфе (10k узлов, 20 итераций): `< 15 мс`;
  - Гибридный поиск (p95 на 50k memories): `< 50 мс`;
  - Качество поиска: падение `recall@5 <= 10%` и `MRR <= 15%` против базового эталона (гейт [`bench.rs`](backend/crates/omnesagent-kag/src/bench.rs)).
- [ ] **Ф20.3. Финальная проверка воркспейса:** `cargo test --workspace` (по измененным крейтам: `omnesagent-kag`, `omnesagent-memory`, `omnesagent-ralph`, `omnesagent-runtime`), `flutter analyze` и `flutter test` во `frontend/{shared,desktop,web}`.

*DoD Ф20:* все тесты и бенчмарк-гейты зелёные, документация синхронизирована, чекбоксы фаз Ф14–Ф20 закрыты.

---

## 5. Матрица рисков, Вехи (M12–M17) и Открытые вопросы

### 5.1. Матрица рисков (R11–R16)

*(Продолжает матрицу рисков предыдущих планов интеграции.)*

| # | Риск | Вероятность | Влияние | Митигация |
|---|---|---|---|---|
| **R11** | Переход на канонический SCIP `node_id` (Ф14.3) оторвёт старые ссылки в `ralph_findings.symbols` и `ast_changes` | Средняя | Высокое | При миграции v9→v10 сохранить таблицу/маппинг или выполнить бэкфилл `node_id` по паре `(file_path, label, node_type)` с сохранением префикса `<project_id>:`. |
| **R12** | Вызов `git log -n 200` (Ф15.4) замедлит `scan_project` на больших монорепо или при отсутствии `git` в PATH | Средняя | Среднее | Выполнять `cochange` как best-effort с жёстким таймаутом 3 с и проверкой кэша `HEAD` SHA в `kv`; при отсутствии `.git` или таймауте пропускать шаг без ошибки скана. |
| **R13** | Расширение `EmbeddingFingerprint` (Ф16.3) заблокирует запись новых векторов до завершения переиндексации | Высокая | Среднее | Запускать фоновый `reindex_project_embeddings` партиями; в период прогрева включать `degraded_mode: "warming_up_embeddings"` на базе FTS5 + U-PPR + сохранённых `bq_mask`. |
| **R14** | Гонки блокировок SQLite (`SQLITE_BUSY`) между воркерами KAG, Dream и несколькими MCP-процессами | Средняя | Высокое | Единый pragma-контракт (`WAL`, `synchronous=NORMAL`, `busy_timeout=5000`), сброс телеметрии строго в `BEGIN IMMEDIATE` и плановый `wal_checkpoint(TRUNCATE)` (Ф17.5). |
| **R15** | Перенасыщение интерфейса оранжевым `#FF6B00` при слепой автозамене всех вхождений `accentCyan` (Ф19.2) | Высокая | Среднее | Токены делятся: `#FF6B00` идёт только в бренд-фокус, активную вкладку/индикатор и промпт `>_` ($\le 10\%$ площади); для информационных ссылок кода и второстепенных тегов остаётся нейтральный графит или спокойный синий `#38BDF8`/`#0284C7`. |
| **R16** | Деградация читаемости AR-визора логотипа на размере `16×16 px` в трее Windows или компактном табе (Ф19.1) | Средняя | Низкое | По правилу §08 брендбука минимальный размер знака с визором в UI — `24×24 px` (в `desktop_titlebar` `26×26 px`); для `16×16 px` favicon/трея генерируется контрастный кроп-рендер с усиленным сглаживанием. |

### 5.2. Вехи (Milestones M12–M17)

*(Продолжают вехи M1–M5 старой спеки и M6–M11 спеки v1.9.9.)*

| Веха | Содержание | Фазы | Зависимости | Оценка |
|---|---|---|---|---|
| **M12 «AST v2 & Схема v10»** | Канонический `node_id`, `Tree::edit` LRU, stubbing генерированных файлов, `TESTED_BY`, `CO_CHANGES_WITH`, миграция v9→v10 | Ф14, Ф15 | — (может стартовать независимо от UI) | 4–5 дней |
| **M13 «Векторный конвейер & BQ»** | `spawn_blocking` пул, AST-чанкинг, асимметричные префиксы, `EmbeddingFingerprint` v2, BQ `popcnt` → SIMD, `file_blobs` | Ф16 | M12 (`bq_mask`, `file_blobs` в схеме v10) | 3–4 дня |
| **M14 «Graph RAG 2.0 & U-PPR»** | Полная формула U-PPR ($\lambda_{\text{code}}=0$, In-Degree Penalty, Mute Node), $k$-hop $p_0$, sub-token FTS5, Elbow Cutoff, WAL | Ф17 | M12 (`subtokens` в схеме v10) | 3–4 дня |
| **M15 «Dream 2.0 & Свежесть»** | 3-уровневая таксономия конфликтов, Entity Overlap, `SUPERSEDES`/`CONTRADICTS`, `git_head_sha`, HITL & Rollback Gate, MCP v2.0 | Ф18 | M12, M14 | 3–4 дня |
| **M16 «Ребрендинг OmnesProducts»** | Векторный знак `omnes-agent.svg`, растры и `.ico`, палитра `#111827`/`#F4F5F7`/`#FF6B00`, шрифты `Montserrat`/`Inter`/`JetBrains Mono`, `DESIGN_SYSTEM.md` | Ф19 | — (полностью независима от backend, может выполняться параллельно) | 2–3 дня |
| **M17 «Golden Bench Gate & Docs»** | Сквозной `bench all` гейт, обновление ADR и `kag_sync.md`, закрытие релиза | Ф20 | M12–M16 | 1 день |

### 5.3. Принятые решения владельца (2026-11-14)
1. **Модель эмбеддингов по умолчанию в OA (RU/EN корпус):** Утверждён плановый переход `LocalBertEmbedding` (`local_bert.rs:39`) с `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` на `intfloat/multilingual-e5-small` (с асимметричными префиксами `"query: "` / `"passage: "` из Ф16.3) **по результатам триплетного RU/EN-бенчмарка**.
2. **Primary Button в тёмной теме (`DesktopTheme.btnPrimaryBg`):** Утверждён графитовый вариант: в тёмной теме `btnPrimaryBg` остаётся `#1F2937` с текстом `#F4F5F7` и фокусом/ховером `#FF6B00`, а сплошная заливка `#FF6B00` резервируется исключительно для главной акцентированной CTA-кнопки («Отправить» / «Запустить») для соблюдения ограничения $\le 10\%$ оранжевого.
3. **Координация и параллельные ветки (`Ф7–Ф13`, `Ф14–Ф18`, `Ф19`):** Утверждено параллельное выполнение работ в отдельных feature-ветках (`feat/kag-v2` для блока памяти/KAG и `feat/rebrand-omnes-v1.1` для ребрендинга) независимо от активных задач `Ф7–Ф13` (`PLAN_OB2H_INTEGRATION_V1.9.9_2026-11-14.md`). Общие точки соприкосновения (`tool_group.rs`, `omnesagent-ralph`, `docs/adr/0001-ob2h-interop.md`) синхронизируются при слиянии веток.

---

## 6. Правила выполнения

1. **Параллельная работа в изолированных ветках:** Блок памяти v2 (`feat/kag-v2`: Ф14–Ф17), ребрендинг (`feat/rebrand-omnes-v1.1`: Ф19) и задачи `Ф7–Ф13` (`PLAN_OB2H_INTEGRATION_V1.9.9_2026-11-14.md`) ведутся параллельно в разных feature-ветках; общие файлы (`tool_group.rs`, `omnesagent-ralph`, `docs/adr/0001-ob2h-interop.md`, актуальные для Ф18.5) синхронизируются последовательно при интеграции/мерже.
2. **Атомарность и проверка на каждом шаге:**
   - После каждой правки Rust: `cargo check -p <crate>` и целевые unit-тесты (`cargo test -p <crate> --lib`);
   - После каждой правки Dart/Flutter: `flutter analyze` во всех затронутых пакетах (`shared`, `desktop`, `web`).
3. **Запрет секретов и личных данных:** соблюдать `AGENTS.md` Safety; тестовые фикстуры и golden-наборы не содержат реальных токенов/путей.

---

## Приложение A. Целевой DDL миграции KAG `SCHEMA_VERSION = 10` (`kag.db` и аддитивные поля `brain.db`)

### A.1. Миграция `kag.db` (`omnesagent-kag/src/schema.rs`, v9 → v10)

```sql
-- 1. Новые колонки graph_nodes (проверять через table_has_column перед ALTER TABLE)
ALTER TABLE graph_nodes ADD COLUMN kind_detail TEXT NOT NULL DEFAULT '';
ALTER TABLE graph_nodes ADD COLUMN bq_mask BLOB;
ALTER TABLE graph_nodes ADD COLUMN subtokens TEXT NOT NULL DEFAULT '';

-- 2. Новые колонки memories и ralph_findings для Git-Anchored Свежести и BQ
ALTER TABLE memories ADD COLUMN git_head_sha TEXT;
ALTER TABLE memories ADD COLUMN valid_until TEXT;
ALTER TABLE memories ADD COLUMN bq_mask BLOB;
ALTER TABLE ralph_findings ADD COLUMN git_head_sha TEXT;

-- 3. Индексы новых колонок
CREATE INDEX IF NOT EXISTS idx_graph_nodes_kind_detail ON graph_nodes(kind_detail);
CREATE INDEX IF NOT EXISTS idx_memories_valid_until ON memories(valid_until);

-- 4. Content-addressed кэш файлов для мгновенного переключения git-веток (Ф16.6 / 72b.5)
CREATE TABLE IF NOT EXISTS file_blobs (
  blake3_hash TEXT PRIMARY KEY,
  parser_version TEXT NOT NULL,
  embedding_fingerprint TEXT NOT NULL,
  ast_json TEXT NOT NULL,
  vector_ids TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- 5. Пересборка FTS5-индекса graph_nodes_fts с поддержкой subtokens (Ф17.3 / 73.9)
DROP TRIGGER IF EXISTS graph_nodes_fts_ai;
DROP TRIGGER IF EXISTS graph_nodes_fts_ad;
DROP TRIGGER IF EXISTS graph_nodes_fts_au;
DROP TABLE IF EXISTS graph_nodes_fts;

CREATE VIRTUAL TABLE IF NOT EXISTS graph_nodes_fts USING fts5(
  label, description, subtokens,
  content='graph_nodes', content_rowid='id', tokenize='trigram'
);

CREATE TRIGGER IF NOT EXISTS graph_nodes_fts_ai AFTER INSERT ON graph_nodes BEGIN
  INSERT INTO graph_nodes_fts (rowid, label, description, subtokens)
    VALUES (new.id, new.label, new.description, new.subtokens);
END;
CREATE TRIGGER IF NOT EXISTS graph_nodes_fts_ad AFTER DELETE ON graph_nodes BEGIN
  INSERT INTO graph_nodes_fts (graph_nodes_fts, rowid, label, description, subtokens)
    VALUES ('delete', old.id, old.label, old.description, old.subtokens);
END;
CREATE TRIGGER IF NOT EXISTS graph_nodes_fts_au AFTER UPDATE OF label, description, subtokens ON graph_nodes BEGIN
  INSERT INTO graph_nodes_fts (graph_nodes_fts, rowid, label, description, subtokens)
    VALUES ('delete', old.id, old.label, old.description, old.subtokens);
  INSERT INTO graph_nodes_fts (rowid, label, description, subtokens)
    VALUES (new.id, new.label, new.description, new.subtokens);
END;

INSERT INTO graph_nodes_fts (rowid, label, description, subtokens)
  SELECT id, label, description, subtokens FROM graph_nodes;
```

### A.2. Аддитивные поля `brain.db` (`omnesagent-memory/src/sqlite.rs`)

```sql
-- Идемпотентно через add_memories_column_if_missing:
ALTER TABLE memories ADD COLUMN git_head_sha TEXT;
ALTER TABLE memories ADD COLUMN valid_until TEXT;
ALTER TABLE memories ADD COLUMN bq_mask BLOB;
-- Единообразие с kag.db: триггер memories_au скоупить на key, content:
DROP TRIGGER IF EXISTS memories_au;
CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE OF key, content ON memories BEGIN
  INSERT INTO memories_fts(memories_fts, rowid, key, content)
    VALUES ('delete', old.rowid, old.key, old.content);
  INSERT INTO memories_fts(rowid, key, content)
    VALUES (new.rowid, new.key, new.content);
END;
```

---

## Приложение B. Паритет-матрица `ob2h v2.0.3` $\leftrightarrow$ `OmnesAgent`

| Компонент / Возможность `ob2h v2.0.3` | Фаза ob2h | Статус / Фаза в OmnesAgent | Примечание |
|---|---|---|---|
| Quality Harness & Golden-гейт (ADR-13) | 69 | Частично (`omnesagent-kag::bench`) + **Ф20.2** | Добросить пороги и E2E `bench all` |
| Tree-sitter + Regex fallback (18 языков вкл. Dart) | 70.1–70.7, 70.9, 70.12 | База в **Ф6.1** (`treesitter.rs` через `inkjet`) | Углубление — **Ф14** |
| Incremental `Tree::edit` + LRU (32–64 дерева) | 70.8 | **Ф14.1** | Ограничение RAM $< 15$ МБ |
| Эвристика `token_tree` в макросах | 70.13 | **Ф14.2** | `provenance = "HEURISTIC"` |
| Канонический SCIP-подобный `node_id` | 70.14 | **Ф14.3** | С префиксом `<project_id>:` по `kag_sync.md` |
| Generated File Stubbing (`*.g.dart`, `*.freezed.dart`, `*.pb.*`) | 70.15 | **Ф14.4** | Критично для Flutter-части OA (`frontend/`) |
| Цепочки вызовов, `TESTED_BY`, `analyze_impact` | 71.1–71.2, 71.6 | **Ф15.1–Ф15.3** | Прямой вывод `affected_tests` |
| Миграция БД v10 (аналог M12 ob2h) | 71.3 | **Ф15.5** | Приложение A |
| SFC `<script>` в `.vue`/`.svelte` на Tree-sitter | 71.5 | **Ф14.5** | Сейчас только regex в `ast/mod.rs` |
| Co-Change Graph (`CO_CHANGES_WITH` из `git log`) | 71.8 | **Ф15.4** | Межъязыковые связи `.rs` ↔ `.dart` ↔ `.sql` |
| Candle `spawn_blocking` pool & batching | 72.1 | **Ф16.1** | Zero C-deps сохраняется |
| ONNX Runtime (`fastembed`/`ort`) | 72.2 | **Не-цель** (ADR Ф6.2 #5) | Candle-first по решению владельца |
| AST-aware чанкинг + заголовок контекста | 72.2a | **Ф16.2** | Пропуск stub-файлов |
| Асимметричные префиксы `query_prefix`/`document_prefix` | 72.2b | **Ф16.3** | На уровне `EmbeddingProvider` |
| `EmbeddingFingerprint` v2 (5 полей вкл. `prefix_version`) | 72.3 | **Ф16.3** | Автодетект несовместимости |
| Stale-While-Revalidate (`degraded_mode`) | 72.5 | **Ф16.4**, **Ф18.5** | Без блокировки холодного старта |
| Санитайзер секретов перед эмбеддингом | 72.6 | **Ф16.4** | Поверх `omnesagent_memory::redact` |
| Mean-Centered BQ (`[u64; 6]`) `popcnt` → SIMD Rescore | 72b.2 | **Ф16.5** | Без C-FFI, `< 0.5 мс` на 100k |
| Пространства имён (`memories` vs `graph_nodes`) & `file_blobs` | 72b.3, 72b.5 | **Ф16.5**, **Ф16.6** | `blake3` кэш ветвления |
| Полная формула U-PPR ($\lambda_{\text{code}}=0$, In-Degree Penalty, Mute Node) | 73.1–73.2, 73.6 | **Ф17.1** | Развитие `pagerank.rs` |
| $k$-hop сабграф + $p_0 = \text{IDF}\times\text{trust}$ + `git diff HEAD` Boost | 73.4–73.5 | **Ф17.2** | Включая исключение `SUPERSEDED` |
| WAL-гигиена (`BEGIN IMMEDIATE` + `wal_checkpoint(TRUNCATE)`) | 73.3 | **Ф17.5** | Против `SQLITE_BUSY` и разбухания WAL |
| Токен-бюджетный AST-скелет + Elbow Cutoff | 73.8 | **Ф17.4** | Развитие `repomap.rs` |
| Sub-token декомпозиция в `graph_nodes_fts` | 73.9 | **Ф17.3** | `CamelCase` / `snake_case` → `subtokens` |
| Dream 2.0: 3 типа конфликтов + Entity Overlap + `git_head_sha` + HITL | 74.1–74.7 | **Ф18.1–Ф18.4** | Томбстоуны `valid_until`, Rollback Gate |
| Ralph `git_head_sha` freshness & tracing spans | 75.1–75.2 | **Ф18.3**, **Ф20.2** | Сопряжение с `omnesagent-ralph` |
| Enterprise Installer `cargo-dist` & `self uninstall` | 76.1–76.9 | **Не-цель** | Относится только к `C:\Projects\ob2h` |

---

## Приложение C. Карта токенов и ассетов ребрендинга (`OmnesProducts` v1.1 $\to$ OmnesAgent)

### C.1. Цветовая палитра (Proportion Rule: 60% Dark Charcoal / 30% Off-White / ≤10% Brand Orange)

| Назначение токена | As-Is (Cyber-Cyan) | To-Be Dark (`#111827` base) | To-Be Light (`#F4F5F7` paper) | Целевые файлы |
|---|---|---|---|---|
| **L0 Shell / Sidebar** | `#141619` / `#09090B` / `#F1F5F9` | `#0D1117` | `#F4F5F7` (`--paper`) | `DesktopTheme`, `ShadcnColors`, `DESIGN_SYSTEM.md` |
| **L1 Primary Surface** | `#16181D` / `#12141A` / `#F8FAFC` | `#111827` (`--charcoal`) | `#FAFBFC` | `DesktopTheme`, `ShadcnColors` |
| **L2 Elevated / Cards** | `#20242D` / `#141720` / `#FFFFFF` | `#18202F` / `#1F2937` | `#FFFFFF` | `DesktopTheme`, `ShadcnColors` |
| **L3 Overlay / Popover** | `#1B1D22` / `#1C202C` / `#FFFFFF` | `#161F2E` | `#FFFFFF` | `DesktopTheme`, `DESIGN_SYSTEM.md` |
| **Border Subtle** | `#22262E` / `#242836` / `#CBD5E1` | `#232C3D` | `#D8DCE3` (`--line`) | `DesktopTheme`, `ShadcnColors` |
| **Border Medium** | `#2C323E` / `#94A3B8` | `#334155` | `#9AA3B2` | `DesktopTheme` |
| **Primary Brand / Focus Accent** | `#00D2FF` / `#00E5FF` / `#0284C7` | `#FF6B00` (`--orange`) | `#E05E00` / `#FF6B00` | `DesktopTheme.borderFocus`, `ShadcnColors.primary` |
| **Accent Hover / Soft Fill** | `rgba(0,210,255,0.12)` | `#FF8533` / `rgba(255,107,0,0.14)` | `rgba(255,107,0,0.10)` | Кнопки, ховеры, активные табы |
| **Text Primary** | `#FFFFFF` / `#020617` | `#F4F5F7` (`--offwhite`) | `#111827` (`--charcoal`) | `DesktopTheme`, `ShadcnColors` |
| **Text Secondary** | `#CBD5E1` / `#1E293B` | `#CBD5E1` | `#1E293B` | `DesktopTheme` |
| **Text Muted** | `#8B949E` / `#334155` | `#94A3B8` | `#5B6472` (`--muted`) | `DesktopTheme` |
| **Primary Button (Solid)** | `#282D37` / `#334155` | `#1F2937` (акцент `#FF6B00` на главной CTA) | `#111827` (текст `#F4F5F7`) | `DesktopTheme.btnPrimaryBg`, `.gemini/style-guide.md` |
| **Code / Secondary Info** | `#38BDF8` / `#0284C7` | `#38BDF8` (без изменений) | `#0284C7` (без изменений) | Ссылки кода, git-ветки |
| **Semantic Status** | `#10B981`, `#F59E0B`, `#EF4444` | `#10B981`, `#F59E0B`, `#EF4444` | `#10B981`, `#F59E0B`, `#EF4444` | Git `+`/`-`, статусы |

### C.2. Типографическая система (§06 брендбука `OmnesProducts`)

| Роль в интерфейсе | As-Is | To-Be (`OmnesProducts` v1.1) | Начертания и трекинг |
|---|---|---|---|
| **Заголовки (`H1–H3`), Топбар `OmnesAgent`, Бренд-бейджи** | `Segoe UI` Bold | **`Montserrat`** (fallback: `Segoe UI`, `Inter`, sans-serif) | `SemiBold (w600)` / `Bold (w700)`, трекинг `0…+2%`, кегль `13–32px` |
| **Основной текст, списки, инпуты, меню** | `Segoe UI` | **`Inter` / `Segoe UI`** | `Regular (w400)` / `Medium (w500)`, кегль `12–16px` |
| **Код, терминал, Git-ветки, тайминги, шорткаты** | `Consolas` | **`JetBrains Mono` / `Consolas`** | `Regular (w400)` / `Medium (w500)`; промпт `>_` в Brand Orange `#FF6B00` |

### C.3. Ассеты логотипа и иконок

| Источник (`OmnesProducts`) | Целевые файлы в `Omnes-agent` | Использование |
|---|---|---|
| `C:\Projects\OmnesProducts\brand\omnes-agent.svg` (`viewBox="0 0 818 815"`, `#111827` / `#F4F5F7` / `#FF6B00`) | `frontend/desktop/assets/Logo/OA_icon.svg`<br>`frontend/web/assets/Logo/OA_icon.svg`<br>`frontend/assets/Logo/OA_icon.svg` | Основная векторная иконка в `desktop_titlebar.dart`, `login_screen.dart` и экранных эмблемах через `flutter_svg` |
| Экспорт из `omnes-agent.svg` (плоский вектор, 0 градиентов) | `app_icon.png`, `app_launcher.png`, `basic_logo_dark.png`, `basic_logo_light.png`, `frontend/desktop/windows/runner/resources/app_icon.ico`, `frontend/web/web/favicon.png`, `Icon-192.png`, `Icon-512.png` | Нативная иконка окна Windows, системный трей, браузерный favicon и PWA-манифест |


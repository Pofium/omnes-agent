# Детальный план полной замены подсистемы памяти OmnesAgent архитектурой KAG и AST-кодинга из OB2H

## Обзор и архитектурная цель

Цель: **Полная замена текущего механизма памяти и базового графа знаний в `Omnes-agent/backend` на архитектуру OB2H (OmnesBot to Harness)**.

Текущая подсистема памяти `Omnes-agent` (наивный `KnowledgeGraph` и `SqliteMemory`) заменяется на единое интеллектуальное ядро памяти, объединяющее:
1. **Детерминированный AST-граф кода (10 языков)**: статический парсинг структуры кода (функции, структуры, классы, интерфейсы, трейты, импорты, вызовы) без расхода LLM-токенов.
2. **Анализ радиуса изменений (Blast Radius / `project_impact`)**: многошаговый расчёт обратных зависимостей и оценка риска рефакторинга (`Low`, `Medium`, `High`).
3. **KAG (Knowledge Augmented Generation) и рассуждения по графу (`graph_reason`)**: многошаговый логический вывод по подграфу с цепочкой рассуждений (evidence chain) и оценкой достоверности (confidence).
4. **Графовая аналитика**: PageRank (ранжирование важности), алгоритм Louvain (кластеризация модулей в сообщества), выявление ключевых архитектурных хабов (**God Nodes**).
5. **Локальные векторные эмбеддинги Candle MiniLM (384d)**: встроенный легковесный инференс нейросети на чистом Rust (CPU) без зависимости от внешних OpenAI API.
6. **Гибридный поиск RRF ($k=60$)**: слияние FTS5 (BM25) и векторного поиска по формуле взаимного ранжирования (Reciprocal Rank Fusion).
7. **Дриминг и версионирование рабочей области**: консолидация сессионных логов в `MEMORY.md`, `SOUL.md`, `USER.md` с фиксацией в локальном Git и возможностью отката к любому коммиту (`dream_restore`).

---

## Архитектурное сопоставление компонентов

| Подсистема | Текущая реализация `Omnes-agent` | Новая реализация на базе `OB2H` |
|---|---|---|
| **Граф знаний** | Базовые таблицы `knowledge_nodes` и `knowledge_edges` в `omnesagent-memory/src/knowledge_graph.rs` | Полнофункциональный KAG-движок с `provenance`, `confidence`, `is_god_node`, PageRank, Louvain и KAG reasoning |
| **Кодовая память** | Отсутствует (только чтение файлов через `file_read`) | Детерминированный AST-граф для 10 языков, инкрементальный сканер с SHA256, расчёт Blast Radius |
| **Эмбеддинги** | Внешние OpenAI/custom HTTP API или Noop | Локальный Candle MiniLM (384d, in-process Rust) + внешние провайдеры как опция |
| **Поиск по памяти** | Простой взвешенный скоринг | Гибридный RRF ($k=60$) по FTS5 (триграммы/юникод) + локальные векторные эмбеддинги |
| **Память проектов** | Слабая изоляция через session/namespace | Полноценная сущность `projects` с привязкой к каталогу, ветке, технологическому стеку и AST-файлам |
| **Консолидация** | Базовый hygiene pass (архивирование старых строк) | Дриминг: ночной/фоновый анализ диалогов, синтез фактов, правка `MEMORY/SOUL/USER` с Git-коммитами |

---

## Единая схема базы данных SQLite (`brain.db`)

Вся память агента консолидируется в единую схему SQLite с версионным мигратором:

```sql
-- 1. Служебная таблица версий
CREATE TABLE IF NOT EXISTS kv (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- 2. Таблица проектов кодовой базы
CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  root_path TEXT NOT NULL,
  description TEXT,
  tech_stack TEXT,
  active_branch TEXT,
  last_scanned_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_projects_root_path ON projects(root_path);

-- 3. Инкрементальное отслеживание файлов проекта по хэшам SHA256
CREATE TABLE IF NOT EXISTS project_files (
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  rel_path TEXT NOT NULL,
  sha256 TEXT NOT NULL,
  file_size INTEGER NOT NULL,
  lines_count INTEGER NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (project_id, rel_path)
);
CREATE INDEX IF NOT EXISTS idx_project_files_project ON project_files(project_id);

-- 4. Долговременная память агента (унифицированная)
CREATE TABLE IF NOT EXISTS memories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  key TEXT UNIQUE NOT NULL,
  content TEXT NOT NULL,
  category TEXT NOT NULL DEFAULT 'general',
  importance REAL NOT NULL DEFAULT 0.5,
  source TEXT DEFAULT 'manual',
  namespace TEXT NOT NULL DEFAULT 'default',
  agent_id TEXT,
  project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
  meta TEXT,
  embedding BLOB,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  access_count INTEGER NOT NULL DEFAULT 0,
  last_accessed TEXT,
  origin TEXT NOT NULL DEFAULT '',
  deleted_at TEXT
);
CREATE INDEX IF NOT EXISTS idx_memories_cat_imp ON memories (category, importance DESC);
CREATE INDEX IF NOT EXISTS idx_memories_project ON memories (project_id);
CREATE INDEX IF NOT EXISTS idx_memories_namespace ON memories (namespace);
CREATE INDEX IF NOT EXISTS idx_memories_agent ON memories (agent_id);

-- 5. Связи между воспоминаниями
CREATE TABLE IF NOT EXISTS memory_relations (
  source_key TEXT NOT NULL REFERENCES memories(key) ON DELETE CASCADE,
  target_key TEXT NOT NULL REFERENCES memories(key) ON DELETE CASCADE,
  relation_type TEXT NOT NULL,
  weight REAL NOT NULL DEFAULT 1.0,
  UNIQUE (source_key, target_key, relation_type)
);

-- 6. Документы и чанки
CREATE TABLE IF NOT EXISTS documents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT,
  path TEXT,
  meta TEXT,
  project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_documents_project ON documents(project_id);

CREATE TABLE IF NOT EXISTS chunks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  doc_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  ordinal INTEGER NOT NULL,
  text TEXT NOT NULL,
  embedding BLOB,
  project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_chunks_doc ON chunks (doc_id);
CREATE INDEX IF NOT EXISTS idx_chunks_project ON chunks (project_id);

-- 7. Узлы графа знаний и AST-кода
CREATE TABLE IF NOT EXISTS graph_nodes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  node_id TEXT UNIQUE NOT NULL,
  label TEXT NOT NULL,
  node_type TEXT NOT NULL,          -- 'function', 'class', 'struct', 'interface', 'trait', 'decision', 'pattern', 'lesson'
  description TEXT,
  val INTEGER NOT NULL DEFAULT 1,
  embedding BLOB,
  project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
  file_path TEXT,
  line_start INTEGER,
  line_end INTEGER,
  provenance TEXT NOT NULL DEFAULT 'manual', -- 'ast', 'llm', 'manual'
  confidence REAL NOT NULL DEFAULT 1.0,
  is_god_node INTEGER NOT NULL DEFAULT 0,
  origin TEXT NOT NULL DEFAULT '',
  deleted_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_label ON graph_nodes (label);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_type ON graph_nodes (node_type);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_project ON graph_nodes (project_id);
CREATE INDEX IF NOT EXISTS idx_graph_nodes_provenance ON graph_nodes (provenance);

-- 8. Рёбра графа знаний и AST-зависимостей
CREATE TABLE IF NOT EXISTS graph_edges (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  source_id INTEGER NOT NULL REFERENCES graph_nodes(id) ON DELETE CASCADE,
  target_id INTEGER NOT NULL REFERENCES graph_nodes(id) ON DELETE CASCADE,
  label TEXT NOT NULL,              -- 'calls', 'imports', 'implements', 'uses', 'replaces', 'applies_to'
  weight REAL NOT NULL DEFAULT 1.0,
  contexts TEXT,
  project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
  provenance TEXT NOT NULL DEFAULT 'manual',
  confidence REAL NOT NULL DEFAULT 1.0,
  origin TEXT NOT NULL DEFAULT '',
  deleted_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL DEFAULT '',
  UNIQUE (source_id, target_id, label)
);
CREATE INDEX IF NOT EXISTS idx_graph_edges_src ON graph_edges (source_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_dst ON graph_edges (target_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_project ON graph_edges (project_id);

-- 9. Журнал дриминга
CREATE TABLE IF NOT EXISTS dream_runs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at TEXT,
  finished_at TEXT,
  status TEXT,
  trigger TEXT,
  phase_log TEXT,
  stats TEXT
);

-- 10. FTS5 полнотекстовые индексы
CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
  content, content='memories', content_rowid='id', tokenize='trigram'
);
-- триггеры синхронизации FTS
CREATE TRIGGER IF NOT EXISTS memories_fts_ai AFTER INSERT ON memories BEGIN
  INSERT INTO memories_fts (rowid, content) VALUES (new.id, new.content);
END;
CREATE TRIGGER IF NOT EXISTS memories_fts_ad AFTER DELETE ON memories BEGIN
  INSERT INTO memories_fts (memories_fts, rowid, content) VALUES ('delete', old.id, old.content);
END;
CREATE TRIGGER IF NOT EXISTS memories_fts_au AFTER UPDATE ON memories BEGIN
  INSERT INTO memories_fts (memories_fts, rowid, content) VALUES ('delete', old.id, old.content);
  INSERT INTO memories_fts (rowid, content) VALUES (new.id, new.content);
END;
```

---

## Детальные фазы выполнения работ

### Фаза 1: Создание базового крейта `omnesagent-kag` и добавление зависимостей

В `backend/crates/omnesagent-kag`:
1. **`Cargo.toml`**:
   - `candle-core` (0.8), `candle-nn`, `candle-transformers`.
   - `tokenizers` (0.21) с выключенными тяжелыми зависимостями (`default-features = false`, `onig`).
   - `hf-hub` (0.4) для кеширования весов MiniLM.
   - `rusqlite` (0.37, bundled), `parking_lot`, `tokio`, `serde`, `serde_json`, `chrono`, `sha2`, `hex`, `regex`, `ignore`, `notify`.
2. **Модули**:
   - `ast/` — парсеры AST для 10 языков: Rust, Python, TS/JS, Go, Dart, PHP, Java, C/C++, SQL.
   - `graph/` — `analytics.rs` (PageRank, Louvain, God Nodes), `service.rs` (поиск, фильтрация, KAG reasoning).
   - `project/` — регистрация проекта, инкрементальный сканер файлов (`ProjectScanner`), расчёт Blast Radius (`ProjectImpact`), генератор контекста (`ProjectContext`).
   - `embedding/` — `CandleEmbedder` (локальный MiniLM 384d).
3. **Регистрация в workspace**:
   - Добавление `crates/omnesagent-kag` в `backend/Cargo.toml` в секцию `[workspace.members]` и `[workspace.dependencies]`.

### Фаза 2: Интеграция базы данных и миграция в `omnesagent-memory`

1. **Рефакторинг `backend/crates/omnesagent-memory/src/sqlite.rs`**:
   - Замена старого `init_schema` на новый унифицированный мигратор схемы OB2H.
   - Функция автоматической плавной миграции: если обнаружена старая база `brain.db` от OmnesAgent, перенести существующие записи из старой таблицы `memories` в новую с сохранением ключей, текста, категорий и `agent_id`.
2. **Реализация гибридного поиска RRF $k=60$**:
   - Слияние результатов FTS5 полнотекстового поиска и векторного сходства (косинусного или скалярного произведения) по стандарту Reciprocal Rank Fusion:
     $$RRF(d) = \sum_{m \in \{fts, vec\}} \frac{w_m}{60 + rank_m(d)}$$
3. **Реализация трейта `Memory`**:
   - Полная поддержка `store()`, `recall()`, `get()`, `forget()`, `list()`, `purge_session()`, `purge_agent()`, `stats()`.

### Фаза 3: Локальный провайдер Candle-эмбеддингов

1. **Интеграция Candle в `omnesagent-memory::embeddings`**:
   - Создание структуры `CandleEmbeddingProvider`:
     - Модель: `all-MiniLM-L6-v2` (384 измерения).
     - Локальное авто-скачивание весов в `~/.cache/omnesagent/models/` или использование встроенных весов.
     - Инференс на CPU через `candle-core` (время генерации одного эмбеддинга < 5 мс).
2. **Настройка конфигурации**:
   - В `omnesagent-config::schema::MemoryConfig` добавление провайдера `"candle"` / `"local"` по умолчанию.

### Фаза 4: AST-парсер и проектная память кодинга

1. **Детерминированный статический анализатор кода**:
   - `ast::parse_file(rel_path, content, project_id)`:
     - Извлечение символов: имя, сигнатура, тип сущности, строки (`line_start`, `line_end`).
     - Извлечение зависимостей: `calls`, `imports`, `implements`, `defines`.
2. **Инкрементальное сканирование (`ProjectScanner`)**:
   - Быстрое сканирование директории репозитория с учётом правил `.gitignore` (через крейт `ignore`).
   - Вычисление SHA256 каждого файла. Сканируются только изменённые или добавленные файлы, удалённые файлы удаляют свои узлы из графа с каскадным удалением рёбер.
3. **Расчёт Blast Radius (`project_impact`)**:
   - Поиск целевого символа или файла в `graph_nodes`.
   - Обход обратных рёбер (`reverse BFS`) до заданной глубины (по умолчанию 3 шага).
   - Подсчёт зависимых функций, классов, файлов.
   - Автоматическая классификация риска:
     - `Low`: затронуто до 3 локальных функций в том же файле.
     - `Medium`: затронуты другие файлы одного модуля (до 10 сущностей).
     - `High`: затронуты публичные интерфейсы, God Nodes или > 10 внешних файлов.

### Фаза 5: KAG Engine и многошаговые рассуждения (`graph_reason`)

1. **Графовая аналитика (`analytics.rs`)**:
   - Расчёт входящей/исходящей связности (Degree Centrality).
   - Расчёт PageRank узлов для выявления ключевых концептов кодовой базы и бизнес-логики.
   - Алгоритм Louvain: разбиение графа на тематические сообщества (кластеры компонентов).
   - Пометка узлов с максимальным рангом флагом `is_god_node = 1`.
2. **Алгоритм KAG Reasoning (`service.rs`)**:
   - При вопросе к графу:
     1. Векторный и лексический поиск начальных узлов («якорей»).
     2. Раскрытие 1-2 hop связей вокруг якорей.
     3. Формирование сжатого связного контекста (evidence path).
     4. Расчёт метрики достоверности ответа (confidence score на основе весов рёбер и provenance).
     5. Формирование ответа с доказательной цепочкой.

### Фаза 6: Замена инструментов в `omnesagent-tools` и `omnesagent-runtime`

1. **Замена `knowledge_tool`**:
   - Текущий `KnowledgeTool` обновляется для работы с новым KAG-движком, получая действия:
     - `graph_search` — поиск по графу (семантический + фильтр по `provenance`: ast/llm/manual).
     - `graph_reason` — вопрос к графу знаний с возвратом цепочки рассуждений.
     - `graph_stats` — сводка по узлам, рёбрам, God Nodes и проектам.
     - `knowledge_extract` — извлечение сущностей из текста в граф.
2. **Новый инструмент `project_code_tool`**:
   - `project_init` — регистрация репозитория в памяти агента.
   - `project_scan` — запуск AST-сканирования кодовой базы (мгновенно, без LLM токенов).
   - `project_context` — получение компактного дайджеста `<project_context>` под задачу.
   - `project_graph_search` — поиск функций/структур по имени или описанию.
   - `project_impact` — расчёт Blast Radius перед рефакторингом.
   - `project_report` — архитектурный дайджест ключевых хабов системы.
3. **Инструменты дриминга (`dream_tool`)**:
   - `dream_run` — принудительный запуск консолидации и дриминга.
   - `dream_status` — статус последнего запуска и состояние воркеров.
   - `dream_restore` — откат рабочей области к указанному Git-коммиту.

### Фаза 7: Встраивание в цикл исполнения агента (`omnesagent-runtime`)

1. **Инъекция в промпт (`memory_inject.rs`)**:
   - Добавление формирования секции `<project_context>` в системный промпт:
     - Если текущая сессия агента работает в директории зарегистрированного проекта, автоматически подгружать архитектурные хабы (God Nodes) и модули, релевантные запросу пользователя.
2. **Фоновые воркеры**:
   - Внедрение `ProjectWatcher` (отслеживание изменений файлов в IDE в реальном времени через `notify` и фоновое обновление AST-графа).
   - Внедрение `AutoDreamWorker` в runtime daemon (ночная консолидация памяти в фоне).

---

## План верификации и тестирования

### Автоматические тесты
1. **Тесты AST-парсеров (`cargo test -p omnesagent-kag --lib ast`)**:
   - Проверка извлечения функций, структур, классов, импортов для Rust, Python, TypeScript, Go, Dart.
2. **Тесты Blast Radius и графовой аналитики**:
   - Проверка правильности вычисления графа вызовов $A \to B \to C$ и обратного поиска риска при изменении $C$.
   - Проверка PageRank и определения God Nodes на синтетическом графе.
3. **Интеграционные тесты `omnesagent-memory`**:
   - Тесты трейта `Memory`: `store`, `recall` (гибридный поиск), `forget`, `purge`.
   - Тесты миграции со старой версии `brain.db`.
4. **Общая проверка сборки и линтинга**:
   - `cargo check --workspace`
   - `cargo clippy --workspace`

### Ручная верификация
1. Инициализация проекта `Omnes-agent` через команду `project_init`.
2. Запуск `project_scan` и проверка скорости парсинга всей кодовой базы на Rust/Flutter (ожидаемое время < 1.5 сек).
3. Проверка инструмента `project_impact` на ключевых функциях (например, `create_memory`).
4. Проверка генерации эмбеддингов Candle без подключения к интернету.

---
name: ob2h
description: Долговременная память, граф знаний и детерминированный AST-анализ кода для Antigravity и OmnesAgent. Используй для поиска фактов о пользователе и проектах, сохранения архитектурных решений, AST-сканирования кодовой базы без расхода токенов и оценки Blast Radius.
---

# OB2H — Долговременная память и AST-кодовый граф (Antigravity & OmnesAgent)

OB2H (OmnesBot to Harness) — локальный «мозг» и граф знаний, работающий поверх общей базы данных на Rust.

## 🗄️ Общая база данных и окружение
- **Исполняемый файл**: `C:\Users\ipres\.cargo\bin\ob2h.exe` (также доступен в `C:\Projects\omnesbot_for_hermes\target\release\ob2h.exe`)
- **Общая база данных (Single Source of Truth)**:
  `C:\Projects\omnesbot_for_hermes\data\ob2h.db` (SQLite WAL + FTS5 trigram + Candle MiniLM 384d).
  *Все агенты (Antigravity, Hermes, Claude, Cursor) работают с единой базой памяти.*
- **Переменная окружения**: `OB2H_DATA_DIR=C:\Projects\omnesbot_for_hermes\data`
- **LLM-ключ**: `OB2H_LLM_API_KEY=DEEPSEEK_API_KEY` (используется для дриминга и экстракции)

## 🛠️ Доступные инструменты (MCP & CLI)

### 1. Долговременная память (Memory)
- `memory_search(query, limit)`: Гибридный семантический + полнотекстовый поиск по фактам, прошлым обсуждениям, решениям и контексту пользователя.
  *Вызывай перед началом сложных задач или при вопросах о предпочтениях.*
- `memory_save(text, category, importance, key)`: Сохранение архитектурных решений, правил и фактов. Для устойчивых параметров задавай `key` (например `omnes.architecture.desktop_split`).

### 2. Граф знаний (Knowledge Graph)
- `graph_search(query, depth)`: Поиск сущностей, связей и триплетов в графе знаний.

### 3. AST-граф кода и анализ кодовой базы (Code Intelligence)
- `project_scan(project_path, force)`: Мгновенное детерминированное построение AST-графа кодовой базы (Rust, Dart, Python, TS/JS) со 100% точностью и без расхода токенов LLM.
  *В OmnesAgent сканирует и backend (Rust workspace), и frontend (Flutter).*
- `project_report(project_path)`: Архитектурный дайджест: ключевые узлы (God Nodes), метрики связности, распределение по языкам и выявление циклических зависимостей.
- `project_impact(target, project_path)`: Анализ радиуса поражения (Blast Radius) перед рефакторингом: находит все функции, трейты, структуры и файлы, зависящие от изменяемого символа.
- `project_graph_search(query, project_path)`: Поиск классов, методов и функций по графу кода.
- `project_context(files, project_path)`: Архитектурный контекст для конкретного списка файлов.

## 📌 Политика публикации памяти (ADR-O1/O3)

Две памяти, две роли — не смешивать:

- **Встроенная память OmnesAgent** — операционный слой харнеса: контекст текущей
  сессии, промежуточные наблюдения, agent-scoped prefetch.
- **ob2h** — долговременный кросс-продуктовый слой: пользователь, проекты,
  принятые решения.

Правила публикации в ob2h:

1. **Что сохранять:** решения владельца, архитектурные факты и устойчивые правила —
   через `memory_save` с `project_id = "omnes-agent"`, стабильным `key`
   (например `omnes.architecture.desktop_split`) и `category` (`decision`,
   `architecture`, `rule`).
2. **Дедупликация обязательна:** перед каждым `memory_save` сначала вызови
   `memory_search` по тому же `key`/смыслу; при совпадении используй `memory_update`,
   а не дубль.
3. **Что НЕ сохранять в ob2h:** секреты, токены, PIN и любой контент с
   `#[credential_class]`; операционный контекст сессии (он остаётся во встроенной
   памяти).
4. **Никакого автосинка:** публикация — только явным вызовом `memory_save`
   (или через мост `[ob2h_bridge]` по политике владельца). Молчаливый автосинк
   всех записей запрещён.

## 🔐 Сейф (Vault) и секреты (ADR-O5, ADR-18)

- **ob2h Vault — межагентный сейф:** токены LLM-провайдеров и другие секреты,
  общие для нескольких агентов. Инструменты: `secrets_status` (read-класс),
  `secrets_get` / `secrets_set` / `secrets_delete` / `secrets_lock`.
- **Форковые config-секреты остаются в форке:** поля с `#[credential_class]`
  (например `McpServerConfig.env`) в ob2h не публикуются и не копируются.
  Правило выбора: межагентное — в сейф ob2h; конфиг серверов/каналов — в конфиг форка.
- **PIN и разблокировка — только через GUI/CLI ob2h.** PIN, парольная фраза или
  любой секрет, введённый пользователем в чат, **никогда** не передаётся
  MCP-аргументом и не сохраняется через `memory_save` (ADR-18).
- `secrets_get` вызывай **только** за значением, реально нужным агенту в рантайме;
  не вызывай его, чтобы вернуть пользователю значение, введённое в чат.
- `secrets_get/set/delete/lock` доступны только интенциям EngineeringTask/SystemAdmin;
  в CodeExploration остаётся лишь `secrets_status`.
- Значения сейфа **не попадают** в память, RAG-чанки, граф знаний и tool-логи.

## 🎓 Уроки (Lessons) — процедурный опыт (ADR-O7/O8)

- `lesson_save(content, context, confidence?, project, tags?, source?)` — **процедурный опыт**
  («что делать / чего избегать / когда применимо»). Это не факт: факты и решения идут
  через `memory_save`.
- `lesson_recall(query, limit?, min_confidence?, project?)` — вызывай **перед нетривиальным
  решением**; `query` = текст задачи + затронутые символы. В цикле Ralph это делает
  `omnesagent-ralph` (Ф8.3) автоматически.
- `lesson_list(...)` — обзор уроков проекта.
- Повторный `lesson_save` того же текста = **reinforce** (ob2h дедуп по fingerprint), а не
  дубль. Не плоди перефразировки одного урока.
- Секреты и `#[credential_class]`-контент в уроки **не писать** (ADR-O5/O11).
- Decay-кандидаты дрима **не удалять автоматически** (ADR-11 ob2h); удаление — явная команда.
- Уроки живут только в ob2h (ADR-O7): в `omnesagent-memory` таблицы lessons **не создаются**.

## 🧰 Слоты (Slots) — кросс-сессийный буфер проекта (ADR-O7)

- `slot_list` / `slot_get` — read-класс; `slot_create` / `slot_append` / `slot_replace` /
  `slot_delete` — write-класс (EngineeringTask/SystemAdmin).
- Seed-дефолты: `pending_items`, `session_guidance`, `project_context`, `user_rules`,
  `self_notes`.
- **413 при `slot_append`** → компакция через `slot_replace` (прочитать → сжать → заменить);
  сообщай владельцу фактические размеры.
- Инъекция в `memory_context` — блок `<agent_slots>`; бюджет `OB2H_SLOTS_MAX_CHARS=1500`,
  pinned по алфавиту; `project`-слот **затеняет** `global`.
- **Запрет split-brain:** не копируй содержимое слотов в память форка и наоборот.
- Секреты/credential-контент в слоты не писать (ADR-O5/O11).

## 🕰️ Хронология, рекап, провенанс, аудит (ADR-O9)

- `memory_timeline(anchor?, before?, after?)` — «что происходило вокруг даты/события X»;
  якорь = ISO-дата или подстрока диалога; без якоря — хвост лога.
- **Провенанс:** перед использованием критичного факта из ob2h — `memory_verify`; в отчёт
  владельцу включай `citations`/`citation_count`. `provenance: none` — честный результат, не
  ошибка. Ограничение контракта: прямой `memory_save` провенанса не даёт (нет `meta`/`source_ids`);
  прослеживаемость — только через цепочку `session_ingest` → `session_recap(save=true)`.
- **Governance:** `memory_forget` вызывай **только с `reason`**; `memory_audit(operation?, limit?)`
  — read-only отчёт операций (actor=agent виден) для владельца.

## 🖥️ Viewer (ADR-O10)

- «Показать память владельцу» → `viewer_open` — **privileged**-инструмент (спавн процесса):
  доступен интенциям EngineeringTask/SystemAdmin, **никогда** не в CodeExploration.
- Окно detached и переживает сессию агента; повторный вызов → `already_running`. Если harness
  форка убивает job-дерево (breakaway не проходит) — фолбэк: CLI `ob2h viewer`.
- Headless/SSH — окно недоступно: сообщи честно, не выдумывай успех и не паникуй.
- **PIN в чате не запрашивать никогда.** Вкладка «Секреты» — только локальный ввод в окне;
  значения секретов не пересказывать в чат даже после раскрытия во viewer.
- Просмотр read-only: вкладки памяти/графа/сессий не пишут в БД.

## 🚀 Сценарии использования в Antigravity IDE

1. **Перед рефакторингом или крупными изменениями**:
   - Вызови `project_impact(target)` или запусти `ob2h project impact <target>` для проверки затронутых компонентов.
2. **При исследовании архитектуры**:
   - `project_report` для нахождения центральных точек входа и God Nodes.
3. **При принятии ключевых решений**:
   - Сохраняй решение в память через `memory_save(category: "decision", importance: 0.9)`.
4. **При вопросах о прошлых договорённостях**:
   - Запрашивай `memory_search`.

## ⚙️ Вызовы через CLI (быстрый доступ)
При необходимости команды можно выполнять напрямую через PowerShell:
```powershell
$env:OB2H_DATA_DIR = "C:\Projects\omnesbot_for_hermes\data"
ob2h stats
ob2h project scan "c:\Projects\Omnes-agent"
ob2h project report "c:\Projects\Omnes-agent"
ob2h project impact "DesktopShell"
```

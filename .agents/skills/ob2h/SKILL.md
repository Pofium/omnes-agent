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

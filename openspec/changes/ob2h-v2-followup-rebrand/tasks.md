# Tasks: ob2h-v2-followup-rebrand (Ф21–Ф23)

## Волна 1 (P0)
- [x] **T-21.1** (`omnesagent-memory/src/consolidation.rs`): Добавить `extract_balanced_json` (вырезание первого сбалансированного JSON-объекта/массива с учётом строк и `\"` escape) перед `serde_json::from_str` + unit-тесты с пояснительным хвостом `Note: ...` и экранированными скобками.
- [x] **T-23.1** (`frontend/{desktop,web,shared}/lib`): Заменить все оставшиеся `0xFF00D2FF`, `0xFF00E5FF`, `0xFF0072FF` (23 файла) на семантические токены `DesktopTheme.accentBrand` / `DesktopTheme.accentBrandDark` / `ShadcnColors.primary` / `ShadcnColors.primaryMuted`.

## Волна 2 (P1)
- [x] **T-21.2** (`omnesagent-memory/src/dream.rs`): Прямое чтение `daily`-записей по курсору `last_daily_cursor` со сдвигом только при успехе прогона.
- [x] **T-21.3** (`omnesagent-ralph/src/loop_driver.rs`): Безопасное сохранение метаданных finding'ов при отсутствии `.git` / `HEAD` (`git_head_sha: None`), покрыто unit-тестом.
- [x] **T-21.4** (`.agents/skills/ob2h/SKILL.md`, `.agents/rules/ob2h.md`): Добавлено правило обязательного `session_ingest` в конце долгих сессий.
- [x] **T-23.2** (`frontend/desktop/pubspec.yaml`, `frontend/desktop/assets/fonts/`): Подключены локальные шрифты `Montserrat`, `Inter`, `JetBrains Mono` в `fonts:` и Google Fonts в `frontend/web/web/index.html`.

## Волна 3 (P2)
- [x] **T-21.5** (`omnesagent-kag/src/ast/{languages.rs, mod.rs}`): Поддержка ассемблера (`asm-gas` `.s`/`.S` / `asm-intel` `.asm`) в AST-реестре и экстракторе.
- [x] **T-22.1** (`omnesagent-kag/src/graph/service.rs`): Добавлено поле `warming_seconds_left` (оценка ETA) к `degraded_mode: "warming_up_embeddings"`.
- [x] **Ф22.2** (`.agents/skills/ob2h/SKILL.md`, `.agents/rules/ob2h.md`): Зафиксированы правила работы с `ob2h secrets ask-pin`, `memory_feedback` (`key`/`node_id`); проверено отсутствие зависающих `rpassword` без `is_terminal()` в `backend/`.
- [x] **Ф23.3** (`frontend/web/web/index.html`, `backend/apps/omnescode`): Добавлен `<meta name="theme-color" content="#111827">` и проверено отсутствие старого циана в `omnescode`.

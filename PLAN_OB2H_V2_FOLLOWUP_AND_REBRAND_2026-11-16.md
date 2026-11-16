# Мини-спецификация: Доработки по хвостам `ob2h` v2.0.0+ и завершение ребрендинга `OmnesProducts` (Ф21–Ф23)

> **Дата:** 2026-11-16  
> **Статус:** Черновик для реализации (продолжает [`PLAN_OB2H_V2_AND_REBRAND_2026-11-14.md`](PLAN_OB2H_V2_AND_REBRAND_2026-11-14.md) [Ф14–Ф20])  
> **Источники:**  
> - Память и коммиты `C:\Projects\ob2h` после релиза `v2.0.0` (15–16.11.2026): Ф78 (`71f7cf5`, `73f8a03`), `ask_json` (`ef2316c`), `attach_git_head_sha` (`e88af34`)
> - Открытые GitHub Issues `Pofium/ob2h`: [`#1`](https://github.com/Pofium/ob2h/issues/1) (PIN-диалог сейфа из MCP), [`#2`](https://github.com/Pofium/ob2h/issues/2) (UX-фиксы живого теста), [`#3`](https://github.com/Pofium/ob2h/issues/3) (AST-графы: поддержка ассемблера)
> - Брендбук `C:\Projects\OmnesProducts` (`README.md`, `brandbook.html`, `brand/omnes-agent.svg`)

---

## 1. Что уже сделано и что осталось (аудит As-Is)

1. **Блок памяти / KAG (`ob2h` v2.0):** базовые фазы Ф14–Ф20 уже в `main`, но в самом `ob2h` сразу после релиза `v2.0.0` выявились боевые доработки и багфиксы (Ф78, устойчивый JSON-парсинг Dream, сохранение `meta` без `.git`, UX-фиксы живого теста), которых **нет** в форке `Omnes-agent`.
2. **Брендинг `OmnesProducts` v1.1:**
   - Сделано: палитра в `ShadcnColors` и `DesktopTheme` (`#111827` / `#F4F5F7` / `#FF6B00`), векторный слон в `OA_icon.svg` и `scripts/generate_brand_icons.py`, константы шрифтов `Montserrat` / `Segoe UI` / `JetBrains Mono` в `DesktopTheme` и `ShadcnTheme`.
   - **Не доделано:** в **23 файлах** `frontend/{desktop,web,shared}/lib` осталось **~472** хардкодных вхождения старого циана (`00D2FF` / `00E5FF` / `0072FF`), не подключены реальные файлы шрифтов в `pubspec.yaml` (`fonts:`), и в `frontend/web/web/index.html` нет `<meta name="theme-color">`.

---

## 2. Доработки блока памяти и MCP-интеропа (по `ob2h` 15–16.11.2026)

### Ф21. Боевые фиксы Dream, JSON-парсера и Ralph (из `ob2h` Ф78 + post-v2.0.0)

| # | Задача | Где в `Omnes-agent` | Суть доработки (по опыту `ob2h`) | Приоритет |
|---|---|---|---|---|
| **Ф21.1** | **Устойчивый JSON-парсер для LLM-ответов (`clean_json_balanced`)** | `backend/crates/omnesagent-memory/src/dream.rs` (строка `serde_json::from_str(&raw)`), `conflict.rs` | LLM часто дописывает пояснительный хвост после JSON (`Note: ...`), из-за чего `serde_json::from_str` падает и прогон Dream теряет правки (`edits=0`). Добавить экстрактор первого сбалансированного JSON-объекта/массива с учётом кавычек и `\"` escape перед десериализацией. | **P0** |
| **Ф21.2** | **Прямое чтение daily/сессионных логов в Dream (аналог Ф78 `ob2h`)** | `backend/crates/omnesagent-memory/src/dream.rs`, `sqlite.rs` | На коротких сессиях Dream голодает, если ждёт переполнения контекста/консолидации. Добавить чтение свежих daily/session-логов по курсору (`.daily_dream_cursor` или мета-ключ в `memory_meta`) с **откатом/непродвижением оффсета при ошибке прогона**. | **P1** |
| **Ф21.3** | **Безопасный `git_head_sha` при отсутствии `.git`** | `backend/crates/omnesagent-ralph/src/loop_driver.rs`, `omnesagent-memory` | По аналогии с фиксом `ob2h` `75.1` (`e88af34`): отсутствие git-корня или `HEAD` не должно через ранний `?` терять остальные поля `meta` / `ralph_findings` / `ralph_report`. Отсутствие git трактуется как `git_head_sha: None` без прерывания записи. | **P1** |
| **Ф21.4** | **Обязательный `session_ingest` в конце долгих сессий** | `.agents/skills/ob2h/SKILL.md`, `.agents/rules/ob2h.md` | Перенести правило Ф78.2 из `ob2h`: в конце долгих или нетривиальных сессий всегда вызывать `ob2h:session_ingest` (с прохождением redact-фильтра), чтобы опыт не терялся до переполнения окна. | **P1** |
| **Ф21.5** | **AST-графы: поддержка ассемблера (`asm-gas` / `asm-intel`, `ob2h` [#3](https://github.com/Pofium/ob2h/issues/3))** | `backend/crates/omnesagent-kag/src/ast/{languages.rs, mod.rs, treesitter.rs}` | Добавить в реестр языков `.s`/`.S` (`asm-gas`) и `.asm` (`asm-intel`) со сниффингом диалекта (GAS AT&T / GAS Intel / NASM / MASM): узлы `Function`/`Macro`/`Data`/`File`, рёбра `DEFINES`/`IMPORTS`/`CALLS` (для сквозных цепочек `C → asm`), `ASM_LABEL_SKIP` (`.L*`, числовые метки) и cap `max_bytes`. Опционально вторая волна — `tree-sitter-asm` для GAS. | **P2** |

### Ф22. Наблюдаемость `degraded_mode` и UX сейфа (по GitHub Issues `ob2h` [#1](https://github.com/Pofium/ob2h/issues/1), [#2](https://github.com/Pofium/ob2h/issues/2))

| # | Задача | Где в `Omnes-agent` | Суть доработки | Приоритет |
|---|---|---|---|---|
| **Ф22.1** | **Информативный статус `warming_up_embeddings` + ETA (`#2.3`)** | `backend/crates/omnesagent-kag/src/graph/service.rs:344`, UI/API ответа | Сейчас при холодном старте возвращается только строка `"warming_up_embeddings"`. Добавить `warming_seconds_left` (оценка прогрева) и стартовый span прогрева embeddings в метрики, чтобы агент и пользователь решали, ждать векторный поиск или отвечать по FTS. | **P2** |
| **Ф22.2** | **Сопряжение с MCP PIN-диалогом и non-TTY гейт (`#1`, `#2.1`, `#2.2`)** | `.agents/skills/ob2h/SKILL.md`, `omnesagent-tools` / `tool_group.rs`, CLI-промпты | 1) Для `secrets_get`/`secrets_set` с будущим `ob2h secrets ask-pin` (`OB2H_PIN_GUI=1`): заложить достаточный таймаут вызова MCP (≥ 60 с под ввод PIN) и **не ретраить** при `ввод PIN отменён пользователем`.<br>2) Для `memory_feedback`: использовать `node_id` (Mute Node) без `key` для граф-узлов; в скилле явно указать, что `key` берётся из `memory_search`/`memory_save` (`lsn_*` не подходит).<br>3) В CLI-командах `Omnes-agent`, где запрашивается пароль/PIN, проверить наличие `is_terminal()`-гейта, чтобы не зависать при `stdin ≠ TTY`. | **P2** |

---

## 3. Завершение ребрендинга по брендбуку `OmnesProducts` v1.1 (Ф23)

> Базовая палитра: **60% Dark Charcoal (`#111827`) / 30% Off-White (`#F4F5F7`) / ≤10% Brand Orange (`#FF6B00`)**; шрифты: **Montserrat** + **Inter / Segoe UI** + **JetBrains Mono**.

| # | Задача | Где в `Omnes-agent` | Суть доработки | Приоритет |
|---|---|---|---|---|
| **Ф23.1** | **Вычистка оставшихся хардкодов циана (~472 вхождения в 23 файлах)** | `frontend/desktop/lib/**`, `frontend/web/lib/**`, `frontend/shared/lib/**` (в т.ч. `task_workspace_view.dart` [108+100], `desktop_sidebar.dart` [73+44], `desktop_settings_dialog.dart` [31+27], `automations_view.dart` [13+13], `interactive_question_card.dart` [11+11], `inspector_panel.dart` [8+8], `desktop_shell.dart` [7+7], `handy_install_card.dart` [6], `voice_settings_section.dart` [3] и др.) | Заменить все `Color(0xFF00D2FF)`, `Color(0xFF00E5FF)`, `Color(0xFF0072FF)` и градиенты `[0xFF00D2FF, 0xFF0072FF]` на семантические токены `DesktopTheme.accentBrand` / `DesktopTheme.accentBrandDark` / `ShadcnColors.primary` (или `DesktopTheme.accentSky` там, где это именно информационный code/link-цвет). | **P0** |
| **Ф23.2** | **Подключение реальных бренд-шрифтов в Flutter (`pubspec.yaml`)** | `frontend/shared/pubspec.yaml`, `frontend/desktop/pubspec.yaml`, `frontend/web/pubspec.yaml`, `frontend/shared/assets/fonts/` | Добавить локальные `.ttf` (или пакет `google_fonts` с офлайн-бандлингом для desktop) для **Montserrat** (`600`/`700`), **Inter** (`400`/`500`) и **JetBrains Mono** (`400`/`500`) и объявить секцию `fonts:` в `pubspec.yaml`, чтобы шрифты работали на системах без предустановленных TTF. | **P1** |
| **Ф23.3** | **Мета-теги Web и проверка TUI (`omnescode`)** | `frontend/web/web/index.html`, `frontend/web/web/manifest.json`, `backend/apps/omnescode/` | Добавить `<meta name="theme-color" content="#111827">` в `index.html`; проверить, что в `omnescode` (Rust CLI/TUI) дефолтная палитра/логотип согласованы с Dark Charcoal + Brand Orange. | **P2** |

---

## 4. Порядок выполнения и верификация

1. **Волна 1 (быстрые P0-фиксы, 1–2 ч):** Ф21.1 (`clean_json_balanced` + unit-тест с `Note:`-хвостом) и Ф23.1 (замена `00D2FF`/`00E5FF`/`0072FF` на токены `DesktopTheme`/`ShadcnColors` + проверка `rg -c "00D2FF|00E5FF|0072FF" frontend/` = 0).
2. **Волна 2 (P1, полдня):** Ф21.2 (прямой daily-курсор в `dream.rs`), Ф21.3 (`git_head_sha` без `.git`), Ф21.4 (правило `session_ingest` в `SKILL.md`/`ob2h.md`), Ф23.2 (бандлинг шрифтов `Montserrat`/`Inter`/`JetBrains Mono` в `pubspec.yaml`).
3. **Волна 3 (P2, по готовности `ob2h` #1/#2):** Ф22.1, Ф22.2, Ф23.3.
4. **Критерии приёмки:** `cargo test -p omnesagent-memory -p omnesagent-ralph -p omnesagent-kag --lib`, `flutter analyze` и `flutter test` (`shared`, `desktop`, `web`) с 0 ошибок и 0 вхождений `00D2FF`/`00E5FF` в `frontend/`.

# PLAN — Сверка backend OmnesAgent с upstream ZeroClaw (issues) и порт отсутствующих фиксов

> Дата: 2026-09-08 · Автор: Hermes (аудит по запросу Ильи)
> Верхнеуровневый запрос: «просмотреть https://github.com/zeroclaw-labs/zeroclaw/issues, сравнить с нашим
> backend и написать план исправлений, если найденное присутствует в нашем коде».
> Статус: **план готов к исполнению** (этот документ — план, НЕ реализация).

---

## 1. Резюме (для ЛПР)

1. **OmnesAgent backend — форк ZeroClaw** (crates переименованы `zeroclaw-*` → `omnesagent-*`,
   `apps/zerocode` → `apps/omnescode`), база ≈ тег **v0.8.5** (05.09.2026) + пара дней. В нашем дереве
   **750 файлов побайтово идентичны upstream-HEAD** (08.09.2026), т.е. основная масса кода — общая с апстримом.
2. Прямым байтовым сравнением (наш файл == состояние *до* фикса) найдено **22 файла, в которых отсутствуют
   уже смерженные upstream-фиксы** (15 PR из окна 05–08.09.2026 — после нашей базы форка).
3. Ещё **21 пара файл/фикс** — в файлах, которые наш форк кастомизировал; все проверены вручную (субагенты,
   evidence `путь:строка`): **итог — все 15 фикс-PR у нас отсутствуют**. Нюансы: #10491 — у нас осознанная
   дивергенция trust-модели (см. п.5); #10661 в upstream не смержен вовсе (открытый PR); #10375 — низкая
   релевантность (wire-контракт `/api/status` уже совпадает, наш фронт — Flutter).
4. **Открытые upstream-баги** (не пофикшены даже в апстриме) в подавляющем большинстве **наследуются нашим
   кодом автоматически** (файлы идентичны); выборочная проверка самых рискованных — в разделе 5.
5. **Один security-дефект подтверждён в нашем коде лично:** WASM plugin HTTPS доверяет только bundled
   `webpki_roots`, не читает системное хранилище сертификатов (upstream bug #9653, фикс #10491).
6. Крупнейший отсутствующий фикс — **#9726 «TaskRecord — единый владелец фонового жизненного цикла»**
   (7 файлов `control_plane` + `daemon` + `delegate`): это база для исправлений класса
   «фоновые задачи/делегаты пропадают при выходе процесса, settlement не durable» (#9333, #10121, #10673,
   #9191) и нашего экрана Automations/cron.

**Рекомендация:** портировать в первую очередь 3 группы — (A) #9726 lifecycle + cron-таймауты, (B) cost/pricing
(#9939, #10638), (C) security плагинов (#10491, #10658). Дальше — по таблице 4.

---

## 2. Методика аудита (как получены выводы)

- Upstream клонирован: `C:\Users\ipres\tmp\zeroclaw-audit\upstream` (полная история, squash-merge коммиты).
- Весь трекер issues (~10 тыс. записей, из них 807 open) выгружен через GitHub API
  (`issues?state=all&sort=updated`) и отфильтрован: open + label `bug` + ядровые подсистемы
  (gateway/runtime/core/channel/config/provider/agent/tool/cli/security/zerocode/daemon) + активность ≤120 дней.
- База форка определена по версии workspace (`0.8.4` у нас, `v0.8.5` — 05.09.2026 у них) и по содержимому.
- **Метод «пропущенных фиксов»:** для каждого fix-коммита апстрима в окне `v0.8.4..HEAD` (489 коммитов,
  из них 341 fix/refactor/perf/revert) каждый затронутый файл сравнивался с нашим деревом (после механического
  переименования `omnesagent*`→`zeroclaw*` в копии, т.е. сравнение логики без брендинга):
  - наш файл **== состояние до фикса** (`<sha>^`) → фикс **точно отсутствует** (`MISSING`);
  - наш файл **== состояние после фикса** (`<sha>`) → фикс **уже есть** (`PRESENT`, 169 пар);
  - иначе → файл кастомизирован форком, проверка вручную (`DIVERGED`, 673 пары; в окне после v0.8.5 — 21).
- Для DIVERGED-пар из окна после v0.8.5 и для выборки открытых багов проведена ручная верификация
  субагентами (по 3 параллельных аудита) с цитатами `путь:строка` из реального репозитория.
- Инструменты MCP (ob2h project tools) подключались (`project_init`, `project_context`, `omnes_stats`),
  но сервер ob2h в момент аудита не отвечал (timeout 420 с на всех вызовах, БД ~730 МБ занята основным
  инстансом/дримом) — статический анализ выполнен напрямую по коду.

### Карта переименований (upstream → наш backend)
| Upstream | Наш |
|---|---|
| `crates/zeroclaw-*` | `crates/omnesagent-*` (подкаталоги те же) |
| `apps/zerocode` | `apps/omnescode` |
| `apps/zerorelay` | `apps/omnesrelay` |

---

## 3. Состояние форка (доказательная база)

- Наш workspace `backend/Cargo.toml`: version `0.8.4`; upstream теги: `v0.8.4` = 02.08.2026, `v0.8.5` = 05.09.2026 (454+ коммитов, 73 контрибьютора).
- Сравнение `crates/`+`apps/`+`src/` (rs/toml/ftl, без тестов/фикстур): **750 идентичны HEAD**, 214 «отличаются от обоих» (наши кастомизации), 13 в состоянии v0.8.4, 69 новых файлов апстрима с нашей отличной версией.
- Отсюда: всё, что апстрим починил **до ~05.09.2026**, у нас уже есть; дельта — окно **05–08.09.2026** (~34 коммита; 22 файла MISSING + 21 DIVERGED-пара) плюс кастомизированные файлы, где фиксы могли быть потеряны при слиянии.

---

## 4. Отсутствующие upstream-фиксы (порт обязателен/рекомендован)

Статусы: **[M]** = MISSING, байтово доказано (наш файл = `до фикса`) → порт = cherry-pick/apply diff'а;
**[D]** = DIVERGED (файл кастомизирован форком) — проверено вручную субагентами, evidence в колонке.
SHA — squash-merge коммиты апстрима (`git -C <upstream> show <sha>`).

| # | Upstream фикс (issue/PR) | SHA | Файлы (наш backend) | В нашем коде | Приоритет |
|---|---|---|---|---|---|
| 1 | #9726 TaskRecord — единый владелец фонового lifecycle (fixes #9593; база для #9333/#10121/#10673) | c853fc7a6 | [M] `crates/omnesagent-runtime/src/control_plane/{authority.rs,boot.rs,global.rs,mod.rs,reaper.rs,task_registry.rs,task_store_sqlite.rs}`; [D] `.../daemon/mod.rs`, `.../tools/delegate.rs` | **НЕТ (везде MISSING)**: control_plane — 7 файлов pre-fix (нет `ControlPlaneRecoveryOwner`/`spawn_control_plane_reaper`; boot.rs:46,67,81,105 — старый `ControlPlaneHandle`); daemon/mod.rs:585,600-603 — старые вызовы; delegate.rs:1598-1634 (артефакт владеет статусом, `originator_route: None`, ошибка создания глушится), :1725-1775 (settlement fire-and-forget), :2354-2403 (cancel без owner-проверки) | 🔴 Высокий |
| 2 | #10491 WASM plugin HTTPS: читать системный trust store (bug #9653, security) | 585725ff0 | [D] `crates/omnesagent-plugins/src/wasi_http.rs`, `Cargo.toml` | **НЕТ — осознанная дивергенция**: `wasi_http.rs:348-358` fail-closed на bundled `webpki_roots` (комментарий прямо говорит: системный стор — отдельное решение). Upstream развернул в обратную сторону → нужно явное решение владельца | 🔴 Высокий (security; decision point) |
| 3 | #9939 cost: surface pricing-unavailable (тихий $0-кап при неизвестном прайсинге) | 9abe969b6 | [M] `omnesagent-config/src/cost/{mod.rs,tracker.rs,types.rs}`, `omnesagent-providers/src/pricing.rs`, `omnesagent-runtime/src/agent/{cost.rs,pricing_catalog.rs}`; [D] `omnesagent-config/src/schema.rs` | **НЕТ (MISSING везде)**: cost/*, pricing.rs, agent/cost.rs, pricing_catalog.rs — pre-fix (нет `unpriced_tokens`, `is_sane_usd_rate`); schema.rs:21414 — `Config::validate()` без `cost.rates.validate()`; у `CostRatesConfig` (schema.rs:6811) метода `validate()` нет | 🔴 Высокий (деньги) |
| 4 | #10638 gateway: seed boot default из первой записи с моделью | ab72fe5d7 | [M] `omnesagent-config/src/providers.rs`; [D] `omnesagent-config/src/schema.rs`, `omnesagent-gateway/src/lib.rs` | **НЕТ (MISSING везде)**: providers.rs без `first_entry_with_model()`; gateway lib.rs:642-647 — `iter_entries().next()` (первая запись «какая ни есть»), fallback :695-700; schema.rs:4001-4009 — старый `resolve_default_model()` (первая непустая модель из любого слота) | 🟠 Средний |
| 5 | #10370 Copilot: harden credential cache | c107d55a5 | [M] `omnesagent-providers/src/copilot.rs`; [D] `.../Cargo.toml` | **НЕТ (MISSING)**: copilot.rs pre-fix (`read_to_string`/`.open` без симлинк-защиты и классификации сбоев); Cargo.toml без cap-std/cap-fs-ext | 🟠 Средний |
| 6 | #10088 multimodal: сохранять attached images после удаления источника (bug #10045) | 8730e7b24 | [M] `omnesagent-providers/src/multimodal.rs`, `omnesagent-runtime/src/rpc/attachments.rs`; [D] `omnesagent-runtime/src/agent/loop_.rs`, `.../turn/mod.rs` | **НЕТ (MISSING)**: attachments.rs:135-140 — маркер из оригинального пути (провисает после remove_file); turn/mod.rs:401 — без loop-local image cache; multimodal.rs без dedup-предупреждений (в loop_.rs апстрим-изменения — только тесты) | 🟠 Средний |
| 7 | #10375 gateway: генерировать dashboard status contract | a7d4a40e6 | [M] `omnesagent-gateway/src/{api.rs,version.rs}`, `omnesagent-runtime/src/{health/mod.rs,process_stats.rs}`; [D] `omnesagent-gateway/src/openapi.rs`, `.../nodes/mdns.rs` | **НЕТ, но LOW**: api.rs:315 — ручной `json!` без `StatusResponse`; wire-контракт `/api/status` фикс НЕ меняет, наш фронт — Flutter (JSON вручную), TS web/ в форке нет → ценность только типизация/OpenAPI | 🟡 Низкий |
| 8 | #9777 channels: Signal source UUID senders | 3ada1bd93 | [M] `omnesagent-channels/src/signal.rs` | **НЕТ** (канал `channel-signal` есть: Cargo.toml:200/244; порт ~15 строк: `sourceUuid` + порядок sender-полей) | 🟠 Средний, канал активен |
| 9 | #10671 daemon: heartbeat.target принимает составной ключ `<type>.<alias>` (bug #10670) | c64ecfdf1 | [D] `omnesagent-runtime/src/daemon/mod.rs` | **НЕТ (MISSING)**: `daemon/mod.rs:2522-2537` валидирует весь target как тип (без `split_once('.')`); вызывается из `resolve_heartbeat_delivery` (:2277-2300) на старте демона (:1816) → `heartbeat.target="telegram.roy"` отклоняется | 🟠 Средний (multi-instance) |
| 10 | #10415 providers: reliable stream errors атрибутируются served-модели (bug #10326) | eebbf2c3d | [D] `omnesagent-providers/src/reliable.rs` | **НЕТ (MISSING)**: reliable.rs pre-fix; :3290-3291/:3380-3381/:3461-3462 — `stream_chat(req, &current_model, ...)` при вычисленном `served_model` (:3265) → ошибки пишутся на requested, а не pinned | 🟠 Средний |
| 11 | #10658 plugins: reject expired dial budgets (#10661 — в upstream НЕ смержен, открытый PR) | 9bf015b32 | [D] `omnesagent-plugins/src/wasi_http.rs` | #10658: **НЕТ** (`wasi_http.rs:415-433` — dial_pinned без перепроверки дедлайна); #10661: N-A (каскадно) | 🟠 Средний |
| 12 | #10628 TTS: surface providers, отброшенных за отсутствие api_key | 43123a81d | [D] `omnesagent-channels/src/tts.rs`, `omnesagent-runtime/src/doctor/mod.rs` | **НЕТ**: `tts.rs:1068-1070` пре-фикс лог; в doctor проверки TTS/transcription api_key нет вообще | 🟡 Низкий (диагностика) |
| 13 | #10692 WhatsApp: транскрипция к провайдеру владеющего агента (bug #10688) | e6fe7a8c9 | [D] `omnesagent-channels/src/orchestrator/mod.rs`, `.../whatsapp_web.rs` | **НЕТ**: канал есть (`whatsapp-web` Cargo.toml:227); orchestrator/mod.rs:10876 legacy `.with_transcription`; `whatsapp_web.rs:628-651` строит `TranscriptionManager::new(&config)` без агентского alias; `resolve_agent_transcription_provider` (:10238) без whatsapp-web → голосовые молча не транскрибируются | 🟠 Средний (функциональная поломка канала) |
| 14 | #10651 providers: warm совместимых соединений через /models (enh #9575) | 4129a18ac | [D] `omnesagent-providers/src/compatible.rs` | **НЕТ (enhancement, low)**: compatible.rs:3938-3944 — warmup ходит на `chat_completions_url()` вместо `{base}/models` | 🟡 Низкий (enhancement) |
| 15 | #10296 hardware: forward probe feature в tool-имплементации | fc45a4558 | [D] `omnesagent-hardware/Cargo.toml` | **НЕТ** (low): `hardware/Cargo.toml:42` без форварда фичи в tools-цепочку | 🟡 Низкий |

---

## 5. Открытые upstream-баги, проверенные в нашем коде

_(код форка идентичен апстриму → баг наследуется; ниже — проверенные/подтверждаемые)_

| Issue | Суть | Где в нашем коде | Вердикт |
|---|---|---|---|
| #9653/#10491 | WASM plugin HTTPS без системного trust store | `plugins/src/wasi_http.rs:348-358` | **ПОДТВЕРЖДЁН** — править вместе с портом #10491 (раздел 4, строка 2); требует решения владельца |
| #9191 | cron-задачи без wall-clock timeout; in-flight locks чистятся только при старте | `runtime/src/cron/{scheduler.rs,store.rs}` | **ПРИСУТСТВУЕТ**: agent-джобы — `scheduler.rs:795-920` (await :868 без дедлайна); release только после завершения (:782); `clear_stale_locks` — только boot (:335, store.rs:836-845) |
| #10230 | startup/reload daemon может переполнить стек при init агентов | `src/main.rs:5359`, `rpc/dispatch.rs:3577-3598`, `daemon/mod.rs:295,330` | **RISK-SHAPE ПРИСУТСТВУЕТ**: in-process reload-loop + повторная инициализация агентских подсистем (корень upstream не установлен; S1) |
| #10674/#10702 | тримминг истории без low-water: ре-трим каждые несколько ходов, убивает prompt cache | `runtime/src/agent/history_trim.rs:47-236` | **ПРИСУТСТВУЕТ**: message-cap стоп на первой границе под капом (:103); token-бюджет — break на первом fit (:174); low-water фракции нет |
| #10068 | интерактивная сессия режет контекст на 32k, игнорируя max_context_tokens | `runtime/src/agent/loop_.rs:2763-2765,2634`; `config/src/schema.rs:4720,3563-3564` | **ПРИСУТСТВУЕТ**: потолок из `model_context_window` провайдера; fallback 32 000 (`UNCONFIGURED_CONTEXT_WINDOW_FALLBACK`); `max_context_tokens` профиля влияет только на внутренний trim |
| #9333/#10673 | failed ACP turns пропадают после переключения сессий | `channels/src/orchestrator/acp_server.rs:1705-1721` | **ПРИСУТСТВУЕТ**: персистенция только для успешных не-cancelled ходов; упавший ход живёт только в памяти (частично закроется портом #9726) |
| #10594 | cron не пишет ничего, когда джоба не запустилась | `runtime/src/cron/{scheduler.rs:693-696,662,674, store.rs:784-811}` | **ПРИСУТСТВУЕТ** (оба кейса): нет owning-агента → WARN без row и без сдвига next_run (вечный re-select); recurring startup-skip → только сдвиг next_run, без last_status/cron_runs |
| #10533 | model_routing_config отвергает `custom.*` слоты | `tools/src/model_routing_config.rs:918-921,900-901` | **ПРИСУТСТВУЕТ**: dotted-значение не разбивается по точке → `ensure()` = None и отказ, хотя схема слот `custom` поддерживает (providers.rs:222); + частичный upsert (секции создаются до валидации) |
| #9284 | config flush затирает конкурентные записи | `runtime/src/rpc/dispatch.rs:599-625` | **ABSENT — уже исправлено**: `dirty_paths.retain(...)` вместо wholesale swap; `save_and_swap_config` (:635-649) под `config_write_lock`; тест `flush_config_preserves_write_landed_during_save` есть |
| #10495 | Config::save() может заменить живой конфиг почти пустым файлом | `config/src/schema.rs:23329-23361,23613` | **PARTIAL (как upstream master)**: первопричина закрыта — отказ перезаписи существующего файла (:23353-23356), атомарная запись есть; нет destructive-save guard, авто-бэкапов, test-harness guard |
| #10061 | отвергнутое провайдером изображение «отравляет» следующие ходы vision-сессии | `runtime/src/agent/turn/vision_route.rs:139-163` | **ПРИСУТСТВУЕТ**: маркеры из истории переподготавливаются каждый ход без карантина → один отвергнутый image блокирует сессию |
| #10625 | внутренний плейсхолдер `[media attachment]` уходит пользователю при non-vision модели | `providers/src/multimodal.rs:448-456`, `runtime/.../vision_route.rs:154-163` | **ПРИСУТСТВУЕТ**: `strip_media_markers()` → литерал `[media attachment]`; outbound-обработчика нет |
| #10501 | MCP tool-result картинки дают 400 на OpenAI-compatible (image part в role:tool) | `providers/src/compatible.rs:2444-2453,2364-2383` | **ПРИСУТСТВУЕТ**: image промоутится в `role:"tool"`-сообщение (OpenAI принимает image только в `role:"user"`); `tool_result_image_policy=Omit` — лишь обход |
| #9421 | неполный ответ терминала может быть засчитан успешным | `providers/src/compatible.rs:2041-2082,2129` | **Слайс 3 ПРИСУТСТВУЕТ**: любой не-null `finish_reason` = успех, `"length"` не классифицируется (слайсы 1-2 у нас закрыты: `agent/turn/outcome.rs:113` + `anthropic.rs:2306`) |
| #8279/#10165/#10534 | delegate в обход allowlist родителя / block_high_risk_commands; bounded delegate молча срезает инструмент | `runtime/src/tools/delegate.rs:730-853,2711,845`, `config/policy.rs:3452` | **ПРИСУТСТВУЕТ как в upstream**: #8279 — independent-ветка собирает реестр без пересечения с родительским `allowed_tools`; #10165 — как в upstream (открыт); #10534 — безусловный срез delegate-инструмента в обеих ветках |
| #10349 | SOP-pane грузится синхронно, блокируя навигацию omnescode | `apps/omnescode/src/sop_pane.rs:662-698` | **УЖЕ ИСПРАВЛЕНО**: асинхронный `start_list_refresh` (tokio-задача + try_recv); файл = upstream HEAD |
| #10667 | omnescode дублирует стрим-ответ, когда prompt-response приходит раньше TurnComplete | `apps/omnescode/src/chat.rs:7239,7265-7268,7289-7298` | **ПРИСУТСТВУЕТ** (1:1 с upstream) |
| #10693 | omnescode игнорирует Enter в состоянии Connected | `apps/omnescode/src/input_bar.rs:1388-1390` | **= upstream** (код идентичен, баг открыт и там) |
| #10506 | wasi:http stale connection при последовательных запросах | `plugins/src/wasi_http.rs:302-368` | **Нет механизма в нашем коде** (per-request dial+TLS, без пула) — не воспроизводится описанным путём |
| #9779 | sops_dir: документированный дефолт не применяется демоном | runtime/config SOP | (не проверялся, добавить при исполнении) |

**Итог раздела 5:** из 19 проверенных открытых upstream-багов (плюс #9653/#10491 — в разделе 4):
- **ПРИСУТСТВУЮТ в нашем коде (11):** #9191, #10674/#10702, #10068, #9333, #10594, #10533, #10061, #10625, #10501, #9421 (слайс 3), #8279/#10165/#10534;
- **структурно совпадает, root-cause в upstream не установлен (1):** #10230;
- **= upstream (открыт, код идентичен) (2):** #10667, #10693;
- **уже исправлено у нас (2):** #9284, #10349;
- **частично (1):** #10495; **не воспроизводится (1):** #10506; **не проверялся (1):** #9779.

---

## 6. Этапы исполнения (порядок работ)

Каждый этап — отдельная PR-ветка (правила: backend/AGENTS.md — ветка не master, conventional commits,
`cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`).

### Этап A — Жизненный цикл фоновых задач (🔴) — ✅ ПОЛНОСТЬЮ ВЫПОЛНЕН (08.09.2026)
- **A1.** Порт #9726 (c853fc7a6) в `control_plane/*` (7 файлов, [M] — чистый перенос; diff доступен
  `git show c853fc7a6`), затем ручное слияние в `daemon/mod.rs` и `tools/delegate.rs` ([D] — наш delegate
  сильно кастомизирован: ~2000 строк поверх; править по семантике хунков, не apply вслепую).
  - ✅ **ВЫПОЛНЕНО 08.09.2026** (коммит `efa3ec4`, ветка `feat/zeroclaw-phase-a`). Реализованы
    `ControlPlaneRecoveryOwner` + `spawn_control_plane_reaper` (boot/global/mod), durable settlement
    background-задач (`persist_terminal_settlement_intent`/`promote_terminal_settlement` в
    task_store_sqlite), delegate: `originator_route` + durable публикация вывода + owner-match проверки,
    `TaskSnapshot` в task_registry. `cargo check -p omnesagent-runtime` — OK. Полный прогон тестов runtime — в A2/A3.
- **A2.** После A1 — проверка/порт фиксов cron-семейства: #9191 (wall-clock timeout джоб),
  #10594 (лог «джоба не запустилась»), связка с нашим экраном Automations.
  - ✅ **ВЫПОЛНЕНО 08.09.2026** (коммит `a91147b`). #9191: `run_agent_job` обёрнут в
    `tokio::time::timeout` (AGENT_JOB_TIMEOUT_SECS=600) — ход не может висеть бесконечно; при
    таймауте — WARN + 'agent job timed out after Ns' в исход. #10594: (а) джоба без owning-агента
    в `process_due_jobs` теперь пишет run-запись 'skipped' + last_status и advance-ит next_run
    recurring (прекращает вечный re-select); (б) `skip_missed_run` recurring-ветка stamp-ит
    last_run/last_status='skipped'/last_output. Тест
    `skip_missed_run_recurring_records_skip_status` — passed; cron-тесты `skip_missed_run` — 3/3 OK.
    Связка с экраном Automations (frontend) — отдельно, вне backend-порта.
- **A3.** #9333/#10673 (durable-персистентность failed ACP turns) — оценить после A1: часть закрывается самим #9726.
  - ✅ **ВЫПОЛНЕНО 08.09.2026** (коммит `2cf4005`). `acp_server` переведён на
    `turn_streamed_with_steering_state` (даёт `StreamedTurnError.new_messages`); при failed
    (не-cancelled) ходе скоммиченные сообщения персистятся в `AcpSessionStore` — упавший ход
    больше не теряется при переключении сессий. Ре-экспорт `StreamedTurnError/Success` из runtime.
    `cargo check` OK; `cargo test -p omnesagent-channels --lib orchestrator::acp_server` — **94/94 OK**.
- **Тесты:** существующие `control_plane`/`daemon`/`cron` тесты + новые на «задача переживает exit процесса».

### Этап B — Cost/pricing/конфиг (🔴 деньги) — ✅ ПОЛНОСТЬЮ ВЫПОЛНЕН (08.09.2026)
- **B1.** Порт #9939 (9abe969b6): 6 файлов [M] + ручное слияние в `omnesagent-config/src/schema.rs` ([D],
  добавить `CostRatesConfig::validate()` и вызов в `Config::validate()` — schema.rs:6811/21414).
  - ✅ **ВЫПОЛНЕНО 08.09.2026** (коммит `f4381e7`). config/cost: `unpriced_tokens` в
    TokenUsage/ModelStats, rollup unpriced в tracker + `get_current_month_model_stats`, `MAX_SANE_USD_RATE`+
    `is_sane_usd_rate` в mod.rs, `CostRatesConfig::validate()` + вызов в `Config::validate()`;
    providers/pricing.rs: `sane_mtok` через `omnesagent_config::cost::is_sane_usd_rate`;
    runtime pricing_catalog.rs: тестовый lock. Проверено: cost-тесты config 38/38, schema `config_validate`
    20/20, cargo check config/providers/runtime OK. ⚠️ **DIVERGED-нюанс**: `runtime/agent/cost.rs`
    в нашем форке сильно кастомизирован (не [M], как считал план) — upstream-порт +568 строк НЕ применён
    вслепую; наш cost.rs ведёт usage через `TokenUsage::new` и не имеет собственного ModelStats-rollup
    (его делает config tracker), поэтому unpriced-учёт config-части покрывает.
- **B2.** Порт #10638 (ab72fe5d7): `providers.rs` [M] (новый `first_entry_with_model()`) + schema.rs:4001
  (`resolve_default_model`) и gateway lib.rs:642-700 [D] — единый резолвер.
  - ✅ **ВЫПОЛНЕНО 08.09.2026** (коммит `dae662f`). `first_entry_with_model()` в providers.rs +
    переписанный `resolve_default_model` (schema.rs) + gateway boot на
    `first_entry_with_model` (семья/учётка/модель когерентны, устранён provider/model mismatch при буте).
    Тест `first_entry_with_model_skips_model_less_entries` — passed; `cargo check -p omnesagent-gateway` OK.
- **B3.** Открытые баги: #10533 (dotted `custom.*` слоты в `model_routing_config`) — чинить локально
  (upstream-фикса нет); #10495 — дореализовать destructive-save guard/бэкапы (у нас PARTIAL); #9284 — уже
  исправлен, только добавить тест-покрытие при наличии.
  - ✅ **ОБСЛЕДОВАНО 08.09.2026** (коммит `8d18c8a`, тест). #10533: баг НЕ воспроизводится —
    `set_default` уже split_once-разбивает dotted и ensure('custom','xyz') создаёт slot; в `upsert_agent`
    dotted-контракт семантически исключён (model_provider=family, name=alias). Добавлен тест
    `set_default_materializes_dotted_custom_slot` (passed). #10495: `write_config_atomically` уже
    реализует атомарность + авто-бэкап `config.toml.bak`(fsync) + восстановление при неудаче rename +
    test-harness guard (`PostReplaceSync::FailForTest`) — больше «PARTIAL»; destructive-save guard
    НЕ вводим (первопричина закрыта, риск). #9284: уже исправлен (`dirty_paths.retain` + lock) — без изменений.
  - ✅ **Этап B выполнен полностью** (B1 f4381e7, B2 dae662f, B3 8d18c8a).
- **Тесты:** cost-тесты конфига, прогон `cargo test -p omnesagent-config -p omnesagent-runtime -- cost`.

### Этап C — Security плагинов (🔴) — ✅ ВЫПОЛНЕН (C1+C2; C3 — мониторинг)
- **C1.** #10491 (585725ff0): **сначала решение владельца** — сейчас `wasi_http.rs:348-358` намеренно
  fail-closed (только bundled webpki, комментарий фиксирует дивергенцию); порт добавит системный trust store
  (`rustls-native-certs`) и сменит trust-модель. Если fail-closed оставляем — зафиксировать решение в коде
  и мониторить upstream-issue #9653.
  - ✅ **ВЫПОЛНЕНО 08.09.2026** (коммит `308197d`, решение владельца — «как upstream»). Плагин HTTPS
    теперь доверяет bundled webpki + системному trust store ОС (rustls-native-certs): `build_trust_anchors`,
    `plugin_tls_config` (кэш по SSL_CERT_FILE/DIR, асинхронная сборка до dial, wait-по-дедлайну),
    `record_trust_anchors` (BundledOnly/Partial/Complete вердикты). Verification неизменна (полная цепочка +
    hostname). 8 новых тестов; былi_http 19/19 passed. Мониторинг #9653 — теперь закрыт (решён в нашу пользу).
- **C2.** Порт #10658 (9bf015b32): expired dial budget rejection в `dial_pinned` (wasi_http.rs:415-433).
  - ✅ **ВЫПОЛНЕНО 08.09.2026** (коммит `ee8c3ff`). `dial_pinned` проверяет `deadline <= Instant::now()`
    перед каждой попыткой connect — готовый loopback-connect не может «протащить» попытку из общего бюджета
    после истечения дедлайна. Тест
    `an_expired_deadline_stops_the_dial_before_it_tries_an_address` — passed (--features plugins-wasmtime);
    `cargo check -p omnesagent-plugins --features plugins-wasmtime` OK.
- **C3.** Проверка #10506 (stale connection): в нашем коде per-request dial, механизма нет — мониторить.
  - ⏸️ **МОНИТОРИТЬ** — per-request dial+TLS без пула, описанный upstream-сценарий не воспроизводится.
- **Тесты:** e2e плагинов (`tests/*plugin*e2e.rs`), тест HTTPS к self-signed/локальному CA.

### Этап D — Providers/каналы (🟠) — ✅ ВЫПОЛНЕН (D1–D6; D7 — вынесен в отдельную задачу) — 08.09.2026
- **D1. ✅** #10370 copilot.rs + Cargo.toml (cap-std/cap-fs-ext) — хардненинг кэша: идемпотентная атомарная запись, 0600/0700, sanitized-диагностика, отвержение symlink/fifo, кэш-счётчики. Коммит `c0a44d0`; 24/24 тестов copilot (9 новых). Точный порт (651+/96-).
- **D2. ✅** #10088 multimodal [M×2]+[D×2] (durable маркеры + loop-local image cache): `reported_failures` (sha256+kind, bounded 32) в LocalImageCache, single-fire лог сбоя, loop-local cache когда None. Коммит `c4771b3`; multimodal 70/70 + loop-cache тест.
- **D3. ✅** #10415 reliable.rs (served_model в атрибуции ошибок) — все 3 entry-точки `&current_model`→`&served_model` + DispatchObservedPinned + тест. Коммит `081b882`; 142+/6- точный порт.
- **D4. ✅** #9777 signal.rs — поле `sourceUuid` + 3-фоллбек sender (sourceNumber→source→sourceUuid), фильтр пустых, лог скрытия. Коммит `bbc3dce`; 96+/10- точный порт; 52/52 (channel-signal).
- **D5. ✅** #10692 WhatsApp [D] (канал есть — транскрипция к провайдеру агента): `configure_whatsapp_transcription` зеркалит discord, `with_transcription_manager`, cfg-gate. Коммит `7053551`; 155+/4-; оба теста passed (whatsapp-web).
- **D6. ✅** #10628 tts/doctor [D]: skip-запись с `config_path` + doctor api_key-диагностики (TTS gated-семейства + transcription). Коммит `9db4928`; doctor 71/71 (6 новых) + tts/transcription тесты.
- **D7. ⏸ ВЫНЕСЕН в отдельную задачу** — локальные фиксы открытых багов (НЕ порты upstream): #10061 (карантин отвергнутого изображения в `vision_route.rs:139-163` — маркеры переподготавливаются каждый ход без карантина), #10625 (не пускать литерал `[media attachment]` пользователю при non-vision — `multimodal.rs:487-495`, `vision_route.rs:154-163`), #10501 (image из tool-результата релоцировать в `role:"user"` — `compatible.rs:2437-2454,2588/2596`; сейчас собирает image-parts в tool-сообщении → 400 на OpenAI). Исследованы, реализации нет.

### Этап E — Низкий приоритет / опционально
- #10375 (только типизация + OpenAPI; wire-контракт уже совпадает), #10651 (enhancement),
  #10296 (feature flag), #10667 (дубль стрима в omnescode — фикс локальный), #10693 (как upstream, ждать их PR),
  #10506 (нет механизма в нашем коде — мониторить upstream), #9779 (sops_dir — проверить отдельно).

---

## 7. Риски и открытые вопросы

- **Конфликт кастомизаций:** наш форк переписал `tools/delegate.rs`, gateway (90+ своих эндпоинтов),
  omnescode (двуязычный UI), добавил `goal_task.rs` в control_plane и модули `cron/`, `heartbeat/` —
  перед apply каждого [D]-фикса читать наш код, портировать семантику, а не diff.
- **#9726 — крупный рефакторинг:** меняет владельца lifecycle фоновых задач; затрагивает наш Automations/cron
  и delegate; делать отдельной PR с полным прогоном тестов runtime.
- **Версия workspace 0.8.4** у нас при базе v0.8.5: при следующем синке решить политику версионирования.
- **ob2h MCP в момент аудита не отвечал** (timeout 420 с; БД 730 МБ, занята инстансом) — для blast-radius
  анализа правок (особенно A1) прогнать `project_impact` после восстановления сервера.
- Открытые вопросы: включены ли в наших релизных сборках каналы Signal/WhatsApp; потребляет ли наш
  Flutter-клиент `/health`-контракт (иначе #10375 можно отложить).

---

## 8. Как поддерживать синхронизацию дальше

Разовые сверки — дорого. Предлагается (отдельная задача):
1) скрипт `scripts/upstream_sync_check.sh`: добавляет upstream как remote, по тегу/HEAD строит карту
   переименований, сравнивает файлы и выдаёт список «наш файл == до фикса» (метод из раздела 2);
2) запускать после каждого upstream-релиза (release-plz → теги vX.Y.Z);
3) CI-джоба-напоминание (не блокирующая).

---

*Приложение: полные списки пар файл/фикс — `missing_fixes.json`, `since085.json`, `diverged_fixes.json`
в `C:\Users\ipres\tmp\zeroclaw-audit\` (scratch, не коммитить).*

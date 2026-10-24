# План синка с апстримом ZeroClaw: v0.8.5 → v0.8.6 (644 коммита)

> **Дата:** 2026-10-24
> **Горизонт:** наш форк отпочкован на `43123a81d` = PR #10628 = релиз **v0.8.5** (2026-09-05). Апстрим с тех пор ушёл на **644 коммита** — весь цикл **v0.8.6** (релиз 2026-10-18) + пост-релизные коммиты (до #11927).
> **Апстрим-клон:** `C:\Users\ipres\tmp\zeroclaw-audit\upstream` (origin/master). Список коммитов: `C:\Users\ipres\tmp\zeroclaw-audit\upstream_since_sync.txt`.
> **Методика:** cherry-pick по одному PR с переименованием `zeroclaw-*` → `omnesagent-*`, `zerocode` → наш бинарник. Вердикты выверены побайтовым сравнением файлов с v0.8.5-блобами: **~40 критичных файлов у нас вообще не разошлись** с точкой форка (policy.rs, security/*, rpc/transport|session|wss, gateway/api_pairing|api_webhook|webhook_ingress|sse|static_files, infra/session_store, approval_gate, tunnel/tailscale) — на них cherry-pick ложится чисто.

## 1. Сводка вердиктов (4 подсистемы, ~340 содержательных коммитов)

| Область | PORT-CLEAN | PORT-ADAPT | ALREADY | N/A |
|---|---|---|---|---|
| security / gateway / rpc / daemon / acp | 22 | 36 | 4 | 16 |
| runtime / turn / parser / multimodal | 28 | 60 | 1 | 35 |
| providers / tools / mcp / cost / deps | 26 | 24 | 3 | 12 |
| channels / config / memory / skills | 26 | 54 | 1 | 16 |
| **Итого** | **~102** | **~174** | **9** | **~79** |

`ALREADY` подтверждены evidence: copilot hardening #10370, served-model attribution #10415, warmup #10651 (портированы в фазе D); env-rebind #11222/#11133 (эквивалент); null-device (частично); paircode admin (частично — см. §3.1).

Ключевые факты о расхождениях: schema.rs у нас **структурно 1:1** с апстримом (43 873 vs 43 872 строк — god-file не сплитнут и у апстрима, заимствовать декомпозицию неоткуда); parser/tool_specs/prompt/multimodal ≈ девственные; stream_guard.rs, agent.rs (~24.6k diff), turn/mod.rs, delegate.rs, channels/orchestrator — сильно переписаны (наши Triage Router, compression, editor).

---

## 2. Этап 0 — подготовка (½ дня)

1. Ветка `feat/fork-sync-v0.8.6` от `main`.
2. Прогнать baseline: `cargo check --workspace`, покрейтовые тесты (api/channels/gateway/runtime), `flutter analyze` — зафиксировать известные pre-existing падения (runtime: 7 flaky denial/stream-тестов — проходят поодиночке; channels: 5 сетевых link_enricher).
3. Правило: **один PR апстрима = один коммит** (conventional, с номером PR в subject), после каждого — `cargo check -p <крейт>` + целевые тесты крейта. Не смешивать с собственными фичами.

## 3. Этап 1 — Security (P0) — Завершено 100%

| # | Коммит | PR | Баг | Наш статус | Коммит в ветке |
|---|---|---|---|---|---|
| 1.1 | `48169a1a3` | #11029 | **Approval-bypass на git**: глобальные опции (`-C`, `-c`, `--git-dir`, `--attr-source`) прятали write-сабкоманду от risk-классификатора. **У нас баг шире**: `command_risk_level` (policy.rs:2110) берёт `args.first()` без скипа глобальных опций → `git -C /repo commit` = Low, без approval | PORT-ADAPT | `86dcdfb5e` |
| 1.2 | `b9ec3e9a8` | #11451 | **Windows key file без DACL** — ключ шифрования секрет-стора читается любым локальным пользователем. Мы под Windows | PORT-CLEAN | `6ce6fa9e1` |
| 1.3 | `ad0dd3dec`+`be354a7ac`+`ece3b744e` | #11253,#11755,#11924 | **wasmtime 47.0.4 → 48.0.7** (RUSTSEC-2026-0313..0316 sandbox escape) | PORT-CLEAN | `889e97dda` |
| 1.4 | `01a6b1e6a` | #11678 | quinn-proto 0.11.15 → 0.11.19 (RUSTSEC-2026-0338..0342) | PORT-CLEAN | `2b9a14fb7` |
| 1.5 | `df6421301`/`0a6974d3e` | #12010,#11998 | tokio 1.53.3 (RUSTSEC-2026-0386) + ctutils 0.4.5 (RUSTSEC-2026-0385) | PORT-CLEAN | `05e92fafd` |
| 1.6 | `baa3a58ef` | #10015 | **SSRF**: datasheet-загрузки без ограничений. Фикс — эталонный паттерн: https-only, allowlist хостов, ревалидация каждого redirect-hop, 25MB cap, таймауты | PORT-CLEAN | `0189987c3` |
| 1.7 | `df941bca5` | #10070 | file_download против SSRF: private-host блокировка по умолчанию, metadata/credential-хосты всегда | PORT-CLEAN | `214d44ff9` |
| 1.8 | `2208f8cec` | #11026 | redact proxy credentials из snapshots | PORT-CLEAN | `8525f6613` |
| 1.9 | `631da1cf3` | #10664 | **Публичный /health отдаёт `last_error` компонентов** (внутренние URL). У нас жив: `/health` в `is_public_path` + `snapshot_json()` (health/mod.rs:13) | PORT-ADAPT | `7b542a2eb` |
| 1.10 | `79987a5c4` | #11455 | TOCTOU/symlink-swap в раздаче static files; cap-std по file-handle | PORT-CLEAN | `45d53bc75` |
| 1.11 | `51bd1d039` | #11656 | `firejail_args` декларирован, но молча игнорируется — security-deception fix | PORT-CLEAN | `602c2f6c3` |
| 1.12 | `949e415d0` | #10381 | **Bare-лаунчеры (`docker`, `firejail`, `bwrap`) резолвятся ПОСЛЕ установки workspace-cwd** → на Windows CreateProcess найдёт подменный exe в workspace агента | PORT-ADAPT | `e7b5e8384` |
| 1.13 | `402cee32c` | #11177 | Сузить `/admin/paircode/new` до admin-сессии (сейчас bearer pairing-токен тоже может ротировать коды) | PORT-ADAPT | `76db7aad7` |

**DoD этапа 1:** все RUSTSEC-позиции в Cargo.lock обновлены (`cargo audit` чист или решения зафиксированы), git-классификатор протестирован на bypass-матрице, `/health` не содержит `last_error`, DLL-паттерны закрыты.

## 4. Этап 2 — Прод-баги ядра (P1)

### 4.1 Промпт-кеш и токены (цепочка, наибольший экономический эффект)
- `6ab38e3a6` (#10959) — **sort tool specs**: 8 строк, стабилизирует префикс кэша. *Лучший ROI всего синка.*
- `7808fcd68` (#10862) — дата вне cached system prompt (иначе кэш умирает ежедневно).
- `bd5494b67` (#11801) — wall-clock дата из channel/agent::run промптов.
- `c10496d63`+`9b9a267ef`+`97ea54ab1` (#10623,#10724,#10895) — **cache passthrough** для всех OpenAI-compatible (breakpoints system+rolling, cache_ttl, фикс rolling breakpoint на image/tool_use-хвостах). Решает нашу известную проблему «кэш только в OpenRouter». Решить: апстримовская раскладка становится канонической, наша openrouter-фича — fallback.
- `ec102ec88` (#10716) — биллинг cache writes (обязательная пара к passthrough).
- `a55179236` (#10890) — per-image стоимость в estimate_history_tokens; `55b078b9f` (#10555) — cap image-promotion (path-listing'и). Бьют в известную проблему structured_history_cap.

### 4.2 Parser и tool-вызовы (реальные сломанные вызовы в проде)
- `3500014d9`+`83a06780a` (#11188,#11025) — **`browser_open`/`web_search` конвертируются в `curl`** у GLM-стиль моделей + алиасы не нормализуются в xml/minimax-форматах. Наш parser ≈ v0.8.5 — порт чистый.
- `44655c34c`+`2d85371f3` (#11479,#10914) — recover structured native tool arguments (стрингифицированные object/array аргументы). Цепочка из 2 коммитов.
- `e8627d2e8` (#11201) — сериализация file_edit+file_write в одном batch (гонка read-modify-write теряет правки).
- `2d85371f3` (base) — preserve string arguments при fallback parsing.

### 4.3 Стримы и надёжность провайдеров
- `f72e4d656` (#10442) — **OpenRouter стрим умирает по request timeout** (наш http_client() вешает timeout на весь запрос — баг жив).
- `9d3fd8db0`+`2d4bed5b7` (#10803,#11170) — retry-ядро reliable.rs: retry budget single-candidate recovery, overload-классификация, retry до send.
- `2eb7df78b` (#10886) — timeout_secs → streaming idle floor 300s.
- `9862c149a`+`e66c219e6` (#10397,#10566) — **MCP token blowup** (CallToolResult дублировал payload) + materialization картинок.

### 4.4 Steering и turn-цикл (адаптировать вокруг нашего Triage Router)
- `3b8ed46fc` (#11617) — **close steering channel при выходе** (поздние сообщения теряются — воспроизводимая потеря).
- `4f41bfa61`+`9ce12530b`+`2125a1301` (#11725,#11181,#11132) — доставка steering на следующем шаге цикла (yield-after-tool-round). Не тащить upstream-механику целиком — встроить идею в наш цикл.
- `64b703012` (#10696) — low-water trim target (основа для dc372cc7a #11067, 06ce69fa4 #9368).
- `c25324e96` (#10935) — stream-guard не глотает прозу с цитатой tool-result (перенести паттерн в наш переписанный guard).

### 4.5 Каналы и память
- `0695c9f8f` (#9753) — **absent vs empty `allowed_tools`** (deny-all семантика): точный баг в нашем policy.rs:3616.
- `87e3c2235` (#11650) — **Telegram 429**: ждать `retry_after`, не долбить в flood.
- `28a34bfc7` (#11669) — Lark flat post-контент молча дропался.
- `2881a7189` (#11258) — transcription `initial_prompt` игнорировался (смежно с нашим transcription.rs:886).
- `927ae8df0` (#11144) + `de0444dcb` (#11143) — memory: FP credential-regex; Qdrant time-bounds pushdown.
- `ecac0dbce`+`3def06fb7` (#10958,#10239) — interruption scope коллизии (`/stop` может cancel'ить чужой turn).
- `130781675`+`caf806ae3` (#10833,#10266) — `is_direct_message` для Signal и WhatsApp Web (DM молча дропались).
- `e6fe7a8c9` (#10692) — per-agent transcription binding.

### 4.6 Windows-пакет (мы целевая платформа)
- `31a7137fb` (#10954) — PowerShell UTF-8 output (у нас кракозябры).
- `c4c2d667a` (#11400) — doctor: замена models_cache.json при открытом reader (file-lock).
- `9c2bfaf50` (#11398) — замена JSONL session-файла, удерживаемого reader.
- `a7dad4ae5` (#10928) — exited Windows task owners.
- `a516d3315`+`3bf862b1f` (#11362,#11363) — SOP: замена файла при открытом reader; directory-триггеры и Windows-пути.
- `a2bfa0c76` (#10735) — heap-pin больших dispatch-веток (stack overflow на Windows 1 MiB).
- `ae9b84be0` (#11285) — tokio `time` feature в omnesagent-api (латентный сбой сборки).

**DoD этапа 2:** все перечисленные PR закоммичены с тестами апстрима (адаптированными), parser-матрица (browser_open/xml/minimax/structured args) зелёная, prompt-cache A/B: повторный вызов той же сессии показывает cache-hit токены.

## 5. Этап 3 — PORT-ADAPT средние (выравнивание с нашими особенностями)

- **Multimodal-кластер**: `d6899dfa1` (#10903 — tool-result картинки доезжают до модели), `8730e7b24` (#10088), `360bcaaf0` (#9819 pixel-валидация), `c1da9332c` (#11582 batch-evict), `4b87318df` (#11200 verbatim user text), `f42b095d9`, `fc4b4c0ea`, `9d268ffb2` — стыковать с нашим image_cache/relocate-фичей.
- **Cost-цепочка**: `36a6f119c` (#11350 — наш `GLOBAL_COST_TRACKER: OnceLock` — тот же баг), `f1527d7e1`, `91958a0b7`, `9abe969b6`, `7007365f4` (#11692 — surface warn_at_percent).
- **Config-батч (schema 1:1 → PORT-CLEAN внутри)**: `ba23a8a2e` (#10612 детерминизм итерации), `6ddfc6cbe` (#10686 openai-chat→openai), `1e1e092e4` (#11616 статические пути в macros), `c813acfee` (#11191 nevis cleanup), `afc47d333` (#11370), `09556e0ab` (#10589), `e16fd898b` (#11260 context budget), `0695c9f8f` если не сделан в этапе 2.
- **Memory-кластер**: `6108618d1`+`3ed28c40b` (sqlite_permissions модуль), `6c23e02b6` (#10567 дата recall — слить с нашими trust/last_feedback_at), `18bc1fccc` (#10252 category-scoped grants), `72324ab20` (#11458).
- **Skills/HTTP**: `7b84f9c80` (#11512 единый deadline — DNS вне reqwest timeout у нас в skill_http.rs:134), `96da19523` (#11804 GIT_TERMINAL_PROMPT=0), `88be24ac4` (#11599 native_tools).
- **RPC-мелочи**: `61b0ec88b` (#11330 session/configure гонка), `083d05ff2` (#11452), `25d543835` (#11151 agent-param в memory/*), `59f21a316` (#11291), `5a2a6250a` (#10775).
- **Провайдеры-мелочи**: `5ea5333c5`+`bab3fb4a7` (grok: резолв бинаря до spawn из workspace — наш TODO-зон; frame limits), `4244a30d2`+`4123877c9` (opencode session header), `88be24ac4`, `a647de344`, `3b61e23b5`, `5ee21b7d6`, `e941e0ca6` (#10519 llm_task alias-aware), `e8d2e5fc8` (#11350 model_switch note).
- **Gateway/daemon-мелочи**: `996c9fa24` (#11083 webhook SOP drive), `49fa46bb1` (#10538 — обрыв WS не убивает ход; session_queue.rs чист), `fa31aee26` (#10637 WS-память на чужом провайдере), `fee19d41` (#11518 EOF ≠ решение оператора), `21fbd9538` (#10093 subprocess изоляция), `24938afc6`+`6c0b1f9ee` (tailscale supervise/report — чисто), `77ae057e0` (#10455), `23950c9a1` (#11901 graceful shutdown — крупный, можно отдельным PR).

## 6. Этап 4 — Архитектурные решения (требуют решения владельца, НЕ чери-пикать бездумно)

1. **Серия #8289 principals/OIDC (~42k строк, 7 коммитов: 59143f9cc, bda1431dd, 579abc8f0, 73e2a4a15, ec17d1046, ebf97ab06 + зависимости 615f63aad, d49925065, eea20ebf1, 0e19debf2, 615f63aad)** — новая модель аутентификации RPC. Без неё непортируем cluster RPC-authz. Рекомендация: отложить до v0.9-решения; взамен **сейчас** закрыть форк-gap: `session/new` принимает клиентский `req.cwd` без confinement (dispatch.rs:1487) — добавить проверку против workspace allowlist.
2. **Config live-revisions цепочка** (`d71f54ce5` #10911 → `ed0183600` #11527 → `4aa83b282` #11466 → `4db7afdcf` #11803) — атомарные live-ревизии конфига. У нас persist = `Arc<RwLock<Config>>`. Портить только целиком.
3. **WeChat staged batches** (`7f7ece374` #9313, 3.5k строк) — закрывает проблему потери сообщений при рестарте и наш placeholder wechat.rs:2573.
4. **Telegram large feats**: multi_message streaming (`441df1313` — даст инфраструктуру и для нашего lark.rs:1219), per_user_session (`cc39235d0`), secure model picker (`097e309c3`), batch media (`cbd1b0adc`) — по мере надобности.
5. **Matrix voice MSC3245 outbound** (`cc512dd24`+`656822b34`), **WhatsApp markdown/polls/previews** (`49353b42c`, `d0d256ca3`, `727aa1a99`…) — логика переносима на наш wacore-стек, код нет.
6. **Sender authorization** (Bluesky/Reddit `96fafbc1e`, LINE `af3064ad4`, Mattermost `aa3a66cf1`) — если эти каналы нужны.
7. **SOP headless/decision-models** (`2f5e85041`, `2b802c739`, `e12c773ed`, `996355b83`) — задевает наш turn-цикл; после закрытия нашего TODO(PR C) в turn/mod.rs:468,676.
8. **Agent bundle export** (`68cc2d5b6`+2 фикса) — новая фича, наша потребность?
9. **Anthropic extended thinking через гейтвеи** (`042739b27`, huge) и **refusals fallback** (`53cc98d84`) — если появятся маршруты через LiteLLM-класс гейтвеи.

## 7. Этап 5 — Не портируем / чиним сами

| Наша проблема | Апстрим |
|---|---|
| whatsapp_storage.rs:1268 cleanup cron (TODO wacore-0.6) | файла нет (чужой стек) — **чинить самим** |
| transcription.rs:886 RAM-буфер аудио | streaming upload апстрим тоже не сделал |
| pairing.rs:82 sync mutex | не чинил |
| schema.rs god-file 43.9k строк | у апстрима тот же размер — **декомпозировать самим** |
| lark.rs:1219 multi_message | порт `441df1313` даст инфраструктуру, lark-карточка отдельной работой |
| ingress SOP-gate no-op (turn/mod.rs TODO(PR C)) | апстрим-механика другая; закрывать нашим PR C |
| eval replay-only | апстримная CI-replay (#9212) не переносится 1:1 |
| acp_channel.listen bails, alias_refs cascade, cloudflare runtime | апстримных фиксов нет |
| Старый `_`-join interruption keys | портим `ecac0dbce` (этап 2) |
| Trust-stamping Phase 1 (safety_net) | перекрывается серией #8289 при её принятии |

N/A-кластеры (переписаны у нас): delegate/agents-bundle/eval/intake, Hailo, seatbelt (macOS), dashboard/web-фиксы (наш Flutter свой), большая часть feat(*).

## 8. Порядок и оценка объёма

| Этап | Позиций | Оценка | Ответвления/риски |
|---|---|---|---|
| 0 подготовка | — | 0.5 дня | — |
| 1 security | 13 | 3–4 дня | 1.12 крупный (~2k строк); deps — только lock |
| 2 прод-баги | ~35 | 8–10 дней | 4.4 — адаптация вокруг Triage Router; agent.rs (~24.6k diff) требует ручной реконсиляции в 4.4/4.1 |
| 3 ADAPT средние | ~40 | 10–12 дней | multimodal/cost/memory стыковка с нашими расширениями |
| 4 архитектурные | 9 решений | по решению | серии #8289, live-revisions — недели |
| 5 | — | отдельно | собственные фиксы |

**Манифест этапа 1–2** (минимальный высокий ROI — ~15 дней суммарно): security-P0 + prompt-cache-цепочка + parser + OpenRouter/reliable + MCP blowup + steering close + Windows-пакет + канальные фиксы (allowed_tools, Telegram 429, Lark, transcription).

## 9. Правила выполнения

1. `git cherry-pick -x <hash>` из апстрим-клона → `sed` rename zeroclaw→omnesagent → правка конфликтов → `cargo check -p <крейт>` → тесты крейта → коммит `fix(scope): … (#<PR>)` без AI-атрибуции.
2. Порт-цепочки (promt-cache, reliable, opencode, #8289) — только целиком, порядок внутри цепочки обязателен.
3. Любое расхождение с нашими подсистемами (Triage Router, compression, stream_guard, editor) — PORT-ADAPT с описанием адаптации в теле коммита.
4. После каждого этапа: полный покрейтовый прогон + `flutter analyze` + обновление этого плана чекбоксами.

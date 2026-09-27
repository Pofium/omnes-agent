# PLAN_FILE_EDITOR_ZED.md — Встроенный редактор файлов в правой панели OmnesAgent (на архитектуре Zed)

**Дата:** 27.09.2026
**Изученный upstream:** `github.com/zed-industries/zed` @ `bda9c0bd43a8d235d82adb01ea5bc875b861ecfc` (2026-09-26, ветка `main`)
**Метод исследования:** репозиторий выкачан локально через `git clone --filter=blob:none --no-checkout` + `git sparse-checkout` (рецепт — приложение B); все приведённые ниже пути, номера строк, LOC и фрагменты кода — реальные, из этого дерева.
**Отправная точка в нашем коде:** `frontend/desktop/lib/features/inspector/inspector_panel.dart`, вкладка `file` (только просмотр).
**Дополнительный upstream (разобран позже):** `github.com/lapce/lapce` @ `b604d57de4a820006d335a3be0d7583eb8fab558` (2026-09-06, ветка `master`) и `github.com/lapce/floem` @ `1351ffb162faeb8be983f1301d718a0d5fb59c27` (подкрейт `editor-core`).

> **СТАТУС ДОКУМЕНТА (важно).** Редакция **v3**: лицензионные ограничения сняты решением владельца (§2), решения зафиксированы в `FRONTEND_SPEC.md` (§3.3-3.4, §5.5) и `BACKEND_SPEC.md` (§3.2-3.3, §9). §1-§10 — редакция v1 (план на архитектуре Zed). **§11 — редакция v2 (дополнение после разбора Lapce)**, и она **меняет ключевое решение**: базой платформы модели текста становится стек Lapce (Apache-2.0 / MIT), а Zed остаётся референсом по функциям, которых у Lapce нет. Читать §0 → §2 → **§11** → остальное. Пометки `ДОПОЛНЕНИЕ v2` в §0 и §2 указывают места, изменённые §11.

---

## 0. Итог в двух абзацах

> **ДОПОЛНЕНИЕ v2 (после разбора Lapce, см. §11):** рекомендация по базовому слою изменилась. Rope, delta-модель правок, курсоры, выделения, слова, отступы, modal-режим и подсветку **берём из Lapce-стека, а не пишем по спецификации Zed**: `lapce-xi-rope` (Apache-2.0) + `floem-editor-core` (MIT, 9102 строки готовой модели редактора) + `lapce-core/src/syntax` (Apache-2.0, компактная tree-sitter-подсветка). Это снимает лицензионный вопрос (§2) и сокращает объём Ф0-Ф2. Zed остаётся источником требований по функциям, которых в Lapce нет или которые там слабее: многослойный display-map (фолды/инлайны/блоки/складки), мультибуфер, широта LSP, git-хунки, темы, vim-редактирование «как в Zed».

Сейчас в правой панели инспектора есть вкладка `file`, которая умеет только показывать текст: `SelectableText` целиком + номера строк, собранные обычным `Column` (`inspector_panel.dart:377-470`). Редактирования нет ни в UI, ни в протоколе шлюза: файлы проекта клиент читает сам через `universal_io.File(...).readAsStringSync()` (`task_workspace_controller.dart:637-653`), пишет файлы только агент — своими инструментами (`backend/crates/omnesagent-tools/src/file_edit.rs`, `file_write.rs`). Задача — превратить эту вкладку в настоящий редактор кода.

Предлагаемая архитектура повторяет разделение Zed, но с разрезом по границе процесса: **модель текста (rope, op-log, display-map, selection, LSP, git-hunks) живёт в Rust внутри шлюза** (новый крейт `omnesagent-editor`), а **Flutter остаётся виртуальным вьюпортом**, который получает по WebSocket не «текст файла», а *render plan* видимых строк (номер строки, текст строки, run-length подсветка, вставки инлайнов, фолды, диагностики) и отправляет назад правки. Это ровно та схема, которую Zed использует для коллаборации (`Buffer` + `OperationQueue` + Lamport-часы): она естественно ложится на случай «человек и агент редактируют один файл одновременно», который для OmnesAgent — не экзотика, а основной сценарий. Подсветку синтаксиса делает сервер (tree-sitter уже есть в нашем воркспейсе через `inkjet`), Flutter только красит — грамматики и парсеры в Dart не тащим.

---

## 1. Что есть сейчас (факты, с точками в коде)

| Что | Где | Состояние |
|---|---|---|
| Вкладка `file` в правой панели | `frontend/desktop/lib/features/inspector/inspector_panel.dart:276-277, 377-470` | Read-only: `SelectableText(content)` + номера строк в `Column` (нет виртуализации, нет текстового слоя) |
| Признак «вьюер, а не редактор» | `inspector_panel.dart:448` | Высота разделителя считается как `lines.length * 15.4` — жёсткая привязка к моноширинному размеру, никакой метрики строк |
| Открытие файла | `desktop_shell.dart:449-455` | `onOpenFile` → `openProjectFile(path)` → `_openInspectorWithTab(5)` (для `.md` — вкладка 2 «Холст») |
| Состояние файла | `task_workspace_controller.dart:158-160` | `selectedFilePath` / `selectedFileContent` — две `RxnString`, без версии, грязного флага и истории |
| Чтение содержимого | `task_workspace_controller.dart:637-653` | **Прямое чтение с диска** `universal_io.File(path).readAsStringSync()` — обход правила «шлюз — единственный владелец состояния» из `FRONTEND_SPEC.md §3` |
| Дерево проекта | `frontend/desktop/lib/widgets/desktop_sidebar.dart:1824-1858, 1986, 2059, 2502` | Обход локальной ФС (`universal_io.FileSystemEntity.isDirectorySync`), без watcher'а и без git-статусов |
| Редактирование агентом | `backend/crates/omnesagent-tools/src/file_edit.rs` (982 строки), `file_write.rs`, `backup_tool.rs` | Есть, но это «записать/пропатчить файл», а не правка разделяемого буфера: перезапись на диске без версии, без уведомления UI |
| WS-каналы шлюза | `backend/crates/omnesagent-gateway/src/lib.rs:2006-2014` | `/ws/chat`, `/ws/sops/runs`, `/ws/canvas/{id}`, `/ws/nodes`, `/ws/terminal/{id}`. Канала редактора нет |
| REST-эндпоинты по файлам **проекта** | `backend/crates/omnesagent-gateway/src/lib.rs` (роуты с 1645), `api.rs` | Отсутствуют. В клиенте есть только работа с workspace **агента**: `workspaceReadFile/workspaceWriteFile/workspaceRawFileBytes/workspaceListFiles` (`frontend/shared/lib/core/gateway/gateway_http.dart:374-412`) |
| tree-sitter в бэкенде | `backend/Cargo.lock:11042` (`tree-sitter 0.23.2`), `:11054` (`tree-sitter-highlight 0.23.2`) | Приходит транзитивно через `inkjet` (`backend/apps/omnescode/Cargo.toml`), причём с включённой грамматикой **dart** — бесплатный фундамент для подсветки |
| Пакеты редактора во Flutter | `frontend/desktop/pubspec.yaml` | Ни одного (есть `webview_windows`, `window_manager`, `path`, `desktop_drop`) |
| Второй «редактор» в репо | `backend/apps/omnescode/src/{editor.rs,file_explorer.rs,diff.rs}` | Это TUI-приложение (ratatui/crossterm, `editor.rs` = 32 строки, только выбор `$EDITOR`). Как переиспользуемая модель не годится |

---

## 2. Источники кода (лицензии не учитываем)

**Решение владельца (27.09.2026):** лицензионные ограничения при переносе кода в OmnesAgent **не учитываются**. Берём код и из Zed, и из Lapce напрямую — переносом и адаптацией под нашу архитектуру, без clean-room и без обязательств по NOTICE. Пометки вида `REFERENCE (GPL)` у фрагментов ниже следует читать как **«фрагмент-ориентир, готовый к переносу»**.

Для справки (на решения в этом плане не влияет):

| Источник | Лицензия |
|---|---|
| Zed: `rope`, `text`, `editor`, `language`, `language_core`, `lsp`, `project`, `vim`, `multi_buffer`, `buffer_diff`, `fuzzy` | GPL-3.0-or-later |
| Zed: `sum_tree`, `collections`, `util`, `gpui` | Apache-2.0 |
| Lapce: `lapce-core`, `lapce-rpc`, `lapce-proxy`, `lapce-app` | Apache-2.0 |
| Lapce: `floem` (`editor-core`) | MIT |
| `lapce-xi-rope` (crates.io) | Apache-2.0 |
| OmnesAgent | MIT |

Единственные ограничения, которые остаются в силе, — технические:
1. Чужой код адаптируется под наши контракты (§4 решение о границе «шлюз ↔ клиент», §5 протокол), а не наоборот.
2. Зависимости не тащим вслепую: см. §11.13.G4 (единая версия tree-sitter в воркспейсе, проверка `ui-events`).
3. Не ломаем сборку: вендоренные подкрейты живут в `omnesagent-editor/vendor/` с фиксацией ревизии upstream (для воспроизводимости, а не из-за лицензии).

---

## 3. Что берём из Zed — по подсистемам

Легенда колонки «Как берём»: **ВЕНДОР** — дословно (Apache-2.0), **ПОРТ** — переписываем по спецификации/поведению, **ИДЕЯ** — берём архитектурное решение, кода не переносим.

> Фрагменты кода ниже — рабочий материал: переносим и адаптируем напрямую (см. §2, редакция v3).

### 3.1. `crates/sum_tree` — 3327 строк, Apache-2.0 → **ВЕНДОР**

Суммирующее B-дерево, на котором в Zed построено **всё**: rope, буфер, display-map, multi_buffer. Каждый узел несёт агрегированный summary поддерева, поэтому «сколько строк до этой позиции» — это навигация O(log n) по дереву, а не проход по тексту.

Что берём (имена — реальные из `crates/sum_tree/src/`):

```rust
// REFERENCE (Apache-2.0, можно вендорить): sum_tree/src/sum_tree.rs
pub trait Item: Clone { type Summary: Summary; fn summary(&self) -> Self::Summary; }
pub trait KeyedItem: Item { type Key: Ord; fn key(&self) -> Self::Key; }
pub trait Summary: Clone { fn zero() -> Self; fn add_summary(&mut self, other: &Self); }
pub trait ContextLessSummary: Clone + Default { fn zero() -> Self; fn add_summary(&mut self, other: &Self); }
pub trait Dimension<'a, S: Summary>: Clone { fn zero(cx: &'a S) -> Self; fn add_summary(&mut self, summary: &'a S, cx: &'a S); }
pub trait SeekTarget<'a, S: Summary, D: Dimension<'a, S>>: Ord { fn cmp(&self, cursor: &Cursor<'a, '_, impl Item<Summary = S>, D>, cx: &'a S) -> Ordering; }
pub enum Bias { Left, Right }
pub struct SumTree<T: Item>(Arc<Node<T>>);
pub enum Edit<T: KeyedItem> { Insert(T), Remove(T), Update(T, T) }
```

Нам это даёт (список — прямая карта в наш крейт):

* `SumTree<Chunk>` — контейнер rope (см. 3.2);
* `SumTree<Fragment>`/`SumTree<InsertionFragment>` — op-log буфера (см. 3.3);
* `TreeMap`/`TreeSet` (`sum_tree/src/tree_map.rs`) — операции буфера по Lamport-ключу;
* `Bias` — тот самый Bias, который нужен `Anchor`'у (курсор «прилипает» влево/вправо от позиции вставки);
* `Cursor::{seek, next, prev, summary}` — все переводы `Offset ⇄ Point ⇄ DisplayRow`;
* `crates/sum_tree/src/property_test.rs` — образец property-теста, копируем подход к тестированию (см. §8).

Объём: 4 файла (`sum_tree.rs` 1903, `cursor.rs` 861, `tree_map.rs` 531, `property_test.rs` 32). Ставим в `backend/crates/omnesagent-editor/vendor/sum_tree/`, пишем `NOTICE` + `THIRD_PARTY.md`, вендор подключаем как отдельный workspace-member (`publish = false`).

### 3.2. `crates/rope` — 4132 строки, GPL → **ПОРТ** (алгоритм)

Что устроено и что повторяем:

```rust
// REFERENCE (GPL): crates/rope/src/rope.rs:25-28, 147-185
pub struct Rope { chunks: SumTree<Chunk> }

pub fn push(&mut self, mut text: &str) {
    self.chunks.update_last(|last_chunk| {
        let split_ix = if last_chunk.text.len() + text.len() <= chunk::MAX_BASE { text.len() }
        else { /* дотягиваем чанк до MIN_BASE по границе символа, остаток — в новые чанки */ };
        let (suffix, remainder) = text.split_at(split_ix);
        last_chunk.push_str(suffix); text = remainder;
    }, ());
    ...
    if text.len() > NUM_CHUNKS * chunk::MAX_BASE - NUM_CHUNKS * 4 { return self.push_large(text); }
}
```

Берём из этого модуля конкретно:

| Артефакт Zed | Файл (LOC) | Зачем нам | Как переносим |
|---|---|---|---|
| `Rope` (чанкованный, `SumTree<Chunk>`) | `rope.rs` (2518) | Основной текст файла; правка/срез/итерация без копирования всего файла | Свой `Rope` в `omnesagent-editor/src/rope/mod.rs`, чанки 1-8 КБ, `MAX_BASE/MIN_BASE`, как в Zed |
| `Chunk`, `ChunkSlice`, `Bitmap` (bitmap «есть ли \n / табы в чанке») | `chunk.rs` (1249) | Ответ «в этом чанке есть перевод строки/таб?» за O(1), отсечение при поиске строк | Порт; битмапы — 64-битные слова на чанк |
| `Point` (row, column) и `PointUtf16` | `point.rs` (146), `point_utf16.rs` (119) | Две системы координат: байты и UTF-16 (нужно для LSP — LSP живёт в UTF-16) | Порт обеих (это же понадобится для `lsp-types`) |
| `OffsetUtf16`, `Unclipped<T>` | `offset_utf16.rs` (49), `unclipped.rs` (51) | Конвертация смещений в UTF-16 и «непроклипленные» позиции для inlay-хитов | Порт |
| `Cursor`, `Chunks`, `Chunks::next_line/prev_line`, `Lines`, `Bytes` | `rope.rs` | Итераторы линий/байтов для рендера и для `similar`-диффа | Порт API + свои итераторы |
| `Bias`-клиппинг: `clip_offset/clip_point/floor_char_boundary/ceil_char_boundary` | `rope.rs` | Курсор никогда не «садится» в середину UTF-8 символа — критично на кириллице | Порт обязательно + unit-тесты на кириллице/эмодзи |
| `Rope::line_len(row)`, `max_point()`, `slice_rows`, `starts_with/ends_with` | `rope.rs` | Ширина строки (горизонтальный скролл), разбиение файла на куски | Порт |

Что из `rope` **не берём**: `rayon`-параллельный `push_large` (у нас файлы открываются по одному; параллелизм добавим позже, если понадобится), `ztracing`-инструментация (у нас `tracing`).

### 3.3. `crates/text` — 6592 строки, GPL → **ПОРТ** (это ключевой крейт для нас)

Именно здесь лежит ответ на вопрос «как человек и агент редактируют один файл, не затирая друг друга».

```rust
// REFERENCE (GPL): crates/text/src/text.rs:59-68 — буфер = снимок + история + отложенные чужие операции
pub struct Buffer {
    snapshot: BufferSnapshot,
    history: History,
    deferred_ops: OperationQueue<Operation>,
    deferred_replicas: HashSet<ReplicaId>,
    pub lamport_clock: clock::Lamport,
    subscriptions: Topic<usize>,
    edit_id_resolvers: HashMap<clock::Lamport, Vec<oneshot::Sender<()>>>,
    wait_for_version_txs: Vec<(clock::Global, oneshot::Sender<()>)>,
}

// crates/text/src/text.rs:113-124 — снимок
pub struct BufferSnapshot {
    visible_text: Rope, deleted_text: Rope,
    fragments: SumTree<Fragment>, insertions: SumTree<InsertionFragment>,
    insertion_slices: TreeSet<InsertionSlice>, undo_map: UndoMap,
    pub version: clock::Global, remote_id: BufferId, replica_id: ReplicaId, line_ending: LineEnding,
}

// crates/text/src/text.rs:619-637 — единица изменения
pub enum Operation { Edit(EditOperation), Undo(UndoOperation) }
pub struct EditOperation { pub timestamp: clock::Lamport, pub version: clock::Global,
                           pub ranges: Vec<Range<FullOffset>>, pub new_text: Vec<Arc<str>> }
pub struct UndoOperation { pub timestamp: clock::Lamport, pub version: clock::Global,
                           pub counts: HashMap<clock::Lamport, u32> }

// crates/text/src/text.rs:870-890 — правка = транзакция + тик Lamport + публикация патча
pub fn edit<R, I, S, T>(&mut self, edits: R) -> Operation {
    self.start_transaction();
    let timestamp = self.lamport_clock.tick();
    let operation = Operation::Edit(self.apply_local_edit(edits, timestamp));
    self.history.push(operation.clone()); self.history.push_undo(operation.timestamp());
    self.snapshot.version.observe(operation.timestamp()); self.end_transaction();
    operation
}
```

```rust
// REFERENCE (GPL): crates/text/src/anchor.rs:10-24 — позиция, устойчивая к чужим правкам выше по тексту
pub struct Anchor {
    pub(crate) timestamp_replica_id: clock::ReplicaId,
    pub(crate) timestamp_value: clock::Seq,
    pub offset: u32,          // смещение внутри текста, вставленного операцией timestamp
    pub bias: Bias,           // прилипание влево/вправо
    pub buffer_id: BufferId,
}
```

```rust
// REFERENCE (GPL): crates/text/src/operation_queue.rs:38-59 — очередь чужих операций, отсортированная по Lamport, с дедупом
pub fn insert(&mut self, mut ops: Vec<T>) {
    ops.sort_unstable_by_key(|op| op.lamport_timestamp());
    ops.dedup_by_key(|op| op.lamport_timestamp());
    self.0.edit(ops.into_iter().map(|op| Edit::Insert(OperationItem(op))).collect(), ());
}
```

Карта переноса по `crates/text/src/`:

| Артефакт | Файл (LOC) | Что берём | Куда в OmnesAgent |
|---|---|---|---|
| `Buffer` / `BufferSnapshot` / `BufferId` / `ReplicaId` | `text.rs` (3844) | Модель: снимок + отложенные операции + версия (version vector) + undo-история | `omnesagent-editor/src/buffer/{mod,state}.rs` |
| `Anchor` + `Bias` | `anchor.rs` (249) | Устойчивые позиции: курсоры, выделения, ханки диффа, комментарии ревью — всё держим на anchor'ах, а не на смещениях | `.../anchor.rs` |
| `Operation`/`EditOperation`/`UndoOperation`, `Transaction`, `HistoryEntry` | `text.rs` | Журнал правок, транзакции (группировка кейстроков в один undo-шаг), `Transaction::merge_in` | `.../op_log.rs` |
| `UndoMap` | `undo_map.rs` (115) | Логическое undo/redo (отменяет «свои» правки, не трогая правки другой реплики — то есть **агент не откатывается вместе с Ctrl+Z человека**) | `.../undo.rs` |
| `OperationQueue<T>` | `operation_queue.rs` (165) | Очередь отложенного применения чужих операций, когда локальная история «занята» (`has_deferred_ops`, `peek_undo_stack`) | `.../op_queue.rs` |
| `Edit<D>` / `Patch<D>` | `text.rs:527`, `patch.rs` (655), `text.rs` | Формат «что изменилось» для всех слоёв display-map и для отдачи дельт в UI | `.../patch.rs` |
| `Locator` | `locator.rs` (177) | Быстрый перевод anchor→offset с кэшем (нужен для тысяч выделений) | `.../locator.rs` |
| `Subscription` / `Topic` | `subscription.rs` (67) | Подписка на правки (в нашем случае — триггер отправки фреймов в UI) | `.../subscription.rs` |
| `wait_for_edits` / `wait_for_anchors` / `wait_for_version` / `fast_forward` | `text.rs:1552-1670` | Синхронизация: «дождаться, пока правка станет видимой», «дождаться версии N» — ровно то, что нужно шлюзу при ответе агенту | `.../sync.rs` |
| `edit_via_marked_text` / `edits_for_marked_text` | `text.rs:1690-1741` | **Прямо наш сценарий агента**: модель отдаёт текст с маркерами изменений, из него выводятся точные правки буфера (вместо «перезаписать файл целиком») | `.../agent_edits.rs` |
| `check_invariants`, `get_random_edits`, `randomly_edit` | `text.rs:1742-1830` | Основа тестов: рандомизированные последовательности правок + проверка инвариантов | `.../tests/randomized.rs` |

### 3.4. `crates/clock` — 338 строк, GPL → **ПОРТ**

```rust
// REFERENCE (GPL): crates/clock/src/clock.rs:58-105
pub type Seq = u32;
pub struct Lamport { pub value: Seq, pub replica_id: ReplicaId }
pub struct Global { values: SmallVec<[u32; 4]> }   // version vector
impl Global {
    pub fn get(&self, replica_id: ReplicaId) -> Seq;
    pub fn observe(&mut self, timestamp: Lamport);
    pub fn join(&mut self, other: &Self); pub fn meet(&mut self, other: &Self);
    pub fn observed(&self, timestamp: Lamport) -> bool;
    pub fn changed_since(&self, other: &Self) -> bool;
}
```

Берём как есть (переписав): `ReplicaId(u16)` + `Global`-version-vector — это основа порядка операций между репликами. **Наши реплики:** `0` — ядро шлюза (sysop-операции, автосохранение, `format on save`), `1` — UI-окно человека, `2..N` — агентные сессии (каждый вызов `write_file`/`patch` агента = реплика своей транзакции), опционально `1024+` — удалённый клиент (web, телефон). Смысл: `UndoMap` отменяет только операции своей реплики, `deferred_ops` принимает чужие независимо от того, где живёт редактор.

### 3.5. Подсветка и язык: `crates/language` (27927 строк) + `crates/language_core` (2570), GPL → **ПОРТ**

Что берём:

```rust
// REFERENCE (GPL): crates/language_core/src/highlight_map.rs — файл целиком 54 строки
pub struct HighlightMap(Arc<[Option<HighlightId>]>);
pub struct CaptureId(pub u32);
pub struct HighlightId(NonZeroU32);
impl HighlightId {
    pub const TABSTOP_INSERT_ID: HighlightId;  pub const TABSTOP_REPLACE_ID: HighlightId;
    pub fn new(capture_id: u32) -> Self { Self(NonZeroU32::new(capture_id + 1).unwrap_or(NonZeroU32::MAX)) }
}
impl HighlightMap {
    pub fn from_ids(highlight_ids: impl IntoIterator<Item = Option<HighlightId>>) -> Self;
    pub fn get(&self, capture_id: CaptureId) -> Option<HighlightId>;
}
```

* `HighlightMap` — отображение «id capture из tree-sitter query → id стиля темы». Реализуем ровно этот контракт: `build_highlight_map(capture_names, syntax_theme) -> HighlightMap` (`language.rs:1261`), плюс `Language::highlight_text` (`language.rs:1120`) как точка входа «текст → подсвеченные чанки».
* `highlight_cache.rs` (339 строк, `language_core`) — `TextHighlightKey`, `ResolvedHighlights`, `TextHighlightCache`/`ChunkHighlightCache` на базе `CostBudgetedLru` с бюджетом байт. Берём идею и структуру: кэш подсветки ключуется по (текст чанка, язык, версия темы), поэтому при вставке символа пересчитывается только затронутый чанк. Это то, что даст нам «подсветка дешевле скролла».
* `SyntaxMap` + слои инъекций (`language/src/syntax_map.rs`, `syntax_map/*`): `SyntaxLayer`, `SyntaxMapCaptures`, `SyntaxMapMatch`, `SyntaxMap::interpolate/reparse/did_parse`, `root_language`, `update_count` — механизм «язык внутри языка» (HTML с JS/CSS, Markdown с кодом, Rust с макросами). Берём структуру слоёв и API; встраивание (injections) включаем в Ф1 только для markdown+html, остальное — по мере надобности.

```rust
// REFERENCE (GPL): crates/language/src/syntax_map.rs:240-247
pub struct SyntaxLayer<'a> {
    pub language: &'a Arc<Language>,
    pub included_sub_ranges: Option<&'a [Range<Anchor>]>,
    pub(crate) depth: usize,
    tree: &'a tree_sitter::Tree,
    pub(crate) offset: (usize, tree_sitter::Point),
}
```

**Грамматики.** Zed объявляет их в корневом `Cargo.toml` (строки 873-896) — берём этот список как эталон паритета; сами грамматики есть на crates.io под MIT:

`tree-sitter` (ядро, git-рев `43623ec9…`), `bash 0.25.1`, `c 0.24.1`, `cpp`, `css 0.23`, `diff 0.1.0`, `elixir 0.3`, `embedded-template 0.23`, `gitcommit`, `go 0.25`, `go-mod`, `gowork`, `heex`, `html 0.23`, `jsdoc 0.23`, `json 0.24`, `md`, `python 0.25`, `regex 0.24`, `ruby 0.23`, `rust 0.24.2`, `typescript`, `yaml`.
Плюс то, что нам нужно и чего у Zed на этом пути нет в списке workspace-deps: **`tree-sitter-dart`** (наш клиент на Dart) — у нас уже доступна грамматика dart через `inkjet` features в `apps/omnescode/Cargo.toml`, значит цепочку сборки на Windows мы уже умеем.

### 3.6. `crates/editor/src/display_map*` — 20 627 строк (4590 + 16 048 подмодулей), GPL → **ПОРТ** (это ядро представления текста)

Шапка модуля — лучшее описание того, что мы переносим (цитата):

```rust
// REFERENCE (GPL): crates/editor/src/display_map.rs:11-20
//! [`DisplayMap`] is conceptually made up of several smaller structures that form a hierarchy (starting at the bottom):
//! - [`InlayMap`] that decides where the [`Inlay`]s should be displayed.
//! - [`FoldMap`] that decides where the fold indicators should be; it also tracks parts of a source file that are currently folded.
//! - [`TabMap`] that keeps track of hard tabs in a buffer.
//! - [`WrapMap`] that handles soft wrapping.
//! - [`BlockMap`] that tracks custom blocks such as diagnostics that should be displayed within buffer.
//! - [`DisplayMap`] that adds background highlights to the regions of text.
```

Слои и их размеры (все — в `crates/editor/src/display_map/`):

| Слой | Файл (LOC) | Ключевые типы (реальные) | Что берём | Что у нас делает |
|---|---|---|---|---|
| `tab_map` | 1849 | `TabMap`, `TabSnapshot`, `TabPoint`, `TabChunks`, `TabStop` | Развёртка табов в пробелы на уровне представления | Табы не портят `Point`‑координаты буфера; `tab_size` из настроек |
| `wrap_map` | 1962 | `WrapMap`, `WrapSnapshot`, `WrapRow`, `WrapPoint`, `WrapChunks`, `WrapRows`, `LineFragmentBuilder` | Перенос строк (soft wrap) с кэшем ширин символов | `soft_wrap: none|editor_width|bounded`, `preferred_line_length` |
| `fold_map` | 2566 | `FoldMap`, `FoldSnapshot`, `FoldPoint`, `Fold`, `FoldRange`, `FoldId`, `FoldPlaceholder` | Регионы фолдов + плейсхолдер «… N строк» | Фолды по отступам/скобкам и LSP `foldingRange` |
| `inlay_map` | 2664 | `InlayMap`, `InlaySnapshot`, `InlayPoint`, `InlayOffset`, `InlayChunk` | Инлайн-текст: подсказки типов (inlay hints), призрак нейросетевой вставки, `deleted_text` | Подсказки LSP и **предложения агента** (text ghost) без порчи буфера |
| `block_map` | 5819 | `BlockMap`, `BlockSnapshot`, `BlockPoint`, `BlockRow`, `BlockPlacement`, `CustomBlockId`, `SpacerId` | Блоки-виджеты между строк (панель диагностики, inline-виджет ревью, `---` раскрытие диффа) | Комментарии ревью и inline-баннеры агента |
| `crease_map` | 520 | `CreaseSnapshot`, `Crease` | «Складки» — диапазоны, которые могут быть свёрнуты (индентация или LSP) | Источник фолдов до применения |
| `invisibles` | 136 | `is_invisible`, `replacement`, `is_standalone_grapheme` | Отрисовка пробелов/переводов строк | `show_whitespaces: none|selection|all` |
| `custom_highlights` | 432 | `Highlights`, `HighlightStyleInterner`, `HighlightStyleId` | Подсветка диапазонов (поиск, выделение совпадений, дифф) | Хайлайт результатов `Ctrl+F` и правок агента |

Контракт слоёв (берём как проектную аксиому — это и есть «магия» корректности Zed):

```rust
// REFERENCE (GPL): crates/editor/src/display_map/wrap_map.rs:68-84
struct Transform { summary: TransformSummary, display_text: Option<&'static str> }
struct TransformSummary { input: TextSummary, output: TextSummary }
impl TransformSummary { fn has_wraps(&self) -> bool { self.input.lines != self.output.lines } }
```

Каждый слой: (1) `Transform` — область, которую слой «переписывает»; (2) `TransformSummary{input, output}` — сколько текста на входе и сколько на выходе; (3) `Snapshot` — состояние слоя; (4) `sync(snapshot, edits: Vec<Edit<D>>) -> (Snapshot, Vec<Edit<S>>)` — пересчёт **только затронутых областей**; (5) конвертеры `<A>_point_to_<B>_point`; (6) итераторы `RowInfo`/`Chunk`. Без этой дисциплины редактор на 100k строк начинает тормозить на первой же правке — поэтому это требование, а не пожелание.

Координатные пространства (берём как есть, это главный источник багов при самодельной реализации):

```rust
// REFERENCE (GPL): crates/editor/src/display_map.rs:2523, 2551-2558
pub struct DisplayPoint(BlockPoint);
pub struct DisplayRow(pub u32);
impl DisplayRow { pub(crate) fn as_display_point(&self) -> DisplayPoint { DisplayPoint::new(*self, 0) } }
```

`Point` (буфер/файл) → `PointUtf16` (для LSP) → `TabPoint` → `WrapPoint` → `FoldPoint` → `InlayPoint` → `BlockPoint`/`DisplayPoint` (то, что реально рисуется) → `DisplayRow` (то, что скроллится).

И главный «двигатель рендера» — та функция, которую мы портируем в наш `render_plan`:

```rust
// REFERENCE (GPL): crates/editor/src/display_map.rs:1847-1871
pub fn highlighted_chunks<'a>(&'a self, display_rows: Range<DisplayRow>,
        language_aware: LanguageAwareStyling, editor_style: &'a EditorStyle)
        -> impl Iterator<Item = HighlightedChunk<'a>> {
    self.chunks(display_rows, language_aware, HighlightStyles { inlay_hint: …, edit_prediction: … })
        .flat_map(|chunk| { let syntax_highlight_style = chunk.syntax_highlight_id.and_then(|id| editor_style.syntax.get(id).cloned()); … })
}
```

Именно эта функция — прообраз нашего WS-фрейма `rows_snapshot`/`rows_changed`: на входе диапазон **display-строк**, на выходе чанки с синтаксическим стилем, хайлайтами, диагностиками и инлайнами. Флаги `language_aware`/`editor_style` у нас превращаются в параметры запроса рендера (какая тема активна, нужны ли инлайны/семантические токены).

### 3.7. `crates/editor` — 182 330 строк, GPL → **ПОРТ** избранного (не всего)

Полный размер ядра говорит о масштабе: переписывать Zed целиком никто не собирается. Ниже — что именно берём, с реальными именами и LOC.

```rust
// REFERENCE (GPL): crates/editor/src/editor.rs:965-1040 — из чего состоит редактор
pub struct Editor {
    buffer: Entity<MultiBuffer>,
    pub display_map: Entity<DisplayMap>,       // где рисовать текст (см. 3.6)
    pub selections: SelectionsCollection,      // множество курсоров/выделений
    pub scroll_manager: ScrollManager,         // автоскролл, «не уезжать» при правке
    ime_transaction: Option<TransactionId>,    // IME (ввод CJK/диакритик) как одна транзакция
    autoclose_regions: Vec<AutocloseRegion>,   // автоскобки/кавычки
    snippet_stack: InvalidationStack<SnippetState>,
    mode: EditorMode,                          // SingleLine | AutoHeight | Full
    breadcrumbs_visibility: BreadcrumbsVisibility,
    show_gutter: bool, show_scrollbars: ScrollbarAxes, minimap_visibility: MinimapVisibility,
    show_git_diff_gutter: Option<bool>, show_line_numbers: Option<bool>, …
}
```

| Модуль Zed (`crates/editor/src/`) | LOC | Что берём | Наш аналог |
|---|---|---|---|
| `editor.rs` | 13 130 | Состав редактора, жизненный цикл, экшены (`actions.rs` 1064), транзакции ввода | `omnesagent-editor/src/editor.rs` (серверная часть) + `EditorController` во Flutter |
| `selections_collection.rs` | 2055 | `SelectionsCollection`, `MutableSelectionsCollection`, `PendingSelection`, `insert_range/select/select_ranges/move_with/move_heads_with/new_selection_id` — модель мультикурсоров | `.../selections.rs` |
| `selection.rs` | 2792 | `Selection<D>`, расширение/сужение, «умное» выделение слов/строк | `.../selection.rs` |
| `movement.rs` | 1648 | `left/right/up/down/line_end/next_word_end/…` как функции над `DisplaySnapshot` (правильная цельность по графемам) | `.../movement.rs` |
| `scroll.rs`, `scroll/` | — | `ScrollManager`, автоскролл за курсором, скроллбар-маркеры, «scroll beyond last line» | `.../scroll.rs` (математика скролла) |
| `fold.rs` + `folding_ranges.rs` | 1099 + 1192 | Порт фолдов: `fold_selected_ranges` (по индентации/скобкам), `fold_ranges` (LSP) | `.../folding.rs` |
| `indent_guides.rs`, `bracket_colorization.rs`, `highlight_matching_bracket.rs` | — | Направляющие отступов, раскраска парных скобок, подсветка пары | `.../guides.rs` → в render plan |
| `input.rs` | 3136 | `handle_input`, `newline`, `insert`, `delete_to_*`, `insert_snippet_at_selections`, `set_use_autoclose`, `set_autoindent`, `replay_insert_event` — **вся семантика ввода**: автоиндент, автоскобки, сниппеты, IME-композиция | `.../input.rs` |
| `git.rs` + `git/` | 3271 | `DiffHunkRenderer` (`DefaultDiffHunkRenderer`/`HiddenDiffHunkRenderer`), `diff_hunks_in_ranges`, `go_to_hunk_before_or_after_position`, gutter-диффы, blame, `Restore`-экшен «откатить ханк» | `.../git_gutter.rs` |
| `semantic_tokens.rs` | 2942 | Семантические токены LSP как ещё один слой подсветки | `.../semantic_tokens.rs` |
| `inlays.rs` + `inlays/` | — | Что показывать в инлайнах (типы, параметры, призрак-правки) | `.../inlays.rs` |
| `diagnostics.rs`, `document_colors.rs`, `document_links.rs` | — | Диагностики как данные + inline-диагностики; цветовые маркеры; кликабельные ссылки | `.../diagnostics.rs` |
| `completions.rs` + `code_context_menus.rs` | 1593 + 2222 | Меню автодополнения, фильтрация/сортировка, контекстные меню | `.../completion_menu.rs` |
| `code_actions.rs`, `signature_help.rs`, `hover_popover.rs`, `hover_links.rs` | 3351+3662 | Хавер-попапы, code actions, подписи сигнатур | `.../popovers.rs` |
| `document_symbols.rs`, `outline` (крейт) | 1499 | Список символов файла + outline-панель | `.../symbols.rs` |
| `navigation.rs` | 2563 | «Перейти к определению», история переходов, а не просто отдача позиции | `.../navigation.rs` |
| `linked_editing_ranges.rs` | — | **Связанные диапазоны**: правка в одном месте отражается в других (аналог «мультиредактирование»), крайне уместно для «агент меняет все вызовы» | `.../linked_edits.rs` |
| `edit_prediction.rs` | 2615 | Призрак-предсказание с accept/reject и учётом сессий | Основа для «предложений агента» (Ф7) |
| `element.rs` (14 007) + `element/mouse.rs` (1294) | | Как строки становятся пикселями: `layout_lines`, `layout_line_numbers`, `EditorLayout`, `CursorLayout`, `PointForPosition` (хит-тест), drag-выделение, `LineHighlightSpec` | **ИДЕЯ** + спец: Flutter-виджет `EditorViewport` с `CustomPainter` |
| `persistence.rs` | — | Восстановление открытых файлов и позиций между запусками | `.../persistence.rs` (настройки UI) |
| `split.rs` (6445), `items.rs`, `split_editor_view.rs` | | Понятие «item» (открытый документ) и разбивка панели | Наш инспектор уже реализует «табы» — берём только модель «item + грязный флаг + undo-история» |
| `editor_settings.rs` | — | Полный список настроек редактора (ниже) | Наши настройки в `config.toml` + UI |

Настройки, которые Zed выносит в конфиг (`editor_settings.rs:20-74`) — это готовый чеклист для Ф8:

```rust
// REFERENCE (GPL): crates/editor/src/editor_settings.rs:20-74 (фрагмент)
pub cursor_blink: bool,
pub cursor_shape: Option<CursorShape>,
pub current_line_highlight: CurrentLineHighlight,
pub selection_highlight: bool,
pub lsp_highlight_debounce: DelayMs,
pub hover_popover_enabled: bool, pub hover_popover_delay: DelayMs,
pub scrollbar: Scrollbar, pub minimap: Minimap, pub gutter: Gutter,
pub scroll_beyond_last_line: ScrollBeyondLastLine,
pub vertical_scroll_margin: f64, pub horizontal_scroll_margin: f32,
pub relative_line_numbers: RelativeLineNumbers,
pub multi_cursor_modifier: MultiCursorModifier,
pub search_wrap: bool, pub search: SearchSettings,
pub auto_signature_help: bool,
pub diagnostics_max_severity: Option<DiagnosticSeverity>,
pub completion_menu_item_kind: CompletionMenuItemKind,
pub diff_view_style: DiffViewStyle, pub file_diff: FileDiffSettings,
```

Значения по умолчанию берём из `assets/settings/default.json` (2999 строк): `tab_size: 4`, `hard_tabs: false`, `soft_wrap: "none"`, `preferred_line_length: 80`, `autosave: "off"`, `format_on_save: "off"`, `remove_trailing_whitespace_on_save: true`, `ensure_final_newline_on_save: true`, `show_whitespaces: "selection"`, `relative_line_numbers: "disabled"`, `scroll_beyond_last_line: "one_page"`, `buffer_font_size: 15`.

### 3.8. LSP: `crates/lsp` (2878) + `crates/project/src/lsp_store.rs` (17 061) + `lsp_command.rs` (6114), GPL → **ИДЕЯ + ПОРТ контрактов**

```rust
// REFERENCE (GPL): crates/lsp/src/lsp.rs:115-140 — процесс языкового сервера
pub struct LanguageServer {
    server_id: LanguageServerId, next_id: AtomicI32,
    outbound_tx: async_channel::Sender<String>,
    notification_tx: async_channel::Sender<NotificationSerializer>,
    name: LanguageServerName, version: Option<SharedString>, process_name: Arc<str>,
    binary: LanguageServerBinary,
    capabilities: RwLock<ServerCapabilities>,
    configuration: Arc<DidChangeConfigurationParams>,
    code_action_kinds: Option<Vec<CodeActionKind>>,
    notification_handlers: Arc<Mutex<HashMap<&'static str, NotificationHandler>>>,
    response_handlers: Arc<Mutex<Option<HashMap<RequestId, ResponseHandler>>>>,
    pending_respond_tasks: PendingRespondTasks,
    io_handlers: Arc<Mutex<HashMap<i32, IoHandler>>>,
}
```

```rust
// REFERENCE (GPL): crates/lsp/src/lsp.rs:1503-1520 — все запросы идут через один таймаутируемый future
pub fn request<T: request::Request>(&self, params: T::Params, request_timeout: Duration)
    -> impl LspRequestFuture<T::Result> + use<T>;
```

Что берём по LSP:

| Артефакт | Откуда | Что это нам даёт |
|---|---|---|
| `LanguageServer` (транспорт, `$/cancelRequest`, обработчики уведомлений и кастомных запросов, лог диалога) | `lsp/src/lsp.rs` | Схема управления дочерним процессом языкового сервера, поверх **`tower-lsp` + `lsp-types` (MIT)** — они дадут протокол, а мы не пишем свой транспорт |
| Набор возможностей из `LspStore` | `project/src/lsp_store.rs` | Готовый перечень фич редактора: `completions` (7770), `resolve_completions` (7967), `apply_additional_edits_for_completion` (8355), `hover` (9153), `definitions/declarations/type_definitions/implementations/references` (7071-7451), `code_actions/resolve_code_action/apply_code_action_kind` (7694/6688/6764), `pull_diagnostics`/`pull_diagnostics_for_buffer` (8512/8961), `inlay_hints` (8662), `symbols` (9226), `signature_help` (9084), `format_ranges_via_lsp` (2494), `on_type_format` (7006) |
| **Как LSP-правки превращаются в правки буфера** | `project/src/lsp_command.rs`: `edits_from_lsp` (3600), `deserialize_text_edits` (3557) | Самый ценный для нас кусок: формат `TextEdit` → `Anchor`-based `Edit`'ы нашего буфера, включая позиции в UTF-16. Это рецепт «не сломать буфер, применив автодополнение» |
| Разделение «локальный сервер» / «сервер на удалённой машине» | `as_local`/`as_remote`, `upstream_client` | В нашей терминологии: языковой сервер может жить на ПК, а клиент — быть web/тонким |

Итог по фазам: полноценный LSP подключаем в Ф5, но **точку расширения закладываем сразу** (фреймы `lsp_*` в протоколе, поле `diagnostics` в render plan), иначе переделка.

### 3.9. Проект, ФС, Git: `crates/project` (82 718), `crates/fs` (7822), `crates/git` (9724), GPL → **ПОРТ контрактов**

| Артефакт Zed | LOC | Что берём |
|---|---|---|
| `Project::{open_buffer (3277), open_buffer_with_lsp, open_buffer_by_id, save_buffer (3455), save_buffer_as (3460), save_buffers (3440), create_entry (2615), rename_entry (2648), delete_entry (2748), find_or_create_worktree (4928)}` | `project/src/project.rs` 7084 | API «проект = рабочие деревья + буферы + сохранение». Нам нужен тот же набор как REST-контракт (см. §5), и `buffer_store.rs` (1911) как реестр открытых буферов с одной записью на путь |
| `fs::Fs` + `fs_watcher` (`notify`) + `PathEvent`/`PathEventKind` (`fs/src/fs.rs:80-88`), `fs/src/fs.rs:152` (ошибки запуска watcher'а → в лог, не в панику) | 7822 | Свой файловый слой поверх крейта `notify` (**MIT/CC0**): события Create/Modify/Remove/Rescan, обработка inotify-лимитов. Прямое требование для «файл изменился на диске — обновить буфер не потеряв правки» |
| `git_store.rs`: `stage_hunks (1417)`, `unstage_uncommitted_hunks (1532)`, `unstage_staged_hunks (1593)`, `open_unstaged_diff/open_staged_diff (1275/1347)`, `get_unstaged_diff/get_staged_diff (2036/2041)`, `stage_entries/unstage_entries (7954/7962)`, `stage_all/unstage_all (8136/8170)`, `restore_checkpoint (2160)` | 12 720 | Полный набор «Git в редакторе»: gutter-диффы, стейджинг по ханкам, откат, чекапойнты |
| `crates/git`: `Repository`, `CommitData`, `blame`, `status`, `stash`, `remote` | 9724 | Blame и статусы для дерева/панели (у нас `git_forge.rs`/`git_operations.rs` уже частично есть — сверяем) |
| `worktree_store.rs` (1522) | | Модель нескольких рабочих деревьев |

Ключевой фрагмент, который показывает, что «стейджить ханк» — это операция над **anchor-диапазонами буфера**, а не над текстом файла:

```rust
// REFERENCE (GPL): crates/project/src/git_store.rs:1417-1448
pub fn stage_hunks(&mut self, buffer: Entity<Buffer>, unstaged_diff: Entity<BufferDiff>,
                   worktree_ranges: Vec<Range<Anchor>>, cx: &mut Context<Self>) -> Result<()> {
    let buffer_snapshot = buffer.read(cx).snapshot();
    let unstaged_snapshot = unstaged_diff.read(cx).snapshot(cx);
    for range in &worktree_ranges {
        unstaged_hunks.extend(unstaged_snapshot.raw_hunks_intersecting_range(range.clone(), &buffer_snapshot));
        index_footprints.push(unstaged_snapshot.base_text_range_for_buffer_range(range.clone(), &buffer_snapshot));
    }
    …
}
```

### 3.10. Диффы: `crates/buffer_diff` (4404), GPL → **ПОРТ**

`BufferDiff`/`BufferDiffSnapshot`/`BufferDiffUpdate`, `DiffHunk`, `DiffHunkStatus { DiffHunkStatusKind, DiffHunkSecondaryStatus }`, `PendingHunk`/`PendingSense` (оптимистичный стейджинг, пока git не подтвердил), `hunks_intersecting_range` (364), `raw_hunks_intersecting_range` (383), `base_text_range_for_buffer_range` (415). Это то, что рисует наш gutter и что валидирует «предложение агента»: базовый текст (HEAD/индекс) против текста буфера. Для самого диффа берём крейт `similar` (**Apache-2.0**), как это делает `apps/omnescode` (`Cargo.toml: similar = { version = "2", features = ["text"] }`).

### 3.11. `crates/multi_buffer` (17 707), GPL → **ПОРТ идеи (позже, Ф6/Ф7)**

Мультибуфер = один «виртуальный документ» из кусков нескольких файлов, с заголовками, «развернуть контекст» и статусами ханков (`Excerpt`, `should_expand_up/down`, `DiffHunkStatus`, `excerpt_context_lines`, `expand_excerpt_lines`). Это то, что даёт Zed «Search All» и «Review Changes» одним списком. Нам нужен для: (а) результатов поиска по проекту, (б) сводного ревью правок агента по нескольким файлам. В MVP не включаем — но структуру крейта учитываем, чтобы не пришлось переписывать выборки.

### 3.12. Навигация: `file_finder` (7864), `search` (17 299), `outline` (1219), `breadcrumbs` (127), `command_palette` (2148) — GPL → **ПОРТ**

* `file_finder`: `FileFinder`, `FileFinderDelegate`, `Match`, `match_count`, `match_score` (665) — быстрый открыватель файлов; матчер у Zed вынесен в `crates/fuzzy`/`fuzzy_nucleo` (обе GPL) → у нас берём **`nucleo-matcher`** или **`fuzzy-matcher`** (MIT/MPL — уточнить лицензию при подключении) либо пишем свой скорер по спецификации Zed (последовательное совпадение + бонусы за границы слов/пути).
* `search`: `buffer_search.rs` (поиск в файле: regex, word-boundary, case/smart-case, «seeded from cursor», `search_wrap`), `project_search.rs`, `text_finder`, `search_bar.rs` — каркас «найти/заменить», включая поиск по всему проекту.
* `outline`, `breadcrumbs`, `command_palette` — модельные списки символов/пути/команд; у нас `command_palette_dialog.dart` уже есть, расширяем его экшенами редактора.

### 3.13. Темы и настройки: `theme` (5885), `theme_settings` (2422), `settings` (9197), GPL → **ПОРТ схемы**

* `Theme::syntax() -> &Arc<SyntaxTheme>` (`theme.rs:275`), `theme/src/styles/syntax.rs` — структура «имя capture → список пар (HighlightStyle, HighlightId)». Наш `build_highlight_map` (3.5) заполняется из неё.
* `theme_settings` — JSON-схема темы (`SyntaxTheme` в файле темы). Забираем **формат** (тема = JSON: цвета UI + таблица syntax-стилей), кладём в `frontend/shared/lib/editor/themes/*.json`, чтобы тема редактора и тема приложения шли из одного файла, а сервер и Flutter использовали одну и ту же таблицу стилей.
* `settings` (дефолты + JSON-схема + merge слоёв «default → user → project → язык») — берём модель слоёв настроек; у нас настройки живут в `config.toml` + `GetStorage`, добавляем секцию `[editor]` и `[editor.languages.<lang>]` (у Zed ровно это: глобальные `tab_size`, а для `python`/`json` — свои).

### 3.14. Vim-режим: `crates/vim` (48 674), GPL → **ИДЕЯ, опциональная фаза**

`motion.rs`, `object.rs`, `normal.rs`, `insert.rs`, `visual.rs`, `command.rs`, `surrounds.rs`, `replace.rs`, `indent.rs`, `digraph.rs`, `helix.rs`, `state.rs`, `mode_indicator.rs`, плюс тестовые данные `crates/vim/test_data/*.json`. Если vim-режим нужен — берём **модель** (режимы как состояния, motions как функции над `DisplaySnapshot`, тесты как data-driven JSON) и реализуем во Flutter по этим контрактам. В MVP не входит.

### 3.15. `crates/gpui` (82 639, Apache-2.0) → **ИДЕЯ, код не переносим**

GPUI — свой GPU-фреймворк (WebGPU/wgpu, своя система элементов, `text_system.rs` с шейпингом и кэшем строк, `FontId`, `TextSystem::layout_line`, `LineWrapper`). Во Flutter он бесполезен (другой рендер-стек), но три вещи оттуда берём как требования к нашему вьюпорту:

1. **Кэш шейпинга/раскладки строк.** У GPUI строка раскладывается один раз и переиспользуется, пока не изменились текст/шрифт/ширина. Во Flutter аналог — `ui.Paragraph`/`TextPainter` кэш на «layout line» + `RepaintBoundary`, а не `Text` на каждую строку (текущий `Column` в `inspector_panel.dart:432-445` — как раз антипаттерн: N `Text`-виджетов и пересчёт при каждом кадре).
2. **Покраска «фрагментов строки»** (`editor/src/element.rs`: `LineFragment`, `HighlightedRange`, `LineHighlightSpec`) — одна строка = список ран с стилями, а не один стиль на строку. Именно это мы кладём в `highlight_runs` нашего render plan.
3. **Курсор/мерцание/анимация** (`blink_manager.rs`, `cursor_animation.rs` 1343, `CursorLayout`): мерцание отдельно от данных, анимация только как визуальный слой.

---

## 4. Целевая архитектура OmnesAgent

### 4.1. Схема

```
┌──────────────────────────── Flutter (desktop + web) ─────────────────────────┐
│  features/inspector/editor/                                                  │
│   ├── editor_controller.dart     (GetX: буфер-вьюпорт, курсоры, undo-стек)    │
│   ├── editor_viewport.dart       (CustomPaint + InputClient, виртуализация)  │
│   ├── editor_gutter.dart         (номера строк, git-маркеры, фолды)          │
│   ├── editor_minimap.dart        (позже)                                     │
│   ├── completion_menu.dart / hover_popover.dart / diagnostics_panel.dart      │
│   └── editor_client.dart         (WS-клиент канала /ws/editor/{buffer_id})    │
└───────────────▲──────────────────────────────┬───────────────────────────────┘
                │ rows_snapshot / rows_changed │ edit_ops, cursor, selection,
                │ diagnostics, lsp_result      │ save, fold, stage_hunk, accept_proposal
                │                              ▼
┌────────────────────────── omnesagent-gateway (Rust, MIT) ────────────────────┐
│  ws_editor.rs         — канал на буфер (аналог ws_terminal.rs по стилю)      │
│  api_editor.rs        — REST: tree/file/save/diff/symbols/search/git          │
│  ┌──────────────────── omnesagent-editor (НОВЫЙ крейт) ───────────────────┐ │
│  │ buffer/  : rope(+vendor/sum_tree), anchor, op_log, undo_map, op_queue   │ │
│  │ display/ : tab, wrap, fold, inlay, block, creases, highlights           │ │
│  │ view     : selections, movement, scroll, folding, indent guides, minimap │ │
│  │ syntax   : language registry, tree-sitter parse, highlight_cache         │ │
│  │ lsp/     : управление языковыми серверами, конвертация TextEdit→Edit     │ │
│  │ git/     : buffer_diff, hunks, stage/unstage/revert, blame               │ │
│  │ project  : worktrees, buffer registry, fs watcher, settings              │ │
│  └──────────────────────────────────────────────────────────────────────────┘ │
│  omnesagent-tools: file_edit.rs / file_write.rs / backup_tool.rs (агент)      │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 4.2. Принятые решения (и почему)

| № | Решение | Почему | Альтернатива, которую отвергаем |
|---|---|---|---|
| Р1 | **Владелец буфера — шлюз**, Flutter не читает и не пишет файлы проекта | Агент пишет те же файлы своими инструментами; две независимые записи = потеря правок. Плюс агенту нужен буфер как объект, а не «файл на диске» | Оставить локальное чтение (`universal_io`) и дописать watcher — конфликт правок не решается, а маскируется |
| Р2 | Подсветка и координаты считаются на сервере; Flutter получает готовые runs | Один tree-sitter на процесс вместо парсера в Dart; кириллица/UTF-16 считаются в одном месте; экономия трафика (стили — это id, не строки) | Тянуть tree-sitter в Dart (`tree_sitter_dart`/`flutter_tree_sitter`) — дубль логики, отставание версий грамматик, вес в Web |
| Р3 | Правки применяются через op-log с Anchor + Lamport (порт `text`) | Позволяет одновременное редактирование человеком и агентом, локальные undo-стеки, «свои» правки не откатываются чужими | «Записать файл целиком при сохранении» — агенту придётся ждать, а UI — перечитывать |
| Р4 | Ввод оптимистичен локально, сервер подтверждает версией | Кейстрок не должен ждать RTT (даже локальный WS — это лишние миллисекунды на каждом символе) | «Каждый символ — round-trip» — заметная задержка набора, «залипание» каретки |
| Р5 | Render plan отправляется только для видимых строк (+шаг предзагрузки) | Стотысячестрочные файлы не должны уезжать в UI | Отправлять весь файл — трафик и тормоза, как в текущем `_buildFileViewerTab` |
| Р6 | Render plan версионируется (`buffer_version` + `display_version`) | Иначе правка во время скролла даёт «сдвиг» каретки | Без версий — гонки, которые невозможно отладить |
| Р7 | Языковые серверы — дочерние процессы шлюза, результаты стримятся в UI | LSP требует файловую систему; клиент может быть web/удалённым | LSP во Flutter — невозможно для rust-analyzer/clangd |
| Р8 | Тема редактора и таблица syntax-стилей — один JSON, общий для сервера и UI | Не разъезжаются цвета подсветки и интерфейса (сейчас тема в `DesktopTheme` + `apps/omnescode` своя) | Две независимые цветовые таблицы |

### 4.3. Что отправляется в UI (render plan, конкретика)

На строку:

| Поле | Тип | Смысл |
|---|---|---|
| `row` | u32 | номер **display**-строки (то, что скроллится; ≠ номер строки файла при wrap/фолдах) |
| `buffer_row` | u32? | номер строки в буфере (для нумерации в gutter; отсутствует для «виртуальных» строк — плейсхолдер фолда, блок) |
| `kind` | `text` \| `fold` \| `block` \| `spacer` | тип строки |
| `text` | string | текст видимой строки (уже с развёрнутыми табами — как `TabSnapshot`) |
| `runs` | `[(len, style_id)]` | run-length разметка: id стиля из общей таблицы (синтаксис, семантические токены, диагностика, поиск) |
| `inlays` | `[(col, text, style_id)]` | инлайны (inlay hints, призрак-предложение агента) |
| `fold` | `{id, hidden_rows, label}`? | свёрнутый регион |
| `indent_guide_cols` | `[u32]` | колонки направляющих |
| `selections` | `[(start_col, end_col, kind)]` | выделения/курсоры на этой строке (шрифт/цвет — из общих настроек) |

Стили (`style_id → {color, bg, underline, bold, italic, strikethrough}`) отправляются один раз на сессию/при смене темы (фрейм `styles`), дальше только id. Это делает дельты строк дешёвыми: правка символа = одна строка + один `runs`.

---

## 5. Протокол

### 5.1. REST (расширение `omnesagent-gateway`, рядом с существующими роутами `lib.rs:1645+`)

| Метод | Путь | Назначение | Аналог в Zed |
|---|---|---|---|
| GET | `/api/editor/tree?root=<path>&depth=<n>` | Дерево проекта (ленивая подгрузка уровней) | `Project`/`worktree_store` |
| GET | `/api/editor/file?path=<path>` | Прочитать файл (в буфер, не «в UI») | `Project::open_buffer` |
| PUT | `/api/editor/file?path=<path>` | Записать (атомарно: temp + rename, как в `file_write.rs`) | `Project::save_buffer_as` |
| GET | `/api/editor/buffers` | Открытые буферы: путь, версия, грязный флаг, кодировка, EOL | `buffer_store.rs` |
| DELETE | `/api/editor/buffer?path=<path>` | Закрыть буфер (с проверкой грязного) | `BufferStore` |
| POST | `/api/editor/git/stage-hunks` | Застейджить ханки по anchor-диапазонам | `git_store.rs::stage_hunks` |
| POST | `/api/editor/git/unstage-hunks` \| `/revert-hunks` | Откат/снятие | `unstage_*`, `Restore` |
| GET | `/api/editor/git/diff?path=<path>&stage=worktree\|index` | Ханки + базовый текст | `open_unstaged_diff`/`get_unstaged_diff` |
| GET | `/api/editor/symbols?path=<path>` | Символы файла (outline) | `document_symbols.rs`, `outline` |
| GET | `/api/editor/search?q=&root=&regex=&case=` | Поиск по проекту (SSE/чанками) | `search/project_search.rs` |
| GET | `/api/editor/find?q=&path=&regex=` | Поиск в файле (поиск и замена) | `search/buffer_search.rs` |

### 5.2. WebSocket `/ws/editor/{buffer_id}` (стиль и авторизация — как в `ws_terminal.rs`)

| Фрейм | Направление | Полезная нагрузка (сокращённо) |
|---|---|---|
| `hello` | S→C | `{buffer_id, path, language, version, display_version, eol, encoding, read_only, styles:{…}, settings:{tab_size, soft_wrap, …}}` |
| `rows_snapshot` | S→C | `{display_version, from_row, rows:[…], total_rows, folded:[…]}` |
| `rows_changed` | S→C | `{display_version, edits:[{row, kind, row:…}], removed:[…]}` |
| `edit_ops` | C→S | `{replica:1, base_version, ops:[{range:[start,end], text}], group:"typing"}` — оптимистично применённые локально |
| `edit_ack` | S→C | `{op_id, version, applied:true, adjusted_ops?}` — если сервер сдвинул правку (конкурентная вставка агента) |
| `remote_ops` | S→C | правки других реплик (агент, другое окно): `{replica:2, ops:[…], author:{tool:"patch", session_id}}` |
| `cursor` / `selection` | обе | `{replica, anchors:[{ts:{replica,value}, offset, bias}]}` — именно anchor'ы, не смещения |
| `presence` | S→C | `{replica:2, kind:"agent", label:"claude-code", color, cursor_range}` — «где сейчас агент» |
| `folds` | C→S | `{op:"fold"\|"unfold"\|"fold_all", ranges:[…]}` |
| `diagnostics` | S→C | `{path, items:[{severity, range_utf16, message, source, code}]}` |
| `lsp_request` / `lsp_response` | C↔S | `{id, kind:"completion"\|"hover"\|"definition"\|"code_action"\|"format"\|"signature"\|"inlay_hints", params, result}` |
| `agent_proposal` | S→C | `{id, replica:2, summary, hunks:[{range_anchors, new_text, accepted:null}]}` — предложение агента; UI рисует inline-виджет accept/reject |
| `agent_proposal_result` | C→S | `{id, accepted:[hunk_ids], rejected:[…]}` |
| `save_state` | обе | `{path, dirty, saved_at, external_change:{mtime, drift}}` — реакция на события ФС |
| `error` | S→C | `{code, message, context}` |

Ключевые инварианты протокола (пункты приёмки):
1. Любой фрейм несёт версии (`buffer_version`, `display_version`) и не применяется, если база не совпадает.
2. `rows_snapshot` запрашивается диапазоном (`from_row`, `count`) и всегда идемпотентен — можно перезапросить и «вылечиться» после сбоя.
3. Правки, пришедшие извне, никогда не меняют `edit_ops`-очередь клиента; их слияние — на сервере (op-log).

---

## 6. Фазы реализации

> **ДОПОЛНЕНИЕ v2:** состав работ по фазам уточнён в §11.11 (что берём готовым из Lapce, что остаётся реализовывать по референсу Zed). Объёмы Ф0-Ф3 при этом уменьшаются.

Обозначения объёма: **S** — до пары дней, **M** — до недели, **L** — 2-3 недели (для одного разработчика, при наличии спецификации; оценки — ориентировочные, уточняются после Ф0).

### Ф0. Спайк: крейт + канал + вкладка (объём M)
Задачи:
1. Вендорить `sum_tree` (Apache-2.0) в `backend/crates/omnesagent-editor/vendor/`, подключить как member воркспейса, `NOTICE` + запись в `THIRD_PARTY.md`.
2. Завести крейт `omnesagent-editor`: `rope` (минимальный: `push/slice/offset_to_point/point_to_offset/line_len`, чанки, кириллица), `buffer` (снимок + `edit` + версия), `anchor` + `clock`.
3. `ws_editor.rs`: канал `/ws/editor/{buffer_id}` (авторизация как в `ws_terminal.rs:44`, форматы JSON) с фреймами `hello`, `rows_snapshot`, `edit_ops`, `edit_ack`.
4. REST `GET /api/editor/file`, `PUT /api/editor/file` (атомарная запись).
5. Во Flutter: вкладка `editor` в `inspector_panel.dart` (новый key `editor`, карточка в `_buildOpenTabChooser`), простой вьюпорт (`CustomPaint`, виртуализация по строкам, gutter), ввод текста через `TextInputClient`.
DoD: файл открывается, редактируется, сохраняется; 10 000 строк скроллятся без залипаний; на кириллице курсор не встаёт в середину символа.
Проверка: unit-тесты rope (кириллица/эмодзи/CRLF), интеграционный тест WS, ручная проверка в desktop-приложении (обязательно с реальным запуском, не только `flutter analyze`).

### Ф1. Read-only паритет + подсветка (объём L)
Задачи: `display/{tab,wrap}`, `syntax` (реестр языков, tree-sitter, `HighlightMap` 1:1 с контрактом Zed, кэш по образцу `highlight_cache.rs`), render plan со стилями и runs; gutter с номерами строк, `relative_line_numbers`; `soft_wrap: none|editor_width|bounded`; `show_whitespaces`; тема синтаксиса из общего JSON; фолды по отступам (`crease_map` → `fold_map`) + LSP-фолды как опция.
DoD: подсветка Dart/Rust/JSON/MD/TOML/YAML/Shell не хуже `inkjet` по времени (бюджеты §7), фолды по индентации работают, wrap не ломает нумерацию.
Проверка: снапшот-тесты подсветки (golden-строки для набора файлов), бенч `criterion` (как `crates/language/benches/highlight_map.rs`).

### Ф2. Полноценное редактирование (объём L)
Задачи: `selections_collection` (мультикурсор, `multi_cursor_modifier`), `movement` (графемы, word/subword), выделение мышью и клавиатурой (включая `columnar_selection`), undo/redo на `UndoMap` + транзакции, копирование/вставка/вырезание, автоиндент, автоскобки/кавычки, сниппеты (`insert_snippet_at_selections`), IME-композиция (одна транзакция), автоскролл за курсором, «умный» дом/конец строки, Ctrl+D-мультиселект, кеймапы (реестр экшенов по образцу `actions.rs` + Flutter `Shortcuts`/`Actions`).
DoD: набор текста без потерь и без скачков каретки (включая быстрый ввод и вставку из буфера), undo/redo согласован с правками агента, мультикурсор работает.
Проверка: property-тесты на буфере (нумерация и стабильность анкоров), e2e-сценарии набора, тесты IME (ввод через кириллицу/диакритику).

### Ф3. Файловые операции и консистентность (объём M)
Задачи: `save`/`save_as`, `autosave` (`off|on_focus_change|on_window_change|after_delay`), `remove_trailing_whitespace_on_save`, `ensure_final_newline_on_save`, dirty-флаг в заголовке вкладки, диалоги «сохранить/отменить», `fs_watcher` (конфликты: файл изменён на диске при грязном буфере → диалог «перечитать/оставить моё/сравнить»), бэкапы через существующий `backup_tool.rs`, **закрытие техдолга**: убрать `universal_io.File`-чтение из `task_workspace_controller.dart:637-653` и обход ФС из `desktop_sidebar.dart:1824-1858` (дерево — через `/api/editor/tree`).
DoD: изменения на диске не теряют правки; ни одного прямого чтения файлов проекта из Flutter.
Проверка: тесты watcher'а (создание/удаление/переименование/быстрая серия правок), ручные сценарии с внешним редактором.

### Ф4. Git в редакторе (объём M)
Задачи: `buffer_diff` (порт), gutter-маркеры (added/modified/deleted), ховер ханка, `stage/unstage/revert hunk`, дерево проекта с git-статусами, blame-инлайн (опция), «просмотр диффа последнего коммита» через `multi_buffer`-подход (пока без мультибуфера — по одному файлу).
DoD: «застейджить ханк» из редактора даёт ожидаемый `git status`; откат ханка не рушит undo-историю буфера.
Проверка: интеграционные тесты на временном git-репозитории (`fs/src/fake_git_repo.rs` — идея для фикстуры), сверка с `git diff` в терминале.

### Ф5. LSP (объём L)
Задачи: `omnesagent-lsp` на `tower-lsp` + `lsp-types`; реестр языковых серверов и их запуск (rust-analyzer, typescript-language-server, ruff/pyright, dart analysis server, clangd); возможности: диагностики (push/pull), автодополнение, сигнатуры, хавер, переход к определению/ссылкам, code actions, форматирование (`format_on_save`, `on_type_format`), семантические токены, inlay hints, document symbols, folding ranges; конвертер `TextEdit → Edit<Anchor>` (по образцу `lsp_command.rs::edits_from_lsp`/`deserialize_text_edits`).
DoD: правки от LSP (в т.ч. автодополнение с дополнительными правками) не теряют данные; диагностики появляются < 1 с после правки (с дебаунсом `lsp_highlight_debounce`).
Проверка: интеграционные тесты против реального ruff (Python) и rust-analyzer, тест «apply completion with additional edits».

### Ф6. Навигация, поиск, символы (объём M)
Задачи: открытие файла по имени (fuzzy, `file_finder`), быстрый переход к строке/символу, outline-панель, breadcrumbs, поиск в файле (regex/case/smart-case, замены, «заменить все»), поиск по проекту с результатами (multibuffer-минимум: список + переход), история навигации.
DoD: Ctrl+P по 50k-файлам отвечает < 100 мс на запрос (индекс в памяти), поиск по проекту стримится.
Проверка: бенчи fuzzy-матчера, тесты поиска с regex-краевыми случаями.

### Ф7. Совместная работа с агентом (объём L) — то, ради чего всё затевается
Задачи: `agent_proposal` (правки агента как предложенные ханки, accept/reject по ханку/файлу/всё), inline-виджет предложения (через `block_map`), `remote_ops` с `presence` («агент сейчас здесь»), связанные диапазоны (`linked_editing_ranges`) для массовых правок, режим «следовать за агентом» (автоскролл к текущей правке), история правок агента с откатом (`UndoOperation` с фильтром по реплике), «diff-ревью» по нескольким файлам (multibuffer), сохранение ревью-комментариев (в терминах Zed — `StoredReviewComment` из `editor/src/git.rs`).
DoD: одновременное редактирование человеком и агентом не теряет правок; Ctrl+Z человека не отменяет принятые правки агента, и наоборот; предложение агента можно принять частично.
Проверка: сценарные тесты (симуляция двух реплик: агент вставляет/удаляет/заменяет, человек набирает и отменяет), инварианты буфера после каждой операции.

### Ф8. Полировка и настройки (объём M)
Задачи: секция `[editor]` в настройках (порт списка `editor_settings.rs`), темы (общий JSON синтаксиса + UI), статус-бар (язык, кодировка, EOL, позиция, отступы, версия/«сохранено»), мини-карта (опция), маркеры скроллбара (ошибки/поиск/правки агента), «перечитать с диска», восстановление сессии (`persistence.rs`: какие файлы и позиции были открыты), большие файлы (порог 10 МБ → подтверждение + read-only или «plain»-режим), лимиты линий по умолчанию.
DoD: настройки применяются без перезапуска; восстановление сессии возвращает вкладки и курсоры.
Проверка: набор UI-тестов, ручной чек-лист (обязательный браузерный/десктопный прогон перед «готово»).

### Ф9 (опционально). Vim-режим (объём L)
Порт модели `crates/vim` (motion/object/visual/surrounds/command) + data-driven тесты из `crates/vim/test_data/*.json`. Включается настройкой `vim_mode: false` по умолчанию.

### Порядок и зависимости

```
Ф0 ─► Ф1 ─► Ф2 ─► Ф3 ─┬─► Ф4 ─┐
                      ├─► Ф5 ─┼─► Ф7 ─► Ф8 ─► Ф9
                      └─► Ф6 ─┘
```
Ф4/Ф5/Ф6 можно вести параллельно после Ф2; Ф7 требует Ф2+Ф3 (иначе нечего синхронизировать) и выигрывает от Ф4 (ревью диффов).

---

## 7. Бюджеты производительности (пункты приёмки)

| Метрика | Цель | Как измеряем |
|---|---|---|
| Открытие файла 1 МБ | < 300 мс до первой отрисованной строки | бенч + лог времени в шлюзе |
| Ввод символа: keystroke → отрисованный кадр | < 16 мс (60 fps), p99 < 33 мс | трассировка во Flutter + серверные таймеры |
| Пересчёт подсветки после правки | < 8 мс на файлах до 5k строк | criterion (по образцу `crates/language/benches/highlight_map.rs`) |
| Скролл в файле 100k строк | без просадок ниже 55 fps, память вьюпорта O(видимые строки) | профилировщик Flutter |
| Размер фрейма `rows_snapshot` для 100 строк | < 64 КБ | тест на размер сериализации |
| Полный файл > 10 МБ | не блокирует UI: подтверждение + «plain»-режим | ручной тест |
| Память сервера на буфер 1 МБ | < 20 МБ (rope + op-log + подсветка) | замер RSS в тесте |

Если бюджет нарушен — это баг фазы, а не «оптимизация потом» (в Zed именно поэтому появились `TransformSummary`/кэши: без них правка перестраивает весь display map).

---

## 8. Тестирование

Порт стратегий тестирования — это, по сути, тоже «код из Zed», который стоит забрать:

| Что тестируем | Как в Zed | Что делаем у себя |
|---|---|---|
| Инварианты rope | `Rope::check_invariants` (`rope.rs`), вызовы после каждой мутации | Свои `check_invariants` (монотонность чанков, отсутствие пустых/слишком мелких чанков, соответствие length/summary) + вызов под `debug_assertions` и в тестах |
| Рандомизированные правки буфера | `text/src/tests.rs` (1057 строк): `random_byte_range`, `get_random_edits`, `randomly_edit`, `EditedBufferSnapshot` | Property-тест: N случайных правок → инварианты, обратимость undo/redo, стабильность анкоров |
| Суммирующее дерево | `sum_tree/src/property_test.rs` | Оставляем как есть (вендор) + прогон в нашем CI |
| Согласованность реплик | `text/src/tests.rs` (операции между репликами), `crates/collab` | Двух-репличный тест: агент и человек правят один буфер, сверяем итог у обоих и у диска |
| Подсветка | `language/benches/highlight_map.rs`, снапшот-тесты | Golden-тесты: для набора файлов одного языка сравниваем `runs` с эталоном |
| Display map | `editor/src/display_map*/**` (тесты внутри модулей: табы/врапы/фолды/инлайны) | Модульные тесты на каждый слой: `input/output` сводки, `sync` по `Edit<D>`, конвертеры точек туда-обратно (round-trip) |
| Git-ханки | `git_store.rs` + `fs/src/fake_git_repo.rs` | Интеграционный тест на временном репозитории: правка → ханк → stage → сверка с `git diff --cached` |
| LSP | `crates/project/src/lsp_command.rs` (6114 строк тестов и логики), `editor/src/code_completion_tests.rs` | Тесты против реального ruff/rust-analyzer + фейковый LSP-сервер (JSON-RPC по stdin/stdout) |
| UI/e2e | `editor_tests.rs` (49 852 строки) — экшены на уровне редактора | Flutter `integration_test` + реальный шлюз: печать, undo, фолды, сохранение, принятие предложения агента |

Отдельное правило проекта (из ваших требований): **после каждой правки UI — реальный запуск приложения и проверка глазами/консолью**, `flutter analyze` недостаточно.

---

## 9. Риски и открытые вопросы

| Риск | Влияние | Что делаем |
|---|---|---|
| **GPL-3.0 у Zed** (см. §2) | Юридический — блокирует копирование | Чистая переработка + вендор только Apache-2.0 (`sum_tree`); при желании переиспользовать дословно — отдельное решение владельца проекта (двойное лицензирование / GPL-сайдкар) |
| IME и не-латинская раскладка во Flutter | Невозможно печатать нормально | IME как одна транзакция (как `ime_transaction` в Zed), тесты на кириллицу/диакритику/эмодзи в Ф0-Ф2, а не «потом» |
| Текстовый слой Flutter vs свой рендер | Производительность и качество шейпинга | Виртуализация + кэш `Paragraph`/`TextPainter` на строку; при проблемах — низкоуровневая отрисовка глифов (`Paragraph` с `TextStyle` фиксированного шрифта) |
| Web-клиент (`frontend/web`) | Редактор в браузере | Модель уже серверная (Р1-Р2), поэтому web-клиент получает тот же WS — плюс архитектуры; отдельно решаем ввод (composition events) и производительность канваса |
| Сборка tree-sitter на Windows | CC-toolchain, время сборки | Уже решено в `apps/omnescode` (inkjet с dart-грамматикой) — переиспользуем ту же цепочку и закреплённые версии |
| Конфликты «агент снаружи»: инструменты могут писать файл, не открытый буфером | Потеря правок | Все записи агента (`file_edit.rs`, `file_write.rs`) переводим на API буфера (открыть → предложить/применить правки → сохранить); прямой обход ФС оставляем только как fallback с инвалидацией буфера |
| Отсутствие языковых серверов у пользователя | LSP-фичи не работают | Диагностика «сервер не найден» + установка по кнопке (как `LanguageServerBinaryOptions` с `allow_binary_download`) |
| Объём работ (Zed = 182k строк только в `editor`) | Расползание скоупа | Строгий DoD фаз; явные не-цели (см. ниже) |

**Не входит в скоуп (осознанно):**
* GPUI, собственный рендер-движок, мини-карта как в Zed (только примитивная);
* Zed-сервер колаборации, `crates/rpc`, аккаунты/каналы;
* AI-подсказки кода уровня `edit_prediction` (наш аналог — предложения агента, Ф7);
* редактор терминала/`omnescode` (это отдельное TUI-приложение);
* полноценный мультибуфер в MVP (только как основа для поиска и ревью).

---

## 10. Приложения

### A. Индекс изученных файлов Zed (из локального дерева ревизии `bda9c0bd`)

| Крейт | LOC `src/**/*.rs` | Файлов | Лицензия | Роль в плане |
|---|---|---|---|---|
| `sum_tree` | 3327 | 4 | Apache-2.0 | **Вендор** (§3.1) |
| `collections` | 418 | 3 | Apache-2.0 | Вендор (опционально) |
| `util` | 10162 | 21 | Apache-2.0 | Вендор точечно (`paths`, `disambiguate`, `size`) |
| `rope` | 4132 | 6 | GPL-3.0-or-later | Порт (§3.2) |
| `clock` | 338 | 2 | GPL-3.0-or-later | Порт (§3.4) |
| `text` | 6592 | 10 | GPL-3.0-or-later | Порт — ядро (§3.3) |
| `language` | 27927 | 22 | GPL-3.0-or-later | Порт (§3.5) |
| `language_core` | 2570 | 11 | GPL-3.0-or-later | Порт (highlight_map/highlight_cache) |
| `editor` | 182330 | 76 | GPL-3.0-or-later | Порт избранного (§3.7) |
| `multi_buffer` | 17707 | 5 | GPL-3.0-or-later | Идея, позже (§3.11) |
| `buffer_diff` | 4404 | 1 | GPL-3.0-or-later | Порт (§3.10) |
| `lsp` | 2878 | 2 | GPL-3.0-or-later | Порт контрактов (§3.8) |
| `project` | 82718 | 66 | GPL-3.0-or-later | Порт контрактов (lsp_store/git_store) |
| `fs` | 7822 | 5 | GPL-3.0-or-later | Порт на `notify` |
| `git` | 9724 | 8 | GPL-3.0-or-later | Порт/сверка с нашим `git_*` |
| `vim` | 48674 | 39 | GPL-3.0-or-later | Опционально (§3.14) |
| `theme` / `theme_settings` / `settings` | 5885 / 2422 / 9197 | 20/3/10 | GPL-3.0-or-later | Порт схемы (§3.13) |
| `search` | 17299 | 9 | GPL-3.0-or-later | Порт контрактов (§3.12) |
| `file_finder`, `outline`, `breadcrumbs`, `command_palette` | 7864 / 1219 / 127 / 2148 | | GPL-3.0-or-later | Порт (§3.12) |
| `fuzzy`, `fuzzy_nucleo` | 1210 / 1314 | 5/4 | GPL-3.0-or-later | Не берём; заменяем permissive-крейтом |
| `gpui` | 82639 | 90 | Apache-2.0 | Только идеи (§3.15) |

Крупные файлы, к которым будем возвращаться: `editor/src/editor.rs` 13130, `editor/src/element.rs` 14007, `editor/src/display_map.rs` 4599, `display_map/block_map.rs` 5819, `text/src/text.rs` 3844, `rope/src/rope.rs` 2518, `language/src/syntax_map.rs`, `project/src/lsp_store.rs` 17061, `project/src/git_store.rs` 12720.

### B. Как воспроизвести исследование (то, что делал я)

```bash
# 1. Частичный клон без истории и бинарных блобов
git clone --filter=blob:none --no-checkout --depth 1 https://github.com/zed-industries/zed.git zed-src
cd zed-src
# 2. Только нужные крейты
git sparse-checkout init --cone
git sparse-checkout set crates/editor crates/rope crates/text crates/language crates/language_core \
  crates/multi_buffer crates/lsp crates/project crates/gpui crates/theme crates/theme_settings \
  crates/vim crates/file_finder crates/buffer_diff crates/settings crates/util crates/fs crates/git \
  crates/clock crates/outline crates/breadcrumbs crates/search crates/command_palette crates/sum_tree \
  crates/collections crates/highlight crates/fuzzy crates/fuzzy_nucleo assets
git checkout
# 3. Фиксируем ревизию, на которую ссылается этот план
git rev-parse HEAD      # bda9c0bd43a8d235d82adb01ea5bc875b861ecfc
```
Полезное: `assets/settings/default.json` (2999 строк, дефолты настроек), `assets/keymaps/default-windows.json` (дефолтные биндинги — источник для нашего реестра экшенов), `crates/language/benches/highlight_map.rs` (образец бенча).

### C. Грамматики tree-sitter (паритет с Zed + наш Dart)

Zed (`Cargo.toml:873-896`): `tree-sitter` (ядро), `bash`, `c`, `cpp`, `css`, `diff`, `elixir`, `embedded-template`, `gitcommit`, `go`, `go-mod`, `gowork`, `heex`, `html`, `jsdoc`, `json`, `md`, `python`, `regex`, `ruby`, `rust`, `typescript`, `yaml`.
Добавляем: `tree-sitter-dart` (наш стек), при необходимости `sql`, `toml`, `dockerfile`, `hcl` (у нас уже есть в `inkjet`-фичах — можно переиспользовать те же версии).
Правило: версии грамматик фиксируем в `Cargo.toml` по образцу Zed (в т.ч. git-ревизии для форков), иначе «поедут» queries и, как следствие, подсветка.

### D. Permissive-зависимости, которыми заменяем Zed-крейты

| Задача в Zed | Крейт Zed | Наша замена | Лицензия |
|---|---|---|---|
| Суммирующее дерево | `crates/sum_tree` | **вендор `sum_tree`** | Apache-2.0 |
| Rope | `crates/rope` | свой rope (§3.2) | — |
| Дифф текста | `similar` (используется в `apps/omnescode`) | `similar` | Apache-2.0 |
| LSP-протокол и транспорт | `lsp-types` + собственный `crates/lsp` | `lsp-types` + `tower-lsp` | MIT |
| Наблюдение за файлами | `notify` | `notify` | MIT/CC0 |
| Fuzzy-матчер | `fuzzy`, `fuzzy_nucleo` (GPL) | `nucleo-matcher` **или** `fuzzy-matcher` (лицензию проверить перед подключением) | MPL-2.0 / MIT |
| Кодировки/EOL | своё | `encoding_rs` + своё определение EOL | Apache-2.0/MIT |
| Форматирование/pretty | `prettier_store` (внешние бинарники) | внешние форматтеры через LSP `textDocument/formatting` | — |

Dart-пакеты (для справки — версии проверены на pub.dev на дату плана): `re_editor 0.10.0`, `flutter_code_editor 0.3.5`, `code_text_field 1.1.0`, `highlight 0.7.0`, `tree_sitter_dart 0.1.1`, `flutter_tree_sitter 0.0.8`.
**Решение:** собственный виджет вьюпорта (`CustomPaint` + `TextInputClient`) — потому что подсветка приходит с сервера, а перечисленные пакеты тянут собственный парсер/модель буфера и воюют с ней. Пакеты используем как ориентир по API и как временную страховку, если понадобится быстрый прототип.

### E. Реестр заимствований (что копируем, что переписываем)

> **ДОПОЛНЕНИЕ v2:** актуальная версия реестра — §11.12. Решением владельца (v3) деление на «можно/нельзя копировать» снято: берём из обоих проектов напрямую.

| Класс | Что | Действие |
|---|---|---|
| Дословное копирование | `crates/sum_tree` (Apache-2.0) | В `vendor/sum_tree`, `NOTICE` + `THIRD_PARTY.md` + фиксация ревизии upstream |
| Переписывание по спецификации | `rope`, `text`, `clock`, `display/*`, `language` (highlight), `buffer_diff`, контракты `lsp`/`project`/`git` | Свой код в `omnesagent-editor`; в doc-комментариях ссылка на исходный модуль Zed как на источник поведения |
| Архитектурные идеи (без кода) | `gpui/text_system`, `element`, `cursor_animation`, `edit_prediction`, `vim`, `multi_buffer` | Реализация во Flutter/своём Rust |
| Не берём | `fuzzy`/`fuzzy_nucleo`, GPUI, collab/`rpc`, `terminal` | Замена permissive-крейтами или отказ от фичи |

### F. Чек-лист «готово» для всего проекта

1. Вкладка `editor` в правой панели с реальным редактированием, подсветкой, gutter, диффами и сохранением.
2. Ни одного прямого чтения/записи файлов проекта из Flutter (кроме явно документированных исключений).
3. Агент и человек редактируют один буфер без потери правок; undo/redo разделены по репликам.
4. Языки: не менее Dart, Rust, JSON, YAML, TOML, Markdown, Shell — с подсветкой по tree-sitter и LSP-диагностикой там, где сервер доступен.
5. Бюджеты §7 подтверждены измерениями, тесты §8 зелёные, UI-проверка выполнена на реальном приложении.
6. Оформлен лицензионный след: `THIRD_PARTY.md`, `NOTICE` (Apache-2.0 — Lapce), MIT-копирайт (floem), ссылки на ревизии Zed и Lapce в doc-комментариях.

---

# 11. ДОПОЛНЕНИЕ v2: разбор Lapce — перенос базы на Apache-2.0/MIT

**Изучено:** `lapce/lapce` @ `b604d57de4a820006d335a3be0d7583eb8fab558` (0.4.6, edition 2024, rust 1.98, ветка `master`, 06.09.2026) + `lapce/floem` @ `1351ffb162faeb8be983f1301d718a0d5fb59c27` (подкрейт `editor-core`). Метод тот же: частичный клон + sparse-checkout (рецепт — приложение §11.13).

## 11.1. Что такое Lapce и почему это меняет план

Lapce — редактор на Rust с **разделением процессов**: UI-процесс (`lapce-app`, фреймворк Floem) общается по JSON-RPC со **служебным процессом `lapce-proxy`**, который владеет файловой системой, буферами, языковыми серверами (LSP), отладчиками (DAP), терминалом, git и плагинами. Плюс «удалённая разработка»: тот же прокси можно поднять на удалённом хосте по SSH, а UI скачивает туда нужный бинарь.

Это **та же граница разреза, которую предлагает наш план** (модель — в Rust за шлюзом, Flutter — вьюпорт), только у Lapce она уже реализована, отлажена и, главное, **лицензирована так, что её код можно брать в MIT-проект**:

| Что | Лицензия | Следствие |
|---|---|---|
| `lapce/lapce` целиком (`lapce-core`, `lapce-rpc`, `lapce-proxy`, `lapce-app`) | **Apache-2.0** (`LICENSE`, README:61) | перенос разрешён с сохранением NOTICE/уведомлений |
| `lapce/floem` → `editor-core` (`floem-editor-core` 0.2.0) | **MIT** (`license = "MIT"` в workspace, LICENSE = MIT Copyright 2023 Floem) | перенос разрешён с сохранением копирайта |
| `lapce-xi-rope` (crates.io, форк rope из xi-editor) | **Apache-2.0** | перенос/использование как зависимости |

## 11.2. Итоговое решение v2: откуда что берём

| Слой | Выбор v2 | Обоснование |
|---|---|---|
| Хранение текста | **`lapce-xi-rope`** (Apache-2.0) | проверенная rope с дельтами (`RopeDelta`, `DeltaBuilder`, `Transformer`, `Interval`, `Spans`) — не пишем сами |
| Модель правок, undo, ревизии | **`floem-editor-core::buffer`** (MIT) | `Buffer` с ревизиями, группами undo, `InvalLines`; 769 строк вместо порта `text` (6592) |
| Курсоры, выделения, слова, отступы, режимы, регистры, команды | **`floem-editor-core`** (MIT) | 9102 строки готовой модели: `Cursor`, `SelRegion`/`Selection`, `WordCursor`, `IndentStyle`, `Mode`, `Register`, `EditCommand`/`MoveCommand`/… |
| Ввод, автоиндент, вставка, движение по графемам | **`floem-editor-core::editor`, `char_buffer`, `word`, `soft_tab`** (MIT) | `Action::insert/do_edit/do_paste`, `EditType::breaks_undo_group`, `CharBuffer` (графемы/IME), snap для soft-tab |
| Подсветка (tree-sitter) | **`lapce-core/src/syntax`** (Apache-2.0, ≈2.3k строк) | `HighlightConfiguration`/`HighlightIter`/`SyntaxLayers`/`Syntax`, инъекции, `find_matching_pair`, `sticky_headers`; вместо Zed `language` (27.9k, GPL) |
| Реестр языков и свойства | **`lapce-core/src/language.rs`** (Apache-2.0, 2075) | `LapceLanguage` со свойствами (comment token, indent unit, sticky-header tags, queries) |
| Протокол UI↔сервис | **`lapce-rpc`** (Apache-2.0, 3760) | готовый DTO-каталог: `PathObject`, `FileNodeItem`, `BufferId`/`NewBufferResponse`/`BufferHeadResponse`, `DiffInfo`/`FileDiff`, `ProxyRequest`/`CoreNotification`, транспорт `stdio` |
| Сервис (буферы, watcher, LSP, git, терминал) | **`lapce-proxy`** (Apache-2.0, 9453) | `dispatch.rs`, `buffer.rs` (rev-гейтинг + генерация `TextDocumentContentChangeEvent`), `watcher.rs`, `plugin/lsp.rs` |
| Слой представления (фолды, врап, инлайны, блоки, складки) | **Zed** (референс, не код) | у Lapce нет многослойного display-map как абстракции — здесь Zed остаётся образцом (§3.6) |
| Широта LSP-фич | **Zed** (референс) | у Lapce каталог уже, чем `lsp_store.rs` (17k) — чеклист берём из Zed (§3.8) |
| Git-действия по хункам | **Zed** (референс) + `git2`/`similar` | у Lapce `DiffInfo`/`FileDiff` — уровень файла, не хунка; gutter и стейджинг хунков делаем сами (§3.9-3.10) |
| Мультибуфер/эксерпты | **Zed** (референс, позже) | у Lapce нет |
| Vim/модальное редактирование | **Lapce** (Apache-2.0) + Zed (референс) | в ядре Lapce уже есть `mode.rs`, `MotionModeCommand`, `CursorMode` — это дешевле, чем порт `crates/vim` (48.7k) |
| UI-фреймворк | **ничего не берём** | `floem` — Rust-UI, во Flutter бесполезен (как и GPUI) |

**Что это даёт по объёму:** вместо «свой rope + свой op-log + порт подсветки» (оценка v1: 1500-2500 строк только на rope/op-log плюс порт Zed language) получаем:
* `lapce-xi-rope` — как зависимость, 0 строк своих;
* `floem-editor-core` — 9102 строки готового кода (MIT), переносится как вендоренный крейт;
* `lapce-core/src/syntax` — ≈2350 строк (Apache-2.0) вместо порта 27927 строк Zed `language`;
* протокол — из готового каталога `lapce-rpc`, а не «придумываем с нуля».
Остаётся написать то, что специфично для нас: display-map (по референсу Zed), серверный цикл подсветки под render plan, LSP-широту, git-хунки, канал агента (`agent_proposal`, presence) и Flutter-вьюпорт.

## 11.3. `lapce-xi-rope` (Apache-2.0) — хранение текста и дельты

Rope из xi-editor с полным набором операций, которые нам нужны: `Rope`, `RopeDelta`, `DeltaBuilder`, `DeltaElement`, `Transformer` (перенос смещений/интервалов через дельту — это то, чем в большинстве случаев заменяется `Anchor` из Zed), `Interval`, `IntervalBounds`, `spans::{Spans, SpansBuilder}`, `multiset::Subset`, `tree::{Node, NodeInfo}`.

Как это уже используется в Lapce (реальное место, где видно, что модель дельт — самодостаточная):

```rust
// REFERENCE (MIT/файл floem-editor-core): editor-core/src/buffer/mod.rs:12-16 — только эти импорты закрывают всю работу с текстом и правками
use lapce_xi_rope::{
    Delta, DeltaBuilder, DeltaElement, Interval, Rope, RopeDelta,
    multiset::Subset,
    tree::{Node, NodeInfo},
};
```

`Transformer` (api из xi-editor — `Transformer::transform(offset, after: bool)`, `transform_interval(interval, after)`) — это механизм «сдвинуть мою позицию, потому что выше по тексту кто-то вставил», то есть ровно то, для чего в Zed заведён `Anchor` с Lamport-таймстампом. Разница: у xi-rope трансформация выполняется явно относительно конкретной дельты (одно направление, один шаг), у Zed — лениво через дерево. Для нас:

* **декор + LSP-позиции + курсоры агента → `Transformer`** (дешево, детерминированно, без CRDT-сложности);
* там, где нужна «живучесть сквозь несколько независимых правок» (ревью-комментарии, ханки диффа, живущие долго) — либо повторное применение `Transformer` по цепочке дельт, либо наш тонкий слой `Anchor` (идея из Zed §3.3, реализация своя, ~150 строк).

## 11.4. `floem-editor-core` (MIT, 9102 строки) — готовая модель редактора

Состав (реально измерено; строки включают тест-модуль `buffer/test.rs` 174):

| Файл | LOC | Что даёт нам |
|---|---|---|
| `buffer/mod.rs` | 769 | `Buffer`, `InvalLines`, ревизии, группы undo, `reload`, `init_content` |
| `buffer/rope_text.rs` | 678 | `RopeText` — трейт «текст» (offset↔line/col, `offset_to_position` для LSP) |
| `buffer/diff.rs` | 242 | `rope_diff`, `DiffLines`, `DiffExpand`, `expand_diff_lines`, `DiffBothInfo` — **готовый дифф строк для gutter** |
| `char_buffer.rs` | 1074 | `CharBuffer` — графемы/комбинированные символы, IME-буфер (болезненная часть, которую лучше не писать) |
| `editor.rs` | 1849 | `Action::{insert, do_edit, do_paste, execute_motion_mode}`, `EditConf`, `EditType` |
| `selection.rs` | 883 | `Selection`, `SelRegion`, `InsertDrift` (мультикурсор/мульти-выделения) |
| `cursor.rs` | 732 | `Cursor`, `CursorMode`, `CursorAffinity`, `ColPosition` |
| `word.rs` | 701 | `WordCursor`, `CharClassification`, `get_char_property` (слова/подслова с учётом языков) |
| `command.rs` | 519 | `EditCommand`, `MoveCommand`, `ScrollCommand`, `FocusCommand`, `MotionModeCommand`, `MultiSelectionCommand` → `Movement` |
| `line_ending.rs` | 434 | Определение и нормализация EOL (CRLF/LF/CR) |
| `soft_tab.rs` | 237 | Прилипание курсора к границам soft-tab при движении/мышью |
| `indent.rs` | 214 | `IndentStyle`, `auto_detect_indent_style` |
| `movement.rs` | 140 | `Movement` — движения курсора |
| `paragraph.rs` | 135 | Разбиение на параграфы |
| `mode.rs` | 99 | `Mode` (модальное редактирование) |
| `chars.rs`, `register.rs`, `util.rs` | 34/41/129 | Графемы, регистры (клипборды), вспомогательное |

Ключевые структуры (для понимания, что мы получаем «бесплатно»):

```rust
// REFERENCE (MIT): editor-core/src/buffer/mod.rs:66-92
#[derive(Debug, Clone)]
pub struct InvalLines {
    pub start_line: usize,
    pub inval_count: usize,
    pub new_count: usize,
    pub old_text: Rope,
}

#[derive(Clone)]
pub struct Buffer {
    rev_counter: u64, pristine_rev_id: u64, atomic_rev: Arc<AtomicU64>,
    text: Rope, revs: Vec<Revision>, cur_undo: usize, undos: BTreeSet<usize>,
    undo_group_id: usize, live_undos: Vec<usize>,
    deletes_from_union: Subset, undone_groups: BTreeSet<usize>, tombstones: Rope,
    this_edit_type: EditType, last_edit_type: EditType,
    indent_style: IndentStyle, …
}
```

```rust
// REFERENCE (MIT): editor-core/src/buffer/mod.rs:33-55 — undo-модель: правки «скармливаются» в группы
enum Contents {
    Edit { undo_group: usize, inserts: Subset, deletes: Subset },   // группы: автоиндент откатывается вместе с newline
    Undo { toggled_groups: BTreeSet<usize>, deletes_bitxor: Subset },
}
```

`EditType::breaks_undo_group(previous: EditType) -> bool` (`editor.rs:73`) — готовая политика «когда нажатие клавиши начинает новую группу undo» (то, что в Zed реализовано через транзакции/групповой интервал).

**Важно (проверять при переносе):** `floem-editor-core` не зависит от UI-фреймворка (зависимости: `lapce-xi-rope`, `ui-events`, `strum`, `itertools`, `bitflags`, `memchr`, `serde` опционально) — то есть модель переносима в наш бэкенд. Единственная «чужая» зависимость — `ui-events` (типы событий ввода; на crates.io поле license не заполнено — **проверить лицензию до вендоринга**, при необходимости вырезать и оставить свои event-типы).

## 11.5. `lapce-core/src/syntax` (Apache-2.0) — компактная подсветка вместо Zed

Что внутри (реально, с именами): `HighlightConfiguration` (+`::new`, `names`, `configure`, `injection_pair`), `HighlightIter` (реализует `Iterator`, а не события-колбэки, как в Zed), `HighlightEvent`, `HighlightIssue`, `Highlight(usize)`, `InjectionLanguageMarker`; `TsParser`; `LanguageLayer`, `SyntaxLayers::new/update/try_tree/highlight_iter`; `Syntax::{init, plaintext, from_language, parse, find_matching_pair, parent_offset, find_tag, sticky_headers, find_enclosing_parentheses, find_enclosing_pair, lens_*}`; `syntax/edit.rs` — `SyntaxEdit` (инкрементальный пересчёт при правке); `BracketParser`/`ASTNode`/`NodeType` — скобки и поиск парного элемента.

Сравнение по объёму (тот же функционал подсветки):

| | Lapce | Zed |
|---|---|---|
| Подсветка | `lapce-core/src/syntax/*` ≈ 2357 строк | `crates/language` 27927 + `language_core` 2570 |
| Реестр языков/свойства | `language.rs` 2075 | внутри `language`/`language_settings` |
| Лицензия | Apache-2.0 → **переносим** | GPL → только референс |

Почти всё, что нам нужно для Ф1 (подсветка + скобки + sticky headers + инъекции), у Lapce уже есть и переносимо. Единственная оговорка — **версия tree-sitter**: Lapce собирается с `tree-sitter = "0.22.6"`, а в нашем воркспейсе уже есть `tree-sitter 0.23.2` (транзитивно через `inkjet`, см. `backend/Cargo.lock:11042`). Две версии tree-sitter в одном процессе — источник бинарных конфликтов грамматик (Zed у себя это даже комментирует в `Cargo.toml`: «two instances of the crate would make grammar…»). Решение — **зафиксировать одну версию на весь воркспейс** и при переносе подсветки адаптировать вызовы под неё (API 0.22→0.23 отличается точечно), либо перенести `inkjet` на совместимую версию.

## 11.6. `lapce-rpc` (Apache-2.0) — готовый каталог протокола

```rust
// REFERENCE (Apache-2.0): lapce-rpc/src/lib.rs:18-30 — форма сообщений (то же, что нам нужно для WS)
pub enum RpcMessage<Req, Notif, Resp> {
    Request(RequestId, Req), Response(RequestId, Resp),
    Notification(Notif), Error(RequestId, RpcError),
}
pub struct RpcError { pub code: i64, pub message: String }
```

Каталог (то, что мы берём, возможно переименовывая):

* **Запросы `ProxyRequest`** (`lapce-rpc/src/proxy.rs` 1220 строк): `NewBuffer`, `BufferHead`, `GlobalSearch`, `GetHover`, `GetSignature`, `GetSelectionRange`, `CompletionResolve`, `CodeActionResolve`, `GotoImplementation`, `GetDefinition`, `GetTypeDefinition`, `GetReferences`, `ShowCallHierarchy`/`CallHierarchyIncoming`, `GetInlayHints`, `GetInlineCompletions`, `GetSemanticTokens`, `LspFoldingRange`, `PrepareRename`/`Rename`, `GetCodeActions`, `GetCodeLens`/`GetCodeLensResolve`, `GetDocumentSymbols`/`GetWorkspaceSymbols`, `GetDocumentFormatting`, `GetOpenFilesContent`, `GetFiles`, `ReadDir`, `Save`, `SaveBufferAs`, `CreateFile`, `CreateDirectory`, `TrashPath`, `DuplicatePath`, `RenamePath`, `TestCreateAtPath`, `GitGetRemoteFileUrl`, DAP-запросы.
* **Уведомления `ProxyNotification`**: `Initialize`, `OpenFileChanged`, `OpenPaths`, `Completion`, `SignatureHelp`, `Update`, `UpdatePluginConfigs`, `NewTerminal`, `InstallVolt`/`RemoveVolt`/`ReloadVolt`/`DisableVolt`/`EnableVolt`, `Shutdown`.
* **Уведомления сервиса в UI `CoreNotification`** (`core.rs` 433): `ProxyStatus{Connecting|Connected|Disconnected}`, `OpenFileChanged`, `CompletionResponse`, `SignatureHelpResponse`, `OpenPaths`, `WorkspaceFileChange`, `PublishDiagnostics`, `ServerStatus`, `WorkDoneProgress` (прогресс LSP!), `ShowMessage`, `LogMessage`, `LspCancel`, `HomeDir`, `VoltInstalled/Installing/Removing/Removed`, `DiffInfo`, `UpdateTerminal`, `TerminalLaunchFailed`, `TerminalProcessId`, `TerminalProcessStopped`, `RunInTerminal`, `Log`, DAP-события. (`CoreRequest` и `CoreResponse` — **пустые enum**: UI не запрашивает у сервиса, только командует через `ProxyRequest` и слушает уведомления. Хороший принцип для нашей модели: один канал команд, один канал событий.)
* **Данные**: `file.rs` 591 — `PathObject` (+`LineCol`) для открытия «файл:строка:колонка», `FileNodeItem`/`FileNodeViewData`/`FileNodeViewKind` — модель дерева файлов, `Naming`/`NamingState`/`Renaming`/`NewNode`/`Duplicating` — инлайн-операции ФС в дереве; `buffer.rs` — `BufferId(u64)`, `NewBufferResponse{content, read_only}`, `BufferHeadResponse{version, content}`; `style.rs` — `Style`/`LineStyle(s)` (готовый формат «стили по строкам» — нам это нужно для render plan!).
* **Git**: `source_control.rs` (полностью, 47 строк): `DiffInfo{head, branches, tags, diffs}`, `FileDiff::{Modified, Added, Deleted, Renamed}(PathBuf)`, `FileDiffKind` — уровень файла (без хунков).
* **Транспорт**: `stdio.rs` — `write_msg`/`read_msg` поверх `serde_json` + два потока (writer/reader) через `crossbeam_channel`; в `lapce-proxy/src/lib.rs` — ещё и локальный сокет (`interprocess::local_socket::LocalSocketListener`, `listen_local_socket`), чтобы CLI-запуск `lapce file:line:col` открывался в уже работающем процессе (`lapce-proxy/src/cli.rs::try_open_in_existing_process`).

## 11.7. `lapce-proxy` (Apache-2.0, 9453 строки) — сервисный слой

| Файл | LOC | Что берём |
|---|---|---|
| `dispatch.rs` | 1764 | Диспетчер: `handle_request(id, rpc)` + `use ProxyRequest::*; match rpc { … }`. В ветке `NewBuffer` видно сразу четыре действия: создать буфер, отправить `did_open_document` в LSP, поставить файл под watcher, ответить клиенту содержимым и флагом `read_only` — готовая архитектура «открыть файл» |
| `buffer.rs` | 351 | `Buffer{rope, rev, read_only, language_id}`, `save(rev, create_parents)`, `update(delta, rev)`, `offset_to_line_col`, `line_to_cow`, `load_file`, `read_path_to_string`, `language_id_from_path`, `get_mod_time` |
| `watcher.rs` | 275 | `FileWatcher`, `WatchToken`, `watch/watch_filtered/unwatch`, `take_events`, трейт `Notify` |
| `plugin/lsp.rs` | 557 | Управление языковым сервером как «плагином» |
| `plugin/{wasi.rs,psp.rs,mod.rs,catalog.rs}` | 629/1440/1797/756 | WASI/WASM-плагины и протокол плагин-сервера — **смотрим, но не берём** (переусложнение для нас) |
| `plugin/dap.rs` | 824 | Отладчик (DAP) — вне скоупа MVP |
| `terminal.rs` | 384 | Терминал — у нас уже есть свой (`ws_terminal.rs` + `DesktopTerminalService`) |
| `lib.rs` | 215 | `mainloop`, `stdio_transport`, `listen_local_socket` (локальный IPC), `get_url` |

Самое ценное для нас — rev-гейтинг и превращение одной дельты в три разные вещи (буфер, LSP, UI):

```rust
// REFERENCE (Apache-2.0): lapce-proxy/src/buffer.rs:116-133 — оптимистичная правка с проверкой ревизии
pub fn update(&mut self, delta: &RopeDelta, rev: u64) -> Option<TextDocumentContentChangeEvent> {
    if self.rev + 1 != rev { return None; }          // правка «не в строю» — отбрасываем
    self.rev += 1;
    let content_change = get_document_content_changes(delta, self);
    self.rope = delta.apply(&self.rope);             // 1) применить к rope
    Some(content_change.unwrap_or_else(|| TextDocumentContentChangeEvent {   // 2) сообщить LSP
        range: None, range_length: None, text: self.get_document(),
    }))
}
```

```rust
// REFERENCE (Apache-2.0): lapce-proxy/src/buffer.rs:305-343 — дельта превращается в LSP-изменение (простое вставки/удаления → точный range, иначе → весь документ)
fn get_document_content_changes(delta: &RopeDelta, buffer: &Buffer) -> Option<TextDocumentContentChangeEvent> {
    let (interval, _) = delta.summary();
    if let Some(node) = delta.as_simple_insert() { /* range + text */ }
    else if delta.is_simple_delete() { /* range + пустой text */ }
    else { None }   // сложное изменение — отдаём документ целиком
}
```

Это тот самый механизм, который в плане v1 надо было проектировать (§5, фреймы `edit_ops`/`rows_changed`). У Lapce он уже есть в двух местах: (а) в прокси — между буфером и LSP, (б) в UI (`lapce-app/src/doc.rs`: `apply_deltas(&[(Rope, RopeDelta, InvalLines)])` + `update_styles/update_inlay_hints/update_diagnostics/update_find_result(delta)` — то есть **список того, что нужно инвалидировать при правке**, и это ровно наш `rows_changed`).

## 11.8. `lapce-app` (Apache-2.0, ≈49k строк) — образец UI-слоя для нашего Flutter

| Файл | LOC | Что берём как образец |
|---|---|---|
| `doc.rs` | 2239 | Модель документа на стороне UI: применение дельт, инвалидация стилей/инлайнов/диагностик/результатов поиска/свёрток; слои стилей (`apply_colorization`, `apply_attr_styles`) |
| `editor.rs` / `editor/view.rs` | 3926 / 2667 | Разделение «данные редактора» и «вью» — прямое указание, как резать наш `EditorController` и `EditorViewport` |
| `main_split.rs` | 3105 | Табы, сплиты, порядок панелей (наш инспектор — упрощённая версия) |
| `keypress/keymap.rs`, `keymap.rs`, `command.rs` | 707, 562, 818 | Реестр команд (`LapceWorkbenchCommand`, `CommandKind`) + разбор биндингов в стиле VSCode (`KeyMapKey`, `KeyMapPress::parse`, `KeymapMatch`) — готовый образец для Flutter `Shortcuts`/`Actions` |
| `history.rs` | — | Undo/redo на уровне UI |
| `find.rs` | 545 | Поиск/замена в файле, прогресс, результаты |
| `snippet.rs` | 859 | Сниппеты и таб-стопы |
| `editor/diff.rs` | 552 | Вид диффа |
| `completion.rs`, `hover.rs`, `code_action.rs`, `code_lens.rs`, `inline_completion.rs`, `lsp.rs` | — | UI-части LSP-фич |
| `palette.rs` | 1700 | Палитра команд/файлов |
| `status.rs`, `db.rs` | 473, 452 | Статус-бар; локальная БД (персистентность состояния) |
| `proxy.rs`, `proxy/remote.rs` | —, 497 | Клиент RPC и **удалённый запуск**: определить ОС/архитектуру удалённого хоста, скачать туда подходящий бинарь прокси, поднять по SSH и подключиться |

Последнее особенно полезно для нашего VPS-сценария: у Lapce есть готовый рецепт «шлюз на удалённой машине» — определить `platform/architecture` хоста (`host_specification`), скачать нужный артефакт, запустить, подключиться по транспорту. Наш аналог: поднять `omnesagent-gateway` на VPS и подключить Flutter-клиент по WS/mTLS, используя ту же логику выбора бинаря.

## 11.9. Где Lapce слабее Zed (и почему Zed остаётся референсом)

| Функция | Lapce | Zed | Вывод для нас |
|---|---|---|---|
| Многослойное представление (fold/inlay/block/crease/wrap как отдельные слои с `Transform`/`InvalLines`) | нет такой абстракции; wrap/fold/sticky реализованы точечно | `display_map` 20 627 строк, строгая система слоёв | фолды/инлайны/блоки — по референсу Zed (§3.6) |
| Мультибуфер/эксерпты (ревью, «поиск по проекту одним документом») | нет | `multi_buffer` 17 707 | берём идею Zed позже (Ф6/Ф7) |
| Широта LSP | уже (нет, например, полного набора code-action/rename-цепочек и работы с «дополнительными правками» автодополнения) | `lsp_store.rs` 17 061 + `lsp_command.rs` 6114 | чеклист функций и логику `TextEdit→правки буфера` берём из Zed |
| Git: хунки, стейджинг по хунку, blame | `DiffInfo`/`FileDiff` по файлам, `BufferHead` (HEAD-версия файла) + `git2` | `git_store.rs` 12 720 (`stage_hunks`, `unstage_*`, `restore_checkpoint`), `git.rs` 3271 (gutter, blame) | gutter-дифф и хунки делаем по Zed; из Lapce берём `BufferHead` (получить содержимое HEAD-версии) |
| Темы/настройки | свои конфиги (`config.rs`, `settings.rs`) | JSON-схема темы + слои настроек | схему темы берём из Zed (§3.13) |
| Модальное редактирование | **в ядре** (`mode.rs`, `MotionModeCommand`, `CursorMode`) | отдельный крейт `vim` 48 674 | Ф9 дешевле делать на базе Lapce |
| Плагины | WASI/WASM (Volt) | нет (расширения — LSP/DAP/ACP) | не берём: не наш сценарий |

## 11.10. Как это меняет архитектурные решения (§4)

| Решение v1 | Изменение v2 |
|---|---|
| Р3: op-log с `Anchor` + Lamport из Zed | **Базовый слой заменяется**: `xi-rope` дельты + ревизии/undo-группы `floem-editor-core`. Lamport/anchor-идея нужна только для многопользовательской части (человек + агент + удалённое окно): реализуем тонкий «сеансовый слой» над буфером (нумерация ревизий, авторы правок, откат чужих правок, 3-way слияние через `rope_diff`/`DiffLines`) |
| Р4: оптимистичный ввод + подтверждение версией | подтверждается и уточняется: берём rev-гейтинг Lapce (`self.rev + 1 != rev → отказ`), добавляя к ревизии буфера id автора (реплику) |
| Р5: render plan только для видимых строк | усиливается: `InvalLines{start_line, inval_count, new_count, old_text}` — готовый формат «какие строки стали невалидными» → наш `rows_changed`; `InvalLines.old_text` позволит решать конфликты на клиенте при расхождении версий |
| Транспорт: WS `/ws/editor/{buffer_id}` | остаётся WS (наш шлюз уже так устроен), но берём из Lapce: (а) форму сообщений `RpcMessage<Req,Notif,Resp>`, (б) принцип «команды только в одну сторону, события — в другую» (`CoreRequest` пустой), (в) локальный IPC-канал для «открыть файл в уже запущенном шлюзе» (`interprocess` + `cli.rs::try_open_in_existing_process`) |
| Карта фреймов | расширяется: `ProxyStatus`→`server_status`, `ServerStatus`/`WorkDoneProgress` (статус и прогресс LSP), `ShowMessage`/`LogMessage`, `WorkspaceFileChange` (пакетное изменение файлов на диске), `DiffInfo` (git-сводка для дерева), `PathObject` (открытие «путь:строка:колонка»), `NamingState` (инлайн-переименование в дереве), `TerminalProcessStopped`/`TerminalLaunchFailed` (у нас уже есть терминал — интегрируем события) |

## 11.11. Уточнения к фазам (§6)

| Фаза | Что меняется |
|---|---|
| Ф0 | вместо «свой rope + свой buffer»: подключить `lapce-xi-rope` (crates.io) + вендорить `floem-editor-core` (MIT) как `omnesagent-editor/vendor/editor-core`; `Buffer::update(delta, rev)` и `InvalLines` берём почти как есть. Объём Ф0 уменьшается с M до **S-M** |
| Ф1 | подсветку не пишем: переносим `lapce-core/src/syntax` (Apache-2.0) и `language.rs`; сводим к одной версии tree-sitter (§11.5). Из Zed берём только структуру render plan (§4.3, §3.6, `highlighted_chunks`) |
| Ф2 | мультикурсор/слова/отступы/soft-tab/графемы — из `floem-editor-core` (MIT); `EditType::breaks_undo_group` даёт политику undo; IME-буфер — `CharBuffer` |
| Ф3 | `lapce-proxy/src/buffer.rs` (save с `create_parents`, `load_file`, `get_mod_time`) + `watcher.rs` (готовая обёртка над `notify`) — берём как есть |
| Ф4 | git: `BufferHead` + `DiffInfo`/`FileDiff` из Lapce (Apache) как «быстрый старт»; хунки/стейджинг — по референсу Zed |
| Ф5 | LSP: `plugin/lsp.rs` (как «плагин-процесс») + чеклист и логику правок из Zed (`lsp_command.rs`) |
| Ф6 | поиск: `GlobalSearch` из `lapce-rpc` + `lapce-proxy` (regex/case/word уже параметрами запроса), палитра — `lapce-app/palette.rs` как образец |
| Ф7 | наш собственный слой (в Lapce аналога нет): сеансовый слой поверх буфера + `agent_proposal`/`presence` |
| Ф9 | vim-режим: вместо порта `crates/vim` (48.7k, GPL) — расширить модальную модель Lapce (`Mode`, `MotionModeCommand`, `CursorMode`) под наши кеймапы |

## 11.12. Реестр заимствований (§10.E, v3)

> Решением владельца лицензионные обязательства не учитываем: всё перечисленное переносится напрямую, колонка «Обязательства» — только про воспроизводимость сборки.

| Класс | Что | Обязательства |
|---|---|---|
| **Дословное копирование (Apache-2.0)** | `lapce-rpc` (DTO), `lapce-core/src/syntax/*`, `lapce-core/src/language.rs`, `lapce-proxy/src/{buffer,watcher,dispatch}.rs` | сохранить NOTICE/уведомления Apache-2.0 и указание файлов-источников + ревизию |
| **Дословное копирование (MIT)** | `floem-editor-core` (весь крейт, 9102 строки) | сохранить MIT-копирайт (Copyright 2023 Floem) |
| **Зависимость (Apache-2.0)** | `lapce-xi-rope` 0.4.0 (проверить отличия API от 0.3.2, которую использует Lapce) | уведомление в `THIRD_PARTY.md` |
| **Дословное копирование (Apache-2.0, Zed)** | `crates/sum_tree` (3327) | NOTICE; нужно только если пойдём в display-map по Zed-модели |
| **Референс (GPL, без кода)** | всё остальное из Zed | в doc-комментариях ссылка «поведение по мотивам `zed/crates/...`», код свой |

## 11.13. Приложения v2

### G1. Индекс Lapce (измерено на ревизии `b604d57d`)

| Крейт | LOC `src/**/*.rs` | Лицензия | Роль в плане |
|---|---|---|---|
| `lapce-xi-rope` (crates.io) | внешний | Apache-2.0 | хранение текста + дельты (зависимость) |
| `floem-editor-core` (`lapce/floem`) | 9102 | MIT | модель редактора: буфер, undo, курсор, выделения, слова, отступы, режимы, команды |
| `lapce-core` | 5211 | Apache-2.0 | `syntax/*` (подсветка, скобки, sticky headers), `language.rs` (реестр), `lens`, `directory`, `encoding`, `style` |
| `lapce-rpc` | 3760 | Apache-2.0 | DTO-каталог протокола + транспорт |
| `lapce-proxy` | 9453 | Apache-2.0 | сервис: буферы, watcher, LSP/DAP/WASI-плагины, терминал, git-хелперы |
| `lapce-app` | 49142 | Apache-2.0 | образец UI-слоя (документ/дельта, вью, команды, палитра, статус, персистентность, remote) |
| `floem` (UI) | — | MIT | не берём (Rust-UI) |

Ключевые файлы-ориентиры: `floem/editor-core/src/buffer/{mod.rs 769, rope_text.rs 678, diff.rs 242}`, `editor-core/src/{char_buffer.rs 1074, editor.rs 1849, selection.rs 883, cursor.rs 732, word.rs 701, command.rs 519, line_ending.rs 434, soft_tab.rs 237, indent.rs 214}`, `lapce-core/src/{language.rs 2075, syntax/mod.rs 1280, syntax/highlight.rs 831, syntax/edit.rs 119}`, `lapce-rpc/src/{proxy.rs 1220, core.rs 433, file.rs 591, source_control.rs 47, stdio.rs 142}`, `lapce-proxy/src/{dispatch.rs 1764, plugin/mod.rs 1797, plugin/psp.rs 1440, plugin/dap.rs 824, plugin/catalog.rs 756, plugin/wasi.rs 629, plugin/lsp.rs 557, terminal.rs 384, buffer.rs 351, watcher.rs 275}`, `lapce-app/src/{doc.rs 2239, editor.rs 3926, editor/view.rs 2667, main_split.rs 3105, palette.rs 1700, snippet.rs 859, command.rs 818, find.rs 545, editor/diff.rs 552, proxy/remote.rs 497, db.rs 452, status.rs 473}`.

### G2. Рецепт клонирования

```bash
# Lapce: крейты лежат в корне репозитория, не в crates/
git clone --filter=blob:none --no-checkout --depth 1 https://github.com/lapce/lapce.git lapce-src
cd lapce-src && git sparse-checkout init --cone \
  && git sparse-checkout set lapce-core lapce-rpc lapce-proxy lapce-app \
  && git checkout && git rev-parse HEAD     # b604d57de4a820006d335a3be0d7583eb8fab558

# Модель редактора (MIT) — в репозитории Floem
git clone --filter=blob:none --no-checkout --depth 1 https://github.com/lapce/floem.git floem-src
cd floem-src && git sparse-checkout init --cone && git sparse-checkout set editor-core \
  && git checkout && git rev-parse HEAD     # 1351ffb162faeb8be983f1301d718a0d5fb59c27
```

### G3. Зависимости Lapce-воркспейса, релевантные нам (для сверки версий)

`lapce-xi-rope 0.3.2` (crates.io: 0.4.0), `tree-sitter 0.22.6` (**конфликт с нашим 0.23.2 — унифицировать**), `lsp-types`, `notify 5.2` (у нас в бэкенде есть свой watcher-путь — сверить), `git2 0.21` (vendored-openssl/libgit2 — тяжело собирается, но даёт git без внешнего бинаря), `interprocess 1.2.1` (локальный IPC), `im` (immutable-структуры), `indexmap`, `smallvec`, `strum`, `regex`, `rayon`, `globset`, `reqwest 0.11` (blocking, для скачивания бинарей прокси), `crossbeam-channel` (транспорт).

### G4. Открытые вопросы v2

1. **Вендорить или форкать?** `floem-editor-core` версионируется вместе с Floem (git-зависимость `rev = 31fa8f44`), изменения там бывают под UI. Предложение: вендорить с фиксацией ревизии, свои патчи держать маленьким патч-файлом, апстрим — по желанию.
2. **`ui-events`** — транзитивная зависимость `floem-editor-core` без явной лицензии на crates.io: проверить (и, если что, заменить своими event-типами).
3. **Версия tree-sitter** — свести к одной в воркспейсе (иначе конфликт грамматик при линковке; см. §11.5).
4. **Смешение моделей**: если оставляем Zed-модель display-map (слои, `Transform`) поверх Lapce-буфера — нужно решить, где живёт «display» (у нас: на сервере, слои над `xi-rope`-деревом). Альтернатива: упрощённая модель (таблицы строк + инлайн-записи), как в `lapce-app/doc.rs`, что дешевле, но беднее по возможностям (нет sticky scroll за пределами `sticky_headers`, нет произвольных блоков виджетов).
5. **Лицензионный след** — добавить в корень `THIRD_PARTY.md` разделы «Zed (Apache-2.0 части: sum_tree)» и «Lapce (Apache-2.0) / Floem editor-core (MIT)» с перечнем перенесённых файлов и ревизий.



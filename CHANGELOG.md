# OmnesAgent — Changelog (fork)

Fork-level changelog for OmnesAgent. Upstream release notes are tracked
separately in [`backend/CHANGELOG-next.md`](./backend/CHANGELOG-next.md).

## ob2h v1.9.9 integration — Memory Lifecycle Pack (contract 56)

Source plan: `PLAN_OB2H_INTEGRATION_V1.9.9_2026-11-14.md` (phases Ф7–Ф13).
Consumes the ob2h Memory Lifecycle Pack. The fork only *consumes* the ob2h MCP
contract; no ob2h-side code is changed here.

### Contract and orchestration (Ф7)

- Expanded the read-only ob2h allowlist in
  `omnesagent-runtime::agent::orchestrator::tool_group` from 11 to **28**
  `ob2h__*` prefixes: the seven read-only lifecycle tools
  (`lesson_recall`, `lesson_list`, `slot_list`, `slot_get`, `memory_timeline`,
  `memory_verify`, `memory_audit`) plus the ten read-only names deferred from the
  Ф0.6 follow-up (`project_scan_status`, `project_call_path`,
  `project_graph_search`, `ast_diff`, `ast_history`, `dream_log`, `dream_status`,
  `ralph_context`, `ralph_report`, `session_log`).
- Classified the new write/privileged lifecycle tools as
  EngineeringTask/SystemAdmin only — never CodeExploration: `lesson_save`,
  `lesson_delete`, `slot_create`, `slot_append`, `slot_replace`, `slot_delete`,
  `session_recap` (write) and `viewer_open` (**privileged**, spawns a GUI
  process). `secrets_get` stays write-class (the Ф4.1 decision is not
  revisited).
- Added unit tests `test_ob2h_v199_lifecycle_classification` and expanded the
  read-only / mutating arrays in
  `test_ob2h_read_only_uses_real_double_underscore_prefix`.

### Ralph experience loop via lessons (Ф8, ADR-O8)

- `Ob2hBridge::publish_lessons` publishes verified findings as `lesson_save`
  (content = "what to do / what to avoid", context = task + affected AST
  symbols, tags = `["ralph", <feature_slug>]`), behind the new opt-in
  `[ob2h_bridge].push_lessons` (default `false`) and the credential-guard.
- `loop_driver` recalls lessons before each iteration
  (`lesson_recall`, query = task text + AST snapshot labels, limit 5,
  min_confidence 0.2) and injects a `<lessons>` block into the context package;
  fail-open with a single log per run.
- `BridgeReport` extended with `lessons_published` / `lessons_skipped` /
  `lessons_failed`.

### Slots as cross-session project buffer (Ф9, ADR-O7)

- Slots are consumed through MCP only; the fork does **not** create a parallel
  lessons/slots table in `omnesagent-memory` (no split-brain). Rules and skill
  document the `<agent_slots>` injection, the `OB2H_SLOTS_MAX_CHARS=1500` budget,
  the 413 → `slot_replace` compaction path, and the no-secrets-in-slots rule.

### Provenance, governance, timeline (Ф10, ADR-O9)

- New opt-in `[ob2h_bridge].recap_on_archive` (default `false`): on run
  archival the bridge ingests the iteration journal
  (`session_ingest`, `session_id = ralph-<run_id>`, `author = omnesagent-ralph`,
  `source = ralph`, credential-guarded) then calls `session_recap(save=true)` —
  the only contract path that yields facts with `meta.source_ids` for
  `memory_verify`.
- Governance rule: `memory_forget` is called **only with `reason`**;
  `memory_audit` is a read-only operation report. `memory_timeline` answers
  "what happened around X".

### Redaction parity and bench metrics (Ф11, ADR-O11)

- `omnesagent-memory::redact` patterns brought to parity with ob2h Ф69.1:
  `sk-…` (OpenAI-style), `ghp_/gho_/ghs_…` (GitHub), `Bearer …`, URL userinfo
  (`://user:pass@`), and `OB2H_[A-Z0-9_]+=…` assignments; capture-template
  replacements keep the scheme/name context. The category model and
  `[memory.policy]` config are preserved.
- `omnesagent-kag::bench` gains `recall_all_at_k` / `any_hit_at_k`
  (об2h Ф69.2 / LongMemEval strict), surfaced as the observational
  `BenchSummary.recall_all5` / `any_hit5`. `GateVerdict` thresholds
  (`RECALL_DROP_LIMIT` / `MRR_DROP_LIMIT`) are **not** changed.

### Viewer (Ф12, ADR-O10)

- `viewer_open` is classified privileged; rules/skill document the detached
  window, the `ob2h viewer` CLI fallback, the honest headless/SSH refusal, and
  that PINs never travel through chat/MCP arguments and secret values are never
  echoed back.

### Documentation and version closure (Ф13)

- `docs/adr/0001-ob2h-interop.md`: added ADR-O7…ADR-O11.
- `docs/kag_sync.md`: added the v1.9.9 parity matrix and the lifecycle-semantics
  transfer regulation.
- `.agents/rules/ob2h.md` and `.agents/skills/ob2h/SKILL.md`: lessons, slots,
  provenance/governance, and viewer sections.
- This changelog.

### Configuration additions (`[ob2h_bridge]`, additive, default `false`)

```toml
[ob2h_bridge]
push_lessons = false        # Ф8.2: verified findings → lesson_save
recap_on_archive = false    # Ф10.1: session_ingest journal + session_recap(save=true)
```

Old TOML remains valid; both fields are `#[serde(default)]` and opt-in (ADR-O3).

### Verification status

- Compile/test verification (`cargo check` / `cargo test`) and the live contract
  smoke (56 tools), golden regeneration, and the Ф9.3/Ф12 live checks are
  **pending** — they require a working terminal and the released ob2h v1.9.9
  binary with the `gui` feature, and were not run in this pass.

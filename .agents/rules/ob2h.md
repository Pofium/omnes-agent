# ob2h MCP Mandatory Usage Rule

<!-- Applied to all AI Agents in OmnesAgent monorepo -->

All AI coding assistants and subagents MUST always use the `ob2h` MCP server tools as the default first-line tools:

1. **Exploration & Context**:
   - Before executing terminal grep/find/cat, call `ob2h:project_scan` or `ob2h:project_context`.
   - Use AST-indexed code knowledge without consuming massive token budgets.

2. **Long-Term Memory**:
   - Query user context and project history via `ob2h:memory_search` and `ob2h:memory_context`.
   - Persist user rules, preferences, and key architectural choices with `ob2h:memory_save`,
     always using `project_id = "omnes-agent"` and an explicit `key`.
   - Before every `memory_save`, run `memory_search` for the same key/meaning first
     (dedup); use `memory_update` on a hit instead of creating a duplicate.
   - Never publish secrets, tokens, PINs, or `#[credential_class]`-classified content,
     and never auto-sync the built-in memory into ob2h (ADR-O3).

3. **Knowledge Graph**:
   - Resolve entity relationships, dependencies, and imports via `ob2h:graph_search` and `ob2h:graph_reason`.
   - Calculate blast radius before refactorings using `ob2h:project_impact`.

4. **Vault & Secrets (ADR-O5)**:
   - The ob2h vault (`secrets_*`) is the *inter-agent* secret store: LLM-provider
     tokens shared across agents. Fork config secrets (`#[credential_class]` fields
     such as `McpServerConfig.env`) stay in the fork and are never copied into ob2h.
   - PIN entry and vault unlock happen **only** through the ob2h GUI/CLI. A PIN,
     passphrase, or any chat-pasted secret is **never** passed as an MCP argument and
     is never written to `memory_save` (ADR-18).
   - Call `secrets_get` **only** to read a value the agent legitimately needs at
     runtime (e.g. a provider token). Never call it to echo back a value the user
     typed into the chat.
   - `secrets_get/set/delete/lock` are EngineeringTask/SystemAdmin-only; only the
     read-class `secrets_status` is exposed in CodeExploration (see
     `filter_tools_for_intent`).
   - Vault values must never be persisted into memory, RAG chunks, the knowledge
     graph, or tool logs.

5. **Lessons vs facts (ADR-O7/O8)**:
   - `lesson_save` stores *procedural* experience ("what to do / what to avoid /
     when it applies"); `memory_save` stores facts and decisions. Do not mix them.
   - Call `lesson_recall` before a non-trivial decision (query = task text +
     affected symbols); the Ralph loop does this automatically before each
     iteration.
   - Re-saving the same lesson text **reinforces** it (ob2h fingerprint dedup) —
     never spawn paraphrases. Lessons live only in ob2h; `omnesagent-memory` must
     not grow a parallel lessons table.
   - Never write secrets or `#[credential_class]` content into lessons. Do not
     auto-delete dream decay candidates (ADR-11 ob2h).

6. **Slots are the cross-session project buffer (ADR-O7)**:
   - `slot_list`/`slot_get` are read-class; `slot_create`/`slot_append`/
     `slot_replace`/`slot_delete` are write-class (EngineeringTask/SystemAdmin).
   - On `413` from `slot_append`, compact via `slot_replace` (read → shrink →
     replace). Never copy slot contents into fork memory, or vice versa
     (split-brain). Never write secrets into slots.

7. **Provenance, governance, viewer (ADR-O9/O10)**:
   - Verify a critical fact with `memory_verify` before relying on it; report
     citations to the owner; `provenance: none` is an honest result, not an error.
   - Call `memory_forget` **only with `reason`**; use `memory_audit` for read-only
     operation reports. `memory_timeline` answers "what happened around X".
   - `viewer_open` is **privileged** (spawns a GUI process): EngineeringTask/
     SystemAdmin only, never CodeExploration. PINs are never requested in chat or
     passed as MCP arguments; secret values are never echoed back into chat.

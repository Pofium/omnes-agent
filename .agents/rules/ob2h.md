# ob2h MCP Mandatory Usage Rule

<!-- Applied to all AI Agents in OmnesAgent monorepo -->

All AI coding assistants and subagents MUST always use the `ob2h` MCP server tools as the default first-line tools:

1. **Exploration & Context**:
   - Before executing terminal grep/find/cat, call `ob2h:project_scan` or `ob2h:project_context`.
   - Use AST-indexed code knowledge without consuming massive token budgets.

2. **Long-Term Memory**:
   - Query user context and project history via `ob2h:memory_search` and `ob2h:memory_context`.
   - Persist user rules, preferences, and key architectural choices with `ob2h:memory_save`.

3. **Knowledge Graph**:
   - Resolve entity relationships, dependencies, and imports via `ob2h:graph_search` and `ob2h:graph_reason`.
   - Calculate blast radius before refactorings using `ob2h:project_impact`.

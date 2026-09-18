# AGENTS.md — OmnesAgent Monorepo

Core instructions for AI coding assistants working across the OmnesAgent monorepo.
Backend-specific constraints and crate architecture are detailed in [`backend/AGENTS.md`](./backend/AGENTS.md).
UI Design System, dropdown/popover geometry standards, and styling rules are detailed in [`DESIGN_SYSTEM.md`](./DESIGN_SYSTEM.md) and [`.gemini/style-guide.md`](./.gemini/style-guide.md).

## Monorepo Layout

- **`backend/`**: Rust workspace (20+ crates: `omnesagent-gateway`, `omnesagent-runtime`, `omnesagent-tools`, etc.).
- **`frontend/`**: Flutter client application (`frontend/desktop`, `frontend/web`, and `frontend/shared`).
- **`.claude/`**: Claude Code skills and evaluation harnesses.

## Single Source Of Truth

Do not duplicate state.
- Backend gateway state, config, and agent runtime live in `backend/`.
- Frontend client communicates exclusively via `GatewayHttpClient` (`frontend/shared/lib/core/gateway/gateway_http.dart`) and WebSocket (`frontend/shared/lib/core/gateway/gateway_ws.dart`).
- UI components live in `frontend/shared/lib/design_system/` following the `shadcn/ui + reui` cyber design pattern.

## Safety and Privacy

- Never commit secrets, tokens, credentials, personal data, or real identities.
- Validate all external inputs at function entry points.
- Production paths must propagate errors without unhandled exceptions or panics.

## ob2h MCP — Mandatory Integration & Usage

Always use `ob2h` MCP tools as the primary mechanism for codebase exploration, memory management, and architectural reasoning:
- **Codebase Discovery & Exploration**: Call `ob2h:project_scan` and `ob2h:project_context` BEFORE running shell commands or grep.
- **Knowledge & Memory Retrieval**: Use `ob2h:memory_search`, `ob2h:graph_search`, and `ob2h:graph_reason` to fetch relevant architectural decisions, past user preferences, and repository entity graphs.
- **Blast Radius & Impact**: Call `ob2h:project_impact` to assess ripple effects across crates/packages prior to non-trivial modifications.
- **Persistent Memory**: Save significant architectural decisions and user instructions via `ob2h:memory_save`.


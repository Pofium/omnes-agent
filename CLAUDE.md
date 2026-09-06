# CLAUDE.md — OmnesAgent Monorepo

> **Core instructions live in [`AGENTS.md`](./AGENTS.md) and [`backend/AGENTS.md`](./backend/AGENTS.md).**
> **UI style guidelines live in [`.gemini/style-guide.md`](./.gemini/style-guide.md).**

## Monorepo Architecture

- **`backend/`**: Rust 2024 / ZeroClaw gateway & runtime. HTTP/WS gateway, multi-agent engine, tools, memory, cron, SOP graphs.
- **`frontend/`**: Flutter (Dart ≥3.4) unified client for mobile (Android APK, iOS) and desktop (Windows Desktop `.exe`). Design system: `shadcn/ui + reui` with cyber aesthetic (`#09090B` background, `#00E5FF` electric cyan accent).
- **`.claude/skills/`**: Shared Claude Code workflow skills (architecture check, issue triage, PR review, etc.).

## Commands & Orchestration

### Unified Scripts (from repo root)
```powershell
.\build.ps1 -Target All -Mode Release   # Builds Rust backend and Flutter APK
.\run.ps1                               # Starts backend gateway and launches client
.\check.ps1                             # Runs full health & lint checks
```

### Backend (Rust)
```bash
cd backend
cargo check -p omnesagent-gateway
cargo test
cargo clippy --all-targets -- -D warnings
```

### Frontend (Flutter / Dart)
```bash
cd frontend
flutter analyze
flutter build apk --debug               # Android APK
flutter build windows                   # Windows desktop (.exe)
```

## Working Rules
1. Never hardcode secrets, tokens, or credentials.
2. In Dart, keep all reusable UI components in `frontend/lib/design_system/` conforming to `shadcn/ui`.
3. In Rust, production code must propagate errors with `Result` and `?` (no bare `unwrap()` or `expect()`).

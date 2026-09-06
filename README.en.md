<div align="center">

<img src="assets/banner.jpg" alt="Omnes Agent Banner" width="100%" style="border-radius: 12px;" />

# Omnes Agent

**Next-Generation Autonomous Personal AI Agent Ecosystem**  
*Ultra-fast Rust Backend · Cyberpunk Flutter Client · Deterministic KAG & AST Code Memory*

[![Rust 2024](https://img.shields.io/badge/Rust-2024_Edition-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Flutter 3.x](https://img.shields.io/badge/Flutter-3.x-02569B.svg?logo=flutter)](https://flutter.dev/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS%20%7C%20Android%20%7C%20iOS-brightgreen.svg)]()

[**English Version**](README.en.md) | [**Русская версия**](README.md)

</div>

---

## 🌟 Overview

**Omnes Agent** is a full-featured execution and orchestration environment for autonomous personal AI assistants:
1. **Pure Rust Core & Gateway**: Reactive async daemon (`omnesagent-gateway`), modular tool execution runtime (`omnesagent-tools`), supporting multi-vendor LLM backends (OpenAI, Anthropic, Gemini, Ollama, DeepSeek, OpenRouter) and cross-channel event processing.
2. **KAG + AST Intelligence**: Deterministic static analysis for 10 programming languages with zero LLM token cost, **Blast Radius** calculation (`project_impact`), local in-process **Candle BERT MiniLM** vector embeddings (CPU), and hybrid **Reciprocal Rank Fusion (RRF $k=60$)** search.
3. **Cross-Platform Flutter Client**: Mobile and desktop application designed with a sleek **Cyber Zinc & Neon Cyan** design system (`shadcn/ui + reui`), featuring live log inspection, Gateway Doctor self-diagnostics, and skill bundle management.
4. **Memory Consolidation & Dreaming**: Background memory synthesis, user persona extraction, and workspace state checkpointing.

---

## 🏛️ Monorepo Architecture

```
Omnes-agent/
├── assets/                  # Brand graphics, emblem, and UI media
├── backend/                 # Rust Workspace (20+ crates)
│   ├── apps/
│   │   ├── omnescode/       # Standalone Terminal TUI coding assistant
│   │   └── omnesrelay/      # End-to-end encrypted P2P WebSocket relay
│   ├── crates/
│   │   ├── omnesagent-kag/     # KAG engine, AST parsers (10 languages), Candle MiniLM, PageRank
│   │   ├── omnesagent-memory/  # Long-term SQLite memory, hybrid RRF search, auto-migrations
│   │   ├── omnesagent-gateway/ # Control plane REST API & WebSocket server
│   │   ├── omnesagent-runtime/ # Agent loop, context injection, scheduler, worker pool
│   │   ├── omnesagent-tools/   # 90+ tools (AST code intelligence, dreaming, browser, shell, git)
│   │   ├── omnesagent-channels/# Telegram, Discord, Slack, WhatsApp, Webhooks, Matrix, AMQP
│   │   ├── omnesagent-providers/# LLM provider adapters (OpenAI, Anthropic, Gemini, DeepSeek, etc.)
│   │   └── omnesagent-config/  # Strongly-typed configuration schemas & security policies
│   └── xtask/               # Developer automation & code generation tasks
├── frontend/                # Cross-platform Flutter client
│   ├── lib/
│   │   ├── core/gateway/    # HTTP REST & WebSocket gateway clients
│   │   ├── design_system/   # Cyber components: shadcn_card, button, badge, dialog, input
│   │   └── features/        # Tools, Skills, Doctor, Logs, Integrations, Device Pairing
├── build.ps1 / build.bat    # Unified multi-target build script (Rust + Flutter)
├── run.ps1 / run.bat        # Unified launch script for gateway daemon & app
└── check.ps1 / check.bat    # Full-stack health check, tests & linting
```

---

## 🧠 KAG & AST Code Memory Architecture

Unlike naive RAG pipelines, Omnes Agent utilizes **Knowledge Augmented Generation**:

```mermaid
flowchart TD
    CodeFiles[Codebase Files] --> ASTScanner[Deterministic AST Scanner]
    ASTScanner -->|10 Languages: Rust, Py, TS, Go, Dart, PHP, Java, SQL, C/C++| GraphNodes[Graph Nodes: functions, structs, classes]
    ASTScanner --> GraphEdges[Graph Edges: calls, imports, inherits, implements]
    
    GraphNodes & GraphEdges --> SQLiteDB[(brain.db)]
    
    SQLiteDB --> PageRank[PageRank & Community Detection]
    PageRank --> GodNodes[Identify Architectural Hubs: God Nodes]
    
    SQLiteDB --> ImpactAnalysis[Blast Radius / project_impact]
    ImpactAnalysis --> RiskEval{Risk Level Evaluation}
    RiskEval -->|Low / Med / High| AgentContext[Prompt Injection <project_context>]
    
    Query[User Query] --> CandleEmbedder[Local Candle MiniLM 384d]
    Query --> FTS5[SQLite FTS5 BM25]
    CandleEmbedder & FTS5 --> RRF[Reciprocal Rank Fusion k=60]
    RRF --> HybridRecall[Hybrid Memory Recall]
```

- **Deterministic AST Parsing**: Structural symbol extraction with zero token overhead and millisecond execution.
- **Blast Radius Analysis (`project_impact`)**: Reverse BFS dependency traversal evaluating refactoring risks before modifying code.
- **Architectural Analytics**: PageRank centrality, Tarjan's strongly connected components (SCC), and module instability metrics (Ca/Ce).
- **In-Process Embeddings**: Pure Rust neural inference powered by `candle-core` running offline on CPU.

---

## ⚡ Quick Start

### Prerequisites
- **Rust**: 1.85+ (Rust 2024 edition)
- **Flutter**: 3.24+
- **OS**: Windows 10/11, Linux, macOS

### 1. Build Everything
```powershell
# Build both backend (omnesagent.exe) and frontend (Desktop / Android APK):
.\build.ps1

# Or via batch file:
build.bat

# Build only the Rust backend:
.\build.ps1 -Target Backend -Mode Release
```

### 2. Run Services
```powershell
# Launch both Gateway and Flutter Client:
.\run.ps1

# Run only the headless Gateway daemon:
.\run.ps1 -Service Backend

# Launch the terminal coding TUI (omnescode):
cargo run --bin omnescode
```

### 3. Verify & Run Tests
```powershell
# Run full-stack health check:
.\check.ps1

# Run KAG and AST parser tests:
cargo test -p omnesagent-kag --lib

# Run local Candle embedding provider tests:
cargo test -p omnesagent-memory --lib factory_candle
```

---

## 🛠️ Built-in Tool Arsenal

Omnes Agent comes out-of-the-box with 90+ tools:
- **`project_code`**: `project_init`, `project_scan`, `project_impact`, `project_context`, `project_graph_search`, `project_report`.
- **`dream_tool`**: `dream_run` (memory consolidation), `dream_status`, `dream_restore`.
- **Developer Tools**: `file_edit`, `file_write`, `file_download`, `git_operations`, `shell_exec`.
- **Research & Web**: `web_search`, `web_fetch`, `content_search`, `browser_open`.
- **Omnichannel**: Inbound/outbound adapters for Telegram, Discord, Slack, Pushover, Email IMAP/SMTP.

---

## 📜 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

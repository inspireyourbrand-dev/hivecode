<p align="center">
  <img src="assets/hivecode-banner.png" alt="HiveCode" width="100%" />
</p>

<h1 align="center">HiveCode</h1>

<p align="center">
  <strong>The model-agnostic AI coding assistant.</strong><br/>
  Use any LLM — Claude, GPT, Gemini, or local models via Ollama — in a blazing-fast native desktop app.
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#architecture">Architecture</a> •
  <a href="#providers">Providers</a> •
  <a href="#building">Building</a> •
  <a href="#license">License</a>
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri_v2-24C8D8?style=for-the-badge&logo=tauri&logoColor=white" />
  <img alt="React" src="https://img.shields.io/badge/React_19-61DAFB?style=for-the-badge&logo=react&logoColor=black" />
  <img alt="TypeScript" src="https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white" />
  <img alt="License" src="https://img.shields.io/badge/License-MIT-F59E0B?style=for-the-badge" />
</p>

---

## Why HiveCode?

| | HiveCode | Claude Code | Cursor | Aider |
|---|---|---|---|---|
| **Model Agnostic** | Any LLM | Claude only | Limited | Yes |
| **Local LLMs** | First-class | No | No | Yes |
| **Desktop UI** | Native (Tauri) | Terminal | Electron | Terminal |
| **Binary Size** | ~8 MB | ~80 MB | ~400 MB | ~50 MB |
| **RAM Usage** | ~50 MB | ~200 MB | ~500 MB | ~100 MB |
| **MCP Support** | Full | Full | Partial | No |
| **Standalone** | Yes | Yes | Yes | Yes |

HiveCode sits at the intersection of **model freedom**, **local LLM support**, **native desktop performance**, and **full MCP extensibility** — in a package 50x smaller than Cursor.

---

## Features

**Model Agnostic** — Switch between Anthropic Claude, OpenAI GPT, Google Gemini, or any local model running on Ollama, LM Studio, vLLM, or llama.cpp. One unified interface, any brain.

**Blazing Fast** — Rust core with zero garbage collection. Sub-100ms startup, 30-50 MB RAM at idle, and ripgrep-speed file search built in.

**Native Desktop** — Tauri v2 uses your OS native webview instead of bundling Chromium. You get a real desktop app in a 3-10 MB download.

**Full Tool Suite** — 15+ tools: file read/write/edit, bash execution, glob, grep, git operations, LSP integration, web fetch, Jupyter notebook editing, and multi-agent spawning.

**MCP Protocol** — Connect any Model Context Protocol server for extensible tool access. GitHub, databases, APIs — if there's an MCP server, HiveCode can use it.

**Security First** — Permission engine with 90+ command pattern classifications, path validation, sensitive file detection, and configurable allow/deny rules.

**Multi-Provider Auth** — API keys, OAuth 2.0 with PKCE for OpenAI/Anthropic platforms, and ChatGPT subscription session tokens. Multiple auth profiles per provider.

**Cloud + Local** — AWS Bedrock, Google Vertex AI, Anthropic Foundry, plus Ollama, LM Studio, vLLM, and any OpenAI-compatible endpoint.

**Session History** — Persistent conversation history with search, export (JSON/Markdown), auto-titling, and token tracking.

**Persistent Memory** — Remember user preferences, project context, code patterns, and corrections across sessions.

**Context Management** — Token tracking with cost calculation, context window monitoring, and conversation compaction to free up space.

**Multi-Agent System** — Spawn sub-agents for parallel task execution: code review, security review, exploration, verification, and custom agents.

**Plan Mode** — Create and execute structured plans with step dependencies, token estimation, and file tracking.

**Plugin System** — Extend HiveCode with tools, providers, themes, skills, and MCP plugins. Install from URLs with dependency management.

**Desktop Notifications** — Native OS notifications for long-running tasks, errors, and important events.

**Keyboard Shortcuts** — Full keyboard shortcut system with command palette (Ctrl+K), customizable bindings, and visual hints.

**Cost Tracking** — Real-time cost monitoring with per-model pricing, spending limits, and threshold alerts.

**Image + PDF** — Read and process images for vision-capable models. Extract text from PDFs with page range selection.

**Auto Updates** — Built-in update checker with stable, beta, and nightly channels via GitHub releases.

**CLI Mode** — Full command-line interface with interactive REPL, streaming output, and 7 subcommands (chat, init, config, auth, plugins, doctor, update).

**Beautiful UI** — TRON-style neon design with dark/light themes, streaming markdown with syntax highlighting, collapsible tool panels, model selector, and settings management.

---

## Quick Start

### One-Command Install (Windows)

Open PowerShell and run:

```powershell
cd "$env:USERPROFILE\Documents\Claude\Projects\HiveCode\hivecode"
powershell -ExecutionPolicy Bypass -File scripts\install-hivecode.ps1
```

This installs Rust (if needed), Node.js (if needed), Tauri CLI, frontend dependencies, builds the app, and offers to launch it.

### Manual Setup

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install Node.js 18+ from https://nodejs.org

# 3. Install Tauri CLI
cargo install tauri-cli --version "^2.0"

# 4. Install frontend dependencies
cd ui && npm install && cd ..

# 5. Run in development mode (hot reload)
cargo tauri dev

# 6. Build for production (generates installer)
cargo tauri build
```

### Configuration

Copy the example config and add your API keys:

```bash
# Windows
copy config.example.toml %USERPROFILE%\.hivecode\config.toml

# macOS/Linux
cp config.example.toml ~/.hivecode/config.toml
```

Edit `~/.hivecode/config.toml`:

```toml
[providers.anthropic]
api_key = "sk-ant-..."

[providers.openai]
api_key = "sk-..."

[providers.ollama]
base_url = "http://localhost:11434"
default_model = "llama3.3:70b"
```

---

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    HIVECODE DESKTOP APP                   │
├──────────────────┬──────────────────┬────────────────────┤
│   UI FRONTEND    │   TAURI BRIDGE   │     RUST CORE      │
│  (React + TS)    │  (IPC Commands)  │     (Engine)       │
├──────────────────┼──────────────────┼────────────────────┤
│ Chat Interface   │ invoke()         │ LLM Providers      │
│ Code Viewer      │ events()         │ Tool Engine        │
│ File Explorer    │ State Sync       │ Shell Manager      │
│ Terminal Panel   │ Stream Relay     │ MCP Client         │
│ Settings UI      │                  │ Permission System  │
│ Model Selector   │                  │ Git Integration    │
└──────────────────┴──────────────────┴────────────────────┘
```

### Crate Structure

| Crate | Purpose |
|---|---|
| `hivecode-core` | State, config, conversation, auth, history, memory, context, agents, plans, plugins, updater, image/PDF processing |
| `hivecode-providers` | LlmProvider trait + OpenAI, Anthropic, Ollama, AWS Bedrock, Google Vertex, Foundry |
| `hivecode-tools` | Tool trait + 15 tools: bash, file ops, glob, grep, git, LSP, notebooks, agents, config, diff |
| `hivecode-security` | Permission engine, shell security, path validation, 90+ command patterns |
| `hivecode-mcp` | MCP JSON-RPC client with stdio transport, tool/resource discovery |
| `hivecode-tauri` | 27 Tauri IPC commands, events, query engine, state management |
| `hivecode-cli` | CLI with 7 subcommands, interactive REPL, streaming output |
| **Frontend** | React 19 + TypeScript + Tailwind CSS + Zustand, 15 components |
| **Total** | **22,000+ lines across 7 crates + frontend** |

---

## Providers

HiveCode supports any LLM through a unified provider interface:

### Cloud Providers
- **Anthropic** — Claude Opus 4, Sonnet 4, Haiku (Messages API with streaming)
- **OpenAI** — GPT-4o, o1, o3 (Chat Completions API)
- **AWS Bedrock** — Claude models via AWS credentials and SigV4 signing
- **Google Vertex AI** — Claude models via Google Cloud project credentials
- **Anthropic Foundry** — Custom Anthropic endpoint for enterprise deployments

### Local Providers
- **Ollama** — Any model: Llama 3.3, CodeLlama, DeepSeek, Mistral, Qwen
- **LM Studio** — GGUF models with OpenAI-compatible API
- **vLLM** — High-throughput inference for self-hosted GPUs
- **llama.cpp server** — Direct HTTP API
- **Any OpenAI-compatible endpoint** — Generic provider

### Authentication
- **API Keys** — Direct API key authentication for all providers
- **OAuth 2.0 (PKCE)** — Secure OAuth flow for OpenAI Platform and Anthropic Console
- **ChatGPT Session** — Use your ChatGPT Plus/Team subscription via session token

The key insight: most local tools expose OpenAI-compatible APIs. A single provider implementation covers the entire local ecosystem.

---

## Building

### Development

```bash
cargo tauri dev          # Launch with hot reload
```

### Production Build

```bash
cargo tauri build        # Generates installer in target/release/bundle/
```

Tauri automatically generates:
- **Windows**: NSIS `.exe` installer + `.msi`
- **macOS**: `.dmg` disk image
- **Linux**: `.AppImage` + `.deb`

### Build Installer Manually

```powershell
# Windows — full build pipeline with progress output
powershell -ExecutionPolicy Bypass -File scripts\build-installer.ps1
```

---

## Project Structure

```
hivecode/
├── Cargo.toml                    # Workspace root
├── config.example.toml           # Example configuration
├── LICENSE
├── crates/
│   ├── hivecode-core/            # State, config, auth, history, memory, agents, plans, plugins
│   ├── hivecode-providers/       # LLM providers: OpenAI, Anthropic, Ollama, Bedrock, Vertex, Foundry
│   ├── hivecode-tools/           # 15 tools: bash, file ops, glob, grep, git, LSP, notebooks
│   ├── hivecode-security/        # Permission engine, shell security, path validation
│   ├── hivecode-mcp/             # MCP JSON-RPC client
│   ├── hivecode-tauri/           # Tauri desktop app: 27 IPC commands, events, query engine
│   └── hivecode-cli/             # CLI mode: REPL, 7 subcommands, terminal rendering
├── ui/                           # React 19 + TypeScript frontend
│   ├── src/
│   │   ├── components/           # 15 components: Chat, Auth, Memory, Notifications, etc.
│   │   ├── stores/               # Zustand: chatStore, appStore, notificationStore
│   │   ├── hooks/                # useAutoScroll, useTheme
│   │   └── lib/                  # Tauri IPC bindings, types
│   ├── package.json
│   └── vite.config.ts
├── assets/                       # Icons, banners, installer assets
├── installers/                   # Platform-specific installer configs
│   ├── windows/                  # Inno Setup + PowerShell build script
│   ├── macos/                    # .app bundle + DMG + notarization
│   └── linux/                    # .deb/.AppImage + desktop entry
├── scripts/                      # Setup, build, installer scripts
└── tests/                        # Integration and E2E tests
```

---

## License

MIT License. See [LICENSE](LICENSE) for details.

---

<p align="center">
  Built by <a href="https://hivepowered.ai">HivePowered</a>
</p>

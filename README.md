# Semaphore

Floating traffic light for AI coding agents. Know at a glance when your agent is idle, thinking, or writing files.

Green = ready for a new task  
Yellow = thinking  
Red = writing / editing files

## One app for end users

Install Semaphore, open it once, connect your tools from Settings — no terminal required.

1. Download a release for your OS
2. Launch Semaphore (stays in the system tray)
3. Open Settings → **Connect tools** (Cursor, Claude Code, …)
4. Use your AI tools normally

Hooks call `semctl` in the background. You never run it manually.

## Supported tools (v0.1)

| Tool | Status |
|------|--------|
| Cursor | Supported via `~/.cursor/hooks.json` |
| Claude Code | Supported via `~/.claude/settings.json` |
| Codex CLI | Planned (limited file-edit hooks today) |
| Gemini CLI | Planned |
| Copilot CLI | Planned |

Semaphore works with any tool that exposes lifecycle hooks. Adapters are pluggable.

## Development

Requirements: Rust, Node.js 20+, npm.

```bash
npm install
npm run tauri dev
```

Build CLI tools:

```bash
cargo build -p semctl --release
cargo build -p sem-core
```

Install hooks from the terminal (optional):

```bash
cargo run -p semctl -- install --all
cargo run -p semctl -- doctor
```

## Architecture

```
AI tool hooks → sem-hook → semctl → Unix socket / named pipe → Semaphore app
```

- **sem-core** — state machine, session aggregation, IPC
- **semctl** — CLI for hooks and installer
- **semaphore** (Tauri) — floating UI, tray, settings

## Stealth mode

Hides the window from many screen-capture tools using OS content protection. Works best on Windows. On macOS 15+ some capture tools may still record the window.

## Themes & i18n

Built-in themes: Classic, Minimal. English (default) and Portuguese included. See `locales/CONTRIBUTING-i18n.md` to add languages.

## License

MIT — see [LICENSE](LICENSE).

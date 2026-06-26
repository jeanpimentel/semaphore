# Claude Code adapter

Config: `~/.claude/settings.json` → `hooks`

| Event | Light |
|-------|-------|
| `UserPromptSubmit` | yellow |
| `PreToolUse` (Write\|Edit\|Bash) | red |
| `PostToolUse` | yellow |
| `Stop` | green (blinking) | Turn finished — waiting for your reply |
| `SessionEnd` | green | Session closed |

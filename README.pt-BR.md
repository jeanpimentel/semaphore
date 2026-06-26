# Semaphore

Semáforo flutuante para agentes de IA. Veja de relance quando o agente está ocioso, pensando ou escrevendo arquivos.

Verde = pronto para nova tarefa  
Amarelo = pensando  
Vermelho = escrevendo / editando arquivos

## Um app só

Instale o Semaphore, abra uma vez e conecte suas ferramentas nas Configurações — sem terminal.

1. Baixe o release para seu sistema
2. Abra o Semaphore (fica no tray)
3. Configurações → **Conectar ferramentas** (Cursor, Claude Code, …)
4. Use suas ferramentas de IA normalmente

## Ferramentas suportadas (v0.1)

| Ferramenta | Status |
|------------|--------|
| Cursor | Suportado |
| Claude Code | Suportado |
| Codex CLI | Planejado |
| Gemini CLI | Planejado |
| Copilot CLI | Planejado |

## Desenvolvimento

```bash
npm install
npm run tauri dev
```

## Licença

MIT — veja [LICENSE](LICENSE).

Documentação completa em inglês: [README.md](README.md).

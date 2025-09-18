## Providers Commands Mapping — Reference

### Gemini CLI (examples)
- One-shot: `gemini chat --model <model> --input "<message>" --project <project> --agent <agent>`
- REPL: `gemini chat --model <model> --repl --project <project> --agent <agent>`

### Claude Code (examples)
- One-shot: `claude code --model <model> --message "<message>" --session new`
- REPL: `claude code --model <model> --repl --session new`

### Cursor Agent CLI (examples)
- One-shot: `cursor-agent send --model <model> --output-format stream-json --message "<message>"`
- REPL: `cursor-agent repl --model <model> --output-format stream-json`

Notes:
- Les flags exacts peuvent varier selon versions/packaging; valider via `doctor`.
- Pour Cursor, le format `stream-json` est requis pour le parsing propre des deltas.


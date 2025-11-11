## Getting started

Looking for something specific? Jump ahead:

- [Tips & shortcuts](#tips--shortcuts) – hotkeys, resume flow, prompts
- [Non-interactive runs](./exec.md) – automate with `adom exec`
- Ready for deeper customization? Head to [`advanced.md`](./advanced.md)

### CLI usage

| Command            | Purpose                            | Example                         |
| ------------------ | ---------------------------------- | ------------------------------- |
| `adom`            | Interactive TUI                    | `adom`                         |
| `adom "..."`      | Initial prompt for interactive TUI | `adom "fix lint errors"`       |
| `adom exec "..."` | Non-interactive "automation mode"  | `adom exec "explain utils.ts"` |

Key flags: `--model/-m`, `--ask-for-approval/-a`.

### Resuming interactive sessions

- Run `adom resume` to display the session picker UI
- Resume most recent: `adom resume --last`
- Resume by id: `adom resume <SESSION_ID>` (You can get session ids from /status or `~/.adom/sessions/`)

Examples:

```shell
# Open a picker of recent sessions
adom resume

# Resume the most recent session
adom resume --last

# Resume a specific session by id
adom resume 7f9f9a2e-1b3c-4c7a-9b0e-123456789abc
```

### Running with a prompt as input

You can also run Adom CLI with a prompt as input:

```shell
adom "explain this codebase to me"
```

### Example prompts

Below are a few bite-size examples you can copy-paste. Replace the text in quotes with your own task.

| ✨  | What you type                                                                   | What happens                                                               |
| --- | ------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| 1   | `adom "Refactor the Dashboard component to React Hooks"`                       | Adom rewrites the class component, runs `npm test`, and shows the diff.   |
| 2   | `adom "Generate SQL migrations for adding a users table"`                      | Infers your ORM, creates migration files, and runs them in a sandboxed DB. |
| 3   | `adom "Write unit tests for utils/date.ts"`                                    | Generates tests, executes them, and iterates until they pass.              |
| 4   | `adom "Bulk-rename *.jpeg -> *.jpg with git mv"`                               | Safely renames files and updates imports/usages.                           |
| 5   | `adom "Explain what this regex does: ^(?=.*[A-Z]).{8,}$"`                      | Outputs a step-by-step human explanation.                                  |
| 6   | `adom "Carefully review this repo, and propose 3 high impact well-scoped PRs"` | Suggests impactful PRs in the current codebase.                            |
| 7   | `adom "Look for vulnerabilities and create a security review report"`          | Finds and explains security bugs.                                          |

Looking to reuse your own instructions? Create slash commands with [custom prompts](./prompts.md).

### Memory with AGENTS.md

You can give Adom extra instructions and guidance using `AGENTS.md` files. Adom looks for them in the following places, and merges them top-down:

1. `~/.adom/AGENTS.md` - personal global guidance
2. Every directory from the repository root down to your current working directory (inclusive). In each directory, Adom first looks for `AGENTS.override.md` and uses it if present; otherwise it falls back to `AGENTS.md`. Use the override form when you want to replace inherited instructions for that directory.

For more information on how to use AGENTS.md, see the [official AGENTS.md documentation](https://agents.md/).

### Tips & shortcuts

#### Use `@` for file search

Typing `@` triggers a fuzzy-filename search over the workspace root. Use up/down to select among the results and Tab or Enter to replace the `@` with the selected path. You can use Esc to cancel the search.

#### Esc–Esc to edit a previous message

When the chat composer is empty, press Esc to prime “backtrack” mode. Press Esc again to open a transcript preview highlighting the last user message; press Esc repeatedly to step to older user messages. Press Enter to confirm and Adom will fork the conversation from that point, trim the visible transcript accordingly, and pre‑fill the composer with the selected user message so you can edit and resubmit it.

In the transcript preview, the footer shows an `Esc edit prev` hint while editing is active.

#### `--cd`/`-C` flag

Sometimes it is not convenient to `cd` to the directory you want Adom to use as the "working root" before running Adom. Fortunately, `adom` supports a `--cd` option so you can specify whatever folder you want. You can confirm that Adom is honoring `--cd` by double-checking the **workdir** it reports in the TUI at the start of a new session.

#### `--add-dir` flag

Need to work across multiple projects in one run? Pass `--add-dir` one or more times to expose extra directories as writable roots for the current session while keeping the main working directory unchanged. For example:

```shell
adom --cd apps/frontend --add-dir ../backend --add-dir ../shared
```

Adom can then inspect and edit files in each listed directory without leaving the primary workspace.

#### Shell completions

Generate shell completion scripts via:

```shell
adom completion bash
adom completion zsh
adom completion fish
```

#### Image input

Paste images directly into the composer (Ctrl+V / Cmd+V) to attach them to your prompt. You can also attach files via the CLI using `-i/--image` (comma‑separated):

```bash
adom -i screenshot.png "Explain this error"
adom --image img1.png,img2.jpg "Summarize these diagrams"
```

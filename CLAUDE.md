# CLAUDE.md — starsight

Architecture, build commands, conventions, current feature surface: see [AGENTS.md](AGENTS.md).
Path-scoped detail (Rust idioms, mark conventions, snapshot tests): `.claude/rules/`.
Slash workflows (`/check`, `/snap`, `/release-prep`, `/quickfix`, `/scout`): `.claude/skills/`.
Subagents (`@snapshot-reviewer`, `@layer-boundary-check`): `.claude/agents/`.
Issue tracker, session-start context, persistent memory: handled by the `bd prime` SessionStart hook — do not duplicate.

## Response style

- Terse output, code first. Skip preamble and trailing summaries unless asked.
- No emoji unless the user requests them.
- Default to no code comments; only add a one-liner when the *why* is non-obvious.
- Match response shape to the task: a small fix gets a one-line confirmation, not headers.

## Session completion contract

A session is **not complete until `git push` succeeds**. The full required sequence:

```bash
git pull --rebase
bd dolt push        # only if a Dolt remote is configured (currently embedded-only)
git push
git status          # must show "up to date with origin"
```

If push fails, resolve and retry — never stop early. Never say "ready to push when you are"; do the push.

If beads has no Dolt remote (current state per the persistent memory), skip the `bd dolt push` step rather than failing on it.

## Hard rules

- Use `bd` for all task tracking. Never use TodoWrite, TaskCreate, or markdown TODO lists.
- Use `bd remember` for persistent knowledge. Never create MEMORY.md files.
- Never amend a commit unless the user explicitly asks. Always create a new commit.
- Never use `--no-verify` to bypass hooks. Fix the underlying issue.
- Library code never `unwrap()` or `expect()` on user-supplied input. Return `Result<T, StarsightError>`.

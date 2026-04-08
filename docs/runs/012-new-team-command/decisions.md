# Decisions — 012-new-team-command

## Task 1: src/new_team.rs

- Exposing `validate_name`, `resolve_target_root`, and `scaffold_team` as `pub(crate)` to make them unit-testable without going through stdin.
- `scaffold_team(name, root)` returns `(team_path, agent_path)` on success. Callers check collision and existing files before calling it.
- Collision check calls `config::discover_teams()` — this requires the builtin teams dir to exist. The integration test (Task 2) will exercise this end-to-end; unit tests stub it by testing the pre-condition logic separately.
- `resolve_target_root` accepts `workflow_dir` and `custom_dir` as plain strings/Options to avoid touching the filesystem in unit tests.

## Task 2: main.rs dispatch

- Integration test uses a tempdir for user-level scaffold to avoid touching `~/.claude-launch/user/`.
- `run_new_team` is a thin wrapper that calls `new_team::run()`, mirroring `run_scheduled`.

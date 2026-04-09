# Decisions Log — 011 Custom Teams

## Task 1: TeamEntry + discover_teams() rewrite

- Replacing existing `discover_teams(teams_dir: &Path) -> Result<Vec<String>>` with new 3-source signature.
- Existing tests for old `discover_teams` in `config/tests.rs` are being replaced with tests for the new API.
- `TeamSource` and `TeamEntry` are added to `src/config.rs` per spec.
- Collision detection covers all three pairs: built-in vs user, built-in vs project, user vs project.
- Missing user dir: silently skipped (user may not have defined custom teams yet).
- Missing project dir when configured (`Some` path that doesn't exist): returns error immediately.

## Task 2: Config custom_dir field

- Adding `custom_dir: Option<String>` to `Config` with `#[serde(default)]`.
- No changes to `Config::default()` — field is `None` by default.

## Task 3: render_prompt() + user dir creation

- `render_prompt()` gains `user_dir: &str` and `project_dir: &str` params.
- All existing call sites updated with empty-string placeholders until Task 4 wires real values.
- `resolve_workflow_dir()` creates `~/.claude-launch/user/teams/` and `~/.claude-launch/user/agents/` via `create_dir_all`.
- `run_install()` in `install.rs` does the same two dirs explicitly.

## Task 4: main.rs wiring

- Both `run()` and `run_scheduled()` updated — easy to miss the scheduled path.
- Template loading switches from path-construction to `TeamEntry.path` directly.
- TUI call site uses `Vec<String>` extracted from `Vec<TeamEntry>`.

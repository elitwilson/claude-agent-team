---
number: 012
status: complete
base_branch: main
---

# Feature: `new-team` Subcommand

## Summary

`claude-launch new-team` is an interactive CLI subcommand that scaffolds the files needed to define a custom team. It prompts for a team name and level (user or project), creates a no-op team entry point and a single placeholder agent file, and prints the paths the user needs to edit. It does not launch the TUI.

---

## Requirements

- `claude-launch new-team` starts an interactive prompt sequence asking for team name and level
- Optionally accepts the team name as a positional argument (`claude-launch new-team my-team`) to skip that prompt
- Level prompt accepts `user` or `project`, defaults to `user` if the user presses Enter without input
- Scaffolded team entry point (`teams/<name>.md`) contains a no-op prompt that immediately exits with a message telling the user to replace it with their own prompt engineering
- Scaffolded agent file (`agents/<name>/agent.md`) contains the same no-op message
- After scaffolding, prints the paths of all created files so the user knows what to edit
- If the team name already exists in any source (built-in, user, or project), fails with a clear error before creating any files
- If `--project` level is requested but `custom_dir` is not configured in `.claude-launch.toml`, fails with a clear error explaining what to add to the config
- If `project` level is selected and `custom_dir` is configured but the directory does not exist on disk, creates it (including `teams/` and `agents/<name>/` subdirs)
- Team name must contain only lowercase letters, digits, and hyphens; any other input fails with a clear error
- If either output file already exists on disk, fails with a clear error before creating any files (no partial writes)

---

## Scope

### In Scope

- `new-team` subcommand parsing in `main.rs`
- New module `src/new_team.rs` owning all scaffolding logic
- Interactive stdin prompts (plain readline — no TUI)
- Scaffolding for user-level and project-level teams
- Collision detection reusing `config::discover_teams()`
- Name validation
- No-op scaffold templates (hardcoded strings, not `.md` files on disk)

### Out of Scope

- Scaffolding multiple agents (always one placeholder named `agent.md`)
- Editing or deleting existing custom teams
- Non-interactive / `--yes` flag mode
- TUI integration

---

## Technical Approach

### Subcommand dispatch in `main.rs`

Add a `new-team` branch to the arg dispatch block, before the TUI path:

```rust
if args.get(1).map(|s| s.as_str()) == Some("new-team") {
    if let Err(e) = run_new_team(&args[2..]) {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
    return;
}
```

`run_new_team` accepts the remaining args slice so a positional name can be parsed from it.

### New module: `src/new_team.rs`

One public function:

```rust
pub fn run(args: &[String]) -> Result<()>
```

**Prompt sequence:**

1. If `args` is non-empty and the first element doesn't start with `-`, treat it as the team name (skip name prompt). Otherwise prompt: `Team name: `
2. Validate name: `^[a-z0-9-]+$`. Fail immediately if invalid.
3. Prompt: `Level (user/project) [user]: `. Empty input → `user`.
4. Resolve target root:
   - `user`: `~/.claude-launch/user/`
   - `project`: load `Config` from CWD, read `custom_dir`. If absent, bail with: `"custom_dir is not set in .claude-launch.toml — add it before creating a project-level team"`
5. Collision check: call `config::discover_teams()` with the same dirs used by `run()`. If the name appears in any source, bail: `"A team named '<name>' already exists"`.
6. Check output paths don't already exist. If either does, bail before writing anything.
7. Create directories and write scaffold files.
8. Print created paths.

**Scaffold content (hardcoded strings):**

Team entry point (`teams/<name>.md`):
```
This is a scaffolded team prompt that has not been configured.
Replace this file with your own prompt engineering before running.
Exiting.
```

Agent file (`agents/<name>/agent.md`):
```
This is a scaffolded agent definition that has not been configured.
Replace this file with your own prompt engineering before running.
Exiting.
```

**Output after success:**

```
Created:
  <path>/teams/<name>.md
  <path>/agents/<name>/agent.md

Edit these files to define your team, then run claude-launch to use it.
```

### Stdin prompts

Use `std::io::stdin().read_line()` directly — no external crate needed.

---

## Success Criteria

- [ ] `claude-launch new-team` prompts for name and level and scaffolds files at the correct location
- [ ] `claude-launch new-team my-team` skips the name prompt
- [ ] Empty level input defaults to `user`
- [ ] Scaffolded team file contains the no-op message
- [ ] Scaffolded agent file exists at `agents/<name>/agent.md`
- [ ] Collision with a built-in team name produces a clear error and creates no files
- [ ] Running the command twice with the same name produces a clear error on the second run
- [ ] `project` level with no `custom_dir` configured produces a clear error
- [ ] Invalid team name (e.g. `My Team`) produces a clear error
- [ ] Printed output lists all created file paths

---

## Tasks

- [ ] **`src/new_team.rs` — prompts, validation, collision check, scaffolding:** Implement `run(args: &[String])`. Prompt for name and level, validate name format, resolve target root, run collision check via `discover_teams()`, check for existing files, create dirs and write scaffold content, print results. Unit-test name validation, path resolution for both levels, and the no-`custom_dir` error case. All tests must pass before proceeding.

- [ ] **`main.rs` dispatch:** Add `new-team` to the arg dispatch block, call `new_team::run()`, wire the module. Add an integration test that calls `run_new_team(&["my-team".to_string()])` against a temp directory and asserts both scaffold files exist with the correct content.

---

## Considerations

- Collision detection calls `discover_teams()` which requires the built-in teams dir to exist (it reads from `~/.claude-launch/prompts/teams/`). This dir is guaranteed to exist after `resolve_workflow_dir()` runs — call that first in `run_new_team` before collision checking.
- For project-level, if `custom_dir` is configured but the directory doesn't exist yet, create it. This is intentional — the user may have just added the config key without manually creating the directory.
- Name validation should reject names that would collide with built-in teams on principle (e.g. `feature-dev`), but the collision check via `discover_teams()` already handles this. Name format validation (`^[a-z0-9-]+$`) is a separate, earlier check.
- The `run_new_team` wrapper in `main.rs` should mirror the pattern of `run_scheduled` — thin wrapper that calls into the module and handles the error.

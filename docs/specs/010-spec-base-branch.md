---
number: 010
status: ready
base_branch: main
---

# Feature: Required `base_branch` Frontmatter and Blocked Spec Popups

## Summary

Specs must declare a `base_branch` in their YAML frontmatter. Without it, the tool has no authoritative source for which branch to base the feature branch from — a gap that caused a real incident where a feature branch was created from `main` when it should have been based on a long-running `modernization` branch. This feature makes `base_branch` a required field, removes the config-level fallback, surfaces blocked specs with a human-readable reason popup in the TUI, and adds a spec content hash check for scheduled runs to prevent executing a run against a spec that changed after it was scheduled.

---

## Requirements

### Frontmatter

- Every spec file must declare `base_branch` in its YAML frontmatter (e.g., `base_branch: main`)
- A spec with proper frontmatter (i.e., has opening `---` delimiters) but missing `base_branch` is treated as `Blocked` with the reason: `"Missing required frontmatter field: base_branch"`
- `base_branch` is read directly from the spec file at launch time (interactive and scheduled); the `Config` struct no longer provides a `base_branch` field or default

### Blocked popup

- Pressing Enter on any `Blocked` spec opens a `BlockedReasonDialog` popup instead of silently doing nothing
- The popup displays the spec name and the block reason as a human-readable message
- Esc dismisses the popup and returns focus to the spec list
- This applies to all blocked specs — whether blocked due to missing `base_branch` or a user-declared `status: blocked`

### Config cleanup

- `Config` struct: remove `base_branch` field, `default_base_branch()` function, and the `Default` impl entry for it
- The `.claude-launch.toml` config file format no longer supports `base_branch`; unknown keys are ignored (no breaking parse error if an old config file still has the field — serde will silently ignore it via `#[serde(deny_unknown_fields)]` being absent)
- All call sites in `main.rs` that pass `config.base_branch` to `run_preflight` must instead read `base_branch` from the spec file's frontmatter

### Scheduled run integrity (spec hash)

- When scheduling a run, compute a SHA-256 hash of the spec file's raw bytes at schedule time
- The hash is passed as `--spec-hash <hex>` in the launchd plist `ProgramArguments`
- `RunArgs` gains an optional `spec_hash: Option<String>` field parsed from `--spec-hash`
- `ScheduledRun` gains a `spec_hash: Option<String>` field
- `generate_plist_xml` accepts `spec_hash: Option<&str>` and appends `--spec-hash <hash>` to the program args when present
- At execution time (the `run` subcommand path in `main.rs`), if `--spec-hash` was provided: re-hash the spec file and compare. If they differ, exit immediately with a clear error: `"Spec '<name>' has changed since it was scheduled (hash mismatch). Re-schedule to run the updated spec."`
- If no `--spec-hash` is provided (manual `claude-bros run` invocation), skip the check

### Existing specs

- All existing spec files in `docs/specs/` that lack `base_branch` must have it added
- All existing specs that will continue to run should use `base_branch: main` unless known otherwise
- `000-test-do-not-run-me.md` should be given `status: blocked` and a note explaining it is not a real spec

---

## Scope

### In Scope

- `SpecEntry` gains `block_reason: Option<String>`
- `parse_frontmatter_status` extended to also extract `base_branch` and return a block reason when missing
- New `parse_spec_frontmatter` function returns both `SpecStatus` and `Option<String>` block reason (and separately, `base_branch` for use at launch time)
- `PopupAction::BlockedReasonDialog { reason: String }` added to the popup enum
- `open_team_popup` routes `Blocked` specs to `BlockedReasonDialog`
- `dismiss_popup` handles `BlockedReasonDialog` (dismiss → `None`, same as `CancelDialog`)
- TUI renders `BlockedReasonDialog` as a centered popup with spec name and reason text
- `Config` struct and tests updated to remove `base_branch`
- `main.rs`: both run paths read `base_branch` from spec frontmatter; error and exit if missing (should not occur if TUI validation worked, but the run path must be defensive)
- `scheduler.rs`: `generate_plist_xml` and `schedule_run` accept and embed `spec_hash`
- `run_cmd.rs`: `RunArgs` and parser support `--spec-hash`
- Unit tests for: block reason parsing, `BlockedReasonDialog` popup routing, hash mismatch abort, `generate_plist_xml` with hash

### Out of Scope

- Adding `base_branch` validation to the Requirements tab (Raw specs have no frontmatter by definition)
- Changing `show_blocked` pref behavior — blocked specs still respect the user's visibility toggle
- Any UI for editing spec frontmatter within the TUI
- Hashing anything other than raw file bytes (no partial/frontmatter-only hashing)

---

## Technical Approach

### `SpecEntry` and frontmatter parsing

Add `block_reason: Option<String>` to `SpecEntry`. Add a `parse_spec_frontmatter` function in `config.rs` that returns a struct or tuple:

```rust
pub struct SpecFrontmatter {
    pub status: SpecStatus,
    pub block_reason: Option<String>,
    pub base_branch: Option<String>,
}

pub fn parse_spec_frontmatter(content: &str) -> SpecFrontmatter
```

Rules:
- No frontmatter → `status: Raw`, everything else `None`
- Has frontmatter, has `status: blocked` or `needs_attention` → `status: Blocked`, `block_reason: Some("Spec is marked blocked — requires human review before running.")`
- Has frontmatter, missing `base_branch` → `status: Blocked`, `block_reason: Some("Missing required frontmatter field: base_branch")`
- Has frontmatter, has valid `status`, has `base_branch` → normal status, no block reason

`discover_specs` uses `parse_spec_frontmatter` and populates `block_reason` on each `SpecEntry`.

`base_branch` is not stored on `SpecEntry` — it is read fresh from the spec file immediately before calling `run_preflight`.

### Popup routing

```rust
pub fn open_team_popup(&mut self) {
    // ...
    if selected.status == SpecStatus::Blocked {
        let reason = selected.block_reason.clone()
            .unwrap_or_else(|| "This spec cannot be run.".to_string());
        self.popup = Some(PopupAction::BlockedReasonDialog { reason });
        return;
    }
    // existing logic
}
```

`dismiss_popup` match arm: `Some(PopupAction::BlockedReasonDialog { .. }) => None`.

### TUI widget

`BlockedReasonDialog` is a new widget in `widgets.rs`:

```rust
pub struct BlockedReasonDialog<'a> {
    pub spec_name: &'a str,
    pub reason: &'a str,
}
```

Renders as a centered popup (similar in style to `CancelDialog`). Content:

```
┌─ Blocked ─────────────────────────┐
│ 003-metrics-query.md              │
│                                   │
│ Missing required frontmatter      │
│ field: base_branch                │
│                                   │
│           [Esc] Dismiss           │
└───────────────────────────────────┘
```

### Spec hash

SHA-256 using the `sha2` crate (already a common transitive dep; add explicitly if not present). Hash is hex-encoded as a lowercase string.

```rust
pub fn hash_spec_file(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path)?;
    let hash = Sha256::digest(&bytes);
    Ok(format!("{hash:x}"))
}
```

`generate_plist_xml` signature gains `spec_hash: Option<&str>`. When `Some`, appends to `program_args`:
```
<string>--spec-hash</string>
<string>{hash}</string>
```

`schedule_run` computes the hash from the spec file path and passes it through.

In `main.rs` at the scheduled-run execution path: after parsing `RunArgs`, if `spec_hash` is `Some`, hash the spec file and compare. Mismatch → print error and exit non-zero before preflight runs.

### Reading `base_branch` at launch time

Extract a standalone helper:

```rust
pub fn read_base_branch(spec_path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(spec_path)?;
    let fm = parse_spec_frontmatter(&content);
    fm.base_branch.ok_or_else(|| anyhow::anyhow!(
        "Spec '{}' is missing required frontmatter field: base_branch",
        spec_path.file_name().unwrap_or_default().to_string_lossy()
    ))
}
```

Both `main.rs` run paths call `read_base_branch` and pass the result to `run_preflight` instead of `config.base_branch`.

---

## Success Criteria

- [ ] A spec with valid frontmatter and `base_branch` set launches normally
- [ ] A spec missing `base_branch` shows `Blocked` in the TUI and opens a `BlockedReasonDialog` on Enter with the text "Missing required frontmatter field: base_branch"
- [ ] A spec with `status: blocked` opens a `BlockedReasonDialog` on Enter with an appropriate human-readable reason
- [ ] Esc on any `BlockedReasonDialog` dismisses it and returns to the spec list
- [ ] `Config` no longer has a `base_branch` field; existing `.claude-launch.toml` files with the field parse without error
- [ ] `main.rs` reads `base_branch` from spec frontmatter, not config
- [ ] If `base_branch` is missing at the run-command path, the process exits with a clear error before preflight
- [ ] Scheduled runs include `--spec-hash` in the plist
- [ ] A scheduled run where the spec file has changed since scheduling fails immediately with a hash-mismatch error
- [ ] A manual `claude-bros run` invocation without `--spec-hash` skips the hash check
- [ ] All existing specs in `docs/specs/` have `base_branch` set
- [ ] All unit tests pass

---

## Tasks

- [ ] **Frontmatter parsing:** Replace `parse_frontmatter_status` with `parse_spec_frontmatter` returning `SpecFrontmatter` (status, block_reason, base_branch). Update `discover_specs`. Add `block_reason` field to `SpecEntry`. Update all call sites and tests.

- [ ] **Config cleanup:** Remove `base_branch` from `Config`, `default_base_branch()`, `Default` impl, and `config/tests.rs`. Update `main.rs` to call `read_base_branch` instead.

- [ ] **Blocked popup — app state:** Add `PopupAction::BlockedReasonDialog { reason: String }`. Update `open_team_popup` to open it for `Blocked` specs. Update `dismiss_popup` to handle it. Update `app/tests.rs`.

- [ ] **Blocked popup — TUI widget:** Implement `BlockedReasonDialog` widget in `widgets.rs`. Wire into popup render match in `ui.rs`. Update `ui/tests.rs`.

- [ ] **Spec hash:** Add `sha2` dependency. Implement `hash_spec_file`. Update `generate_plist_xml`, `schedule_run`, `ScheduledRun`, `RunArgs`, and `parse_run_args` to carry the hash. Add hash check in `main.rs` run path. Update `scheduler/tests.rs` and `run_cmd/tests.rs`.

- [ ] **Existing specs:** Add `base_branch` to all spec files in `docs/specs/`. Mark `000-test-do-not-run-me.md` as blocked with explanation.

---

## Considerations

- **`parse_frontmatter_status` is a public function** — any external callers or tests using it directly will break. Replace entirely with `parse_spec_frontmatter`; do not keep the old function as a shim.
- **`BlockedReasonDialog` carries both fields** — the variant is `BlockedReasonDialog { spec_name: String, reason: String }`, consistent with how `CancelDialog` carries `spec_slug`. `open_team_popup` has `visible_specs()[self.spec_index].name` available to populate it. `ui.rs` unpacks both fields directly into the widget struct at render time — no need to reach back into `app` state.
- **Hash mismatch is a hard stop** — do not warn and continue. The scheduled run must fail cleanly so the user notices and re-schedules with the updated spec.
- **`sha2` crate**: check `Cargo.toml` before adding — it may already be a transitive dependency via another crate. Add it explicitly regardless for clarity.
- **`status: complete` specs and `base_branch`**: complete specs are non-runnable but may still lack `base_branch`. The simplest rule: treat missing `base_branch` as blocked regardless of declared status — a complete spec with missing `base_branch` still shows blocked in the TUI if `show_blocked` is on. This is consistent and avoids a special case.

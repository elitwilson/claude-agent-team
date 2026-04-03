# Decisions Log — 010-spec-base-branch

## Task breakdown

Split spec into 4 tasks (3-5 per solo-dev role):

1. **Frontmatter parsing + Config cleanup** — `parse_spec_frontmatter`, `block_reason` on `SpecEntry`, `read_base_branch`, remove `base_branch` from `Config`, update `main.rs` call sites
2. **Blocked popup** — `PopupAction::BlockedReasonDialog { spec_name, reason }`, `open_team_popup` routing, `dismiss_popup`, `BlockedReasonDialog` widget, event loop wiring
3. **Spec hash** — `sha2` dep, `hash_spec_file`, update `generate_plist_xml` / `schedule_run` / `ScheduledRun` / `RunArgs` / hash check in `main.rs`
4. **Existing specs** — Add `base_branch` to all `docs/specs/*.md`, mark `000-test-do-not-run-me.md` as blocked

## Decisions

- `sha2` is not already in Cargo.toml — adding it explicitly per spec guidance
- `parse_frontmatter_status` is removed entirely (spec says no shim); all call sites updated
- `BlockedReasonDialog` carries both `spec_name: &str` and `reason: &str` per spec considerations section
- `status: complete` specs with missing `base_branch` are still treated as Blocked (spec: consistent rule, no special case)

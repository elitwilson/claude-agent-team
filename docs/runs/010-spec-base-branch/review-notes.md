# Review Notes

## Task 1: Frontmatter parsing + Config cleanup

**Verdict: APPROVED (with one minor note)**

### Requirements checklist vs. test coverage

| Requirement | Test(s) | Status |
|---|---|---|
| `parse_spec_frontmatter` returns `SpecFrontmatter { status, block_reason, base_branch }` | `test_parse_spec_frontmatter_ready_with_base_branch` + others | Covered |
| No frontmatter → `Raw`, everything `None` | `test_parse_spec_frontmatter_no_frontmatter_is_raw` | Covered |
| `status: blocked` → `Blocked` + specific block_reason message | `test_parse_spec_frontmatter_explicit_blocked_status` | Covered |
| `status: needs_attention` → `Blocked` + block_reason | `test_parse_spec_frontmatter_needs_attention_is_blocked` | Partially covered (see note) |
| Missing `base_branch` → `Blocked` + specific message | `test_parse_spec_frontmatter_missing_base_branch_is_blocked` | Covered |
| Valid status + `base_branch` → normal status, no block_reason | `test_parse_spec_frontmatter_ready_with_base_branch`, `test_parse_spec_frontmatter_complete_with_base_branch` | Covered |
| `complete` + missing `base_branch` → `Blocked` | `test_parse_spec_frontmatter_complete_missing_base_branch_is_blocked` | Covered |
| `SpecEntry.block_reason: Option<String>` field exists | `discover_specs` tests accessing `specs[0].block_reason` | Covered (compile-time) |
| `discover_specs` populates `block_reason` | `test_discover_specs_sets_block_reason_for_missing_base_branch`, `test_discover_specs_no_block_reason_for_valid_spec`, `test_discover_specs_block_reason_for_explicit_blocked` | Covered |
| `read_base_branch` returns value from frontmatter | `test_read_base_branch_returns_value_from_frontmatter` | Covered |
| `read_base_branch` errors when `base_branch` missing | `test_read_base_branch_errors_when_missing` | Covered |
| `read_base_branch` errors on missing file | `test_read_base_branch_errors_on_missing_file` | Covered |
| Old config with `base_branch` field parses without error | `test_load_ignores_base_branch_key` | Covered |
| `parse_frontmatter_status` removed (no shim) | N/A — compile-time, not unit-testable | N/A |

### Note

`test_parse_spec_frontmatter_needs_attention_is_blocked` asserts only `block_reason.is_some()`. The spec groups `needs_attention` with `blocked` and specifies the same message: `"Spec is marked blocked — requires human review before running."` The test does not verify the exact string. This is not a blocking issue — the behavior is constrained by the `blocked` test which pins the message — but tightening the assertion would make the intent explicit. Not flagging as a gap; noting for awareness.

No spec requirements are missing coverage. Tests check observable behavior (return values, error conditions) rather than implementation internals.

---

## Task 2: Blocked popup (app state + TUI widget)

**Verdict: FLAGGED**

### Requirements checklist vs. test coverage

| Requirement | Test(s) | Status |
|---|---|---|
| `PopupAction::BlockedReasonDialog { spec_name, reason }` variant exists | All tests use the variant (compile-time) | Covered |
| Confirm on `Blocked` spec opens `BlockedReasonDialog` instead of silently doing nothing | `test_blocked_spec_confirm_opens_blocked_reason_dialog` | Covered |
| `BlockedReasonDialog` carries correct `spec_name` and `reason` | `test_blocked_reason_dialog_carries_spec_name_and_reason`, `test_blocked_reason_dialog_uses_fallback_reason_when_none` | Covered |
| `dismiss_popup` on `BlockedReasonDialog` returns popup to `None` | `test_dismiss_popup_on_blocked_reason_dialog_returns_none` | Covered |
| `popup_move_down`/`popup_move_up` handle `BlockedReasonDialog` without panicking | (none) | **MISSING** |
| TUI renders `BlockedReasonDialog` showing spec name and reason text | `test_render_blocked_reason_dialog_shows_spec_name_and_reason` | Covered |

### Gap

The task description explicitly requires: "`popup_move_down`/`popup_move_up` handle the new variant without panicking."

No test exercises either `popup_move_down` or `popup_move_up` when `app.popup` is `Some(PopupAction::BlockedReasonDialog { .. })`. If the match arms are exhaustive in the existing handlers, the compiler would catch an unhandled variant — but if the handlers use a wildcard or the new variant is simply silently routed incorrectly, this gap would go undetected. A test calling `app.popup_move_down()` and `app.popup_move_up()` with the popup set (asserting no panic and popup state unchanged) is needed.

### Action required

Add tests for `popup_move_down` and `popup_move_up` when `BlockedReasonDialog` is active before proceeding to implementation.

---

## Task 3: Spec hash

**Verdict: APPROVED (with one clarification note)**

### Requirements checklist vs. test coverage

| Requirement | Test(s) | Status |
|---|---|---|
| `hash_spec_file` returns 64-char lowercase hex string | `test_hash_spec_file_returns_hex_string` | Covered |
| `hash_spec_file` is deterministic | `test_hash_spec_file_is_deterministic` | Covered |
| `hash_spec_file` produces distinct hash when content changes | `test_hash_spec_file_differs_when_content_changes` | Covered |
| `hash_spec_file` errors on missing file | `test_hash_spec_file_errors_on_missing_file` | Covered |
| `generate_plist_xml` appends `--spec-hash <hash>` when `Some` | `test_plist_includes_spec_hash_when_provided` | Covered |
| `generate_plist_xml` omits `--spec-hash` when `None` | `test_plist_excludes_spec_hash_when_none` | Covered |
| `parse_plist` extracts `--spec-hash` into `ScheduledRun.spec_hash` | `test_parse_plist_extracts_spec_hash_when_present` | Covered |
| `parse_plist` returns `spec_hash: None` for legacy plist without the flag | `test_parse_plist_spec_hash_is_none_for_legacy_plist` | Covered |
| `parse_run_args` parses `--spec-hash <value>` into `RunArgs.spec_hash` | `test_parse_spec_hash_flag` | Covered |
| `--spec-hash` absent → `spec_hash: None` | `test_spec_hash_defaults_to_none` | Covered |
| `--spec-hash` with no value → parse error | `test_spec_hash_flag_missing_value_returns_error` | Covered |
| `main.rs` hash mismatch → exit with specific error message | None (by design) | See note |
| `main.rs` `spec_hash: None` → skip check | None (by design) | See note |

### Note

The scope list in the spec says "Unit tests for: hash mismatch abort" but the task description immediately contradicts this: "The `main.rs` hash-mismatch check terminates the process, which makes it hard to unit test. That behavior is verified by inspection (no unit test for the main.rs wiring)."

The task description takes precedence here — the `main.rs` wiring is a process-exit path that is not realistically unit-testable without refactoring the abort logic out of `main`. Accepting inspection-only coverage for that path is reasonable. No gap is being raised; this is noted for awareness so the developer knows the scope list's mention of "hash mismatch abort" tests refers to any future extraction of that logic, not the current `main.rs` direct exit.

All testable spec requirements have coverage. Tests check observable behavior (output format, XML content, parse results, error conditions) — no implementation detail testing observed.

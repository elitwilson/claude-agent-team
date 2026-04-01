# Decisions — 005 Scheduled Runs

## run_cmd module instead of inline main.rs tests

Created a separate `run_cmd` module for arg parsing rather than testing inline in `main.rs`. Binary crate roots can't easily use the `mod tests;` (separate file) pattern that the rest of the codebase uses. The `run_cmd` module keeps the pattern consistent.

## Manual XML parsing instead of plist crate

Used line-by-line XML string parsing for plist generation and parsing rather than adding the `plist` crate dependency. The XML structure is well-defined (we control generation), so manual parsing is sufficient. This avoids a new dependency without explicit approval.

## list_pending_in(dir) for testability

Added `list_pending_in(dir)` as a testable variant of `list_pending()`. The public `list_pending()` delegates to `list_pending_in(~/Library/LaunchAgents/)`. Tests use temp directories with fixture plists.

## cleanup_plist checks stderr for launchctl failures

`launchctl unload` returns exit code 0 even when it fails — it prints "Unload failed" to stderr instead. `cleanup_plist` captures stderr and checks for "Unload failed" to properly detect failures. This is critical because missed cleanup turns a one-shot run into a silently recurring annual job.

## generate_plist_xml separated from schedule_run

Per spec guidance, plist XML generation is a pure function separate from the `schedule_run` orchestrator. This keeps the shell-out surface (launchctl load) thin and lets the plist content be fully unit tested.

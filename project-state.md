# Project State

> Last updated: 2026-05-28

---

## Overview

`claude-launch` v1 is functionally complete. All core launcher features described in the vision are implemented: TUI spec browser, team picker, preflight, interactive/headless run modes, scheduled runs via launchd, metrics collection, multi-account support, and the `new-team` scaffold command. The project is now in v2 territory — the next significant work is the Auto-Plan tab (spec 013, ready to implement). The Drafter agent and related Raw Inputs / auto-chain features from the original v2 vision may be superseded by the auto-plan approach and are under review.

---

## What's Built

- **TUI spec browser** — spec list with status filtering (ready/blocked/complete), persistent prefs via `~/.claude/claude-agent-team-prefs.toml`; color-coded status, Options panel with headless/show-complete/show-blocked toggles
- **Team picker** — built-in teams (`feature-dev`, `solo-dev`, `solo-with-subagent-review`, `investigation`); custom user-level and project-level teams discoverable at runtime
- **`new-team` subcommand** — interactive CLI wizard for scaffolding user-level or project-level custom teams
- **Pre-flight git setup** — validates clean working tree, checks out base branch, creates feature branch; skips pull if no upstream
- **Interactive and headless run modes** — `runner.rs` spawns the `claude` CLI; headless flag passed through from TUI prefs
- **Scheduled runs via launchd** — `scheduler.rs` generates plists; cancel flow removes the plist and any pending launchd job; schedule picker TUI for time selection
- **Metrics collection** — token usage per run written to SQLite via `metrics/db.rs`; metrics query view in TUI
- **Multi-account support** — OAuth tokens stored/retrieved via macOS Keychain (`security` CLI); account picker dialog in TUI
- **Spec base_branch frontmatter** — required field enforced at discovery; blocked popup shown if missing
- **First-time install** (`install.rs`) — symlinks rules, registers hooks in `settings.json`
- **Spec frontmatter parsing** — `config.rs` parses status, base_branch, number; supports ready/blocked/complete/idea

---

## In Progress

- **Auto-Plan Tab** (spec 013, status: ready) — adds a Plan tab to TUI that triggers the `auto-plan` skill; also deploys `skills/` and `agents/` directories via `install.rs`; no implementation started yet

---

## Tech Debt & Known Issues

- `docs/specs/TODO-spec-dependencies.md` (status: blocked) — spec dependency tracking is a noted gap; no resolution path defined yet
- `docs/specs/TODO-vue-testing-rules.md` (status: idea) — stale/misfiled; likely from a different project and can be ignored

---

## Recent Changes

- Added `vision.md` and spec 013 (auto-plan tab) — 2026-05-28
- Completed `new-team` subcommand (spec 012) with dialoguer-based wizard and integration tests
- Removed tmux support spec (was specced then abandoned)
- Fixed regression: scheduled specs had `.md` stripped from path (introduced in `0a398d9`, fixed with test)
- Removed `git pull` from preflight; skip pull if no upstream configured

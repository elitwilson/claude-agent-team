# Vision: claude-launch

> A macOS TUI launcher that takes a spec and a team, handles all the ceremony, and gets out of the way so Claude Code agent runs happen reliably — interactively or unattended.

---

## Problem Statement

Running Claude Code agent teams requires repetitive, error-prone ceremony: clean git state, branch creation, token selection, prompt construction, headless/interactive decision, log capture, and metrics collection. None of that is interesting — it's scaffolding that gets between you and the actual work. Without a launcher, every agent run is a manual assembly job, which defeats the purpose of automation.

---

## Product Positioning

`claude-launch` is a personal dev tool for macOS developers who use Claude Code agent teams to implement features autonomously. It provides a spec-driven TUI that handles everything from pre-flight to metrics collection, including scheduled (overnight) runs via launchd. It's opinionated by design: every run is spec-driven, TDD is non-negotiable, and the built-in teams enforce both without requiring any per-project configuration.

---

## Users & Personas

**Solo developer using Claude Code for autonomous implementation** — knows how to write specs and agent teams, wants to queue work before going to bed and review the output in the morning. Values reliability over flexibility: would rather have a narrower tool that works every time than a general tool that requires setup.

---

## Core Features

- TUI spec browser: browse, select, and launch specs by status (ready / blocked / complete)
- Team picker: select built-in or custom agent teams per run
- Pre-flight: validates clean git state, checks out base branch, creates feature branch
- Interactive and headless run modes
- Scheduled runs via launchd (one-shot, fire-and-forget)
- Metrics collection: token usage per run to SQLite
- Multi-account support: store and switch between Claude accounts via macOS Keychain
- Custom team scaffolding: `new-team` command for user-level and project-level custom teams
- Drafter: takes a raw requirements file and produces a ready-or-blocked spec

---

## Explicit Non-Goals

- Linux or Windows support — relies on launchd and macOS Keychain by design
- Multi-project management — `claude-launch` runs from within a single project directory
- Web UI or remote access — this is a local terminal tool
- General-purpose task runner — only runs Claude Code agent sessions against specs
- Prompt engineering iteration surface — agent prompt quality is a separate concern from the launcher

---

## Product Constraints

- macOS only — launchd and Keychain are non-negotiable platform dependencies; the binary refuses to compile on other platforms
- Single binary, cargo-installed — no daemons, no background services beyond registered launchd jobs
- Scheduled runs always headless — no interactive session when running unattended
- Agents must not push to remote — push and MR creation is out of scope for the agent runs themselves

---

## Tech Stack

- Rust (edition 2024)
- ratatui + crossterm for TUI
- rusqlite (bundled) for metrics storage
- toml + serde for config and prefs
- chrono for scheduling
- macOS launchd for scheduled jobs
- macOS Keychain (`security` CLI) for credential storage
- Claude Code CLI as the execution engine

---

## Milestones & Phasing

### v1 — Core Launcher
- TUI spec browser with status filtering and persistent prefs
- Team picker (built-in: `feature-dev`, `solo-dev`, `solo-with-subagent-review`, `investigation`)
- Pre-flight git setup
- Interactive and headless run modes
- Scheduled runs via launchd with cancel support
- Metrics collection (token usage) to SQLite
- Multi-account support via Keychain
- Custom team scaffolding (`new-team` command, user-level and project-level)
- First-time install: symlink rules, register hooks in `settings.json`

### v2 — Drafter + Spec Panel Polish
- Drafter agent: raw requirements file → ready-or-blocked spec
- Raw Inputs tab in TUI for raw files without frontmatter
- Auto-chain to team run when Drafter produces a `ready` spec
- Spec panel as sortable table with created/completed date columns

---

## Success Criteria

- **v1:** User can select a spec, pick a team, launch it interactively or schedule it unattended, and review token metrics the next morning — with no manual git setup, no prompt construction, and no config beyond `.claude-launch.toml` optional overrides.
- **v2:** User can drop a raw requirements file into the specs directory, select it in the Raw Inputs tab, and get a structured spec (or a blocked spec with explicit reasons) without writing spec format manually.

---

#!/usr/bin/env python3
"""Installs the agent-team workflow into ~/.claude/.

Safe to re-run — symlinks are idempotent, settings.json and CLAUDE.md
entries are only added if not already present.
"""

import json
import sys
from pathlib import Path

WORKFLOW_DIR = Path(__file__).parent
CLAUDE_DIR = Path.home() / ".claude"


def link_rules(workflow_dir: Path, claude_dir: Path) -> None:
    rules_dir = claude_dir / "rules"
    rules_dir.mkdir(parents=True, exist_ok=True)
    link = rules_dir / "agent-workflow"

    if link.is_symlink():
        print("  rules symlink already exists, skipping")
        return
    if link.exists():
        print(f"  ERROR: {link} exists but is not a symlink. Remove it manually and re-run.")
        sys.exit(1)

    link.symlink_to(workflow_dir / "rules")
    print(f"  linked rules/ → {link}")


# Note: no CLAUDE.md import needed — role definitions are passed explicitly
# via spawn prompts at runtime, not loaded globally.


def register_hooks(workflow_dir: Path, claude_dir: Path) -> None:
    settings_path = claude_dir / "settings.json"

    if not settings_path.exists():
        settings_path.write_text("{}")

    settings = json.loads(settings_path.read_text())
    hooks = settings.setdefault("hooks", {})

    hook_scripts = {
        "TaskCompleted": workflow_dir / "hooks" / "task-completed.sh",
        "TaskCreated":   workflow_dir / "hooks" / "task-created.sh",
        "TeammateIdle":  workflow_dir / "hooks" / "teammate-idle.sh",
    }

    changed = False
    for event, script_path in hook_scripts.items():
        command = str(script_path)
        entries = hooks.setdefault(event, [])
        already_registered = any(
            h.get("command") == command
            for entry in entries
            for h in entry.get("hooks", [])
        )
        if not already_registered:
            entries.append({"hooks": [{"type": "command", "command": command}]})
            changed = True

    if changed:
        settings_path.write_text(json.dumps(settings, indent=2))
        print("  registered hooks in settings.json")
    else:
        print("  hooks already registered, skipping")


def main() -> None:
    print(f"Installing agent-team-workflow from: {WORKFLOW_DIR}")
    link_rules(WORKFLOW_DIR, CLAUDE_DIR)
    register_hooks(WORKFLOW_DIR, CLAUDE_DIR)
    print("\nInstall complete.")
    print("\nAdd scripts to PATH by adding this to your shell profile:")
    print(f'  export PATH="{WORKFLOW_DIR / "scripts"}:$PATH"')


if __name__ == "__main__":
    main()

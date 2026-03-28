#!/usr/bin/env python3
"""Orchestrates an agent team run against a feature spec.

Usage:
  run-agent-team.py <spec-file>           # unattended, logs to file
  run-agent-team.py <spec-file> --watch   # interactive, output in terminal
"""

import os
import subprocess
import sys
from datetime import date
from pathlib import Path
from string import Template

WORKFLOW_DIR = Path(__file__).parent.parent


def load_oauth_token(service: str = "claude-token-1", account: str = "claude") -> str | None:
    result = subprocess.run(
        ["security", "find-generic-password", "-w", "-s", service, "-a", account],
        capture_output=True, text=True
    )
    return result.stdout.strip() if result.returncode == 0 else None


def build_prompt(template_path: Path, variables: dict) -> str:
    return Template(template_path.read_text()).safe_substitute(variables)


def run_preflight(spec_file: Path, base_branch: str) -> str:
    subprocess.run(
        ["python3", str(WORKFLOW_DIR / "scripts" / "preflight.py"), str(spec_file), base_branch],
        check=True
    )
    return Path("/tmp/current-agent-branch").read_text().strip()


def run_agent(prompt: str, log_file: Path, max_turns: int = 200) -> int:
    """Run agent team non-interactively, logging output to file."""
    log_file.parent.mkdir(parents=True, exist_ok=True)
    env = {**os.environ, "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"}
    with log_file.open("a") as log:
        result = subprocess.run(
            [
                "claude", "--print",
                "--max-turns", str(max_turns),
                "--dangerously-skip-permissions",  # TODO: replace with pre-approved permissions before production use
                "--teammate-mode", "in-process",
                prompt,
            ],
            stdout=log, stderr=log, env=env
        )
    return result.returncode


def run_agent_watch(prompt: str, max_turns: int = 200) -> int:
    """Run agent team interactively so output is visible in the terminal.
    Skips logging — MR creation is also skipped since the user is present.
    """
    env = {**os.environ, "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"}
    result = subprocess.run(
        [
            "claude",
            "--max-turns", str(max_turns),
            "--dangerously-skip-permissions",
            "--teammate-mode", "in-process",
            prompt,
        ],
        env=env
    )
    return result.returncode


def create_mr(branch_name: str, spec_file: Path, base_branch: str, log_file: Path) -> None:
    feature_slug = spec_file.stem
    run_date = date.today().isoformat()

    description = "\n".join([
        "Automated implementation by Claude Code Agent Teams.",
        "",
        f"**Spec:** `{spec_file}`",
        f"**Branch:** `{branch_name}`",
        f"**Run date:** {run_date}",
        "",
        f"Review `docs/specs/{feature_slug}-decisions.md` for assumptions made during the run.",
        f"Review `docs/specs/{feature_slug}-review-notes.md` for Reviewer gate outcomes.",
        "",
        f"Log: `{log_file}`",
    ])

    subprocess.run([
        "git", "push", "origin", branch_name,
        "-o", "merge_request.create",
        "-o", f"merge_request.target={base_branch}",
        "-o", f"merge_request.title=feat: {feature_slug} (agent run {run_date})",
        "-o", f"merge_request.description={description}",
        "-o", "merge_request.remove_source_branch",
    ], check=True)


def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: run-agent-team.py <spec-file> [--watch]", file=sys.stderr)
        sys.exit(1)

    spec_file = Path(sys.argv[1])
    watch = "--watch" in sys.argv
    base_branch = os.environ.get("BASE_BRANCH", "main")
    feature_slug = spec_file.stem
    log_file = Path(f"logs/agent-runs/{feature_slug}-{date.today().strftime('%Y%m%d')}.log")

    token = load_oauth_token()
    if token:
        os.environ["CLAUDE_OAUTH_TOKEN"] = token
    else:
        print("WARNING: No OAuth token found in Keychain (claude-token-1). Using default auth.")

    try:
        branch_name = run_preflight(spec_file, base_branch)
    except subprocess.CalledProcessError:
        print("ERROR: Pre-flight failed.", file=sys.stderr)
        sys.exit(1)

    prompt = build_prompt(
        WORKFLOW_DIR / "prompts" / "orchestration.md",
        {
            "SPEC_FILE": str(spec_file),
            "FEATURE_SLUG": feature_slug,
            "WORKFLOW_DIR": str(WORKFLOW_DIR),
        }
    )

    if watch:
        print(f"Running in watch mode. Branch: {branch_name}")
        print("MR creation skipped — run manually when ready.\n")
        run_agent_watch(prompt)
    else:
        exit_code = run_agent(prompt, log_file)
        print(f"Agent run complete. Branch: {branch_name}, Log: {log_file}")
        if exit_code != 0:
            print(f"WARNING: Agent exited with code {exit_code}. Review log before creating MR.")
        create_mr(branch_name, spec_file, base_branch, log_file)
        print(f"MR created for branch: {branch_name}")


if __name__ == "__main__":
    main()

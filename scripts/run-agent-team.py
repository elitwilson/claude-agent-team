#!/usr/bin/env python3
"""Orchestrates an overnight agent team run against a feature spec."""

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


def run_agent(prompt: str, log_file: Path, headless: bool = False, max_turns: int = 200) -> int:
    log_file.parent.mkdir(parents=True, exist_ok=True)
    env = {**os.environ, "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS": "1"}
    cmd = [
        "claude",
        "--max-turns", str(max_turns),
        "--dangerously-skip-permissions",  # TODO: replace with pre-approved permissions before production use
        "--teammate-mode", "in-process",
        prompt,
    ]
    if headless:
        cmd.insert(1, "--print")
        with log_file.open("a") as log:
            result = subprocess.run(cmd, stdout=log, stderr=log, env=env)
    else:
        tee = subprocess.Popen(["tee", "-a", str(log_file)], stdin=subprocess.PIPE)
        result = subprocess.run(cmd, stdout=tee.stdin, env=env)
        tee.stdin.close()
        tee.wait()
    return result.returncode


def create_mr(branch_name: str, spec_file: Path, base_branch: str, log_file: Path, exit_code: int = 0) -> None:
    feature_slug = spec_file.stem
    spec_title = spec_file.read_text().splitlines()[0].lstrip("# ")
    run_date = date.today().isoformat()

    failed = exit_code != 0
    status_banner = f"\n> **WARNING: Agent exited with code {exit_code}. Run may be incomplete. Review log before merging.**\n" if failed else ""
    title_prefix = "INCOMPLETE: " if failed else ""

    description = "\n".join([
        f"Automated implementation by Claude Code Agent Teams.{status_banner}",
        "",
        f"**Spec:** `{spec_file}`",
        f"**Branch:** `{branch_name}`",
        f"**Run date:** {run_date}",
        f"**Agent exit code:** `{exit_code}`",
        "",
        f"Review `docs/specs/{feature_slug}/decisions.md` for assumptions made during the run.",
        f"Review `docs/specs/{feature_slug}/review-notes.md` for Reviewer gate outcomes.",
        "",
        f"Log: `{log_file}`",
    ])

    subprocess.run([
        "git", "push", "origin", branch_name,
        "-o", "merge_request.create",
        "-o", f"merge_request.target={base_branch}",
        "-o", f"merge_request.title={title_prefix}feat: {spec_title} (agent run {run_date})",
        "-o", f"merge_request.description={description}",
        "-o", "merge_request.remove_source_branch",
    ], check=True)


def main() -> None:
    import argparse
    parser = argparse.ArgumentParser(description="Run an agent team against a feature spec.")
    parser.add_argument("spec_file", type=Path, help="Path to the spec file")
    parser.add_argument("--team", default="feature-dev", help="Team type to use (default: feature-dev)")
    parser.add_argument("--headless", action="store_true", help="Run without interactive output, logging only (for cron/overnight use)")
    args = parser.parse_args()

    spec_file = args.spec_file
    team = args.team
    headless = args.headless
    base_branch = os.environ.get("BASE_BRANCH", "main")
    feature_slug = spec_file.stem
    log_file = Path(f"logs/agent-runs/{feature_slug}-{date.today().strftime('%Y%m%d')}.log")

    team_prompt = WORKFLOW_DIR / "prompts" / "teams" / f"{team}.md"
    if not team_prompt.exists():
        print(f"ERROR: No prompt template found for team '{team}' at {team_prompt}", file=sys.stderr)
        sys.exit(1)

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
        team_prompt,
        {
            "SPEC_FILE": str(spec_file),
            "FEATURE_SLUG": feature_slug,
            "WORKFLOW_DIR": str(WORKFLOW_DIR),
        }
    )

    exit_code = run_agent(prompt, log_file, headless=headless)
    print(f"Agent run complete. Branch: {branch_name}, Log: {log_file}")

    if exit_code != 0:
        print(f"WARNING: Agent exited with code {exit_code}. Run may be incomplete.")

    create_mr(branch_name, spec_file, base_branch, log_file, exit_code)
    print(f"MR created for branch: {branch_name}")


if __name__ == "__main__":
    main()

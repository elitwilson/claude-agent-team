#!/usr/bin/env python3
"""Pre-flight checks: validates environment and creates the feature branch."""

import subprocess
import sys
from datetime import date
from pathlib import Path


def check_spec_exists(spec_file: Path) -> None:
    if not spec_file.exists():
        raise FileNotFoundError(f"Spec file not found: {spec_file}")


def check_clean_working_tree() -> None:
    result = subprocess.run(
        ["git", "status", "--porcelain"],
        capture_output=True, text=True, check=True
    )
    if result.stdout.strip():
        raise RuntimeError("Working tree is dirty. Commit or stash changes first.")


def pull_latest(base_branch: str) -> None:
    subprocess.run(["git", "checkout", base_branch], check=True)
    subprocess.run(["git", "pull", "origin", base_branch], check=True)


def create_feature_branch(feature_slug: str) -> str:
    branch_name = f"feature/{feature_slug}-{date.today().strftime('%Y%m%d')}"
    subprocess.run(["git", "checkout", "-b", branch_name], check=True)
    return branch_name


def run_preflight(spec_file: Path, base_branch: str = "main") -> str:
    check_spec_exists(spec_file)
    check_clean_working_tree()
    pull_latest(base_branch)
    branch_name = create_feature_branch(spec_file.stem)
    print(f"Pre-flight passed. Branch: {branch_name}")
    return branch_name


def main() -> None:
    if len(sys.argv) < 2:
        print("Usage: preflight.py <spec-file> [base-branch]", file=sys.stderr)
        sys.exit(1)

    spec_file = Path(sys.argv[1])
    base_branch = sys.argv[2] if len(sys.argv) > 2 else "main"

    try:
        branch_name = run_preflight(spec_file, base_branch)
        Path("/tmp/current-agent-branch").write_text(branch_name)
    except (FileNotFoundError, RuntimeError) as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()

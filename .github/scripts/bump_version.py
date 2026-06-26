#!/usr/bin/env python3
"""Bump version based on conventional commits since the last release tag.

Outputs GitHub Actions outputs:
  - version: the new version (e.g. "0.4.0")
  - tag: the new tag (e.g. "v0.4.0")
  - released: "true" if a new release was created, "false" otherwise
"""
from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

try:
    import tomllib  # py3.11+
except ImportError:
    import tomli as tomllib  # type: ignore[no-redef]

ROOT = Path(__file__).resolve().parents[2]
PYPROJECT = ROOT / "pyproject.toml"

MAJOR = "feat!"
MINOR = "feat"
PATCH = ("fix", "perf", "refactor")


def run(cmd: list[str], **kw) -> str:
    return subprocess.check_output(cmd, cwd=ROOT, text=True, **kw)


def latest_tag() -> str | None:
    """Return the highest semver tag (without leading 'v') or None."""
    out = run(["git", "tag", "--list", "v*", "--sort=-v:refname"])
    tags = [t.strip() for t in out.splitlines() if t.strip()]
    return tags[0].lstrip("v") if tags else None


def commits_since(sha: str | None) -> list[str]:
    rng = sha if sha else "HEAD"
    out = run(["git", "log", f"{rng}..HEAD", "--pretty=%s"])
    return [line for line in out.splitlines() if line.strip()]


def next_version(curr: str | None, msgs: list[str]) -> str | None:
    if not msgs:
        return None
    if curr is None:
        # Bootstrap: start at 0.1.0.
        return "0.1.0"
    major, minor, patch = (int(x) for x in curr.split("."))
    bumped = False
    for m in msgs:
        if m.startswith(MAJOR):
            return f"{major + 1}.0.0"
        if m.startswith(MINOR):
            return f"{major}.{minor + 1}.0"
        if any(m.startswith(p + ":") or m.startswith(p + "(") for p in PATCH):
            bumped = True
    if bumped:
        return f"{major}.{minor}.{patch + 1}"
    return None


def write_outputs(d: dict[str, str]) -> None:
    out = os.environ.get("GITHUB_OUTPUT", "")
    if out:
        with open(out, "a") as f:
            for k, v in d.items():
                f.write(f"{k}={v}\n")


def main() -> int:
    curr = latest_tag()
    msgs = commits_since(curr)
    new = next_version(curr, msgs)
    if new is None:
        print(f"No release-worthy commits since v{curr or '(none)'}; skipping.")
        write_outputs({"released": "false"})
        return 0

    print(f"Current: v{curr}  ->  New: v{new}")
    for m in msgs:
        print(f"  · {m}")

    # Update pyproject.toml.
    with open(PYPROJECT, "rb") as f:
        data = tomllib.load(f)
    data["project"]["version"] = new
    text = PYPROJECT.read_text()
    text = re.sub(
        r'^version\s*=\s*"[^"]+"',
        f'version = "{new}"',
        text,
        count=1,
        flags=re.MULTILINE,
    )
    PYPROJECT.write_text(text)

    # Commit, tag, push.
    tag = f"v{new}"
    env = {**os.environ, "GIT_AUTHOR_NAME": "github-actions[bot]",
           "GIT_AUTHOR_EMAIL": "41898282+github-actions[bot]@users.noreply.github.com",
           "GIT_COMMITTER_NAME": "github-actions[bot]",
           "GIT_COMMITTER_EMAIL": "41898282+github-actions[bot]@users.noreply.github.com"}
    run(["git", "config", "user.name", "github-actions[bot]"])
    run(["git", "config", "user.email", "41898282+github-actions[bot]@users.noreply.github.com"])
    run(["git", "add", "pyproject.toml"])
    run(["git", "commit", "-m", f"chore(release): {tag}"])
    run(["git", "tag", tag])
    run(["git", "push", "origin", "main", tag])

    write_outputs({"version": new, "tag": tag, "released": "true"})
    return 0


if __name__ == "__main__":
    sys.exit(main())
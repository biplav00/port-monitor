#!/usr/bin/env python3
"""Create (or update) the GitHub Release for the tag set by bump_version.py.

Reads `tag` and `version` from $GITHUB_OUTPUT written by the previous step.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def run(cmd: list[str], **kw) -> str:
    return subprocess.check_output(cmd, cwd=ROOT, text=True, **kw)


def main() -> int:
    tag = os.environ.get("RELEASE_TAG", "")
    version = os.environ.get("RELEASE_VERSION", "")
    if not tag or not version:
        # Fall back to reading pyproject.toml
        import re
        m = re.search(r'^version\s*=\s*"([^"]+)"', (ROOT / "pyproject.toml").read_text(),
                      re.MULTILINE)
        if not m:
            print("Cannot determine version", file=sys.stderr)
            return 1
        version = m.group(1)
        tag = f"v{version}"

    log = run(["git", "log", "--pretty=%s", "HEAD~10..HEAD"]) if False else ""
    body = (
        f"## Port Monitor {tag}\n\n"
        f"See [README](README.md) for what changed.\n"
    )

    gh = os.environ.get("GH_TOKEN", "")
    if not gh:
        print("GH_TOKEN not set", file=sys.stderr)
        return 1

    # Use the gh CLI if available, otherwise fall back to the REST API.
    import shutil
    if shutil.which("gh"):
        subprocess.check_call(
            [
                "gh", "release", "create", tag,
                "--title", tag,
                "--notes", body,
                "--target", "main",
            ],
            env={**os.environ, "GH_TOKEN": gh},
            cwd=ROOT,
        )
    else:
        import urllib.request
        req = urllib.request.Request(
            f"https://api.github.com/repos/biplav00/port-monitor/releases",
            method="POST",
            data=json.dumps({
                "tag_name": tag,
                "name": tag,
                "body": body,
                "draft": False,
                "prerelease": False,
            }).encode(),
            headers={
                "Authorization": f"token {gh}",
                "Accept": "application/vnd.github+json",
                "Content-Type": "application/json",
                "User-Agent": "release-script",
            },
        )
        with urllib.request.urlopen(req) as resp:
            print(resp.read().decode())

    return 0


if __name__ == "__main__":
    sys.exit(main())
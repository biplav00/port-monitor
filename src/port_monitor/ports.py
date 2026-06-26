"""List and kill processes on listening TCP ports.

Uses `lsof` in field-output mode — no sudo needed for the current user's own
processes, which is exactly the dev-server-janitor scope.
"""
from __future__ import annotations

import os
import signal
import subprocess

from .types import Port


def list_listening() -> list[Port]:
    try:
        out = subprocess.run(
            ["lsof", "+c", "0", "-nP", "-iTCP", "-sTCP:LISTEN", "-F", "pcLn"],
            capture_output=True,
            text=True,
            timeout=5,
        ).stdout
    except (subprocess.SubprocessError, FileNotFoundError):
        return []

    # lsof -F emits one field per line: p<pid>, c<command>, L<login>, then
    # f<fd>/n<addr> per open file. Track the current process; each n is a listener.
    by_port: dict[int, tuple[Port, bool]] = {}
    pid = 0
    command = ""
    user = ""
    for line in out.splitlines():
        if not line:
            continue
        tag, val = line[0], line[1:]
        if tag == "p":
            pid = int(val)
        elif tag == "c":
            command = val
        elif tag == "L":
            user = val
        elif tag == "n":
            # val: "*:3000" | "127.0.0.1:3000" | "[::1]:3000"
            try:
                port = int(val.rsplit(":", 1)[1])
            except (ValueError, IndexError):
                continue
            is_v4 = not val.startswith("[")
            entry = Port(port=port, pid=pid, command=command, user=user, is_v4=is_v4)
            # Dedupe by port, preferring the IPv4 binding.
            existing = by_port.get(port)
            if existing is None or (not existing[1] and is_v4):
                by_port[port] = (entry, is_v4)

    return sorted((e for e, _ in by_port.values()), key=lambda p: p.port)


def kill(pid: int, force: bool = False) -> None:
    os.kill(pid, signal.SIGKILL if force else signal.SIGTERM)


if __name__ == "__main__":
    # Smoke check: list our own listening ports.
    for p in list_listening():
        print(f":{p.port:<6} {p.command} (pid {p.pid} · {p.user})")

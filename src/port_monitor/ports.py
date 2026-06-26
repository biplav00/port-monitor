"""List and kill processes on listening TCP ports.

Uses `lsof` in field-output mode — no sudo needed for the current user's own
processes, which is exactly the dev-server-janitor scope.

The seam is the subprocess: `list_listening` takes a `run` callable (the real
lsof by default) so tests drive the whole command->parse path through the same
interface callers use. The `-F` field spec and the parser that reads it live
side by side below — they must agree, so they're kept together.
"""
from __future__ import annotations

import getpass
import os
import signal
import subprocess
from collections.abc import Callable

from .types import Port

# lsof -F emits one field per line, tagged by the IDs requested here:
#   p<pid>  c<command>  L<login>  n<addr>  (plus f<fd> we skip).
# _parse below reads exactly these tags; change one, change both.
_FIELDS = "pcLn"
_LSOF_CMD = ["lsof", "+c", "0", "-nP", "-iTCP", "-sTCP:LISTEN", "-F", _FIELDS]


def _run_lsof() -> str:
    return subprocess.run(
        _LSOF_CMD, capture_output=True, text=True, timeout=5
    ).stdout


def list_listening(run: Callable[[], str] = _run_lsof, me: str | None = None) -> list[Port]:
    """Current listening TCP ports, deduped and sorted. Empty on any lsof error."""
    if me is None:
        me = getpass.getuser()
    try:
        out = run()
    except (subprocess.SubprocessError, FileNotFoundError):
        return []
    return _parse(out, me)


def _parse(out: str, me: str) -> list[Port]:
    """Pure transform of `lsof -F pcLn` output -> sorted, deduped ports."""
    # Track the current process; each n line is a listener for that process.
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
            entry = Port(
                port=port,
                pid=pid,
                command=command,
                user=user,
                is_v4=is_v4,
                is_current_user=(user == me),
            )
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

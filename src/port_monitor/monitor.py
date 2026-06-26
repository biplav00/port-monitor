"""The port-monitoring domain, free of AppKit.

`Monitor` owns the current listening-port snapshot and the kill operation. The
controller in `app.py` schedules a timer and renders the view; everything about
*what a port is and how it dies* lives here, behind `ports` / `refresh` / `kill`,
so it can be tested without a running NSApplication.
"""
from __future__ import annotations

from collections.abc import Callable

from . import ports as _ports
from .types import Port


class Monitor:
    def __init__(
        self,
        lister: Callable[[], list[Port]] = _ports.list_listening,
        killer: Callable[[int, bool], None] = _ports.kill,
    ):
        self._lister = lister
        self._killer = killer
        self.ports: list[Port] = []

    def refresh(self) -> list[Port]:
        """Re-read the listening ports into `self.ports` and return them."""
        self.ports = self._lister()
        return self.ports

    def kill(self, pid: int, force: bool = False) -> None:
        """Signal the process; a gone/forbidden pid is a no-op, not an error."""
        try:
            self._killer(pid, force)
        except (ProcessLookupError, PermissionError):
            pass

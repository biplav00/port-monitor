from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Port:
    """One listening TCP port as shown to the user."""

    port: int
    pid: int
    command: str
    user: str
    is_v4: bool
    is_current_user: bool

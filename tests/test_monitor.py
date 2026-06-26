from port_monitor.monitor import Monitor
from port_monitor.types import Port

P = Port(port=3000, pid=1, command="node", user="me", is_v4=True, is_current_user=True)


def test_refresh_stores_and_returns_snapshot():
    m = Monitor(lister=lambda: [P])
    assert m.ports == []
    assert m.refresh() == [P]
    assert m.ports == [P]


def test_kill_forwards_pid_and_force():
    calls = []
    m = Monitor(killer=lambda pid, force: calls.append((pid, force)))
    m.kill(1, force=True)
    assert calls == [(1, True)]


def test_kill_swallows_dead_or_forbidden_process():
    def gone(pid, force):
        raise ProcessLookupError

    Monitor(killer=gone).kill(99)  # must not raise

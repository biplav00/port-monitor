from port_monitor.ports import list_listening

# Two listeners on the same port (v6 then v4) → IPv4 wins; ports sorted;
# the current user's process is flagged, another user's isn't.
SAMPLE = "\n".join(
    [
        "p100", "cnode", "Lalice", "f3", "n[::1]:3000",
        "p101", "cnginx", "Lroot", "f4", "n*:3000",   # dup port, v4 → preferred
        "p200", "credis", "Lalice", "f5", "n127.0.0.1:6379",
    ]
)


def test_dedup_sort_and_current_user():
    # Drive the whole command→parse path through the public seam with a fake run.
    ports = list_listening(run=lambda: SAMPLE, me="alice")
    assert [p.port for p in ports] == [3000, 6379]      # sorted, deduped

    p3000 = ports[0]
    assert p3000.pid == 101 and p3000.command == "nginx"  # IPv4 binding kept
    assert p3000.is_current_user is False                 # owned by root

    p6379 = ports[1]
    assert p6379.is_current_user is True                  # owned by alice


def test_empty_input():
    assert list_listening(run=lambda: "", me="alice") == []


def test_lsof_failure_is_empty():
    def boom():
        raise FileNotFoundError("lsof missing")

    assert list_listening(run=boom, me="alice") == []

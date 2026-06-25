use crate::port_enum::types::{PortEntry, RawScan};
use std::collections::HashMap;

/// Pure transform: raw OS facts → the listening-port view.
/// Owns every decision — current-user match, IPv4-over-IPv6 dedup, sort —
/// so they're testable without binding a real port.
pub fn assemble(raw: RawScan) -> Vec<PortEntry> {
    // Dedupe by port, preferring the IPv4 binding.
    let mut by_port: HashMap<u16, (PortEntry, bool /* is_v4 */)> = HashMap::new();

    for l in raw.listeners {
        // Windows usernames are case-insensitive and sysinfo may return a
        // stale-cased form; the PID match is the reliable same-process fallback.
        let is_current_user =
            l.pid == raw.current_pid || l.user.eq_ignore_ascii_case(&raw.current_user);

        let entry = PortEntry {
            port: l.port,
            pid: l.pid,
            process_name: l.process_name,
            user: l.user,
            is_current_user,
        };

        match by_port.get(&l.port) {
            // Already have the IPv4 binding for this port: keep it.
            Some((_, existing_v4)) if *existing_v4 => {}
            _ => {
                by_port.insert(l.port, (entry, l.is_v4));
            }
        }
    }

    let mut out: Vec<PortEntry> = by_port.into_values().map(|(e, _)| e).collect();
    out.sort_by_key(|e| e.port);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port_enum::types::RawListener;

    fn listener(port: u16, pid: u32, is_v4: bool, user: &str) -> RawListener {
        RawListener {
            port,
            pid,
            is_v4,
            process_name: "proc".into(),
            user: user.into(),
        }
    }

    fn scan(listeners: Vec<RawListener>) -> RawScan {
        RawScan {
            listeners,
            current_user: "alice".into(),
            current_pid: 1000,
        }
    }

    #[test]
    fn dedup_prefers_ipv4_regardless_of_order() {
        // v6 first, then v4
        let a = assemble(scan(vec![
            listener(8080, 1, false, "alice"),
            listener(8080, 2, true, "alice"),
        ]));
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].pid, 2);

        // v4 first, then v6
        let b = assemble(scan(vec![
            listener(8080, 2, true, "alice"),
            listener(8080, 1, false, "alice"),
        ]));
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].pid, 2);
    }

    #[test]
    fn current_user_match_is_case_insensitive() {
        let out = assemble(scan(vec![listener(3000, 5, true, "ALICE")]));
        assert!(out[0].is_current_user);
    }

    #[test]
    fn current_pid_overrides_user_mismatch() {
        // user differs, but pid == current_pid → still "mine"
        let out = assemble(scan(vec![listener(3000, 1000, true, "root")]));
        assert!(out[0].is_current_user);
    }

    #[test]
    fn foreign_user_is_not_current() {
        let out = assemble(scan(vec![listener(3000, 7, true, "root")]));
        assert!(!out[0].is_current_user);
    }

    #[test]
    fn sorted_by_port() {
        let out = assemble(scan(vec![
            listener(9000, 1, true, "alice"),
            listener(3000, 2, true, "alice"),
            listener(5000, 3, true, "alice"),
        ]));
        let ports: Vec<u16> = out.iter().map(|e| e.port).collect();
        assert_eq!(ports, vec![3000, 5000, 9000]);
    }
}

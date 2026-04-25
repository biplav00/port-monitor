use crate::port_enum::types::PortEntry;

#[derive(Debug, Clone, Copy)]
pub struct FilterOpts {
    pub port_range_min: u16,
    pub port_range_max: u16,
    pub show_system_ports: bool,
    pub show_all_users: bool,
}

pub fn apply(entries: &[PortEntry], opts: FilterOpts) -> Vec<PortEntry> {
    entries
        .iter()
        .filter(|e| e.port >= opts.port_range_min && e.port <= opts.port_range_max)
        .filter(|e| opts.show_system_ports || e.port >= 1024)
        .filter(|e| opts.show_all_users || e.is_current_user)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port_enum::types::Proto;

    fn entry(port: u16, is_me: bool) -> PortEntry {
        PortEntry {
            port,
            proto: Proto::Tcp,
            pid: 1,
            process_name: "p".into(),
            user: if is_me { "me".into() } else { "root".into() },
            is_current_user: is_me,
        }
    }

    fn defaults() -> FilterOpts {
        FilterOpts {
            port_range_min: 1024,
            port_range_max: 65535,
            show_system_ports: false,
            show_all_users: false,
        }
    }

    #[test]
    fn drops_entries_outside_range() {
        let e = vec![entry(500, true), entry(3000, true), entry(1023, true)];
        let out = apply(&e, defaults());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].port, 3000);
    }

    #[test]
    fn drops_non_current_user_by_default() {
        let e = vec![entry(3000, true), entry(3001, false)];
        let out = apply(&e, defaults());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].port, 3000);
    }

    #[test]
    fn show_all_users_keeps_other_users() {
        let e = vec![entry(3000, true), entry(3001, false)];
        let out = apply(
            &e,
            FilterOpts {
                show_all_users: true,
                ..defaults()
            },
        );
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn show_system_ports_keeps_low_ports_in_range() {
        let e = vec![entry(80, true), entry(3000, true)];
        let out = apply(
            &e,
            FilterOpts {
                port_range_min: 1,
                port_range_max: 65535,
                show_system_ports: true,
                show_all_users: false,
            },
        );
        assert_eq!(out.len(), 2);
    }
}

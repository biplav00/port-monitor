// ponytail: TCP-only by scope (local dev-server janitor). If UDP listeners
// ever matter, add a `proto` field back and widen the netstat2 query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortEntry {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub user: String,
    pub is_current_user: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_entry_eq_by_fields() {
        let a = PortEntry {
            port: 8080,
            pid: 42,
            process_name: "node".into(),
            user: "alice".into(),
            is_current_user: true,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}

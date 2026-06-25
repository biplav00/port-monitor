// ponytail: TCP-only by scope (local dev-server janitor). If UDP listeners
// ever matter, add a `proto` field back and widen the netstat2 query.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PortEntry {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub user: String,
    pub is_current_user: bool,
}

/// Raw OS facts gathered by `source::collect_raw`, consumed by the pure
/// `assemble`. This data type *is* the seam: prod fills it from the OS, tests
/// construct it as a literal — no trait, no mocks.
#[derive(Debug, Clone)]
pub struct RawScan {
    pub listeners: Vec<RawListener>,
    pub current_user: String,
    pub current_pid: u32,
}

#[derive(Debug, Clone)]
pub struct RawListener {
    pub port: u16,
    pub pid: u32,
    pub is_v4: bool,
    pub process_name: String,
    pub user: String,
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

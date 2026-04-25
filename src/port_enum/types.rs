#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortEntry {
    pub port: u16,
    pub proto: Proto,
    pub pid: u32,
    pub process_name: String,
    pub user: String,
    pub is_current_user: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proto {
    Tcp,
    Udp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_entry_eq_by_fields() {
        let a = PortEntry {
            port: 8080,
            proto: Proto::Tcp,
            pid: 42,
            process_name: "node".into(),
            user: "alice".into(),
            is_current_user: true,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}

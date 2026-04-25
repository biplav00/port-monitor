use crate::port_enum::types::{PortEntry, Proto};
use anyhow::{Context, Result};
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use std::collections::HashMap;
use sysinfo::{Pid, System, Users};

pub fn list_listening() -> Result<Vec<PortEntry>> {
    let sockets = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP,
    )
    .context("get_sockets_info")?;

    let mut sys = System::new();
    sys.refresh_processes();
    let users = Users::new_with_refreshed_list();

    let current_user = whoami::username();
    let mut by_port: HashMap<u16, (PortEntry, bool /* is_v4 */)> = HashMap::new();

    for info in sockets {
        let tcp = match info.protocol_socket_info {
            ProtocolSocketInfo::Tcp(ref t) => t.clone(),
            _ => continue,
        };
        if tcp.state != TcpState::Listen {
            continue;
        }
        let Some(&pid) = info.associated_pids.first() else {
            continue;
        };
        let port = tcp.local_port;
        let is_v4 = tcp.local_addr.is_ipv4();

        let (process_name, user) = sys
            .process(Pid::from_u32(pid))
            .map(|p| {
                let name = p.name().to_string();
                let user = p
                    .user_id()
                    .and_then(|uid| users.get_user_by_id(uid))
                    .map(|u| u.name().to_string())
                    .unwrap_or_else(|| "?".into());
                (name, user)
            })
            .unwrap_or_else(|| ("?".into(), "?".into()));

        // Windows usernames are case-insensitive; sysinfo may also return a stale-cased
        // form. Match the current PID directly as a fallback for the same-process case.
        let is_current_user = pid == std::process::id() || user.eq_ignore_ascii_case(&current_user);

        let new_entry = PortEntry {
            port,
            proto: Proto::Tcp,
            pid,
            process_name,
            user,
            is_current_user,
        };

        // Dedupe by port: prefer IPv4 over IPv6.
        match by_port.get(&port) {
            Some((_, existing_v4)) if *existing_v4 => {}
            _ => {
                by_port.insert(port, (new_entry, is_v4));
            }
        }
    }

    let mut out: Vec<PortEntry> = by_port.into_values().map(|(e, _)| e).collect();
    out.sort_by_key(|e| e.port);
    Ok(out)
}

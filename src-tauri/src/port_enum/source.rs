use crate::port_enum::types::{RawListener, RawScan};
use anyhow::{Context, Result};
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use sysinfo::{Pid, System, Users};

/// The single OS adapter: gather raw listening-socket facts. No dedup, no
/// current-user logic, no filtering — those live in the pure `assemble`.
pub fn collect_raw() -> Result<RawScan> {
    let sockets = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP,
    )
    .context("get_sockets_info")?;

    let mut sys = System::new();
    sys.refresh_processes();
    let users = Users::new_with_refreshed_list();

    let mut listeners = Vec::new();
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

        listeners.push(RawListener {
            port: tcp.local_port,
            pid,
            is_v4: tcp.local_addr.is_ipv4(),
            process_name,
            user,
        });
    }

    Ok(RawScan {
        listeners,
        current_user: whoami::username(),
        current_pid: std::process::id(),
    })
}

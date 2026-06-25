use std::net::TcpListener;

#[test]
fn list_listening_finds_our_bound_port() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let bound_port = listener.local_addr().unwrap().port();
    let our_pid = std::process::id();

    let entries = port_monitor::port_enum::list_listening().expect("list");
    let hit = entries.iter().find(|e| e.port == bound_port);

    assert!(
        hit.is_some(),
        "bound port {bound_port} not found in {entries:?}"
    );
    let hit = hit.unwrap();
    assert_eq!(hit.pid, our_pid, "pid mismatch: expected {our_pid}");
    assert!(hit.is_current_user);

    drop(listener);
}

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn wait_for_exit(child: &mut std::process::Child, within: Duration) {
    let deadline = Instant::now() + within;
    loop {
        if child.try_wait().unwrap().is_some() {
            return;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            panic!("child did not exit within {:?}", within);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(unix)]
#[test]
fn sigterm_terminates_child() {
    let mut child = Command::new("sleep")
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sleep");
    let pid = child.id();
    port_monitor::port_enum::kill(pid, false).expect("kill");
    wait_for_exit(&mut child, Duration::from_secs(3));
}

#[cfg(unix)]
#[test]
fn sigkill_terminates_child() {
    let mut child = Command::new("sleep")
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sleep");
    let pid = child.id();
    port_monitor::port_enum::kill(pid, true).expect("kill");
    wait_for_exit(&mut child, Duration::from_secs(3));
}

#[cfg(windows)]
#[test]
fn terminate_process_ends_child() {
    let mut child = Command::new("cmd")
        .args(["/C", "ping", "-n", "30", "127.0.0.1"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ping");
    let pid = child.id();
    port_monitor::port_enum::kill(pid, false).expect("kill");
    wait_for_exit(&mut child, Duration::from_secs(3));
}

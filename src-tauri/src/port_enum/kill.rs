use anyhow::Result;

#[cfg(unix)]
pub fn kill(pid: u32, force: bool) -> Result<()> {
    use nix::sys::signal::{kill as nix_kill, Signal};
    use nix::unistd::Pid;
    let sig = if force {
        Signal::SIGKILL
    } else {
        Signal::SIGTERM
    };
    nix_kill(Pid::from_raw(pid as i32), sig)?;
    Ok(())
}

#[cfg(windows)]
pub fn kill(pid: u32, _force: bool) -> Result<()> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)?;
        let term = TerminateProcess(handle, 1);
        let _ = CloseHandle(handle);
        term?;
    }
    Ok(())
}

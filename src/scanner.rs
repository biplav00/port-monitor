use crate::port_enum::{apply_filter, list_listening, FilterOpts};
use crate::state::SharedState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Scanner {
    stop: Arc<AtomicBool>,
    // `bool` is the wake predicate: set by trigger_refresh/set_interval/stop so
    // a notify sent while the thread is mid-scan isn't lost (it's seen on the
    // next wait). Without it, Refresh could stall up to a full interval.
    wake: Arc<(Mutex<bool>, Condvar)>,
    interval: Arc<Mutex<Duration>>,
    handle: Option<JoinHandle<()>>,
}

impl Scanner {
    pub fn spawn(state: SharedState, ctx: egui::Context) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let wake = Arc::new((Mutex::new(false), Condvar::new()));
        let initial = {
            let s = state.read().unwrap();
            Duration::from_secs_f64(s.settings.refresh_interval_secs)
        };
        let interval = Arc::new(Mutex::new(initial));

        let handle = {
            let stop = stop.clone();
            let wake = wake.clone();
            let interval = interval.clone();
            let state = state.clone();
            thread::Builder::new()
                .name("port-scanner".into())
                .spawn(move || Self::run(state, ctx, stop, wake, interval))
                .expect("spawn scanner thread")
        };

        Scanner {
            stop,
            wake,
            interval,
            handle: Some(handle),
        }
    }

    pub fn trigger_refresh(&self) {
        self.signal_wake();
    }

    pub fn set_interval(&self, d: Duration) {
        *self.interval.lock().unwrap() = d;
        self.signal_wake();
    }

    pub fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.signal_wake();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    fn signal_wake(&self) {
        *self.wake.0.lock().unwrap() = true;
        self.wake.1.notify_all();
    }

    fn run(
        state: SharedState,
        ctx: egui::Context,
        stop: Arc<AtomicBool>,
        wake: Arc<(Mutex<bool>, Condvar)>,
        interval: Arc<Mutex<Duration>>,
    ) {
        while !stop.load(Ordering::SeqCst) {
            Self::scan_once(&state);
            ctx.request_repaint();

            let dur = *interval.lock().unwrap();
            let (lock, cvar) = &*wake;
            let guard = lock.lock().unwrap();
            // Wait until woken (predicate handles lost notifies + spurious wakeups).
            let (mut woken, _) = cvar
                .wait_timeout_while(guard, dur, |woken| !*woken)
                .unwrap();
            *woken = false;
        }
    }

    fn scan_once(state: &SharedState) {
        let opts = {
            let s = state.read().unwrap();
            FilterOpts {
                port_range_min: s.settings.port_range_min,
                port_range_max: s.settings.port_range_max,
                show_system_ports: s.settings.show_system_ports,
                show_all_users: s.settings.show_all_users,
            }
        };
        match list_listening() {
            Ok(all) => {
                let filtered = apply_filter(&all, opts);
                let mut s = state.write().unwrap();
                s.ports = filtered;
                s.last_error = None;
            }
            Err(e) => {
                log::warn!("scan failed: {e:#}");
                let mut s = state.write().unwrap();
                s.last_error = Some(format!("Scan failed: {e}"));
            }
        }
    }
}

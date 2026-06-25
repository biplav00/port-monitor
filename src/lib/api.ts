import { invoke } from "@tauri-apps/api/core";

// Ports are polled via `listPorts` (invoke), not pushed via events —
// Rust→JS event delivery wasn't reaching the webview reliably.

// SOURCE OF TRUTH: these mirror the Rust structs `PortEntry` (port_enum/types.rs)
// and `Settings` (settings.rs) — they're serialized as snake_case across the IPC
// seam. Change a field there → update it here. (Too few types to justify codegen.)
export interface Port {
  port: number;
  pid: number;
  process_name: string;
  user: string;
  is_current_user: boolean;
}

export type Appearance = "system" | "light" | "dark";

export interface Settings {
  refresh_interval_secs: number;
  port_range_min: number;
  port_range_max: number;
  show_system_ports: boolean;
  show_all_users: boolean;
  appearance: Appearance;
  launch_at_login: boolean;
}

export const listPorts = () => invoke<Port[]>("list_ports");
export const getSettings = () => invoke<Settings>("get_settings");
export const setSettings = (next: Settings) =>
  invoke<Settings>("set_settings", { new: next });
export const killPort = (pid: number, force: boolean) =>
  invoke("kill_port", { pid, force });

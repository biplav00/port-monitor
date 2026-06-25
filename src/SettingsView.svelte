<script lang="ts">
  import type { Settings } from "./lib/api";
  let {
    settings,
    onsave,
    onback,
  }: { settings: Settings; onsave: (s: Settings) => void; onback: () => void } =
    $props();

  let s = $state({ ...settings });
  // onchange (commit), not oninput — no debounce needed.
  const commit = () => onsave({ ...s });
</script>

<header>
  <button class="back" onclick={onback} aria-label="Back">‹</button>
  <h1>Settings</h1>
</header>

<div class="form">
  <label class="field">
    <span class="lbl">Refresh interval <em>{s.refresh_interval_secs}s</em></span>
    <input type="range" min="1" max="30" step="0.5" bind:value={s.refresh_interval_secs} onchange={commit} />
  </label>

  <label class="field">
    <span class="lbl">Port range</span>
    <span class="range">
      <input type="number" min="0" max="65535" bind:value={s.port_range_min} onchange={commit} />
      <span class="dash">–</span>
      <input type="number" min="0" max="65535" bind:value={s.port_range_max} onchange={commit} />
    </span>
  </label>

  <label class="check">
    <input type="checkbox" bind:checked={s.show_system_ports} onchange={commit} />
    Show system ports (&lt; 1024)
  </label>

  <label class="check">
    <input type="checkbox" bind:checked={s.show_all_users} onchange={commit} />
    Show all users
  </label>

  <label class="field">
    <span class="lbl">Appearance</span>
    <select bind:value={s.appearance} onchange={commit}>
      <option value="system">System</option>
      <option value="light">Light</option>
      <option value="dark">Dark</option>
    </select>
  </label>

  <label class="check">
    <input type="checkbox" bind:checked={s.launch_at_login} onchange={commit} />
    Launch at login
  </label>
</div>

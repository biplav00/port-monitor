<script lang="ts">
  import { onMount } from "svelte";
  import {
    listPorts,
    getSettings,
    setSettings,
    type Port,
    type Settings,
  } from "./lib/api";
  import PortRow from "./PortRow.svelte";
  import SettingsView from "./SettingsView.svelte";

  let ports = $state<Port[]>([]);
  let error = $state<string | null>(null);
  let view = $state<"list" | "settings">("list");
  let settings = $state<Settings | null>(null);
  let timer: ReturnType<typeof setInterval> | undefined;

  function applyTheme(s: Settings) {
    const mode =
      s.appearance === "system"
        ? matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light"
        : s.appearance;
    document.documentElement.dataset.theme = mode;
  }

  async function poll() {
    try {
      ports = await listPorts();
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  function startPolling(secs: number) {
    clearInterval(timer);
    timer = setInterval(poll, secs * 1000);
  }

  onMount(() => {
    getSettings().then((s) => {
      settings = s;
      applyTheme(s);
      startPolling(s.refresh_interval_secs);
    });
    poll();
    return () => clearInterval(timer);
  });

  async function save(next: Settings) {
    settings = await setSettings(next);
    applyTheme(settings);
    startPolling(settings.refresh_interval_secs);
  }
</script>

<main>
  {#if view === "list"}
    <header>
      <h1>Ports <span class="count">{ports.length}</span></h1>
      <div class="actions">
        <button onclick={poll} title="Refresh" aria-label="Refresh">⟳</button>
        <button onclick={() => (view = "settings")} title="Settings" aria-label="Settings">⚙</button>
      </div>
    </header>

    {#if error}
      <div class="error">
        <span>{error}</span>
        <button onclick={() => (error = null)} aria-label="Dismiss">✕</button>
      </div>
    {/if}

    {#if ports.length === 0}
      <p class="empty">No listening ports in range</p>
    {:else}
      <ul class="list">
        {#each ports as p (p.pid + ":" + p.port)}
          <PortRow {p} onkilled={poll} />
        {/each}
      </ul>
    {/if}
  {:else if settings}
    <SettingsView {settings} onsave={save} onback={() => (view = "list")} />
  {/if}
</main>

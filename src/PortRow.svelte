<script lang="ts">
  import { killPort, type Port } from "./lib/api";
  let { p, onkilled }: { p: Port; onkilled: () => void } = $props();
  // Shift-click = force kill (SIGKILL on unix; Windows is always forced).
  const kill = async (e: MouseEvent) => {
    await killPort(p.pid, e.shiftKey);
    onkilled();
  };
</script>

<li class="row" class:foreign={!p.is_current_user}>
  <span class="port">:{p.port}</span>
  <span class="meta">
    <span class="name">{p.process_name}</span>
    <span class="sub">pid {p.pid} · {p.user}</span>
  </span>
  <button
    class="kill"
    disabled={!p.is_current_user}
    onclick={kill}
    title="Kill — hold Shift to force"
    aria-label="Kill {p.process_name}">✕</button>
</li>

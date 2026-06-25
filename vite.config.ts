import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Strip Vite's `crossorigin` from emitted <script>/<link> — under Tauri's
// custom asset protocol it blocks the module bundle from executing (blank page).
const stripCrossorigin = {
  name: "strip-crossorigin",
  transformIndexHtml: (html: string) => html.replace(/\s+crossorigin/g, ""),
};

// Tauri serves the dev frontend on a fixed port.
export default defineConfig({
  plugins: [svelte(), stripCrossorigin],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
});

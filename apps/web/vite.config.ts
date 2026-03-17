import tailwindcss from "@tailwindcss/vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
  plugins: [tsconfigPaths(), tailwindcss(), tanstackStart(), viteReact()],
  worker: {
    format: "es",
  },
  server: {
    port: 5020,
    fs: {
      // Allow serving WASM files from the kernel-wasm package
      allow: ["../.."],
    },
  },
  optimizeDeps: {
    // Exclude WASM package from pre-bundling (it has side-effects on import)
    exclude: ["@blockCAD/kernel-wasm"],
  },
});

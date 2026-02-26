import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],

  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port: 1421,
    strictPort: true,
    host: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});

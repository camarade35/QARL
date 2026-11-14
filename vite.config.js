import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import adapter from "@sveltejs/adapter-static";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [
    sveltekit({
      adapter: adapter({
        fallback: 'index.html'
      })
    }),
    tailwindcss()
  ],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
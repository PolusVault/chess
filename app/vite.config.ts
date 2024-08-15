import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
    build: {
        target: "esnext",
    },
    plugins: [react()],
    esbuild: {
        drop: ["console", "debugger"],
    },
    // server: {
    //     host: true,
    // },
});

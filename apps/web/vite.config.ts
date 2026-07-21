import { execSync } from "node:child_process";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

function resolveSoftwareCommit(): string {
  if (process.env.ECHLUB_SOFTWARE_COMMIT) return process.env.ECHLUB_SOFTWARE_COMMIT;
  try {
    return execSync("git rev-parse HEAD", { encoding: "utf-8" }).trim();
  } catch {
    return "";
  }
}

export default defineConfig({
  plugins: [react()],
  define: {
    __ECHLUB_SOFTWARE_COMMIT__: JSON.stringify(resolveSoftwareCommit()),
  },
  server: { port: 5173 },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: "./src/test/setup.ts",
  },
});

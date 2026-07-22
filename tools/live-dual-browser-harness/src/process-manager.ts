import { execSync, spawn, type ChildProcess } from "node:child_process";
import { appendFileSync } from "node:fs";
import { REPO_ROOT, WEB_ORIGIN } from "./constants.js";

export function resolveAuthorizedCommit(): string {
  return execSync("git rev-parse HEAD", { cwd: REPO_ROOT, encoding: "utf8" }).trim();
}

export class ProcessManager {
  private readonly children: ChildProcess[] = [];
  private readonly logPath: string;

  constructor(logPath: string) {
    this.logPath = logPath;
  }

  verifyGitCommit(): string {
    const head = resolveAuthorizedCommit();
    if (!/^[0-9a-f]{40}$/.test(head)) {
      throw new Error(`unexpected git HEAD ${head}`);
    }
    const status = execSync("git status --short", { cwd: REPO_ROOT, encoding: "utf8" }).trim();
    if (status.length > 0) {
      throw new Error(`working tree not clean:\n${status}`);
    }
    return head;
  }

  start(name: string, command: string, args: string[], env: Record<string, string> = {}): ChildProcess {
    const child = spawn(command, args, {
      cwd: REPO_ROOT,
      env: { ...process.env, ...env },
      stdio: ["ignore", "pipe", "pipe"],
    });
    child.stdout?.on("data", (chunk) => this.log(name, chunk.toString()));
    child.stderr?.on("data", (chunk) => this.log(name, chunk.toString()));
    child.on("exit", (code, signal) => {
      this.log(name, `exit code=${code ?? "null"} signal=${signal ?? "null"}\n`);
    });
    this.children.push(child);
    return child;
  }

  async startServers(softwareCommit: string): Promise<void> {
    this.start("control-plane", "cargo", ["run", "-p", "echlub-control-plane", "--bin", "control-plane"], {
      ECHLUB_BIND_ADDR: "127.0.0.1:8080",
    });
    this.start("web-dev", "corepack", ["pnpm", "--filter", "@echlub/web", "dev", "--host", "127.0.0.1", "--port", "5173"], {
      ECHLUB_SOFTWARE_COMMIT: softwareCommit,
    });
    await waitForHttp(`${WEB_ORIGIN}/`);
    await waitForHttp("http://127.0.0.1:8080/healthz");
  }

  terminateAll(): void {
    for (const child of this.children) {
      if (child.exitCode === null && !child.killed) {
        child.kill("SIGTERM");
      }
    }
  }

  async terminateAllAndWait(timeoutMs = 5_000): Promise<void> {
    this.terminateAll();
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      if (this.children.every((child) => child.exitCode !== null || child.killed)) {
        return;
      }
      await sleep(100);
    }
    for (const child of this.children) {
      if (child.exitCode === null && !child.killed) {
        child.kill("SIGKILL");
      }
    }
  }

  get trackedChildren(): ChildProcess[] {
    return [...this.children];
  }

  private log(name: string, message: string): void {
    appendFileSync(this.logPath, `[${name}] ${message}`);
  }
}

export async function waitForHttp(url: string, timeoutMs = 60_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch {
      /* retry */
    }
    await sleep(500);
  }
  throw new Error(`timed out waiting for ${url}`);
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

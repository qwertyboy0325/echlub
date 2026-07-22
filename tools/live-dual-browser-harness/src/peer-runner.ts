import { readFileSync } from "node:fs";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { chromium, type BrowserContext, type Download, type Page } from "playwright";
import {
  COMPLETED_TIMEOUT_MS,
  LIVE_URL,
  OBSERVATION_SECONDS,
  READY_TIMEOUT_MS,
  SIGNALING_URL,
  type ConnectOrder,
  type PeerDiagnostics,
} from "./constants.js";
import { sleep } from "./process-manager.js";

export interface PeerConfig {
  role: "peer_a" | "peer_b";
  correlationId: string;
  fakeAudioPath: string;
  userDataDir: string;
  diagnosticsDir: string;
}

export interface PeerSession {
  role: "peer_a" | "peer_b";
  context: BrowserContext;
  page: Page;
  consoleLogs: string[];
  pageErrors: string[];
}

export async function launchPeer(config: PeerConfig): Promise<PeerSession> {
  const consoleLogs: string[] = [];
  const pageErrors: string[] = [];
  const fakeAudio = resolve(config.fakeAudioPath);
  const context = await chromium.launchPersistentContext(config.userDataDir, {
    headless: false,
    args: [
      "--use-fake-device-for-media-stream",
      "--use-fake-ui-for-media-stream",
      `--use-file-for-fake-audio-capture=${fakeAudio}`,
      "--autoplay-policy=no-user-gesture-required",
    ],
    permissions: ["microphone"],
    acceptDownloads: true,
    viewport: { width: 1280, height: 900 },
  });
  await context.tracing.start({ screenshots: true, snapshots: true, sources: true });
  const page = context.pages()[0] ?? (await context.newPage());
  page.on("console", (message) => {
    consoleLogs.push(`[${message.type()}] ${message.text()}`);
  });
  page.on("pageerror", (error) => {
    pageErrors.push(error.message);
  });
  await page.goto(LIVE_URL, { waitUntil: "domcontentloaded", timeout: 60_000 });
  await page.getByRole("button", { name: "Live Session" }).click();
  await configurePeer(page, config);
  return { role: config.role, context, page, consoleLogs, pageErrors };
}

async function configurePeer(page: Page, config: PeerConfig): Promise<void> {
  const panel = page.locator("section.performance-panel");
  await panel.getByRole("heading", { name: /Live Two-Peer Observation/i }).waitFor({ timeout: 60_000 });
  await panel.locator("label").filter({ hasText: "Session correlation ID" }).locator("input").fill(config.correlationId);
  await panel.locator("select").nth(0).selectOption(config.role);
  await panel.locator("label").filter({ hasText: "Signaling URL" }).locator("input").fill(SIGNALING_URL);
  await panel.locator("select").nth(1).selectOption("browser_default");
  await panel.getByRole("checkbox").check();
  await panel.getByRole("button", { name: /^Prepare$/i }).click();
  await panel.getByRole("button", { name: /Enable Microphone/i }).click();
}

export async function connectPeer(page: Page): Promise<void> {
  const panel = page.locator("section.performance-panel");
  await panel.getByRole("button", { name: /^Connect$/i }).click();
}

export async function waitForReady(page: Page, timeoutMs = READY_TIMEOUT_MS): Promise<void> {
  const panel = page.locator("section.performance-panel");
  await panel.getByText(/Phase: Ready To Observe/i).waitFor({ timeout: timeoutMs });
}

export async function startObservation(page: Page): Promise<void> {
  const panel = page.locator("section.performance-panel");
  await panel.getByRole("button", { name: /Start 60s Observation/i }).click();
}

export async function waitForCompleted(page: Page, timeoutMs = COMPLETED_TIMEOUT_MS): Promise<void> {
  const panel = page.locator("section.performance-panel");
  await panel.getByText(/Phase: Completed/i).waitFor({ timeout: timeoutMs });
}

export async function exportFinalized(page: Page, destination: string): Promise<void> {
  const panel = page.locator("section.performance-panel");
  const downloadPromise = page.waitForEvent("download", { timeout: 30_000 });
  await panel.getByRole("button", { name: /Export Finalized Endpoint/i }).click();
  const download = await downloadPromise;
  await download.saveAs(destination);
}

export async function readDiagnostics(page: Page): Promise<PeerDiagnostics> {
  const bodyText = await page.locator("section.performance-panel").innerText();
  const phase = bodyText.match(/Phase: (.+)/)?.[1] ?? null;
  const connection = bodyText.match(/Connection: ([^\|]+)/)?.[1]?.trim() ?? null;
  const ice = bodyText.match(/ICE: ([^\|]+)/)?.[1]?.trim() ?? null;
  const dc = bodyText.match(/DataChannel: (.+)/)?.[1]?.trim() ?? null;
  const samples = bodyText.match(/Samples: (\d+)/)?.[1] ?? null;
  const probes = bodyText.match(/Probes: (\d+)/)?.[1] ?? null;
  const error = (await page.locator(".error").allTextContents()).join("; ") || null;
  return {
    phase,
    connection,
    ice,
    dataChannel: dc,
    samples,
    probes,
    error,
    console: [],
    pageErrors: [],
  };
}

export async function capturePeerArtifacts(
  session: PeerSession,
  diagnosticsDir: string,
  prefix: "peer-a" | "peer-b",
): Promise<PeerDiagnostics> {
  const diagnostics = await readDiagnostics(session.page);
  diagnostics.console = [...session.consoleLogs];
  diagnostics.pageErrors = [...session.pageErrors];
  await session.page.screenshot({ path: join(diagnosticsDir, `${prefix}-screenshot.png`), fullPage: true });
  await session.context.tracing.stop({ path: join(diagnosticsDir, `${prefix}-trace.zip`) });
  return diagnostics;
}

export async function closePeer(session: PeerSession): Promise<void> {
  await session.context.close();
}

export function createTempUserDataDir(prefix: string): string {
  return mkdtempSync(join(tmpdir(), prefix));
}

export function removeTempUserDataDir(path: string): void {
  rmSync(path, { recursive: true, force: true });
}

export function resolveConnectSequence(order: ConnectOrder): Array<"peer_a" | "peer_b"> {
  return order === "peer_a_first" ? ["peer_a", "peer_b"] : ["peer_b", "peer_a"];
}

export async function runConnectOrder(
  order: ConnectOrder,
  peerA: PeerSession,
  peerB: PeerSession,
): Promise<void> {
  for (const next of resolveConnectSequence(order)) {
    await connectPeer(next === "peer_a" ? peerA.page : peerB.page);
  }
}

export async function synchronizedObservationStart(peerA: PeerSession, peerB: PeerSession): Promise<number> {
  const startedAt = Date.now();
  await startObservation(peerA.page);
  await sleep(250);
  await startObservation(peerB.page);
  return Date.now() - startedAt;
}

export function assertObservationStartWindow(deltaMs: number, maxMs: number): void {
  if (deltaMs > maxMs) {
    throw new Error(`observation start delta ${deltaMs}ms exceeds ${maxMs}ms`);
  }
}

export async function assertPeerReadyGuards(peer: PeerSession): Promise<void> {
  const diagnostics = await readDiagnostics(peer.page);
  if (diagnostics.dataChannel !== "open") {
    throw new Error(`${peer.role} DataChannel not open`);
  }
  if (!diagnostics.probes || Number(diagnostics.probes) < 1) {
    throw new Error(`${peer.role} clock preflight incomplete`);
  }
  if (!diagnostics.samples || Number(diagnostics.samples) < 1) {
    throw new Error(`${peer.role} stats preflight incomplete`);
  }
  const hasRemoteAudio = await peer.page.evaluate(() => {
    const audio = document.getElementById("remote-audio") as HTMLAudioElement | null;
    const stream = audio?.srcObject as MediaStream | null;
    return Boolean(stream?.getAudioTracks().some((track) => track.readyState === "live"));
  });
  if (!hasRemoteAudio) {
    throw new Error(`${peer.role} remote audio track missing`);
  }
}

export function assertEndpointNegotiationRoles(peerAPath: string, peerBPath: string): void {
  const peerA = JSON.parse(readFileSync(peerAPath, "utf8")) as {
    peerRole?: string;
    dataChannel?: { owner?: boolean };
  };
  const peerB = JSON.parse(readFileSync(peerBPath, "utf8")) as {
    peerRole?: string;
    dataChannel?: { owner?: boolean };
  };
  if (peerA.peerRole !== "peer_a" || peerB.peerRole !== "peer_b") {
    throw new Error("exported endpoint roles do not match peer_a / peer_b");
  }
  if (peerA.dataChannel?.owner !== true) {
    throw new Error("peer_a must own the initial DataChannel offer");
  }
  if (peerB.dataChannel?.owner !== false) {
    throw new Error("peer_b must not originate the initial DataChannel offer");
  }
}

export async function assertPeerReadyStates(peerA: PeerSession, peerB: PeerSession): Promise<void> {
  await Promise.all([waitForReady(peerA.page), waitForReady(peerB.page)]);
  await assertPeerReadyGuards(peerA);
  await assertPeerReadyGuards(peerB);
}

export async function assertPeerCompletedStates(peerA: PeerSession, peerB: PeerSession): Promise<void> {
  await Promise.all([waitForCompleted(peerA.page), waitForCompleted(peerB.page)]);
}

export async function saveDownloads(peerA: PeerSession, peerB: PeerSession, peerAPath: string, peerBPath: string) {
  await exportFinalized(peerA.page, peerAPath);
  await exportFinalized(peerB.page, peerBPath);
}

export type { Download };

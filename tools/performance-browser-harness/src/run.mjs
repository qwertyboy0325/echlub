import { execSync } from "node:child_process";
import { mkdirSync, writeFileSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import puppeteer from "puppeteer-core";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "../../..");
const CORRECTED_DIR = join(ROOT, "evidence/performance-baseline/corrected");
const WEB_URL = process.env.ECHLUB_WEB_URL ?? "http://localhost:4173";
const RUN_ID =
  process.env.ECHLUB_RUN_ID ??
  new Date().toISOString().replace(/[:.]/g, "-").replace("T", "T").replace("Z", "Z");
const ARTIFACT_PATH =
  process.env.ECHLUB_ARTIFACT_PATH ?? join(CORRECTED_DIR, RUN_ID, "observation.json");

function detectBrowserExecutable() {
  const candidates = [
    process.env.CHROME_PATH,
    process.env.CHROMIUM_PATH,
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/usr/bin/google-chrome",
    "/usr/bin/chromium-browser",
    "/usr/bin/chromium",
  ].filter(Boolean);

  for (const path of candidates) {
    if (existsSync(path)) return path;
  }

  try {
    const chromePath = execSync("mdfind \"kMDItemCFBundleIdentifier == 'com.google.Chrome'\" | head -1", {
      encoding: "utf-8",
    }).trim();
    if (chromePath) {
      const exe = join(chromePath, "Contents/MacOS/Google Chrome");
      if (existsSync(exe)) return exe;
    }
  } catch {
    /* mdfind unavailable */
  }

  return null;
}

async function runSyntheticHarness() {
  const executablePath = detectBrowserExecutable();
  if (!executablePath) {
    console.error("NO_BROWSER: system browser not found");
    process.exit(2);
  }

  console.log(`Using browser: ${executablePath}`);

  const browser = await puppeteer.launch({
    executablePath,
    headless: true,
    args: [
      "--headless=new",
      "--use-fake-ui-for-media-stream",
      "--use-fake-device-for-media-stream",
      "--autoplay-policy=no-user-gesture-required",
      "--no-sandbox",
    ],
  });

  const page = await browser.newPage();
  await page.goto(`${WEB_URL}?section=synthetic`, { waitUntil: "networkidle0", timeout: 30000 });

  await page.waitForFunction(() => typeof window.__echlubSyntheticHarness === "function", {
    timeout: 15000,
  });

  const result = await page.evaluate(async () => {
    const fn = window.__echlubSyntheticHarness;
    if (!fn) throw new Error("harness function unavailable");
    return fn();
  });

  await browser.close();

  mkdirSync(dirname(ARTIFACT_PATH), { recursive: true });
  writeFileSync(ARTIFACT_PATH, JSON.stringify(result, null, 2));
  console.log(`Evidence written: ${ARTIFACT_PATH}`);
  return ARTIFACT_PATH;
}

runSyntheticHarness().catch((err) => {
  console.error("Harness failed:", err.message);
  process.exit(1);
});

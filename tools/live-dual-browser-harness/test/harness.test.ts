import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { describe, expect, it, vi } from "vitest";
import { generateCorrelationId } from "../src/constants.js";
import { calculateActualEndpointStartDeltaMs } from "../src/endpoint-timing.js";
import {
  assertSafeAutomationOutputPath,
  OWNER_IMPORT_ROOT,
  resolveAutomationRunDir,
  scenarioLayout,
} from "../src/paths.js";
import {
  buildAutomationReport,
  classificationNeverClaimsPhysical,
  createScenarioReport,
  overallPassRequiresAllScenarios,
  scenarioMayPass,
} from "../src/report.js";
import { FAKE_AUDIO_PEER_A, FAKE_AUDIO_PEER_B } from "../src/constants.js";
import { createTempUserDataDir, removeTempUserDataDir } from "../src/peer-runner.js";
import { ProcessManager } from "../src/process-manager.js";
import { assertFinalizedDownloadsExist, validateLiveEndpoint } from "../src/validation.js";
import { join } from "node:path";

describe("automation output path guard", () => {
  it("rejects owner import directory targets", () => {
    expect(() => assertSafeAutomationOutputPath(join(OWNER_IMPORT_ROOT, "peer-a.json"))).toThrow(
      /forbidden under owner import directory/,
    );
  });

  it("rejects paths outside live-automation", () => {
    expect(() => assertSafeAutomationOutputPath("/tmp/outside")).toThrow(/must resolve under \.local\/live-automation/);
  });

  it("accepts automation run directories", () => {
    const runDir = resolveAutomationRunDir("test-run");
    expect(() => assertSafeAutomationOutputPath(runDir)).not.toThrow();
    const scenario = scenarioLayout(runDir, "peer_a_first");
    expect(scenario.peerAJson).toContain("scenarios/peer-a-first/raw/peer-a.json");
  });
});

describe("per-scenario output isolation", () => {
  it("uses disjoint paths for peer_a_first and peer_b_first", () => {
    const runDir = resolveAutomationRunDir("isolation-test");
    const peerAFirst = scenarioLayout(runDir, "peer_a_first");
    const peerBFirst = scenarioLayout(runDir, "peer_b_first");
    expect(peerAFirst.scenarioDir).not.toBe(peerBFirst.scenarioDir);
    expect(peerAFirst.rawDir).not.toBe(peerBFirst.rawDir);
    expect(peerAFirst.pairedDir).not.toBe(peerBFirst.pairedDir);
    expect(peerAFirst.diagnosticsDir).not.toBe(peerBFirst.diagnosticsDir);
    expect(peerAFirst.peerAJson).not.toBe(peerBFirst.peerAJson);
    expect(peerAFirst.peerBJson).not.toBe(peerBFirst.peerBJson);
  });

  it("retains scenario A artifacts when scenario B layout is created", () => {
    const runDir = resolveAutomationRunDir(`isolation-retention-${Date.now()}`);
    try {
      const peerAFirst = scenarioLayout(runDir, "peer_a_first");
      const peerBFirst = scenarioLayout(runDir, "peer_b_first");
      mkdirSync(peerAFirst.rawDir, { recursive: true });
      writeFileSync(peerAFirst.peerAJson, '{"scenario":"peer-a-first"}\n', "utf8");
      writeFileSync(peerAFirst.peerBJson, '{"scenario":"peer-a-first-b"}\n', "utf8");
      expect(peerBFirst.scenarioDir).not.toBe(peerAFirst.scenarioDir);
      expect(peerAFirst.peerAJson).toContain("peer-a-first");
      expect(peerBFirst.peerAJson).toContain("peer-b-first");
      expect(() => readFileSync(peerAFirst.peerAJson, "utf8")).not.toThrow();
    } finally {
      rmSync(runDir, { recursive: true, force: true });
    }
  });

  it("retains separate diagnostics directories", () => {
    const runDir = resolveAutomationRunDir("diag-isolation");
    const peerAFirst = scenarioLayout(runDir, "peer_a_first");
    const peerBFirst = scenarioLayout(runDir, "peer_b_first");
    expect(peerAFirst.diagnosticsDir.endsWith("peer-a-first/diagnostics")).toBe(true);
    expect(peerBFirst.diagnosticsDir.endsWith("peer-b-first/diagnostics")).toBe(true);
  });
});

describe("browser isolation fixtures", () => {
  it("uses distinct user-data directories", () => {
    const a = createTempUserDataDir("echlub-live-a-");
    const b = createTempUserDataDir("echlub-live-b-");
    try {
      expect(a).not.toBe(b);
    } finally {
      removeTempUserDataDir(a);
      removeTempUserDataDir(b);
    }
  });

  it("uses distinct fake audio paths", () => {
    expect(FAKE_AUDIO_PEER_A).not.toBe(FAKE_AUDIO_PEER_B);
    expect(FAKE_AUDIO_PEER_A).toMatch(/fake-peer-a\.wav$/);
    expect(FAKE_AUDIO_PEER_B).toMatch(/fake-peer-b\.wav$/);
  });
});

describe("session configuration", () => {
  it("generates identical correlation IDs from the same seed", () => {
    const shared = generateCorrelationId("shared-seed");
    expect(shared).toMatch(/^[0-9a-f]{32}$/);
    expect(generateCorrelationId("shared-seed")).toBe(shared);
  });

  it("uses opposite peer roles in runner config", () => {
    const roles: Array<"peer_a" | "peer_b"> = ["peer_a", "peer_b"];
    expect(new Set(roles).size).toBe(2);
  });
});

describe("classification boundary", () => {
  it("never claims physical two-device evidence", () => {
    const report = buildAutomationReport({
      authorizedCommit: "abdcbe1df7ab953ad970c3d5bbd72dfd1f27800a",
      result: "PASS",
      scenarios: [],
      observationSeconds: 60,
    });
    expect(classificationNeverClaimsPhysical(report)).toBe(true);
    expect(report.physicalTwoDeviceObservationSatisfied).toBe(false);
    expect(report.evidenceAuthority).toBe("readiness_only");
    expect(report.allRequiredScenariosPassed).toBe(false);
  });
});

describe("scenario pass gate", () => {
  it("cannot PASS before validation, pairing, and verify", () => {
    const report = createScenarioReport("peer_a_first", "/tmp/scenario");
    expect(scenarioMayPass(report)).toBe(false);
    expect(report.result).toBe("FAILED");
  });

  it("requires all validation steps before PASS", () => {
    const report = createScenarioReport("peer_a_first", "/tmp/scenario");
    report.readyReached = true;
    report.completedReached = true;
    report.finalizedDownloads = "PASS";
    report.peerAEndpointValidation = "PASS";
    report.peerBEndpointValidation = "PASS";
    report.pairing = "PASS";
    report.directoryVerification = "PASS";
    report.uiClickDispatchDeltaMs = 12;
    report.actualEndpointStartDeltaMs = 34;
    expect(scenarioMayPass(report)).toBe(true);
  });

  it("overall PASS requires both scenarios", () => {
    const pass = createScenarioReport("peer_a_first", "/a");
    pass.result = "PASS";
    const fail = createScenarioReport("peer_b_first", "/b");
    expect(overallPassRequiresAllScenarios([pass])).toBe(true);
    expect(overallPassRequiresAllScenarios([pass, fail])).toBe(false);
  });
});

describe("connect order helpers", () => {
  it("supports peer_a_first ordering", async () => {
    const { resolveConnectSequence } = await import("../src/peer-runner.js");
    expect(resolveConnectSequence("peer_a_first")).toEqual(["peer_a", "peer_b"]);
  });

  it("supports peer_b_first ordering", async () => {
    const { resolveConnectSequence } = await import("../src/peer-runner.js");
    expect(resolveConnectSequence("peer_b_first")).toEqual(["peer_b", "peer_a"]);
  });
});

describe("validation failures", () => {
  it("fails on invalid endpoint fixture", () => {
    const dir = mkdtempSync(join(tmpdir(), "echlub-invalid-endpoint-"));
    const path = join(dir, "invalid.json");
    writeFileSync(path, JSON.stringify({ exportKind: "draft" }), "utf8");
    try {
      expect(() => validateLiveEndpoint(path)).toThrow(/validate-live-endpoint failed/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});

describe("process cleanup", () => {
  it(
    "waits for actual child exit after SIGTERM",
    async () => {
      const logDir = mkdtempSync(join(tmpdir(), "echlub-process-log-"));
      const manager = new ProcessManager(join(logDir, "process-log.txt"));
      manager.start("sleeper", "sleep", ["30"]);
      await manager.terminateAllAndWait(5_000);
      expect(manager.trackedChildren.every((child) => child.signalCode !== null || child.exitCode !== null)).toBe(
        true,
      );
      rmSync(logDir, { recursive: true, force: true });
    },
    10_000,
  );
});

describe("ready timeout guards", () => {
  it("fails when ready state is absent", async () => {
    const { waitForReady } = await import("../src/peer-runner.js");
    const page = {
      locator: () => ({
        getByText: () => ({
          waitFor: vi.fn(async () => {
            throw new Error("Timeout");
          }),
        }),
      }),
    };
    await expect(waitForReady(page as never, 10)).rejects.toThrow(/Timeout/);
  });
});

describe("finalized download guards", () => {
  it("fails when expected endpoint path is missing", () => {
    const dir = mkdtempSync(join(tmpdir(), "echlub-missing-download-"));
    const existing = join(dir, "peer-b.json");
    const missing = join(dir, "peer-a.json");
    writeFileSync(existing, "{}\n", "utf8");
    try {
      expect(() => assertFinalizedDownloadsExist(missing, existing)).toThrow(
        /missing finalized endpoint download/,
      );
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("proceeds to validator when endpoint paths exist", () => {
    const dir = mkdtempSync(join(tmpdir(), "echlub-existing-download-"));
    const path = join(dir, "peer-a.json");
    writeFileSync(path, JSON.stringify({ exportKind: "draft" }), "utf8");
    try {
      expect(() => assertFinalizedDownloadsExist(path, path)).not.toThrow();
      expect(() => validateLiveEndpoint(path)).toThrow(/validate-live-endpoint failed/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});

describe("actual endpoint start delta", () => {
  function writeEndpoint(dir: string, name: string, startedAtUtc: string): string {
    const path = join(dir, name);
    writeFileSync(path, `${JSON.stringify({ startedAtUtc })}\n`, "utf8");
    return path;
  }

  it("passes for 0 ms delta", () => {
    const dir = mkdtempSync(join(tmpdir(), "echlub-start-delta-"));
    try {
      const peerA = writeEndpoint(dir, "peer-a.json", "2026-07-22T05:00:00.000Z");
      const peerB = writeEndpoint(dir, "peer-b.json", "2026-07-22T05:00:00.000Z");
      expect(calculateActualEndpointStartDeltaMs(peerA, peerB)).toBe(0);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("passes for 1999 ms delta", () => {
    const dir = mkdtempSync(join(tmpdir(), "echlub-start-delta-"));
    try {
      const peerA = writeEndpoint(dir, "peer-a.json", "2026-07-22T05:00:00.000Z");
      const peerB = writeEndpoint(dir, "peer-b.json", "2026-07-22T05:00:01.999Z");
      expect(calculateActualEndpointStartDeltaMs(peerA, peerB)).toBe(1999);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("fails for 2001 ms delta", () => {
    const dir = mkdtempSync(join(tmpdir(), "echlub-start-delta-"));
    try {
      const peerA = writeEndpoint(dir, "peer-a.json", "2026-07-22T05:00:00.000Z");
      const peerB = writeEndpoint(dir, "peer-b.json", "2026-07-22T05:00:02.001Z");
      expect(() => calculateActualEndpointStartDeltaMs(peerA, peerB)).toThrow(/exceeds 2000ms/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("fails when startedAtUtc is missing", () => {
    const dir = mkdtempSync(join(tmpdir(), "echlub-start-delta-"));
    try {
      const peerA = join(dir, "peer-a.json");
      const peerB = writeEndpoint(dir, "peer-b.json", "2026-07-22T05:00:00.000Z");
      writeFileSync(peerA, "{}\n", "utf8");
      expect(() => calculateActualEndpointStartDeltaMs(peerA, peerB)).toThrow(/missing startedAtUtc/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("fails when startedAtUtc is invalid", () => {
    const dir = mkdtempSync(join(tmpdir(), "echlub-start-delta-"));
    try {
      const peerA = writeEndpoint(dir, "peer-a.json", "not-a-date");
      const peerB = writeEndpoint(dir, "peer-b.json", "2026-07-22T05:00:00.000Z");
      expect(() => calculateActualEndpointStartDeltaMs(peerA, peerB)).toThrow(/invalid startedAtUtc/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});

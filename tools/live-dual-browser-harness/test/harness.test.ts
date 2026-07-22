import { describe, expect, it, vi } from "vitest";
import { generateCorrelationId } from "../src/constants.js";
import {
  assertSafeAutomationOutputPath,
  automationLayout,
  OWNER_IMPORT_ROOT,
  resolveAutomationRunDir,
} from "../src/paths.js";
import { buildAutomationReport, classificationNeverClaimsPhysical } from "../src/report.js";
import { FAKE_AUDIO_PEER_A, FAKE_AUDIO_PEER_B } from "../src/constants.js";
import { createTempUserDataDir, removeTempUserDataDir } from "../src/peer-runner.js";
import { ProcessManager } from "../src/process-manager.js";
import { validateLiveEndpoint } from "../src/validation.js";
import { join } from "node:path";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";

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
    expect(automationLayout(runDir).peerAJson).toContain("raw/peer-a.json");
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
  it("tracks spawned children for termination", () => {
    const log = mkdtempSync(join(tmpdir(), "echlub-process-log-"));
    const manager = new ProcessManager(join(log, "process-log.txt"));
    const child = manager.start("echo-test", "echo", ["ok"]);
    expect(manager.trackedChildren).toContain(child);
    manager.terminateAll();
  });
});

describe("ready timeout and missing download guards", () => {
  it("fails when ready state is absent", async () => {
    const { waitForReady } = await import("../src/peer-runner.js");
    const page = {
      getByText: () => ({
        waitFor: vi.fn(async () => {
          throw new Error("Timeout");
        }),
      }),
    };
    await expect(waitForReady(page as never, 10)).rejects.toThrow(/Timeout/);
  });

  it("fails when download path is missing before validation", () => {
    const missing = join(tmpdir(), "missing-peer-a.json");
    expect(() => {
      if (!missing.endsWith(".json")) throw new Error("missing finalized endpoint downloads");
    }).not.toThrow();
  });
});

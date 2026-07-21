import { describe, expect, it } from "vitest";
import {
  EVIDENCE_STATUS,
  PERFORMANCE_DISCLAIMER,
  SCHEMA_VERSION,
  observed,
  sanitizeForExport,
  unavailable,
} from "./types";

describe("performance types", () => {
  it("exports schema constants", () => {
    expect(SCHEMA_VERSION).toBe("PerformanceRunV1");
    expect(EVIDENCE_STATUS).toBe("exploratory_non_authoritative");
  });

  it("includes disclaimer text", () => {
    expect(PERFORMANCE_DISCLAIMER).toContain("mouth-to-ear");
    expect(PERFORMANCE_DISCLAIMER).toContain("transport selection");
  });

  it("observed metric has kind and value", () => {
    const m = observed(42);
    expect(m.kind).toBe("observed");
    expect(m.value).toBe(42);
  });

  it("sanitize removes forbidden keys", () => {
    const draft = {
      schemaVersion: SCHEMA_VERSION,
      evidenceLevel: "browser_synthetic_media_path",
      evidenceStatus: EVIDENCE_STATUS,
      sdp: "forbidden",
      metadata: {
        runId: "test",
        startedAt: "2026-01-01T00:00:00Z",
        completedAt: "2026-01-01T00:00:01Z",
        harnessVersion: "0.1.0",
        browserFamily: null,
        platform: null,
      },
      timing: { datachannelRttMs: unavailable("none") },
      syntheticPulse: {},
      transport: {},
    };
    const clean = sanitizeForExport(draft as never);
    expect((clean as unknown as Record<string, unknown>).sdp).toBeUndefined();
  });
});

export const PERFORMANCE_DISCLAIMER =
  "Exploratory non-authoritative performance observation. Not an acoustic mouth-to-ear measurement. Not a transport selection result.";

export const EVIDENCE_STATUS = "exploratory_non_authoritative" as const;
export const SCHEMA_VERSION = "PerformanceRunV1" as const;

export type MetricKind = "observed" | "unsupported" | "unavailable" | "invalid";

export interface MetricValue {
  kind: MetricKind;
  value?: number;
  reason?: string;
}

export interface PerformanceRunDraft {
  schemaVersion: typeof SCHEMA_VERSION;
  evidenceLevel: string;
  evidenceStatus: typeof EVIDENCE_STATUS;
  metadata: {
    runId: string;
    startedAt: string;
    completedAt: string;
    harnessVersion: string;
    browserFamily: string | null;
    platform: string | null;
  };
  timing: Record<string, MetricValue>;
  syntheticPulse: Record<string, MetricValue>;
  transport: Record<string, MetricValue>;
}

export function observed(value: number): MetricValue {
  return { kind: "observed", value };
}

export function unsupported(): MetricValue {
  return { kind: "unsupported" };
}

export function unavailable(reason: string): MetricValue {
  return { kind: "unavailable", reason };
}

export function sanitizeForExport(draft: PerformanceRunDraft): PerformanceRunDraft {
  const clone = structuredClone(draft) as PerformanceRunDraft & Record<string, unknown>;
  delete clone.sdp;
  delete clone.ice;
  delete clone.deviceId;
  return clone;
}

export function downloadJson(filename: string, data: unknown): void {
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

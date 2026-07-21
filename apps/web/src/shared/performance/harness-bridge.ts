import { buildPerformanceRunDraft, runSyntheticLoopback } from "../../adapters/webrtc/synthetic/loopback";
import type { PerformanceRunDraft } from "./types";

declare global {
  interface Window {
    __echlubSyntheticHarness?: () => Promise<PerformanceRunDraft & { outcome?: string }>;
  }
}

window.__echlubSyntheticHarness = async () => {
  const runId =
    typeof window !== "undefined" &&
    (window as unknown as { __ECHLUB_RUN_ID__?: string }).__ECHLUB_RUN_ID__
      ? (window as unknown as { __ECHLUB_RUN_ID__: string }).__ECHLUB_RUN_ID__
      : undefined;

  const result = await runSyntheticLoopback({ runId });
  const draft = buildPerformanceRunDraft(result) as PerformanceRunDraft & {
    outcome?: string;
    limitationReason?: string;
    pulseRecords?: unknown;
    decodedPulseDetectorUsed?: boolean;
  };
  draft.outcome = result.outcome;
  if (result.limitationReason) draft.limitationReason = result.limitationReason;
  draft.pulseRecords = result.syntheticPulse.pulseRecords;
  draft.decodedPulseDetectorUsed = result.decodedPulseDetectorUsed;
  return draft;
};

export {};

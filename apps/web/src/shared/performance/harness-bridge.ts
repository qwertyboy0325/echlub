import { buildPerformanceRunDraft, runSyntheticLoopback } from "../../adapters/webrtc/synthetic/loopback";
import type { PerformanceRunDraft } from "./types";

declare global {
  interface Window {
    __echlubSyntheticHarness?: () => Promise<PerformanceRunDraft>;
  }
}

window.__echlubSyntheticHarness = async () => {
  const result = await runSyntheticLoopback();
  return buildPerformanceRunDraft(result);
};

export {};

import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mockStartObservation = vi.fn();
const mockPrepare = vi.fn();
const mockEnableMicrophone = vi.fn().mockResolvedValue(undefined);
const mockConnect = vi.fn().mockResolvedValue(undefined);
const mockStop = vi.fn();
let phaseCallback: ((phase: string) => void) | null = null;

vi.mock("../../adapters/webrtc/live/session", () => ({
  LiveWebRtcSession: vi.fn().mockImplementation((_config, callbacks) => {
    phaseCallback = callbacks.onPhaseChange;
    return {
      prepare: mockPrepare,
      enableMicrophone: mockEnableMicrophone,
      connect: mockConnect,
      startObservation: mockStartObservation,
      stop: mockStop,
      exportFinalizedEndpoint: vi.fn(),
      exportEndpointDraft: vi.fn(),
    };
  }),
}));

import { LivePerformancePanel } from "./LivePerformancePanel";

function prepareFlow() {
  fireEvent.click(screen.getByRole("checkbox"));
  fireEvent.click(screen.getByRole("button", { name: /^Prepare$/i }));
  fireEvent.click(screen.getByRole("button", { name: /Enable Microphone/i }));
  fireEvent.click(screen.getByRole("button", { name: /^Connect$/i }));
}

describe("LivePerformancePanel", () => {
  beforeEach(() => {
    mockStartObservation.mockReset();
    mockPrepare.mockReset();
    mockEnableMicrophone.mockReset();
    mockConnect.mockReset();
    mockStop.mockReset();
    phaseCallback = null;
    vi.useRealTimers();
  });

  it("shows privacy disclaimer", () => {
    render(<LivePerformancePanel />);
    expect(screen.getByText(/Use headphones/i)).toBeInTheDocument();
    expect(screen.getByText(/not an acoustic mouth-to-ear measurement/i)).toBeInTheDocument();
  });

  it("requires headphone acknowledgement before microphone", () => {
    render(<LivePerformancePanel />);
    const micButton = screen.getByRole("button", { name: /Enable Microphone/i });
    expect(micButton).toBeDisabled();
  });

  it("enables microphone after acknowledgement and prepare", () => {
    render(<LivePerformancePanel />);
    fireEvent.click(screen.getByRole("checkbox"));
    fireEvent.click(screen.getByRole("button", { name: /^Prepare$/i }));
    const micButton = screen.getByRole("button", { name: /Enable Microphone/i });
    expect(micButton).not.toBeDisabled();
  });

  it("disables finalized export until completed phase", () => {
    render(<LivePerformancePanel />);
    expect(screen.getByRole("button", { name: /Export Finalized Endpoint/i })).toBeDisabled();
  });

  it("disables start observation before genuine readiness", () => {
    render(<LivePerformancePanel />);
    prepareFlow();
    expect(screen.getByRole("button", { name: /Start 60s Observation/i })).toBeDisabled();
  });

  it("enables start observation only in ready_to_observe phase", () => {
    render(<LivePerformancePanel />);
    prepareFlow();
    act(() => {
      phaseCallback?.("ready_to_observe");
    });
    expect(screen.getByRole("button", { name: /Start 60s Observation/i })).not.toBeDisabled();
  });

  it("displays error and clears timer when observation start is rejected", async () => {
    vi.useFakeTimers();
    mockStartObservation.mockRejectedValue(new Error("not ready to observe"));
    render(<LivePerformancePanel />);
    prepareFlow();
    act(() => {
      phaseCallback?.("ready_to_observe");
    });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /Start 60s Observation/i }));
      await Promise.resolve();
    });
    expect(screen.getByText(/not ready to observe/i)).toBeInTheDocument();
    await act(async () => {
      vi.advanceTimersByTime(5000);
    });
    expect(screen.getByText(/Elapsed: 0s/i)).toBeInTheDocument();
  });

  it("clears timer after successful observation start completes", async () => {
    vi.useFakeTimers();
    mockStartObservation.mockImplementation(async () => {
      await new Promise((resolve) => setTimeout(resolve, 100));
    });
    render(<LivePerformancePanel />);
    prepareFlow();
    act(() => {
      phaseCallback?.("ready_to_observe");
    });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /Start 60s Observation/i }));
    });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(100);
    });
    await act(async () => {
      vi.advanceTimersByTime(5000);
    });
    expect(screen.getByText(/Elapsed: 0s/i)).toBeInTheDocument();
  });

  it("keeps phase and elapsed display coherent after rejection", async () => {
    mockStartObservation.mockRejectedValue(new Error("not ready to observe"));
    render(<LivePerformancePanel />);
    prepareFlow();
    act(() => {
      phaseCallback?.("ready_to_observe");
    });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: /Start 60s Observation/i }));
    });
    await waitFor(() => {
      expect(screen.getByText(/Phase: Ready To Observe/i)).toBeInTheDocument();
      expect(screen.getByText(/Elapsed: 0s/i)).toBeInTheDocument();
    });
  });
});

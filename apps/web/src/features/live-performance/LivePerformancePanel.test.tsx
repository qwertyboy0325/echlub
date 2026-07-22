import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  generateSessionCorrelationId,
  normalizeSessionCorrelationId,
} from "../../adapters/webrtc/live/types";

const mockStartObservation = vi.fn();
const mockPrepare = vi.fn();
const mockEnableMicrophone = vi.fn().mockResolvedValue(undefined);
const mockConnect = vi.fn().mockResolvedValue(undefined);
const mockStop = vi.fn();
const mockSessionConstructor = vi.fn();
let phaseCallback: ((phase: string) => void) | null = null;

vi.mock("../../adapters/webrtc/live/session", () => ({
  LiveWebRtcSession: vi.fn().mockImplementation((config, callbacks) => {
    mockSessionConstructor(config);
    phaseCallback = callbacks.onPhaseChange;
    return {
      prepare: () => {
        mockPrepare();
        callbacks.onPhaseChange("prepared");
      },
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

const SHARED_ID = "0123456789abcdef0123456789abcdef";
const UPPERCASE_ID = "0123456789ABCDEF0123456789ABCDEF";

function correlationInput() {
  return screen.getByLabelText(/Session correlation ID/i);
}

function prepareOnly() {
  fireEvent.click(screen.getByRole("button", { name: /^Prepare$/i }));
}

function prepareFlow() {
  fireEvent.click(screen.getByRole("checkbox"));
  prepareOnly();
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
    mockSessionConstructor.mockReset();
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
    prepareOnly();
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

describe("LivePerformancePanel correlation ID setup", () => {
  beforeEach(() => {
    mockSessionConstructor.mockReset();
    mockPrepare.mockReset();
    phaseCallback = null;
  });

  it("accepts a valid copied correlation ID on the second device", () => {
    render(<LivePerformancePanel />);
    fireEvent.change(correlationInput(), { target: { value: SHARED_ID } });
    expect(correlationInput()).toHaveValue(SHARED_ID);
    expect(screen.getByRole("button", { name: /^Prepare$/i })).not.toBeDisabled();
  });

  it("normalizes uppercase correlation IDs on Prepare", () => {
    render(<LivePerformancePanel />);
    fireEvent.change(correlationInput(), { target: { value: UPPERCASE_ID } });
    prepareOnly();
    expect(mockSessionConstructor.mock.calls[0]?.[0]).toMatchObject({
      sessionCorrelationId: SHARED_ID,
    });
    expect(correlationInput()).toHaveValue(SHARED_ID);
  });

  it.each([
    ["31 chars", "0123456789abcdef0123456789abcde"],
    ["33 chars", "0123456789abcdef0123456789abcdef0"],
    ["non-hex", "0123456789abcdef0123456789abcdeg"],
  ])("rejects invalid correlation ID: %s", (_label, value) => {
    render(<LivePerformancePanel />);
    fireEvent.change(correlationInput(), { target: { value } });
    expect(screen.getByRole("button", { name: /^Prepare$/i })).toBeDisabled();
    expect(screen.getByText(/exactly 32 hexadecimal characters/i)).toBeInTheDocument();
    prepareOnly();
    expect(mockSessionConstructor).not.toHaveBeenCalled();
  });

  it("passes the exact validated shared ID to LiveWebRtcSession", () => {
    render(<LivePerformancePanel />);
    fireEvent.change(correlationInput(), { target: { value: SHARED_ID } });
    prepareOnly();
    expect(mockSessionConstructor).toHaveBeenCalledTimes(1);
    expect(mockSessionConstructor.mock.calls[0]?.[0]).toMatchObject({
      sessionCorrelationId: SHARED_ID,
    });
  });

  it("freezes correlation ID after Prepare", () => {
    render(<LivePerformancePanel />);
    prepareOnly();
    expect(correlationInput()).toHaveAttribute("readonly");
    expect(screen.getByRole("button", { name: /^Generate$/i })).toBeDisabled();
    fireEvent.change(correlationInput(), { target: { value: "ffffffffffffffffffffffffffffffff" } });
    expect(correlationInput()).not.toHaveValue("ffffffffffffffffffffffffffffffff");
  });

  it("returns to editable idle state after Reset", () => {
    render(<LivePerformancePanel />);
    prepareOnly();
    fireEvent.click(screen.getByRole("button", { name: /^Reset$/i }));
    expect(correlationInput()).not.toHaveAttribute("readonly");
    expect(screen.getByRole("button", { name: /^Generate$/i })).not.toBeDisabled();
    expect(screen.getByText(/Phase: Idle/i)).toBeInTheDocument();
  });
});

describe("LivePerformancePanel configuration freeze", () => {
  beforeEach(() => {
    mockSessionConstructor.mockReset();
    phaseCallback = null;
  });

  it("freezes role, signaling URL, and capture profile after Prepare", () => {
    render(<LivePerformancePanel />);
    const roleSelect = screen.getByLabelText(/^Role$/i);
    const signalingInput = screen.getByLabelText(/Signaling URL/i);
    const profileSelect = screen.getByLabelText(/Capture profile/i);

    fireEvent.change(signalingInput, { target: { value: "ws://example.test/ws" } });
    fireEvent.change(roleSelect, { target: { value: "peer_b" } });
    fireEvent.change(profileSelect, { target: { value: "music_low_latency" } });
    prepareOnly();

    expect(roleSelect).toBeDisabled();
    expect(signalingInput).toHaveAttribute("readonly");
    expect(profileSelect).toBeDisabled();
  });

  it("creates the session exactly once with the displayed configuration", () => {
    render(<LivePerformancePanel />);
    fireEvent.change(correlationInput(), { target: { value: SHARED_ID } });
    fireEvent.change(screen.getByLabelText(/Signaling URL/i), {
      target: { value: "ws://127.0.0.1:9090/v1/signaling/ws" },
    });
    fireEvent.change(screen.getByLabelText(/^Role$/i), { target: { value: "peer_b" } });
    fireEvent.change(screen.getByLabelText(/Capture profile/i), {
      target: { value: "music_low_latency" },
    });
    prepareOnly();
    expect(mockSessionConstructor).toHaveBeenCalledTimes(1);
    expect(mockSessionConstructor.mock.calls[0]?.[0]).toMatchObject({
      sessionCorrelationId: SHARED_ID,
      localPeerId: "peer_b",
      signalingUrl: "ws://127.0.0.1:9090/v1/signaling/ws",
      captureProfile: "music_low_latency",
    });
  });

  it("restores editable configuration after Reset", () => {
    render(<LivePerformancePanel />);
    prepareOnly();
    fireEvent.click(screen.getByRole("button", { name: /^Reset$/i }));
    expect(screen.getByLabelText(/^Role$/i)).not.toBeDisabled();
    expect(screen.getByLabelText(/Signaling URL/i)).not.toHaveAttribute("readonly");
    expect(screen.getByLabelText(/Capture profile/i)).not.toBeDisabled();
  });
});

describe("session correlation ID helpers", () => {
  it("generates valid lowercase IDs", () => {
    const id = generateSessionCorrelationId();
    expect(normalizeSessionCorrelationId(id)).toBe(id);
  });
});

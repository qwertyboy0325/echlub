import { afterEach, describe, expect, it, vi } from "vitest";
import { SignalingClient } from "./client";

class MockWebSocket {
  static instances: MockWebSocket[] = [];
  readyState = WebSocket.OPEN;
  onmessage: ((event: { data: string }) => void) | null = null;
  onopen: (() => void) | null = null;
  onerror: (() => void) | null = null;

  constructor(_url: string) {
    MockWebSocket.instances.push(this);
  }

  send = vi.fn();
  close = vi.fn();
}

describe("SignalingClient peer discovery", () => {
  afterEach(() => {
    MockWebSocket.instances = [];
    vi.unstubAllGlobals();
  });

  it("notifies joiners about other peers but not themselves", () => {
    vi.stubGlobal("WebSocket", MockWebSocket as unknown as typeof WebSocket);

    const onPeerJoined = vi.fn();
    const client = new SignalingClient({
      sessionId: "0123456789abcdef0123456789abcdef",
      peerId: "peer_a",
      onMessage: vi.fn(),
      onPeerJoined,
    });
    client.connect();
    const ws = MockWebSocket.instances[0]!;

    ws.onmessage?.({
      data: JSON.stringify({ type: "peer_joined", peer_id: "peer_b" }),
    });
    expect(onPeerJoined).toHaveBeenCalledWith("peer_b");

    onPeerJoined.mockClear();
    ws.onmessage?.({
      data: JSON.stringify({ type: "peer_joined", peer_id: "peer_a" }),
    });
    expect(onPeerJoined).not.toHaveBeenCalled();
  });
});

export const DEFAULT_SIGNALING_URL = "ws://127.0.0.1:8080/v1/signaling/ws";

export interface SignalingClientOptions {
  sessionId: string;
  peerId: string;
  url?: string;
  onMessage: (payload: unknown) => void;
  onPeerJoined?: (peerId: string) => void;
  onPeerLeft?: (peerId: string) => void;
  onError?: (error: string) => void;
}

export class SignalingClient {
  private ws: WebSocket | null = null;
  private readonly options: SignalingClientOptions;

  constructor(options: SignalingClientOptions) {
    this.options = options;
  }

  connect(): void {
    const base = this.options.url ?? DEFAULT_SIGNALING_URL;
    const url = `${base}?session_id=${encodeURIComponent(this.options.sessionId)}&peer_id=${encodeURIComponent(this.options.peerId)}`;
    this.ws = new WebSocket(url);

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data as string) as {
          type: string;
          peer_id?: string;
          from?: string;
          payload?: unknown;
        };
        switch (msg.type) {
          case "peer_joined":
            this.options.onPeerJoined?.(msg.peer_id ?? "");
            break;
          case "peer_left":
            this.options.onPeerLeft?.(msg.peer_id ?? "");
            break;
          case "relay":
            this.options.onMessage(msg.payload ?? msg);
            break;
          default:
            this.options.onMessage(msg);
        }
      } catch {
        this.options.onError?.("failed to parse signaling message");
      }
    };

    this.ws.onerror = () => {
      this.options.onError?.("signaling connection error");
    };
  }

  relay(payload: unknown): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: "relay", payload }));
    }
  }

  disconnect(): void {
    this.ws?.close();
    this.ws = null;
  }
}

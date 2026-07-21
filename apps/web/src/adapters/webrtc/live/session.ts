export interface LiveSessionConfig {
  sessionId: string;
  localPeerId: string;
  remotePeerId: string;
  signalingUrl?: string;
}

export interface LiveSessionCallbacks {
  onConnectionState: (state: RTCPeerConnectionState) => void;
  onRemoteStream: (stream: MediaStream) => void;
  onDataChannelMessage: (data: string) => void;
  onClockProbe?: (rttMs: number) => void;
  onError: (message: string) => void;
}

export class LiveWebRtcSession {
  private pc: RTCPeerConnection | null = null;
  private dc: RTCDataChannel | null = null;
  private localStream: MediaStream | null = null;
  private signaling: import("../../signaling/client").SignalingClient | null = null;
  private readonly config: LiveSessionConfig;
  private readonly callbacks: LiveSessionCallbacks;
  private isInitiator = false;

  constructor(config: LiveSessionConfig, callbacks: LiveSessionCallbacks) {
    this.config = config;
    this.callbacks = callbacks;
  }

  async start(): Promise<void> {
    this.pc = new RTCPeerConnection({ iceServers: [] });
    this.pc.onconnectionstatechange = () => {
      this.callbacks.onConnectionState(this.pc!.connectionState);
    };
    this.pc.ontrack = (event) => {
      this.callbacks.onRemoteStream(event.streams[0]);
    };
    this.pc.ondatachannel = (event) => {
      this.setupDataChannel(event.channel);
    };

    this.dc = this.pc.createDataChannel("clock-probe");
    this.setupDataChannel(this.dc);

    const { SignalingClient } = await import("../../signaling/client");
    this.signaling = new SignalingClient({
      sessionId: this.config.sessionId,
      peerId: this.config.localPeerId,
      url: this.config.signalingUrl,
      onMessage: (payload) => this.handleSignaling(payload),
      onPeerJoined: (peerId) => {
        if (peerId !== this.config.localPeerId && !this.isInitiator) {
          this.isInitiator = true;
          void this.createOffer();
        }
      },
      onError: this.callbacks.onError,
    });
    this.signaling.connect();
  }

  async enableMicrophone(): Promise<void> {
    this.localStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
    for (const track of this.localStream.getTracks()) {
      this.pc?.addTrack(track, this.localStream);
    }
  }

  sendClockProbe(): void {
    if (this.dc?.readyState === "open") {
      this.dc.send(JSON.stringify({ type: "clock_probe", sentAt: performance.now() }));
    }
  }

  private setupDataChannel(channel: RTCDataChannel): void {
    this.dc = channel;
    channel.onmessage = (event) => {
      this.callbacks.onDataChannelMessage(event.data as string);
      try {
        const msg = JSON.parse(event.data as string) as { type: string; sentAt?: number };
        if (msg.type === "clock_probe" && msg.sentAt !== undefined) {
          const rtt = performance.now() - msg.sentAt;
          this.callbacks.onClockProbe?.(rtt);
          channel.send(JSON.stringify({ type: "clock_pong", sentAt: msg.sentAt }));
        } else if (msg.type === "clock_pong" && msg.sentAt !== undefined) {
          const rtt = performance.now() - msg.sentAt;
          this.callbacks.onClockProbe?.(rtt);
        }
      } catch {
        /* non-json messages ignored */
      }
    };
  }

  private async handleSignaling(payload: unknown): Promise<void> {
    const msg = payload as { type?: string; sdp?: RTCSessionDescriptionInit; candidate?: RTCIceCandidateInit };
    if (!this.pc) return;

    if (msg.type === "offer" && msg.sdp) {
      await this.pc.setRemoteDescription(msg.sdp);
      const answer = await this.pc.createAnswer();
      await this.pc.setLocalDescription(answer);
      this.signaling?.relay({ type: "answer", sdp: this.pc.localDescription });
    } else if (msg.type === "answer" && msg.sdp) {
      await this.pc.setRemoteDescription(msg.sdp);
    } else if (msg.type === "ice_candidate" && msg.candidate) {
      await this.pc.addIceCandidate(msg.candidate);
    }
  }

  private async createOffer(): Promise<void> {
    if (!this.pc) return;
    const offer = await this.pc.createOffer();
    await this.pc.setLocalDescription(offer);
    this.pc.onicecandidate = (event) => {
      if (event.candidate) {
        this.signaling?.relay({ type: "ice_candidate", candidate: event.candidate });
      }
    };
    this.signaling?.relay({ type: "offer", sdp: this.pc.localDescription });
  }

  stop(): void {
    this.localStream?.getTracks().forEach((t) => t.stop());
    this.signaling?.disconnect();
    this.pc?.close();
    this.pc = null;
  }
}

import type { SignalingEnvelope } from "./types";

export interface NegotiationState {
  makingOffer: boolean;
  ignoreOffer: boolean;
  isSettingRemoteAnswerPending: boolean;
  polite: boolean;
}

export function createNegotiationState(polite: boolean): NegotiationState {
  return {
    makingOffer: false,
    ignoreOffer: false,
    isSettingRemoteAnswerPending: false,
    polite,
  };
}

export async function applySignalingDescription(
  pc: RTCPeerConnection,
  state: NegotiationState,
  envelope: SignalingEnvelope,
  sendDescription: (type: RTCSdpType, sdp: RTCSessionDescriptionInit) => void,
): Promise<void> {
  if (envelope.type === "description" && envelope.sdp) {
    const description = envelope.sdp;
    const readyForOffer =
      !state.makingOffer && (pc.signalingState === "stable" || state.isSettingRemoteAnswerPending);
    const offerCollision = description.type === "offer" && !readyForOffer;

    state.ignoreOffer = !state.polite && offerCollision;
    if (state.ignoreOffer) return;

    state.isSettingRemoteAnswerPending = description.type === "answer";
    await pc.setRemoteDescription(description);
    state.isSettingRemoteAnswerPending = false;

    if (description.type === "offer") {
      const answer = await pc.createAnswer();
      await pc.setLocalDescription(answer);
      if (pc.localDescription) {
        sendDescription("answer", pc.localDescription);
      }
    }
  }
}

export async function createAndSendOffer(
  pc: RTCPeerConnection,
  state: NegotiationState,
  sendDescription: (type: RTCSdpType, sdp: RTCSessionDescriptionInit) => void,
): Promise<void> {
  state.makingOffer = true;
  try {
    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);
    if (pc.localDescription) {
      sendDescription("offer", pc.localDescription);
    }
  } finally {
    state.makingOffer = false;
  }
}

export class IceCandidateBuffer {
  private readonly buffer: RTCIceCandidateInit[] = [];
  private applied = new Set<string>();

  add(candidate: RTCIceCandidateInit): void {
    const key = `${candidate.sdpMid}:${candidate.sdpMLineIndex}:${candidate.candidate ?? ""}`;
    if (this.applied.has(key)) return;
    this.buffer.push(candidate);
  }

  async flush(pc: RTCPeerConnection): Promise<void> {
    if (!pc.remoteDescription) return;
    for (const candidate of this.buffer) {
      const key = `${candidate.sdpMid}:${candidate.sdpMLineIndex}:${candidate.candidate ?? ""}`;
      if (this.applied.has(key)) continue;
      try {
        await pc.addIceCandidate(candidate);
        this.applied.add(key);
      } catch {
        /* ignore invalid duplicate */
      }
    }
    this.buffer.length = 0;
  }

  clear(): void {
    this.buffer.length = 0;
    this.applied.clear();
  }
}

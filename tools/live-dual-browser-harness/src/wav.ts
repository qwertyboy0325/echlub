import { writeFileSync } from "node:fs";
import { FAKE_AUDIO_PEER_A, FAKE_AUDIO_PEER_B, FIXTURES_DIR } from "./constants.js";
import { ensureDir } from "./constants.js";

const SAMPLE_RATE_HZ = 48_000;
const CHANNELS = 1;
const BIT_DEPTH = 16;
const DURATION_SECONDS = 90;

function writeWavHeader(buffer: Buffer, dataBytes: number): void {
  buffer.write("RIFF", 0);
  buffer.writeUInt32LE(36 + dataBytes, 4);
  buffer.write("WAVE", 8);
  buffer.write("fmt ", 12);
  buffer.writeUInt32LE(16, 16);
  buffer.writeUInt16LE(1, 20);
  buffer.writeUInt16LE(CHANNELS, 22);
  buffer.writeUInt32LE(SAMPLE_RATE_HZ, 24);
  buffer.writeUInt32LE((SAMPLE_RATE_HZ * CHANNELS * BIT_DEPTH) / 8, 28);
  buffer.writeUInt16LE((CHANNELS * BIT_DEPTH) / 8, 32);
  buffer.writeUInt16LE(BIT_DEPTH, 34);
  buffer.write("data", 36);
  buffer.writeUInt32LE(dataBytes, 40);
}

export function generateToneWav(frequencyHz: number, outputPath: string): void {
  const sampleCount = SAMPLE_RATE_HZ * DURATION_SECONDS;
  const dataBytes = sampleCount * 2;
  const buffer = Buffer.alloc(44 + dataBytes);
  writeWavHeader(buffer, dataBytes);

  for (let i = 0; i < sampleCount; i += 1) {
    const t = i / SAMPLE_RATE_HZ;
    const pulse = Math.sin(2 * Math.PI * 2 * t) > 0 ? 1 : 0;
    const sample = Math.max(
      -32767,
      Math.min(32767, Math.round(0.3 * 32767 * pulse * Math.sin(2 * Math.PI * frequencyHz * t))),
    );
    buffer.writeInt16LE(sample, 44 + i * 2);
  }

  writeFileSync(outputPath, buffer);
}

export function ensureFakeAudioFixtures(): { peerA: string; peerB: string } {
  ensureDir(FIXTURES_DIR);
  generateToneWav(697, FAKE_AUDIO_PEER_A);
  generateToneWav(1209, FAKE_AUDIO_PEER_B);
  return { peerA: FAKE_AUDIO_PEER_A, peerB: FAKE_AUDIO_PEER_B };
}

export const WAV_SPEC = {
  format: "PCM WAV",
  sampleRateHz: SAMPLE_RATE_HZ,
  channels: CHANNELS,
  bitDepth: BIT_DEPTH,
  durationSeconds: DURATION_SECONDS,
};

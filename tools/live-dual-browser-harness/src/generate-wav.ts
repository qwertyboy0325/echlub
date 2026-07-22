#!/usr/bin/env tsx
import { ensureFakeAudioFixtures } from "./wav.js";

const fixtures = ensureFakeAudioFixtures();
console.log(`Generated ${fixtures.peerA}`);
console.log(`Generated ${fixtures.peerB}`);

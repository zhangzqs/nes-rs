import { test } from "node:test";
import assert from "node:assert/strict";
import { AudioPlayback, AUDIO_LEAD, SAMPLE_RATE } from "../audio.js";

class AudioContextStub {
  currentTime = 0;
  sampleRate = 48000;
  state = "suspended";
  scheduled = [];
  createGain() { return { connect() {}, gain: { value: 1, setTargetAtTime() {} } }; }
  createBuffer(channels, length, rate) {
    assert.equal(channels, 1);
    assert.equal(rate, SAMPLE_RATE);
    return { duration: length / rate, copyToChannel() {} };
  }
  createBufferSource() {
    const owner = this;
    return { connect() {}, disconnect() {}, stop() { this.stopped = true; },
      start(time) { owner.scheduled.push({ time, duration: this.buffer.duration, source: this }); } };
  }
  async resume() { this.state = "running"; }
  async suspend() { this.state = "suspended"; }
}

test("five minutes of variable NES frame lengths stay contiguous on a 48 kHz device", () => {
  const context = new AudioContextStub();
  const playback = new AudioPlayback(context);
  let frames = 0;
  let consumed = 0;
  // Poll at irregular animation intervals while the device consumes real time.
  while (context.currentTime < 300) {
    while (playback.buffered < AUDIO_LEAD) {
      playback.enqueue(new Float32Array(++frames % 3 === 0 ? 733 : 734));
    }
    context.currentTime += frames % 7 === 0 ? 0.035 : 0.016;
    while (consumed < context.scheduled.length) {
      const item = context.scheduled[consumed];
      if (item.time + item.duration > context.currentTime) break;
      item.source.onended?.();
      consumed++;
    }
  }
  for (let i = 1; i < context.scheduled.length; i++) {
    const previous = context.scheduled[i - 1];
    assert.ok(Math.abs(context.scheduled[i].time - previous.time - previous.duration) < 1e-10);
  }
  assert.equal(playback.underruns, 0);
  assert.ok(playback.sources.size < 10, "completed sources must be released");
});

test("pause cancels queued sound and resume starts with fresh timing", async () => {
  const context = new AudioContextStub();
  const playback = new AudioPlayback(context);
  await playback.resume();
  playback.enqueue(new Float32Array(735));
  await playback.pause();
  assert.equal(context.state, "suspended");
  assert.equal(playback.sources.size, 0);
  assert.ok(context.scheduled.every((item) => item.source.stopped));
  context.currentTime = 10;
  await playback.resume();
  playback.enqueue(new Float32Array(735));
  assert.equal(playback.underruns, 0);
  assert.equal(context.scheduled.at(-1).time, 10.04);
});

test("a genuine scheduling stall re-primes the buffer without replaying a backlog", () => {
  const context = new AudioContextStub();
  const playback = new AudioPlayback(context);
  playback.enqueue(new Float32Array(735));
  context.currentTime = 2;
  playback.enqueue(new Float32Array(735));
  assert.equal(playback.underruns, 1);
  assert.equal(context.scheduled.at(-1).time, 2.04);
  assert.ok(playback.buffered < AUDIO_LEAD);
});

export const SAMPLE_RATE = 44100;
export const AUDIO_LEAD = 0.12;

// Schedule against the audio device clock, never a 60 Hz wall-clock timer.
// AudioBuffer's source rate also lets the browser resample to 48 kHz devices.
export class AudioPlayback {
  constructor(context = new AudioContext({ latencyHint: "interactive" })) {
    this.context = context;
    this.gain = context.createGain();
    this.gain.connect(context.destination);
    this.gain.gain.value = 0.65;
    this.sources = new Set();
    this.nextTime = 0;
    this.underruns = 0;
  }

  get buffered() {
    return Math.max(0, this.nextTime - this.context.currentTime);
  }

  async resume() {
    await this.context.resume();
  }

  enqueue(samples) {
    if (!samples.length) return;
    const now = this.context.currentTime;
    if (this.nextTime < now || this.nextTime === 0) {
      if (this.nextTime > 0) this.underruns++;
      this.nextTime = now + 0.04;
    }
    const buffer = this.context.createBuffer(1, samples.length, SAMPLE_RATE);
    buffer.copyToChannel(samples, 0);
    const source = this.context.createBufferSource();
    source.buffer = buffer;
    source.connect(this.gain);
    this.sources.add(source);
    source.onended = () => {
      this.sources.delete(source);
      source.disconnect();
    };
    source.start(this.nextTime);
    this.nextTime += samples.length / SAMPLE_RATE;
  }

  clear() {
    for (const source of this.sources) {
      source.onended = null;
      source.stop();
      source.disconnect();
    }
    this.sources.clear();
    this.nextTime = 0;
  }

  async pause() {
    this.clear();
    await this.context.suspend();
  }

  setVolume(volume) {
    this.gain.gain.setTargetAtTime(volume, this.context.currentTime, 0.01);
  }
}

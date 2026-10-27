import init, { Emulator } from "./pkg/nes_web.js";

const ready = init();
let emulator;
let frames = 0;

self.onmessage = async ({ data }) => {
  try {
    await ready;
    if (data.type === "load") {
      emulator = new Emulator(data.rom);
      if (data.save?.length) emulator.load_save_ram(data.save);
      self.postMessage({ type: "ready", mapper: emulator.mapper() });
    } else if (data.type === "frame") {
      emulator.buttons(0, data.pads[0]);
      emulator.buttons(1, data.pads[1]);
      for (let i = 0; i < data.count; i++) {
        const pixels = emulator.frame();
        const audio = emulator.audio();
        const save = ++frames % 60 === 0 ? emulator.save_ram() : null;
        self.postMessage({ type: "frame", epoch: data.epoch, pixels, audio, save },
          [pixels.buffer, audio.buffer]);
      }
    } else if (data.type === "reset") {
      emulator.reset();
      frames = 0;
      self.postMessage({ type: "ready", mapper: emulator.mapper() });
    }
  } catch (error) {
    self.postMessage({ type: "error", message: String(error) });
  }
};

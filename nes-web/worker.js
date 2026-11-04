import init, { Emulator } from "./pkg/nes_wasm.js";

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
    } else if (data.type === "memory-snapshot") {
      const bytes = emulator.ram_snapshot();
      self.postMessage({ type: "memory-snapshot", requestId: data.requestId, bytes }, [bytes.buffer]);
    } else if (data.type === "memory-write") {
      if (!Number.isInteger(data.address) || data.address < 0 || data.address >= 0x800 ||
          !Number.isInteger(data.value) || data.value < 0 || data.value > 255) {
        throw new Error("内存地址或数值超出范围");
      }
      emulator.write_ram(data.address, data.value);
      self.postMessage({ type: "memory-written", requestId: data.requestId, address: data.address, value: data.value });
    }
  } catch (error) {
    self.postMessage({ type: data.type.startsWith("memory-") ? "memory-error" : "error",
      requestId: data.requestId, message: String(error) });
  }
};

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import init, { Emulator } from "../pkg/nes_web.js";

await init({ module_or_path: await readFile(new URL("../pkg/nes_web_bg.wasm", import.meta.url)) });

function rom() {
  const bytes = new Uint8Array(16 + 0x8000);
  bytes.set([0x4e, 0x45, 0x53, 0x1a, 2, 0, 0x12]); // MMC1 with battery RAM.
  bytes.set([0xad, 0, 0x60, 0x18, 0x69, 1, 0x8d, 0, 0x60, 0x4c, 0x09, 0x80], 16);
  bytes.set([0x00, 0x80], 16 + 0x7ffc);
  return bytes;
}

test("the actual WASM core renders RGBA and preserves battery RAM across reloads", () => {
  const first = new Emulator(rom());
  assert.equal(first.mapper(), 1);
  let sampleCount = 0;
  for (let i = 0; i < 120; i++) {
    const image = first.frame();
    assert.equal(image.length, 256 * 240 * 4);
    assert.equal(image[3], 255);
    const audio = first.audio();
    assert.ok(audio.every(Number.isFinite));
    sampleCount += audio.length;
  }
  assert.ok(sampleCount > 87000 && sampleCount < 89000);
  const save = first.save_ram();
  assert.equal(save.length, 8192);
  assert.equal(save[0], 1);
  first.free();
  const second = new Emulator(rom());
  second.load_save_ram(save);
  second.frame();
  assert.equal(second.save_ram()[0], 2);
  second.reset();
  second.frame();
  assert.equal(second.save_ram()[0], 3);
  assert.throws(() => second.load_save_ram(new Uint8Array(1)), /8 KiB/);
  second.free();
});

test("malformed and unsupported ROMs are reported through the WASM boundary", () => {
  assert.throws(() => new Emulator(new Uint8Array(1)), /Truncated iNES header/);
  const unsupported = rom();
  unsupported[6] = 0x50;
  assert.throws(() => new Emulator(unsupported), /mapper 5/i);
});

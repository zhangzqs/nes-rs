import { test } from "node:test";
import assert from "node:assert/strict";
import { MemorySearch, RAM_SIZE } from "../memory-search.js";

test("unknown search narrows candidates against each previous snapshot", () => {
  const search = new MemorySearch();
  const bytes = new Uint8Array(RAM_SIZE);
  assert.equal(search.start(bytes), RAM_SIZE);
  bytes[0x75a] = 2;
  bytes[0x30] = 1;
  assert.equal(search.filter(bytes, "increased"), 2);
  bytes[0x75a] = 1;
  bytes[0x30] = 2;
  assert.equal(search.filter(bytes, "decreased"), 1);
  assert.deepEqual(search.entries(), [{ address: 0x75a, value: 1 }]);
  search.recordWrite(0x75a, 9);
  assert.deepEqual(search.entries(), [{ address: 0x75a, value: 9 }]);
  assert.equal(search.filter(bytes, "changed"), 1);
  assert.equal(search.filter(bytes, "unchanged"), 1);
});

test("exact search, exclusion, and reset keep only valid byte addresses", () => {
  const search = new MemorySearch();
  const bytes = new Uint8Array(RAM_SIZE);
  bytes[7] = bytes[9] = 3;
  assert.equal(search.start(bytes, 3), 2);
  bytes[9] = 4;
  assert.equal(search.filter(bytes, "not-equal", 3), 1);
  assert.deepEqual(search.entries(), [{ address: 9, value: 4 }]);
  assert.equal(search.filter(bytes, "equal", 4), 1);
  assert.throws(() => search.recordWrite(7, 99), /候选地址/);
  assert.throws(() => search.filter(bytes, "equal", 256), /0–255/);
  search.reset();
  assert.throws(() => search.filter(bytes, "changed"), /先开始/);
  assert.throws(() => search.start(new Uint8Array(1)), /快照无效/);
});

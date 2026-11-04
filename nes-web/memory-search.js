export const RAM_SIZE = 0x800;

export class MemorySearch {
  constructor() { this.reset(); }

  reset() {
    this.previous = null;
    this.candidates = [];
  }

  start(bytes, value = null) {
    const current = this.#copy(bytes);
    if (value !== null) this.#checkValue(value);
    this.candidates = Array.from({ length: RAM_SIZE }, (_, address) => address)
      .filter((address) => value === null || current[address] === value);
    this.previous = current;
    return this.candidates.length;
  }

  filter(bytes, rule, value = null) {
    if (!this.previous) throw new Error("请先开始搜索");
    const current = this.#copy(bytes);
    if (rule === "equal" || rule === "not-equal") this.#checkValue(value);
    const matches = {
      equal: (address) => current[address] === value,
      "not-equal": (address) => current[address] !== value,
      increased: (address) => current[address] > this.previous[address],
      decreased: (address) => current[address] < this.previous[address],
      changed: (address) => current[address] !== this.previous[address],
      unchanged: (address) => current[address] === this.previous[address],
    };
    if (!matches[rule]) throw new Error("未知的筛选规则");
    this.candidates = this.candidates.filter(matches[rule]);
    this.previous = current;
    return this.candidates.length;
  }

  recordWrite(address, value) {
    if (!this.previous || !this.candidates.includes(address)) throw new Error("请选择候选地址");
    this.#checkValue(value);
    this.previous[address] = value;
  }

  entries(limit = 32) {
    return this.candidates.slice(0, limit).map((address) => ({ address, value: this.previous[address] }));
  }

  #copy(bytes) {
    if (!(bytes instanceof Uint8Array) || bytes.length !== RAM_SIZE) throw new Error("工作内存快照无效");
    return Uint8Array.from(bytes);
  }

  #checkValue(value) {
    if (!Number.isInteger(value) || value < 0 || value > 255) throw new Error("数值需在 0–255 之间");
  }
}

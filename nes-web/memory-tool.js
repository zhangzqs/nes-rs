import { MemorySearch } from "./memory-search.js";

const $ = (id) => document.getElementById(id);
const addressText = (address) => `$${address.toString(16).toUpperCase().padStart(4, "0")}`;

export class MemoryTool {
  constructor(send) {
    this.send = send;
    this.search = new MemorySearch();
    this.loaded = false;
    this.pending = null;
    this.requestId = 0;
    this.selected = null;
    this.message = "可先暂停游戏再写入，避免数值立刻被游戏覆盖。";
    $("memory-start").onclick = () => this.#run(() => {
      const value = this.#readValue("memory-value", true);
      this.#request("start", { value });
    });
    $("memory-filter").onclick = () => this.#run(() => {
      const rule = $("memory-rule").value;
      const value = rule === "equal" || rule === "not-equal" ? this.#readValue("memory-value") : null;
      this.#request("filter", { rule, value });
    });
    $("memory-write").onclick = () => this.#run(() => {
      if (this.selected === null) throw new Error("请先选择候选地址");
      const value = this.#readValue("memory-write-value");
      this.#request("write", { address: this.selected, value });
    });
    $("memory-reset").onclick = () => this.reset(this.loaded);
    this.#render();
  }

  reset(loaded = false) {
    this.search.reset();
    this.loaded = loaded;
    this.pending = null;
    this.selected = null;
    this.message = loaded ? "输入数值开始搜索，或留空搜索所有地址。" : "载入游戏后可搜索。";
    this.#render();
  }

  setLoaded() {
    this.loaded = true;
    this.#render();
  }

  handle(data) {
    const action = this.pending;
    if (!action || data.requestId !== action.requestId) return;
    this.pending = null;
    try {
      if (data.type === "memory-error") throw new Error(data.message);
      if (data.type === "memory-snapshot") {
        if (action.kind === "start") this.search.start(data.bytes, action.value);
        else this.search.filter(data.bytes, action.rule, action.value);
        if (!this.search.candidates.includes(this.selected)) this.selected = null;
        this.message = this.search.candidates.length
          ? "继续操作游戏，再按规则过滤；点击候选地址即可修改。"
          : "没有匹配地址，请重新搜索。";
      } else if (data.type === "memory-written") {
        this.search.recordWrite(data.address, data.value);
        this.message = `已将 ${addressText(data.address)} 写为 ${data.value}。`;
      }
    } catch (error) {
      this.message = String(error.message || error);
    }
    this.#render();
  }

  #run(action) {
    try { action(); }
    catch (error) { this.message = String(error.message || error); this.#render(); }
  }

  #readValue(id, optional = false) {
    const text = $(id).value.trim();
    if (optional && text === "") return null;
    const value = Number(text);
    if (text === "" || !Number.isInteger(value) || value < 0 || value > 255) {
      throw new Error("请输入 0–255 的整数");
    }
    return value;
  }

  #request(kind, options) {
    if (!this.loaded || this.pending) return;
    const requestId = ++this.requestId;
    this.pending = { kind, requestId, ...options };
    this.message = kind === "write" ? "正在写入…" : "正在读取内存…";
    this.#render();
    try {
      this.send({ type: kind === "write" ? "memory-write" : "memory-snapshot", requestId, ...options });
      $("screen").focus({ preventScroll: true });
    } catch (error) {
      this.pending = null;
      throw error;
    }
  }

  #render() {
    const ready = this.loaded && !this.pending;
    const started = this.search.previous !== null;
    $("memory-start").disabled = !ready;
    $("memory-filter").disabled = !ready || !started;
    $("memory-write").disabled = !ready || this.selected === null;
    $("memory-reset").disabled = !ready || !started;
    const count = this.search.candidates.length;
    $("memory-count").textContent = started
      ? `候选地址：${count}${count > 32 ? "（仅显示前 32 项）" : ""}`
      : this.loaded ? "尚未搜索。" : "载入游戏后可搜索。";
    $("memory-status").textContent = this.message;
    const list = $("memory-candidates");
    list.replaceChildren();
    for (const { address, value } of this.search.entries()) {
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = `${addressText(address)}　${value}（$${value.toString(16).toUpperCase().padStart(2, "0")}）`;
      button.setAttribute("aria-pressed", String(this.selected === address));
      button.onclick = () => { this.selected = address; this.#render(); };
      list.append(button);
    }
  }
}

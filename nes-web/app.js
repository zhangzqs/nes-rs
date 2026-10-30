import { AudioPlayback, AUDIO_LEAD } from "./audio.js";

const $ = (id) => document.getElementById(id);
const canvas = $("screen");
const context = canvas.getContext("2d", { alpha: false });
const pads = [0, 0];
const keys = [
  ["KeyZ", "KeyX", "ShiftRight", "Enter", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"],
  ["KeyG", "KeyH", "KeyR", "KeyT", "KeyW", "KeyS", "KeyA", "KeyD"],
];
let audio, worker, saveKey, running = false, loaded = false, inFlight = 0, priming = false;
let epoch = 0, animation = 0, frameCount = 0, loadRequest = 0;

function status(text, error = false) {
  $("status").textContent = text;
  $("status").classList.toggle("error", error);
}

function releaseButtons() {
  pads.fill(0);
}

async function pause() {
  running = false;
  priming = false;
  epoch++;
  inFlight = 0;
  cancelAnimationFrame(animation);
  releaseButtons();
  $("pause").textContent = "继续";
  $("light").classList.remove("on");
  if (loaded) status("已暂停，点击继续或按 P 返回游戏。");
  await audio?.pause();
}

function fail(error) {
  void pause();
  status(`无法运行：${String(error)}。请检查 ROM 后重新载入。`, true);
}

function requestFrame() {
  if (!running || inFlight || audio.buffered >= AUDIO_LEAD) return;
  inFlight = Math.max(2, Math.min(8, Math.ceil((AUDIO_LEAD - audio.buffered) * 60.1)));
  worker.postMessage({ type: "frame", epoch, pads, count: inFlight });
}

function tick() {
  if (!running) return;
  if (!priming && audio.context.state !== "running") {
    void pause();
    return;
  }
  requestFrame();
  animation = requestAnimationFrame(tick);
}

async function play() {
  if (!loaded || running || document.hidden) return;
  const requestedEpoch = ++epoch;
  await audio.pause();
  if (epoch !== requestedEpoch || document.hidden) return;
  running = true;
  priming = true;
  inFlight = 0;
  $("pause").textContent = "暂停";
  $("light").classList.add("on");
  status("正在运行 · Enter 开始游戏");
  canvas.focus({ preventScroll: true });
  tick();
}

async function unlockAudio() {
  audio ??= new AudioPlayback();
  audio.setVolume(Number($("volume").value) / 100);
  // Called directly from a user gesture, before reading or fetching the ROM.
  await audio.resume();
}

async function loadRom(bytes, name, request) {
  await pause();
  if (request !== loadRequest) return;
  loaded = false;
  $("pause").disabled = $("reset").disabled = true;
  worker?.terminate();
  status("正在载入卡带…");
  const hash = await crypto.subtle.digest("SHA-256", bytes);
  if (request !== loadRequest) return;
  saveKey = "nes-rs-save-" + [...new Uint8Array(hash)].map((n) => n.toString(16).padStart(2, "0")).join("");
  let save;
  try {
    const saved = JSON.parse(localStorage.getItem(saveKey));
    if (Array.isArray(saved) && saved.length === 8192) save = new Uint8Array(saved);
  } catch { /* Storage may be unavailable; playing does not require it. */ }
  frameCount = 0;
  audio.underruns = 0;
  // Resume while the load gesture is still active; no samples play until ready.
  await audio.resume();
  if (request !== loadRequest) return;
  const current = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
  worker = current;
  current.onerror = (event) => {
    if (worker === current) fail(event.message || "模拟核心加载失败");
  };
  current.onmessage = ({ data }) => {
    if (worker !== current) return;
    if (data.type === "ready") {
      loaded = true;
      $("empty").hidden = true;
      $("game-name").textContent = name.replace(/\.nes$/i, "");
      $("pause").disabled = $("reset").disabled = false;
      void play().catch(fail);
    } else if (data.type === "frame") {
      if (data.save?.length) {
        try { localStorage.setItem(saveKey, JSON.stringify([...data.save])); }
        catch { status("存档空间不可用，本次进度无法自动保存。", true); }
      }
      if (!running || data.epoch !== epoch) return;
      inFlight--;
      frameCount++;
      audio.enqueue(data.audio);
      if (priming && audio.buffered >= AUDIO_LEAD) {
        const requestedEpoch = epoch;
        audio.resume().then(() => {
          if (epoch === requestedEpoch) priming = false;
        }).catch(fail);
      } else {
        requestFrame();
      }
      context.putImageData(new ImageData(new Uint8ClampedArray(data.pixels.buffer), 256, 240), 0, 0);
    } else if (data.type === "error") {
      fail(data.message);
    }
  };
  current.postMessage({ type: "load", rom: new Uint8Array(bytes), save }, [bytes]);
}

async function openFile(file) {
  if (!file) return;
  const request = ++loadRequest;
  await unlockAudio();
  const bytes = await file.arrayBuffer();
  if (request === loadRequest) await loadRom(bytes, file.name, request);
}

$("load").onclick = () => $("rom-file").click();
$("rom-file").onchange = (event) => {
  void openFile(event.target.files[0]).catch(fail);
  event.target.value = "";
};
$("pause").onclick = () => { void (running ? pause() : play()).catch(fail); };
async function resetGame() {
  if (!loaded) return;
  await pause();
  loaded = false;
  $("pause").disabled = $("reset").disabled = true;
  frameCount = 0;
  worker.postMessage({ type: "reset" });
}
$("reset").onclick = () => { void resetGame().catch(fail); };
$("volume").oninput = (event) => {
  $("volume-value").value = `${event.target.value}%`;
  audio?.setVolume(Number(event.target.value) / 100);
};
$("fullscreen").onclick = () => {
  const operation = document.fullscreenElement ? document.exitFullscreen() : canvas.parentElement.requestFullscreen();
  operation.catch((error) => status(`无法进入全屏：${error.message}`, true));
};

function handleKey(event, down) {
  if (!loaded || (down && event.target.matches("input, summary"))) return;
  if (event.code === "F2") {
    event.preventDefault();
    if (down && !event.repeat) void resetGame().catch(fail);
    return;
  }
  if (event.code === "KeyP") {
    event.preventDefault();
    if (down && !event.repeat) void (running ? pause() : play()).catch(fail);
    return;
  }
  keys.forEach((bindings, player) => {
    const bit = bindings.indexOf(event.code);
    if (bit < 0) return;
    event.preventDefault();
    if (down) pads[player] |= 1 << bit;
    else pads[player] &= ~(1 << bit);
  });
}
window.addEventListener("keydown", (event) => handleKey(event, true));
window.addEventListener("keyup", (event) => handleKey(event, false));
window.addEventListener("blur", releaseButtons);
document.addEventListener("visibilitychange", () => { if (document.hidden) void pause(); });
window.addEventListener("pagehide", () => { void pause(); });
window.addEventListener("dragover", (event) => { event.preventDefault(); document.body.classList.add("dragging"); });
window.addEventListener("dragleave", (event) => { if (!event.relatedTarget) document.body.classList.remove("dragging"); });
window.addEventListener("drop", (event) => {
  event.preventDefault();
  document.body.classList.remove("dragging");
  void openFile(event.dataTransfer.files[0]).catch(fail);
});

// A development server can offer an explicitly supplied local ROM. Nothing is bundled.
const query = new URLSearchParams(location.search);
if (query.has("rom")) {
  const url = new URL(query.get("rom"), location.href);
  if (url.origin === location.origin) {
    const name = query.get("name") || "本地游戏";
    $("demo").hidden = false;
    $("demo").textContent = `载入${name}`;
    $("demo").onclick = async () => {
      try {
        const request = ++loadRequest;
        await unlockAudio();
        const response = await fetch(url);
        if (!response.ok) throw new Error(`读取游戏失败 (${response.status})`);
        const bytes = await response.arrayBuffer();
        if (request === loadRequest) await loadRom(bytes, name, request);
      } catch (error) { fail(error); }
    };
  }
}

// Read-only runtime counters for browser regression tests and troubleshooting.
Object.defineProperty(window, "nesDiagnostics", {
  get: () => ({ running, frames: frameCount, buffered: audio?.buffered ?? 0,
    underruns: audio?.underruns ?? 0, audioState: audio?.context.state ?? "uninitialized",
    audioTime: audio?.context.currentTime ?? 0, sampleRate: audio?.context.sampleRate ?? 0 }),
});

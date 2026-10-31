// Browser-level QA of the *deployed* site, using Chrome DevTools Protocol and Node 22.
// No npm dependency, mocks, screenshots, or changes to the public dictionary.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
import { existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const url = "https://sguzman.github.io/mexilex/";
const chrome = process.env.CHROME_BIN
  || ["/usr/bin/google-chrome", "/usr/bin/google-chrome-stable", "/usr/bin/chromium"].find(existsSync);
if (!chrome) throw new Error("Chrome/Chromium executable not found");
const profile = await mkdtemp(join(tmpdir(), "mexilex-cdp-"));
const processChrome = spawn(chrome, [
  "--headless=new", "--no-sandbox", "--disable-gpu", "--disable-dev-shm-usage",
  "--no-first-run", "--no-default-browser-check",
  "--remote-debugging-port=0", "--user-data-dir=" + profile,
  "--window-size=1280,960", "about:blank",
], { stdio: ["ignore", "ignore", "pipe"] });

const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
let socket;
const connection = new Promise((resolve, reject) => {
  let output = "";
  processChrome.stderr.on("data", chunk => {
    output += chunk.toString();
    const match = output.match(/DevTools listening on (ws:\/\/[^\s]+)/);
    if (match) resolve(new URL(match[1]).port);
    if (output.length > 12000) output = output.slice(-12000);
  });
  processChrome.on("exit", code => reject(new Error("Chromium exited early: " + code)));
});
try {
  const port = await Promise.race([
    connection,
    new Promise((_, reject) => setTimeout(() => reject(new Error("DevTools startup timeout")), 20000)),
  ]);
  let target;
  for (let i = 0; i < 50; i++) {
    const pages = await fetch("http://127.0.0.1:" + port + "/json/list").then(r => r.json());
    target = pages.find(page => page.type === "page");
    if (target?.webSocketDebuggerUrl) break;
    await sleep(100);
  }
  assert.ok(target?.webSocketDebuggerUrl);
  socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  let nextId = 0;
  const pending = new Map();
  socket.addEventListener("message", event => {
    const response = JSON.parse(event.data);
    if (!response.id) return;
    const callback = pending.get(response.id);
    if (callback) {
      pending.delete(response.id);
      if (response.error) callback.reject(new Error(JSON.stringify(response.error)));
      else callback.resolve(response.result);
    }
  });
  function call(method, params = {}) {
    const id = ++nextId;
    const outcome = new Promise((resolve, reject) => pending.set(id, {resolve, reject}));
    socket.send(JSON.stringify({id, method, params}));
    return outcome;
  }
  async function evaluate(expression) {
    const reply = await call("Runtime.evaluate", {
      expression, awaitPromise: true, returnByValue: true,
    });
    if (reply.exceptionDetails) throw new Error("Browser exception: " + reply.exceptionDetails.text);
    return reply.result.value;
  }
  async function viewport(width, height, mobile) {
    await call("Emulation.setDeviceMetricsOverride", {
      width, height, deviceScaleFactor: 1, mobile
    });
    await call("Emulation.setTouchEmulationEnabled", {
      enabled: mobile, maxTouchPoints: 1
    });
  }
  async function navigate() {
    await call("Page.navigate", {url});
    await evaluate(`(async () => {
      const limit = Date.now() + 20000;
      while (Date.now() < limit) {
        if (document.readyState === "complete"
            && document.querySelector("#count-entries")?.textContent.includes("813,928"))
          return true;
        await new Promise(resolve => setTimeout(resolve, 150));
      }
      throw Error("Public site failed to load statistics");
    })()`);
  }
  await call("Page.enable");
  await call("Runtime.enable");
  await viewport(1365, 900, false);
  await navigate();
  const desktop = await evaluate(`(async () => {
    const wait = async (predicate, label) => {
      const until = Date.now() + 20000;
      while (Date.now() < until) {
        if (predicate()) return;
        await new Promise(resolve => setTimeout(resolve, 120));
      }
      throw Error("Timeout: " + label);
    };
    const input = document.querySelector("#query");
    input.value = "banqueta";
    input.dispatchEvent(new Event("input", {bubbles: true}));
    await wait(() => document.querySelector("#results .result-row"), "banqueta search");
    const first = document.querySelector("#results .result-row");
    if (!first.textContent.includes("banqueta") || !first.textContent.toLowerCase().includes("sidewalk"))
      throw Error("Mexican meaning missing from search preview");
    first.click();
    await wait(() => document.querySelector("#detail")?.textContent.toLowerCase().includes("sidewalk")
      && !document.querySelector("#detail")?.textContent.includes("Loading entry"), "live entry details");
    const theme = document.querySelector("#theme");
    theme.value = "dark";
    theme.dispatchEvent(new Event("change", {bubbles: true}));
    if (document.documentElement.dataset.theme !== "dark") throw Error("Dark mode failed");
    theme.value = "system";
    theme.dispatchEvent(new Event("change", {bubbles: true}));
    if (document.documentElement.dataset.theme !== "") throw Error("System mode failed");
    return {
      results: document.querySelectorAll("#results .result-row").length,
      detail: document.querySelector("#detail").textContent.slice(0, 120),
      noOverflow: document.documentElement.scrollWidth <= window.innerWidth + 2,
    };
  })()`);
  assert.ok(desktop.noOverflow, "Desktop has horizontal overflow");

  await viewport(390, 844, true);
  await navigate();
  const mobile = await evaluate(`(async () => {
    const wait = async (predicate, label) => {
      const until = Date.now() + 20000;
      while (Date.now() < until) {
        if (predicate()) return;
        await new Promise(resolve => setTimeout(resolve, 150));
      }
      throw Error("Timeout: " + label);
    };
    const currentWidth = document.documentElement.scrollWidth;
    if (currentWidth > window.innerWidth + 2)
      throw Error("Mobile horizontal overflow: " + currentWidth + " > " + window.innerWidth);
    document.querySelector("#browse-dialogue").click();
    await wait(() => document.querySelectorAll("#results .result-row").length === 60,
      "Mexican dialogue browse");
    if (document.querySelector("#mexico-only").checked) throw Error("Dialogue browse incorrectly forces Mexico-specific meanings");
    const first = document.querySelector("#results .result-row");
    first.click();
    await wait(() => document.querySelector("#detail")?.textContent.includes("film dialogue"),
      "Frequency evidence in entry detail");
    document.querySelector("#browse-mexico").click();
    await wait(() => document.querySelectorAll("#results .result-row").length === 60,
      "Mexico-tagged meanings browse");
    if (!document.querySelector("#mexico-only").checked)
      throw Error("Mexico-specific meaning filter not set");
    return {
      width: window.innerWidth,
      pageWidth: currentWidth,
      dialogueCount: 60,
      mexicoCount: document.querySelectorAll("#results .result-row").length,
    };
  })()`);
  console.log("PASS live desktop and mobile Chromium UI", JSON.stringify({desktop, mobile}));
} finally {
  socket?.close();
  processChrome.kill("SIGKILL");
  await rm(profile, {recursive: true, force: true});
}

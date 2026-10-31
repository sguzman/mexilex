import { fold, bucketOf, resultMatches, rankMexicanFrequency } from "./search.mjs";
"use strict";
const REPOSITORY = "sguzman/mexilex";
const root = document.documentElement;
const byId = id => document.getElementById(id);
const results = byId("results");
const detail = byId("detail");
const queryInput = byId("query");
const mexicoOnly = byId("mexico-only");
const includeForms = byId("include-forms");
const rankMexico = byId("rank-mx");
const cache = new Map();
const fmt = new Intl.NumberFormat("en");
let config = null;
let selectedId = null;
let sequence = 0;
let activeSearch = [];


function el(tag, className, text) {
  const n = document.createElement(tag);
  if (className) n.className = className;
  if (text !== undefined) n.textContent = String(text);
  return n;
}
function setTheme(theme) {
  root.dataset.theme = theme === "system" ? "" : theme;
  byId("theme").value = theme;
  try { localStorage.setItem("mexilex-theme", theme); } catch (_) {}
}
let savedTheme = "system";
try { savedTheme = localStorage.getItem("mexilex-theme") || "system"; } catch (_) {}
setTheme(["system", "light", "dark"].includes(savedTheme) ? savedTheme : "system");
byId("theme").addEventListener("change", event => setTheme(event.target.value));

function displayEmpty(container, message) {
  container.replaceChildren(el("p", "empty", message));
}
function updateLocation(q) {
  const url = new URL(window.location.href);
  if (q) url.searchParams.set("q", q);
  else url.searchParams.delete("q");
  history.replaceState(null, "", url);
}
function showStatus(message) { byId("result-count").textContent = message; }
async function getIndex(bucket, forms = false) {
  const path = (forms ? "forms/" : "index/") + bucket + ".json";
  if (cache.has(path)) return cache.get(path);
  const promise = (async () => {
    const response = await fetch("./" + path);
    if (response.status === 404) return [];
    if (!response.ok) throw new Error("Index fetch failed: " + response.status);
    return response.json();
  })();
  cache.set(path, promise);
  try { return await promise; } catch (err) { cache.delete(path); throw err; }
}

function renderResults(matches, total, sample = false) {
  const fragment = document.createDocumentFragment();
  matches.forEach(row => {
    const btn = el("button", "result-row");
    btn.type = "button";
    btn.setAttribute("aria-pressed", String(row[1] === selectedId));
    const head = el("div", "result-head");
    head.append(el("span", "word", row[0]), el("span", "pos", row[2]));
    if (row[4]) head.append(el("span", "tag", "MEXICO"));
    btn.append(head, el("div", "preview", row[3]));
    btn.addEventListener("click", () => selectEntry(row));
    fragment.append(btn);
  });
  if (matches.length) results.replaceChildren(fragment);
  else displayEmpty(results, "No matching entries. Try removing a filter or searching a different word.");
  showStatus(sample ? `${matches.length} sampled from ${fmt.format(total)}` : total > matches.length ? `First ${matches.length} of ${fmt.format(total)}` : `${fmt.format(total)} result${total === 1 ? "" : "s"}`);
}
async function runSearch() {
  const turn = ++sequence;
  const query = queryInput.value.trim();
  updateLocation(query);
  selectedId = null;
  activeSearch = [];
  displayEmpty(detail, "Select an entry to inspect its individual meanings.");
  const bucket = bucketOf(query);
  if (!bucket) {
    showStatus("Enter 2+ characters");
    displayEmpty(results, "Enter at least two characters to search.");
    return;
  }
  showStatus("Searching…");
  try {
    const ordinary = await getIndex(bucket);
    const inflections = includeForms.checked ? await getIndex(bucket, true) : [];
    if (turn !== sequence) return;
    const index = ordinary.concat(inflections);
    index.sort(rankMexico.checked ? rankMexicanFrequency : (a, b) =>
      fold(a[0]).localeCompare(fold(b[0]), "es") || Number(a[5]) - Number(b[5]) || a[2].localeCompare(b[2], "en"));
    const q = fold(query);
    const matches = [];
    let total = 0;
    for (const row of index) {
      if (!resultMatches(row, q, mexicoOnly.checked, includeForms.checked)) continue;
      total++;
      if (matches.length < 80) matches.push(row);
    }
    activeSearch = matches;
    renderResults(matches, total);
    if (matches.length && (fold(matches[0][0]) === q || total === 1)) selectEntry(matches[0]);
  } catch (error) {
    if (turn !== sequence) return;
    showStatus("Unavailable");
    displayEmpty(results, "Could not retrieve the search index. Check your connection and try again.");
  }
}
let mexicanDialogueCache;
let mexicoBrowseCache;
byId("browse-mexico").addEventListener("click", async () => {
  const turn = ++sequence;
  queryInput.value = "";
  updateLocation("");
  mexicoOnly.checked = true;
  includeForms.checked = false;
  selectedId = null;
  displayEmpty(detail, "Choose a Mexico-tagged word to explore its meanings.");
  showStatus("Loading…");
  try {
    if (!mexicoBrowseCache) {
      const response = await fetch("./mexico.json");
      if (!response.ok) throw new Error("No Mexico vocabulary index");
      mexicoBrowseCache = await response.json();
    }
    if (turn !== sequence) return;
    // Shuffle a small list of verified region-tagged headwords rather than
    // downloading the entire dictionary for a random browse page.
    const candidates = mexicoBrowseCache.slice();
    for (let i = candidates.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [candidates[i], candidates[j]] = [candidates[j], candidates[i]];
    }
    activeSearch = candidates.slice(0, 60);
    renderResults(activeSearch, candidates.length, true);
  } catch (_) {
    if (turn !== sequence) return;
    showStatus("Unavailable");
    displayEmpty(results, "Could not load Mexico-tagged vocabulary. Try searching a word.");
  }
});
byId("browse-dialogue").addEventListener("click", async () => {
  const turn = ++sequence;
  queryInput.value = "";
  updateLocation("");
  mexicoOnly.checked = false;
  includeForms.checked = false;
  rankMexico.checked = true;
  selectedId = null;
  displayEmpty(detail, "Choose a word attested in Mexican film dialogue to inspect its meanings and frequency.");
  showStatus("Loading…");
  try {
    if (!mexicanDialogueCache) {
      const response = await fetch("./dialogue.json");
      if (!response.ok) throw new Error("No dialogue frequency index");
      mexicanDialogueCache = await response.json();
    }
    if (turn !== sequence) return;
    activeSearch = mexicanDialogueCache.slice(0, 60);
    renderResults(activeSearch, mexicanDialogueCache.length);
  } catch (_) {
    if (turn !== sequence) return;
    showStatus("Unavailable");
    displayEmpty(results, "Could not load the film-dialogue vocabulary index. Try searching a word.");
  }
});
let timer;
queryInput.addEventListener("input", () => { clearTimeout(timer); timer = setTimeout(runSearch, 160); });
mexicoOnly.addEventListener("change", runSearch);
includeForms.addEventListener("change", runSearch);
rankMexico.addEventListener("change", runSearch);
document.querySelectorAll("[data-sample]").forEach(button => button.addEventListener("click", () => {
  queryInput.value = button.dataset.sample;
  runSearch();
  queryInput.focus();
}));
document.addEventListener("keydown", event => {
  if (event.key === "/" && !["INPUT", "TEXTAREA", "SELECT"].includes(document.activeElement.tagName)) {
    event.preventDefault();
    queryInput.focus();
  }
});

function recordLink(word) {
  return "https://en.wiktionary.org/wiki/" + encodeURIComponent(word) + "#Spanish";
}
function row(tag, text, className) { return el(tag, className || "", text); }
function sourceLink(anchor, text, url) {
  const link = row("a", text);
  link.href = url;
  link.target = "_blank";
  link.rel = "noopener noreferrer";
  anchor.append(link);
}
async function selectEntry(selected) {
  selectedId = selected[1];
  Array.from(results.children).forEach((btn, i) => btn.setAttribute("aria-pressed", String(activeSearch[i]?.[1] === selectedId)));
  displayEmpty(detail, "Loading entry…");
  try {
    if (!config || !/^[0-9a-f]{40}$/.test(config.commit)) throw new Error("Missing pinned source revision");
    const id = selected[1];
    if (!/^[0-9a-f]{64}$/.test(id)) throw new Error("Invalid lexical identifier");
    const path = `corpus/${id.slice(0, 2)}/${id.slice(2, 3)}.jsonl`;
    const url = `https://raw.githubusercontent.com/${REPOSITORY}/${config.commit}/${path}`;
    const response = await fetch(url);
    if (!response.ok) throw new Error("Unable to retrieve lexical shard");
    let record = null;
    const lines = (await response.text()).split("\n");
    for (const line of lines) {
      if (!line) continue;
      const candidate = JSON.parse(line);
      if (candidate.id === id) { record = candidate; break; }
    }
    if (selectedId !== id) return;
    if (!record) throw new Error("The selected entry was not found in its source shard");
    showEntry(record, selected[6]);
  } catch (_) {
    if (selectedId !== selected[1]) return;
    const note = el("p", "empty", "This entry could not be loaded. The original Wiktionary entry is still available:");
    const links = el("p", "entry-links");
    sourceLink(links, "View source →", recordLink(selected[0]));
    detail.replaceChildren(note, links);
  }
}
function showEntry(record, dialogueFrequency) {
  const container = el("div", "detail-content");
  const head = el("div", "detail-header");
  const titlebox = el("div");
  titlebox.append(row("h2", record.word), row("div", record.part_of_speech, "pos"));
  head.append(titlebox);
  container.append(head);
  if (Array.isArray(dialogueFrequency) && dialogueFrequency.length === 4) {
    const [mx, es, ar, dispersion] = dialogueFrequency;
    const frequencies = el("section", "frequency-evidence");
    frequencies.append(el("strong", "", "Attested in Mexican film dialogue"));
    const comparison = el("p", "", `Occurrences per 100,000 words: Mexico ${mx.toLocaleString("en")} · Spain ${es.toLocaleString("en")} · Argentina ${ar.toLocaleString("en")}. Mexican film dispersion: ${dispersion} / 10.`);
    frequencies.append(comparison);
    const note = el("small", "", "ACTIV-ES v0.2 · Film-subtitle word frequency (surface spelling), not a regional-sense label.");
    frequencies.append(note);
    container.append(frequencies);
  }
  const senses = Array.isArray(record.senses) ? record.senses : [];
  senses.forEach((sense, index) => {
    const section = el("section", "sense");
    const labels = el("div", "sense-tags");
    labels.append(el("span", "pos", `MEANING ${index + 1}`));
    if (sense.mexican_usage) labels.append(el("span", "tag", "MEXICO-SPECIFIC"));
    section.append(labels, el("p", "sense-definition", sense.definition_en));
    if (Array.isArray(sense.examples)) {
      for (const sample of sense.examples.slice(0, 2)) {
        if (!sample || typeof sample !== "object" || !sample.text) continue;
        const example = el("div", "example");
        example.append(row("span", sample.text));
        if (sample.english) example.append(row("span", sample.english));
        section.append(example);
      }
    }
    container.append(section);
  });
  if (Array.isArray(record.forms) && record.forms.length) {
    const forms = record.forms.slice(0, 12).map(form => form.form).filter(Boolean);
    if (forms.length) container.append(el("p", "forms", "Related forms: " + forms.join(" · ")));
  }
  const links = el("div", "entry-links");
  sourceLink(links, "Original Wiktionary entry ↗", recordLink(record.word));
  sourceLink(links, "Lexical record on GitHub ↗", `https://github.com/${REPOSITORY}/blob/${config.commit}/corpus/${record.id.slice(0,2)}/${record.id[2]}.jsonl`);
  container.append(links);
  detail.replaceChildren(container);
}
async function initialize() {
  try {
    const response = await fetch("./config.json");
    if (!response.ok) throw new Error("Missing site configuration");
    config = await response.json();
    byId("count-entries").textContent = fmt.format(config.entries);
    byId("count-senses").textContent = fmt.format(config.senses);
    byId("count-mexican").textContent = fmt.format(config.mexican_senses);
    const q = new URL(window.location.href).searchParams.get("q");
    if (q) { queryInput.value = q; runSearch(); }
  } catch (_) {
    showStatus("Unavailable");
    displayEmpty(results, "Site data could not be loaded. Please try again.");
  }
}
initialize();

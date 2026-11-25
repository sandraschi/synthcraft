#!/usr/bin/env python3
"""Generate the self-contained interactive HTML documentation site for SynthCraft.

Reads all 14 workbench sub-READMEs in docs/workbenches/*.md and produces
docs/site/index.html with zero external dependencies (works via file:// or GitHub Pages).
"""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WB_DIR = ROOT / "docs" / "workbenches"
OUT_HTML = ROOT / "docs" / "site" / "index.html"


def parse_workbench(md_path: Path) -> dict:
    text = md_path.read_text(encoding="utf-8")
    lines = text.splitlines()
    title = md_path.stem
    intro_lines: list[str] = []
    subpages: list[dict] = []

    current_sub: dict | None = None
    in_intro = False

    for line in lines:
        if line.startswith("# ") and not in_intro and current_sub is None:
            title = line[2:].strip()
            in_intro = True
            continue
        if line.startswith("## Subpage "):
            in_intro = False
            if current_sub is not None:
                current_sub["body"] = "\n".join(current_sub["lines"]).strip()
                del current_sub["lines"]
                subpages.append(current_sub)
            heading = line[3:].strip()
            sub_id = re.sub(r"[^a-z0-9]+", "-", heading.lower()).strip("-")
            current_sub = {"id": sub_id, "title": heading, "lines": []}
            continue
        if current_sub is not None:
            current_sub["lines"].append(line)
        elif in_intro and line.strip() != "---":
            intro_lines.append(line)

    if current_sub is not None:
        current_sub["body"] = "\n".join(current_sub["lines"]).strip()
        del current_sub["lines"]
        subpages.append(current_sub)

    return {
        "file": md_path.name,
        "id": md_path.stem,
        "title": title,
        "intro": "\n".join(intro_lines).strip(),
        "subpages": subpages,
    }


HTML_TEMPLATE = r"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>SynthCraft Studio Handbook -- 14 Workbenches, 37 Subpages &amp; Synth Lineage</title>
<style>
  :root {
    --bg: #0a0d14;
    --bg-elev: #111622;
    --bg-card: #161c2c;
    --bg-code: #0c1018;
    --border: #263046;
    --text: #e6ecf5;
    --muted: #93a1ba;
    --amber: #f5a623;
    --amber-soft: rgba(245, 166, 35, 0.12);
    --cyan: #38bdf8;
    --cyan-soft: rgba(56, 189, 248, 0.10);
    --magenta: #e879f9;
    --magenta-soft: rgba(232, 121, 249, 0.10);
    --green: #34d399;
    --shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
  }
  body.light {
    --bg: #f6f7fb;
    --bg-elev: #ffffff;
    --bg-card: #ffffff;
    --bg-code: #f0f3f8;
    --border: #d7dfec;
    --text: #141a26;
    --muted: #52607a;
    --amber: #b45309;
    --amber-soft: rgba(180, 83, 9, 0.08);
    --cyan: #0369a1;
    --cyan-soft: rgba(3, 105, 161, 0.08);
    --magenta: #86198f;
    --magenta-soft: rgba(134, 25, 143, 0.08);
    --green: #047857;
    --shadow: 0 8px 24px rgba(15, 23, 42, 0.08);
  }
  * { box-sizing: border-box; }
  body {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Inter, Roboto, sans-serif;
    background: var(--bg);
    color: var(--text);
    line-height: 1.65;
    display: flex;
    min-height: 100vh;
  }
  aside.sidebar {
    width: 315px;
    flex-shrink: 0;
    background: var(--bg-elev);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    height: 100vh;
    position: sticky;
    top: 0;
  }
  .brand {
    padding: 20px 20px 14px;
    border-bottom: 1px solid var(--border);
  }
  .brand h1 {
    margin: 0;
    font-size: 1.25rem;
    letter-spacing: 0.03em;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .brand .badge {
    font-size: 0.7rem;
    padding: 2px 7px;
    border-radius: 999px;
    background: var(--amber-soft);
    color: var(--amber);
    border: 1px solid var(--amber);
    font-weight: 700;
  }
  .brand p {
    margin: 6px 0 0;
    font-size: 0.8rem;
    color: var(--muted);
  }
  .search-wrap {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
  }
  .search-wrap input {
    width: 100%;
    padding: 9px 12px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--bg);
    color: var(--text);
    font-size: 0.88rem;
    outline: none;
  }
  .search-wrap input:focus { border-color: var(--cyan); }
  nav.tree {
    overflow-y: auto;
    padding: 10px 10px 28px;
    flex: 1;
  }
  .nav-home {
    width: 100%;
    text-align: left;
    background: transparent;
    border: 1px solid transparent;
    color: var(--text);
    padding: 9px 12px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    margin-bottom: 8px;
  }
  .nav-home.active, .nav-home:hover {
    background: var(--amber-soft);
    border-color: var(--amber);
    color: var(--amber);
  }
  .wb-group { margin-bottom: 6px; }
  .wb-btn {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    color: var(--text);
    font-weight: 600;
    font-size: 0.85rem;
    padding: 7px 10px;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .wb-btn:hover { background: var(--bg-card); }
  .wb-subs {
    padding-left: 12px;
    margin-top: 2px;
    border-left: 1px solid var(--border);
    margin-left: 10px;
  }
  .sub-btn {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    color: var(--muted);
    font-size: 0.82rem;
    padding: 6px 10px;
    border-radius: 6px;
    cursor: pointer;
    display: block;
    margin: 2px 0;
  }
  .sub-btn:hover { color: var(--text); background: var(--bg-card); }
  .sub-btn.active {
    background: var(--cyan-soft);
    color: var(--cyan);
    font-weight: 600;
  }
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  header.topbar {
    position: sticky;
    top: 0;
    z-index: 10;
    background: var(--bg-elev);
    border-bottom: 1px solid var(--border);
    padding: 12px 32px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .crumb {
    font-size: 0.88rem;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .crumb strong { color: var(--text); }
  .top-actions { display: flex; gap: 10px; }
  .pill-btn {
    background: var(--bg-card);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 6px 13px;
    border-radius: 999px;
    font-size: 0.82rem;
    cursor: pointer;
    font-weight: 600;
  }
  .pill-btn:hover { border-color: var(--amber); color: var(--amber); }
  .content {
    max-width: 960px;
    width: 100%;
    margin: 0 auto;
    padding: 32px 36px 64px;
  }
  .hero {
    background: linear-gradient(135deg, var(--bg-card), var(--bg-elev));
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 30px 32px;
    box-shadow: var(--shadow);
    margin-bottom: 28px;
  }
  .hero h2 {
    margin: 0 0 10px;
    font-size: 2rem;
    line-height: 1.2;
    background: linear-gradient(90deg, var(--amber), var(--cyan), var(--magenta));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  .hero p { color: var(--muted); margin: 0 0 18px; font-size: 1.02rem; }
  .stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 12px;
  }
  .stat {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 12px 14px;
  }
  .stat .num { font-size: 1.4rem; font-weight: 800; color: var(--amber); }
  .stat .lbl { font-size: 0.78rem; color: var(--muted); }
  .grid-cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(275px, 1fr));
    gap: 16px;
    margin-top: 20px;
  }
  .wb-card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 18px;
    cursor: pointer;
    transition: transform 0.14s ease, border-color 0.14s ease;
  }
  .wb-card:hover { transform: translateY(-2px); border-color: var(--cyan); }
  .wb-card h3 { margin: 0 0 8px; font-size: 1.02rem; color: var(--amber); }
  .wb-card p { margin: 0 0 12px; font-size: 0.86rem; color: var(--muted); }
  .wb-card .tag {
    display: inline-block;
    font-size: 0.73rem;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--cyan-soft);
    color: var(--cyan);
    margin: 2px 4px 2px 0;
  }
  .subpage-header {
    margin-bottom: 22px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border);
  }
  .subpage-header .kicker {
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--amber);
    font-weight: 700;
  }
  .subpage-header h2 {
    margin: 6px 0 0;
    font-size: 1.85rem;
    line-height: 1.25;
  }
  .section-card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 20px 24px;
    margin-bottom: 18px;
  }
  .section-card.lineage {
    border-left: 4px solid var(--cyan);
    background: linear-gradient(180deg, var(--cyan-soft), var(--bg-card) 42%);
  }
  .section-card.tracks {
    border-left: 4px solid var(--magenta);
    background: linear-gradient(180deg, var(--magenta-soft), var(--bg-card) 42%);
  }
  .section-card.cli {
    border-left: 4px solid var(--green);
  }
  .section-card h3 {
    margin: 0 0 12px;
    font-size: 1.08rem;
    color: var(--amber);
  }
  .section-card.lineage h3 { color: var(--cyan); }
  .section-card.tracks h3 { color: var(--magenta); }
  .section-card.cli h3 { color: var(--green); }
  .section-card ul, .section-card ol { padding-left: 20px; margin: 8px 0; }
  .section-card li { margin-bottom: 7px; }
  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    background: var(--bg-code);
    padding: 2px 6px;
    border-radius: 5px;
    font-size: 0.86em;
    border: 1px solid var(--border);
  }
  pre {
    background: var(--bg-code);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 14px 16px;
    overflow-x: auto;
    position: relative;
    margin: 10px 0 0;
  }
  pre code { border: none; padding: 0; background: transparent; font-size: 0.86rem; }
  .copy-btn {
    position: absolute;
    top: 8px;
    right: 8px;
    background: var(--bg-card);
    border: 1px solid var(--border);
    color: var(--muted);
    font-size: 0.75rem;
    padding: 4px 9px;
    border-radius: 6px;
    cursor: pointer;
  }
  .copy-btn:hover { color: var(--green); border-color: var(--green); }
  .pager {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    margin-top: 28px;
    padding-top: 18px;
    border-top: 1px solid var(--border);
  }
  .pager button {
    background: var(--bg-card);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 10px 16px;
    border-radius: 10px;
    cursor: pointer;
    font-weight: 600;
    font-size: 0.86rem;
  }
  .pager button:hover { border-color: var(--amber); color: var(--amber); }
  .search-hit {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 14px 18px;
    margin-bottom: 12px;
    cursor: pointer;
  }
  .search-hit:hover { border-color: var(--cyan); }
  .search-hit .meta { font-size: 0.78rem; color: var(--amber); font-weight: 700; }
  .search-hit h4 { margin: 4px 0 6px; font-size: 1.02rem; }
  .search-hit p { margin: 0; font-size: 0.86rem; color: var(--muted); }
  @media (max-width: 860px) {
    body { flex-direction: column; }
    aside.sidebar { width: 100%; height: auto; position: static; }
    .content { padding: 20px 18px 48px; }
  }
</style>
</head>
<body>
<aside class="sidebar">
  <div class="brand">
    <h1>SynthCraft <span class="badge">v0.4.0</span></h1>
    <p>14 Workbenches &middot; 37 Subpages &middot; 60 MCP Tools</p>
  </div>
  <div class="search-wrap">
    <input id="searchInput" type="search" placeholder="Search synths, tracks, 303, Daft Punk, Buchla...">
  </div>
  <nav class="tree" id="sidebarTree"></nav>
</aside>
<main>
  <header class="topbar">
    <div class="crumb" id="breadcrumb"><strong>SynthCraft Studio Handbook</strong></div>
    <div class="top-actions">
      <button class="pill-btn" id="homeBtn">Atlas Home</button>
      <button class="pill-btn" id="themeBtn">Daylight Mode</button>
    </div>
  </header>
  <div class="content" id="content"></div>
</main>
<script>
const WORKBENCHES = __DATA_JSON__;

const flatSubpages = [];
WORKBENCHES.forEach((wb, wIdx) => {
  wb.subpages.forEach((sub, sIdx) => {
    flatSubpages.push({ wIdx, sIdx, wb, sub });
  });
});

let currentView = { mode: "home", wIdx: 0, sIdx: 0 };

function escapeHtml(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function formatInline(s) {
  let out = escapeHtml(s);
  out = out.replace(/`([^`]+)`/g, "<code>$1</code>");
  out = out.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  out = out.replace(/\*([^*]+)\*/g, "<em>$1</em>");
  return out;
}

function renderMarkdownSections(md) {
  const parts = md.split(/\n(?=### )/);
  let html = "";
  for (const part of parts) {
    const lines = part.trim().split("\n");
    if (!lines.length) continue;
    let heading = "";
    let bodyLines = lines;
    if (lines[0].startsWith("### ")) {
      heading = lines[0].slice(4).trim();
      bodyLines = lines.slice(1);
    }
    let cardClass = "section-card";
    if (/Provenance|Lineage/i.test(heading)) cardClass += " lineage";
    else if (/Famous Tracks/i.test(heading)) cardClass += " tracks";
    else if (/CLI & MCP/i.test(heading)) cardClass += " cli";

    let inner = heading ? `<h3>${escapeHtml(heading)}</h3>` : "";
    let inCode = false;
    let codeBuf = [];
    let inUl = false;
    let inOl = false;

    const closeLists = () => {
      if (inUl) { inner += "</ul>"; inUl = false; }
      if (inOl) { inner += "</ol>"; inOl = false; }
    };

    for (const raw of bodyLines) {
      const line = raw.trimEnd();
      if (line.startsWith("```")) {
        closeLists();
        if (inCode) {
          const codeText = codeBuf.join("\n");
          inner += `<pre><button class="copy-btn" onclick="copyCode(this)">Copy</button><code>${escapeHtml(codeText)}</code></pre>`;
          codeBuf = [];
          inCode = false;
        } else {
          inCode = true;
        }
        continue;
      }
      if (inCode) { codeBuf.push(line); continue; }
      if (!line.trim()) { closeLists(); continue; }
      if (line.startsWith("- ")) {
        if (inOl) { inner += "</ol>"; inOl = false; }
        if (!inUl) { inner += "<ul>"; inUl = true; }
        inner += `<li>${formatInline(line.slice(2))}</li>`;
      } else if (/^\d+\.\s+/.test(line)) {
        if (inUl) { inner += "</ul>"; inUl = false; }
        if (!inOl) { inner += "<ol>"; inOl = true; }
        inner += `<li>${formatInline(line.replace(/^\d+\.\s+/, ""))}</li>`;
      } else {
        closeLists();
        inner += `<p>${formatInline(line)}</p>`;
      }
    }
    closeLists();
    html += `<section class="${cardClass}">${inner}</section>`;
  }
  return html;
}

window.copyCode = function(btn) {
  const code = btn.nextElementSibling.textContent;
  navigator.clipboard.writeText(code).then(() => {
    btn.textContent = "Copied!";
    setTimeout(() => { btn.textContent = "Copy"; }, 1400);
  });
};

function renderSidebar() {
  const tree = document.getElementById("sidebarTree");
  let html = `<button class="nav-home ${currentView.mode === "home" ? "active" : ""}" onclick="showHome()">Studio Atlas &amp; Overview</button>`;
  WORKBENCHES.forEach((wb, wIdx) => {
    html += `<div class="wb-group">
      <button class="wb-btn" onclick="showSubpage(${wIdx}, 0)">
        <span>${escapeHtml(wb.title.replace(/^Workbench\s+/, ""))}</span>
      </button>
      <div class="wb-subs">`;
    wb.subpages.forEach((sub, sIdx) => {
      const isActive = currentView.mode === "sub" && currentView.wIdx === wIdx && currentView.sIdx === sIdx;
      const shortTitle = sub.title.replace(/^Subpage\s+[\d.]+:\s*/, "");
      html += `<button class="sub-btn ${isActive ? "active" : ""}" onclick="showSubpage(${wIdx}, ${sIdx})">${escapeHtml(shortTitle)}</button>`;
    });
    html += `</div></div>`;
  });
  tree.innerHTML = html;
}

function showHome() {
  currentView = { mode: "home", wIdx: 0, sIdx: 0 };
  document.getElementById("searchInput").value = "";
  document.getElementById("breadcrumb").innerHTML = "<strong>SynthCraft Studio Handbook</strong> &middot; 14 Workbenches / 37 Subpages";
  let cards = WORKBENCHES.map((wb, wIdx) => {
    const tags = wb.subpages.map(s => `<span class="tag">${escapeHtml(s.title.replace(/^Subpage\s+[\d.]+:\s*/, ""))}</span>`).join("");
    return `<div class="wb-card" onclick="showSubpage(${wIdx}, 0)">
      <h3>${escapeHtml(wb.title)}</h3>
      <p>${formatInline(wb.intro)}</p>
      <div>${tags}</div>
    </div>`;
  }).join("");

  document.getElementById("content").innerHTML = `
    <div class="hero">
      <h2>SynthCraft Studio Handbook</h2>
      <p>Five decades of synthesizer history -- from 1970 Moog ladders, Don Buchla wavefolders, 1980s Roland TB-303 / TR-808 / TR-909, Korg M1 &amp; Yamaha DX7 to modern 2D FDTD physical modeling, 8x Wave Digital Filter circuits, Neural DDSP timbre transfer, and multi-core Genopatch -- explained in plain English with full hardware provenance and famous tracks.</p>
      <div class="stats">
        <div class="stat"><div class="num">14</div><div class="lbl">Hierarchical Workbenches</div></div>
        <div class="stat"><div class="num">${flatSubpages.length}</div><div class="lbl">Dedicated Subpage Guides</div></div>
        <div class="stat"><div class="num">60</div><div class="lbl">CLI &amp; MCP 2026-07-28 Tools</div></div>
        <div class="stat"><div class="num">CLAP + VCV</div><div class="lbl">Live DAW &amp; Rack 2 Verified</div></div>
      </div>
    </div>
    <h3>Explore All 14 Workbenches</h3>
    <div class="grid-cards">${cards}</div>
  `;
  renderSidebar();
  window.scrollTo({ top: 0, behavior: "smooth" });
}

function showSubpage(wIdx, sIdx) {
  currentView = { mode: "sub", wIdx, sIdx };
  const wb = WORKBENCHES[wIdx];
  const sub = wb.subpages[sIdx];
  const flatIdx = flatSubpages.findIndex(x => x.wIdx === wIdx && x.sIdx === sIdx);
  const prev = flatIdx > 0 ? flatSubpages[flatIdx - 1] : null;
  const next = flatIdx < flatSubpages.length - 1 ? flatSubpages[flatIdx + 1] : null;

  document.getElementById("breadcrumb").innerHTML =
    `${escapeHtml(wb.title)} &rsaquo; <strong>${escapeHtml(sub.title)}</strong>`;

  const pagerHtml = `<div class="pager">
    ${prev ? `<button onclick="showSubpage(${prev.wIdx}, ${prev.sIdx})">&larr; ${escapeHtml(prev.sub.title)}</button>` : "<span></span>"}
    ${next ? `<button onclick="showSubpage(${next.wIdx}, ${next.sIdx})">${escapeHtml(next.sub.title)} &rarr;</button>` : "<span></span>"}
  </div>`;

  document.getElementById("content").innerHTML = `
    <div class="subpage-header">
      <div class="kicker">${escapeHtml(wb.title)}</div>
      <h2>${escapeHtml(sub.title)}</h2>
    </div>
    ${renderMarkdownSections(sub.body)}
    ${pagerHtml}
  `;
  renderSidebar();
  window.scrollTo({ top: 0, behavior: "smooth" });
}

function runSearch(query) {
  const q = query.trim().toLowerCase();
  if (!q) {
    if (currentView.mode === "sub") showSubpage(currentView.wIdx, currentView.sIdx);
    else showHome();
    return;
  }
  const hits = flatSubpages.filter(item =>
    item.wb.title.toLowerCase().includes(q) ||
    item.sub.title.toLowerCase().includes(q) ||
    item.sub.body.toLowerCase().includes(q)
  );
  document.getElementById("breadcrumb").innerHTML = `Search results for <strong>"${escapeHtml(query)}"</strong> (${hits.length} matches)`;
  if (!hits.length) {
    document.getElementById("content").innerHTML = `<div class="section-card"><h3>No matches found</h3><p>Try searching for <code>303</code>, <code>808</code>, <code>Daft Punk</code>, <code>Buchla</code>, <code>DX7</code>, <code>OTT</code>, <code>vocoder</code>, or <code>MPE</code>.</p></div>`;
    return;
  }
  const html = hits.map(item => {
    const idx = item.sub.body.toLowerCase().indexOf(q);
    const start = Math.max(0, idx - 80);
    const snippet = item.sub.body.slice(start, start + 220).replace(/\n+/g, " ");
    return `<div class="search-hit" onclick="showSubpage(${item.wIdx}, ${item.sIdx})">
      <div class="meta">${escapeHtml(item.wb.title)}</div>
      <h4>${escapeHtml(item.sub.title)}</h4>
      <p>...${escapeHtml(snippet)}...</p>
    </div>`;
  }).join("");
  document.getElementById("content").innerHTML = html;
}

document.getElementById("searchInput").addEventListener("input", (e) => runSearch(e.target.value));
document.getElementById("homeBtn").addEventListener("click", showHome);
document.getElementById("themeBtn").addEventListener("click", () => {
  document.body.classList.toggle("light");
  document.getElementById("themeBtn").textContent =
    document.body.classList.contains("light") ? "Obsidian Dark" : "Daylight Mode";
});

showHome();
</script>
</body>
</html>
"""


def main() -> None:
    wb_files = sorted(WB_DIR.glob("*.md"))
    workbenches = [parse_workbench(p) for p in wb_files]
    total_subs = sum(len(w["subpages"]) for w in workbenches)
    OUT_HTML.parent.mkdir(parents=True, exist_ok=True)
    data_json = json.dumps(workbenches, ensure_ascii=True)
    html = HTML_TEMPLATE.replace("__DATA_JSON__", data_json)
    OUT_HTML.write_text(html, encoding="utf-8", newline="\n")
    print(f"Generated {OUT_HTML} ({len(workbenches)} workbenches, {total_subs} subpages, {len(html):,} bytes)")


if __name__ == "__main__":
    main()

#!/usr/bin/env node
/**
 * Post-build script: inject llms.txt discovery signals into every built HTML page.
 *
 * Agents that fetch the HTML version of a page have no built-in way to discover that a
 * documentation index exists, or that a clean markdown version of the page is available.
 * The Agent-Friendly Documentation spec (afdocs.dev) addresses both:
 *
 *   1. A visually-hidden link to the documentation index, placed right after the opening
 *      <body> tag (before the React root, so there is no hydration mismatch).
 *   2. A <link rel="alternate" type="text/markdown"> in <head> advertising the clean
 *      markdown sibling the SignalWire llms-txt plugin emits for the page. Only injected
 *      when the sibling .md actually exists in the build output, so the href is never
 *      broken.
 *
 * The site is served under the `/docs/` baseUrl, so both hrefs are `/docs/`-prefixed.
 * Both injections are idempotent: pages already carrying them are skipped.
 */

const fs = require("fs");
const path = require("path");

const BUILD_DIR = path.join(__dirname, "..", "build");
const BASE = "/docs";

const MARKER = "data-llms-txt-directive";

// Visually-hidden via the standard clip-rect pattern so the directive is in the DOM body
// for agents but invisible to human readers.
const DIRECTIVE =
  `<div ${MARKER} style="position:absolute;width:1px;height:1px;padding:0;` +
  `margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0;">` +
  `For the complete documentation index, see ` +
  `<a href="${BASE}/llms.txt" tabindex="-1">llms.txt</a>.</div>`;

const ALTERNATE_REL = 'rel="alternate" type="text/markdown"';
const BODY_TAG = /<body[^>]*>/i;
const HEAD_CLOSE = /<\/head>/i;

function* walkHtml(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walkHtml(full);
    } else if (entry.isFile() && entry.name.endsWith(".html")) {
      yield full;
    }
  }
}

/**
 * Map a built HTML file to the build-relative path of its markdown sibling, if one
 * exists: build/X/index.html -> X.md ; build/X.html -> X.md ; build/index.html ->
 * index.md. Returns the build-relative md path (forward slashes) or null.
 */
function markdownSibling(htmlFile) {
  const rel = path.relative(BUILD_DIR, htmlFile).split(path.sep).join("/");
  let base;
  if (rel === "index.html") {
    base = "index";
  } else if (rel.endsWith("/index.html")) {
    base = rel.slice(0, -"/index.html".length);
  } else if (rel.endsWith(".html")) {
    base = rel.slice(0, -".html".length);
  } else {
    return null;
  }
  for (const candidate of [`${base}.md`, `${base}/index.md`]) {
    if (fs.existsSync(path.join(BUILD_DIR, candidate))) return candidate;
  }
  return null;
}

function main() {
  if (!fs.existsSync(BUILD_DIR)) {
    console.error(
      `Error: ${BUILD_DIR} not found. Run the build before this script.`,
    );
    process.exit(1);
  }

  let bodyInjected = 0;
  let altInjected = 0;
  let skipped = 0;
  let noBody = 0;

  for (const file of walkHtml(BUILD_DIR)) {
    let html = fs.readFileSync(file, "utf-8");
    let changed = false;

    // 1. Per-page markdown alternate in <head>.
    const md = markdownSibling(file);
    if (md && !html.includes(ALTERNATE_REL)) {
      const headMatch = html.match(HEAD_CLOSE);
      if (headMatch) {
        const link = `<link ${ALTERNATE_REL} href="${BASE}/${md}">`;
        html = html.slice(0, headMatch.index) + link + html.slice(headMatch.index);
        altInjected++;
        changed = true;
      }
    }

    // 2. Visually-hidden documentation-index link after <body>.
    if (html.includes(MARKER)) {
      skipped++;
    } else {
      const match = html.match(BODY_TAG);
      if (!match) {
        noBody++;
      } else {
        const insertAt = match.index + match[0].length;
        html = html.slice(0, insertAt) + DIRECTIVE + html.slice(insertAt);
        bodyInjected++;
        changed = true;
      }
    }

    if (changed) fs.writeFileSync(file, html);
  }

  console.log(
    `Injected llms.txt directive into ${bodyInjected} page(s) and a markdown ` +
      `alternate into ${altInjected} page(s) ` +
      `(${skipped} already had the directive, ${noBody} had no <body>).`,
  );
}

main();

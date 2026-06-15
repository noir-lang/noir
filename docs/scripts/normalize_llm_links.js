#!/usr/bin/env node
/**
 * Post-build script: collapse a doubled `/docs/` base in the generated markdown links.
 *
 * The site is served under the `/docs/` baseUrl. When `relativePaths` is false, the
 * @signalwire/docusaurus-plugin-llms-txt plugin builds absolute URLs for the llms.txt
 * index correctly (`https://noir-lang.org/docs/<page>`), but for links *inside* a page's
 * rendered content it prepends the baseUrl to an href that already carries it, producing
 * a broken `…/docs/docs/<page>`.
 *
 * With baseUrl `/docs/` and routeBasePath `/`, every generated route is served at
 * `/docs/<page>` — none under `/docs/docs/` — so collapsing every `/docs/docs/` to
 * `/docs/` is safe and fixes only the doubled links. Runs over every generated page .md
 * plus the aggregate llms.txt / llms-full.txt.
 *
 * Idempotent: a second run finds nothing left to collapse.
 */

const fs = require("fs");
const path = require("path");

const BUILD_DIR = path.join(__dirname, "..", "build");
const DOUBLED = /\/docs\/docs\//g;

function* walkTargets(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walkTargets(full);
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      yield full;
    }
  }
}

function main() {
  if (!fs.existsSync(BUILD_DIR)) {
    console.error(
      `Error: ${BUILD_DIR} not found. Run the build before this script.`,
    );
    process.exit(1);
  }

  const targets = [...walkTargets(BUILD_DIR)];
  for (const name of ["llms.txt", "llms-full.txt"]) {
    const p = path.join(BUILD_DIR, name);
    if (fs.existsSync(p)) targets.push(p);
  }

  let filesFixed = 0;
  let occurrences = 0;
  for (const file of targets) {
    const content = fs.readFileSync(file, "utf-8");
    const matches = content.match(DOUBLED);
    if (!matches) continue;
    fs.writeFileSync(file, content.replace(DOUBLED, "/docs/"));
    filesFixed++;
    occurrences += matches.length;
  }

  console.log(
    `Collapsed ${occurrences} doubled /docs/docs/ link(s) across ${filesFixed} file(s).`,
  );
}

main();

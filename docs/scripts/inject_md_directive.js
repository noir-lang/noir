#!/usr/bin/env node
/**
 * Post-build script: prepend an llms.txt discovery pointer to every generated per-page
 * markdown file.
 *
 * Agents that fetch the markdown version of a page benefit from a pointer back to the
 * documentation index. The Agent-Friendly Documentation spec (afdocs.dev) satisfies this
 * with a blockquote near the top of each markdown page:
 *
 *   > For the complete documentation index, see [llms.txt](/docs/llms.txt)
 *
 * The SignalWire llms-txt plugin writes a .md sibling for every route into the build
 * output but does not add this pointer, so we inject it here. The aggregate indexes
 * (llms.txt / llms-full.txt) are .txt and never matched.
 *
 * Idempotent: pages already carrying the pointer are skipped.
 */

const fs = require("fs");
const path = require("path");

const BUILD_DIR = path.join(__dirname, "..", "build");

const DIRECTIVE =
  "> For the complete documentation index, see [llms.txt](/docs/llms.txt)";

function* walkMarkdown(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      yield* walkMarkdown(full);
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      yield full;
    }
  }
}

/**
 * Insert the directive after a leading YAML frontmatter block if present, so we never
 * break frontmatter parsing; otherwise prepend it to the top of the file.
 */
function withDirective(content) {
  const block = `${DIRECTIVE}\n\n`;
  const frontMatter = content.match(/^---\r?\n[\s\S]*?\r?\n---\r?\n/);
  if (frontMatter) {
    const end = frontMatter[0].length;
    return content.slice(0, end) + "\n" + block + content.slice(end);
  }
  return block + content;
}

function main() {
  if (!fs.existsSync(BUILD_DIR)) {
    console.error(
      `Error: ${BUILD_DIR} not found. Run the build before this script.`,
    );
    process.exit(1);
  }

  let injected = 0;
  let skipped = 0;

  for (const file of walkMarkdown(BUILD_DIR)) {
    const content = fs.readFileSync(file, "utf-8");
    if (content.includes(DIRECTIVE)) {
      skipped++;
      continue;
    }
    fs.writeFileSync(file, withDirective(content));
    injected++;
  }

  console.log(
    `Injected llms.txt directive into ${injected} markdown page(s) ` +
      `(${skipped} already had it).`,
  );
}

main();

const fs = require('fs');
const path = require('path');
const childProcess = require('child_process');

// Keep the noirjs_app tutorial's hardcoded published-package versions in sync
// with the versions this repo actually ships, so they cannot silently drift.
//
// The tutorial pins published versions (noir_js, @aztec/bb.js,
// vite-plugin-node-polyfills) in prose. Those are not `#include_code` snippets,
// so nothing else keeps them current. This script is the single source of truth.
//
// Both `--check` and `--write` resolve the same way: read the last release tag
// `v<manifest>` if it exists, otherwise the working tree. On a normal PR the tag
// exists, so the tutorial is pinned to the last release. On the release-please PR
// (which cuts the docs version before the tag is published) the tag is absent, so
// the working tree is used, which on that branch already is the new release.
//
// `.release-please-manifest.json["."]` is the noir/noir_js version; it is "latest
// release" on master and "next release" on the release-please PR.

const ROOT = path.resolve(__dirname, '../..');
const TUTORIAL = 'docs/docs/tutorials/noirjs_app.md';
const MANIFEST = '.release-please-manifest.json';
const INSTALL_BB = 'scripts/install_bb.sh';
const BROWSER_PKG = 'examples/browser/package.json';

const WORKTREE = 'WORKTREE';

function tagExists(tag) {
  try {
    childProcess.execFileSync('git', ['rev-parse', '--verify', '--quiet', `refs/tags/${tag}`], { cwd: ROOT });
    return true;
  } catch (err) {
    return false;
  }
}

function isShallowRepository() {
  const out = childProcess.execFileSync('git', ['rev-parse', '--is-shallow-repository'], { cwd: ROOT });
  return out.toString().trim() === 'true';
}

function readAtRef(ref, relPath) {
  if (ref === WORKTREE) return fs.readFileSync(path.resolve(ROOT, relPath), 'utf-8');
  return childProcess.execFileSync('git', ['show', `${ref}:${relPath}`], { cwd: ROOT }).toString();
}

function bbVersionFrom(text) {
  const match = text.match(/^VERSION="([^"]+)"/m);
  if (!match) throw new Error(`Could not parse VERSION from ${INSTALL_BB}`);
  return match[1];
}

function polyfillsVersionFrom(text) {
  const version = JSON.parse(text).devDependencies?.['vite-plugin-node-polyfills'];
  if (!version) throw new Error(`Could not find vite-plugin-node-polyfills in ${BROWSER_PKG}`);
  return version;
}

function occurrenceError(label, count) {
  return `${label}: expected exactly 1 occurrence, found ${count} (did the surrounding text change?)`;
}

// Each target anchors on stable surrounding text and captures (prefix, version, suffix).
function targets({ noir, bb, poly }) {
  return [
    { label: 'noir_js dependency', regex: /(@noir-lang\/noir_js@)(\S+?)(\s)/g, want: noir },
    { label: 'bb.js dependency', regex: /(@aztec\/bb\.js@)(\S+?)(\s)/g, want: bb },
    {
      label: 'vite-plugin-node-polyfills dependency',
      regex: /(vite-plugin-node-polyfills@)(\S+?)(\s|$)/gm,
      want: poly,
    },
    { label: 'noirup install command', regex: /(noirup -v )([^\s\x60]+)()/g, want: noir },
    { label: 'nargo version sentence', regex: /(also need version )(\S+?)( nargo)/g, want: noir },
    { label: 'pinned-versions note', regex: /(versions pinned to )(\S+?)(\. )/g, want: noir },
    { label: 'Barretenberg version note', regex: /(Barretenberg version )([^,\s]+)(,)/g, want: bb },
  ];
}

function resolveVersions(expect) {
  const noir = JSON.parse(fs.readFileSync(path.resolve(ROOT, MANIFEST), 'utf-8'))['.'];
  if (!noir) throw new Error(`Could not read version from ${MANIFEST}`);
  if (expect && expect.replace(/^v/, '') !== noir) {
    throw new Error(`Expected version ${expect} does not match ${MANIFEST} (${noir}).`);
  }

  // Resolve bb.js/polyfills from the last release tag. A missing tag is only
  // trusted as "this version is not tagged yet" (the release-please PR that cuts
  // it) when the clone is complete; in a shallow clone a missing tag may just be
  // unfetched, so we fail rather than risk pinning master's incompatible versions.
  const tag = `v${noir}`;
  let ref;
  if (tagExists(tag)) {
    ref = tag;
  } else if (isShallowRepository()) {
    throw new Error(`Release tag ${tag} is unavailable in a shallow clone. Checkout with fetch-depth: 0.`);
  } else {
    ref = WORKTREE;
  }
  console.log(
    `[sync_tutorial_versions] noir_js ${noir}; bb.js/polyfills source: ${ref === WORKTREE ? 'working tree' : tag}`,
  );

  return {
    noir,
    bb: bbVersionFrom(readAtRef(ref, INSTALL_BB)),
    poly: polyfillsVersionFrom(readAtRef(ref, BROWSER_PKG)),
  };
}

function check(content, want) {
  const errors = [];
  for (const { label, regex, want: expected } of targets(want)) {
    const matches = [...content.matchAll(regex)];
    if (matches.length !== 1) {
      errors.push(occurrenceError(label, matches.length));
      continue;
    }
    if (matches[0][2] !== expected) errors.push(`${label}: found "${matches[0][2]}", expected "${expected}"`);
  }
  return errors;
}

function write(content, want) {
  let updated = content;
  for (const { label, regex, want: expected } of targets(want)) {
    const matches = [...updated.matchAll(regex)];
    if (matches.length !== 1) {
      throw new Error(occurrenceError(label, matches.length));
    }
    updated = updated.replace(regex, (_match, prefix, _version, suffix) => `${prefix}${expected}${suffix}`);
  }
  return updated;
}

function main() {
  const mode = process.argv[2];
  if (mode !== '--check' && mode !== '--write') {
    console.error('Usage: node docs/scripts/sync_tutorial_versions.js --check|--write [--expect <version>]');
    process.exit(2);
  }

  const expectIndex = process.argv.indexOf('--expect');
  const expect = expectIndex === -1 ? null : process.argv[expectIndex + 1];

  const want = resolveVersions(expect);
  const tutorialPath = path.resolve(ROOT, TUTORIAL);
  const content = fs.readFileSync(tutorialPath, 'utf-8');
  const summary = `noir_js ${want.noir}, bb.js ${want.bb}, polyfills ${want.poly}`;

  if (mode === '--check') {
    const errors = check(content, want);
    if (errors.length) {
      for (const error of errors) console.error(`::error file=${TUTORIAL}::${error}`);
      console.error(
        `\n${TUTORIAL} is out of sync. Run \`node docs/scripts/sync_tutorial_versions.js --write\` to fix.`,
      );
      process.exit(1);
    }
    console.log(`[sync_tutorial_versions] ${TUTORIAL} is in sync (${summary}).`);
    return;
  }

  const updated = write(content, want);
  if (updated === content) {
    console.log(`[sync_tutorial_versions] ${TUTORIAL} already up to date (${summary}).`);
    return;
  }
  fs.writeFileSync(tutorialPath, updated);
  console.log(`[sync_tutorial_versions] updated ${TUTORIAL} (${summary}).`);
}

try {
  main();
} catch (err) {
  console.error(`::error file=${TUTORIAL}::${err.message}`);
  process.exit(1);
}

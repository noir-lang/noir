// Shared helpers for the route-snapshot guard: read the routes Docusaurus actually emitted for
// the unversioned (`current`) docs, and resolve a path against the redirect table.
//
// Reading the emitted routes rather than walking `docs/` means the snapshot follows `slug:` and
// `id:` frontmatter, `routeBasePath`, and anything else that decides a page's URL. A path-based
// model would miss a frontmatter `slug:` edit, which changes a URL with no file rename at all.

const fs = require('fs');
const path = require('path');

const { legacyPathRedirects } = require('../redirects');

const DOCS_DIR = path.join(__dirname, '..');
const GLOBAL_DATA_PATH = path.join(DOCS_DIR, '.docusaurus', 'globalData.json');
const SNAPSHOT_PATH = path.join(DOCS_DIR, 'routes.snapshot.json');
const ALLOWLIST_PATH = path.join(DOCS_DIR, 'removed-urls.json');

// Typedoc regenerates the NoirJS reference from the exported symbols of `tooling/noir_js` and
// `tooling/noir_wasm`, so its routes track the TypeScript API rather than anything an author
// writes here. Snapshotting them would demand a snapshot refresh on every JS API change while
// protecting URLs nobody deep-links from outside the site.
const EXCLUDED_PREFIXES = ['/reference/NoirJS/'];

/** Site-relative, version-agnostic form: no trailing slash, `/` for the docs root. */
function normalizePath(pathname) {
  const trimmed = pathname.replace(/\/+$/, '');
  return trimmed === '' ? '/' : trimmed;
}

/**
 * Paths of every page in the `current` docs version, as `/guides/oracles` rather than the
 * `/docs/dev/guides/oracles` the built site serves them at. This is the same space
 * `legacyPathRedirects` is written in, so the two can be compared directly.
 */
function readCurrentVersionPaths() {
  if (!fs.existsSync(GLOBAL_DATA_PATH)) {
    throw new Error(
      `${path.relative(DOCS_DIR, GLOBAL_DATA_PATH)} not found. Run \`yarn build\` in docs/ first — ` +
        'the route list is produced by a Docusaurus build.',
    );
  }

  const globalData = JSON.parse(fs.readFileSync(GLOBAL_DATA_PATH, 'utf-8'));
  const docsPlugin = globalData['docusaurus-plugin-content-docs'];
  const versions = docsPlugin && docsPlugin.default && docsPlugin.default.versions;
  if (!versions) {
    throw new Error('Could not find docs plugin versions in globalData.json.');
  }

  const current = versions.find((version) => version.name === 'current');
  if (!current) {
    throw new Error("Could not find the 'current' docs version in globalData.json.");
  }

  const versionPrefix = normalizePath(current.path);
  const paths = current.docs.map((doc) => {
    const normalized = normalizePath(doc.path);
    if (versionPrefix === '/') return normalized;
    if (normalized === versionPrefix) return '/';
    if (!normalized.startsWith(`${versionPrefix}/`)) {
      throw new Error(`Route ${doc.path} is not under the current version prefix ${versionPrefix}.`);
    }
    return normalized.slice(versionPrefix.length);
  });

  return paths.filter((p) => !EXCLUDED_PREFIXES.some((prefix) => p.startsWith(prefix))).sort();
}

/**
 * First matching rule, mirroring Netlify's `_redirects` semantics: rules are tried in order, a
 * source ending in `/*` matches anything below that directory, and `:splat` in the target is
 * replaced by the matched remainder.
 *
 * `/a/b/*` also matches a bare `/a/b`, with an empty splat, because Netlify matches the empty
 * remainder: a request for `/docs/noir/concepts/data_types` is served by the
 * `/docs/noir/concepts/data_types/*` rule as `/docs/language/data_types/`, trailing slash and all.
 * Letting a bare path fall through to a broader rule instead would resolve it against a rule
 * Netlify never reaches, so a table that 404s in production could pass this check.
 */
function matchRedirect(pathname, rules = legacyPathRedirects) {
  for (const [from, to] of rules) {
    if (from.endsWith('/*')) {
      const prefix = from.slice(0, -1);
      const remainder = pathname === prefix.slice(0, -1) ? '' : null;
      if (remainder !== null || pathname.startsWith(prefix)) {
        const splat = remainder !== null ? remainder : pathname.slice(prefix.length);
        // An empty splat leaves the target with a trailing slash, which names the same page.
        return { from, to, target: normalizePath(to.replace(':splat', splat)) };
      }
    } else if (pathname === from) {
      return { from, to, target: to };
    }
  }
  return null;
}

function readSnapshot() {
  if (!fs.existsSync(SNAPSHOT_PATH)) return null;
  return JSON.parse(fs.readFileSync(SNAPSHOT_PATH, 'utf-8')).paths;
}

function writeSnapshot(paths) {
  const contents = { generatedBy: 'yarn routes:snapshot', paths };
  fs.writeFileSync(SNAPSHOT_PATH, `${JSON.stringify(contents, null, 2)}\n`);
}

/** Paths that are allowed to 404 because the page has no successor. */
function readAllowlist() {
  if (!fs.existsSync(ALLOWLIST_PATH)) return new Set();
  const parsed = JSON.parse(fs.readFileSync(ALLOWLIST_PATH, 'utf-8'));
  return new Set((parsed.urls || []).map((entry) => entry.path));
}

module.exports = {
  ALLOWLIST_PATH,
  DOCS_DIR,
  EXCLUDED_PREFIXES,
  SNAPSHOT_PATH,
  legacyPathRedirects,
  matchRedirect,
  normalizePath,
  readAllowlist,
  readCurrentVersionPaths,
  readSnapshot,
  writeSnapshot,
};

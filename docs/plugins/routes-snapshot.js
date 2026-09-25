// Fails the build when a docs page that used to be served has no redirect pointing at its
// successor, and when a redirect in `redirects.js` points at a page that no longer exists.
//
// External sites, blog posts and search results link straight at page URLs, and `onBrokenLinks`
// cannot see them — it only validates links inside this site. `routes.snapshot.json` records the
// URL of every page in the unversioned (`current`) docs so a removal can be noticed at all.
//
// The routes come from the docs plugin's loaded content rather than from walking `docs/`, so the
// snapshot follows `slug:` and `id:` frontmatter, `routeBasePath`, and anything else that decides
// a page's URL. A path-based model would miss a frontmatter `slug:` edit, which changes a URL with
// no file rename at all.
//
// `ROUTES_SNAPSHOT=update` rewrites the snapshot from the build instead of checking it.

const fs = require('fs');
const path = require('path');

const { legacyPathRedirects } = require('../redirects');

const DOCS_DIR = path.join(__dirname, '..');
const SNAPSHOT_PATH = path.join(DOCS_DIR, 'routes.snapshot.json');
const ALLOWLIST_PATH = path.join(DOCS_DIR, 'removed-urls.json');
const SNAPSHOT_FILE = path.relative(DOCS_DIR, SNAPSHOT_PATH);
const ALLOWLIST_FILE = path.relative(DOCS_DIR, ALLOWLIST_PATH);

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
function currentVersionPaths(docsContent) {
  const current = docsContent.loadedVersions.find((version) => version.versionName === 'current');
  if (!current) {
    throw new Error("Could not find the 'current' docs version.");
  }

  const versionPrefix = normalizePath(current.path);
  const paths = current.docs.map((doc) => {
    const normalized = normalizePath(doc.permalink);
    if (versionPrefix === '/') return normalized;
    if (normalized === versionPrefix) return '/';
    if (!normalized.startsWith(`${versionPrefix}/`)) {
      throw new Error(`Route ${doc.permalink} is not under the current version prefix ${versionPrefix}.`);
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

function checkRemovedPaths(removed, currentPaths, allowlist) {
  const errors = [];

  for (const removedPath of removed) {
    if (allowlist.has(removedPath)) continue;

    const match = matchRedirect(removedPath);
    if (!match) {
      errors.push(
        `${removedPath} is no longer served and nothing redirects it.\n` +
          `    Add a rule to redirects.js, e.g. ['${removedPath}', '<new path>'], or record the\n` +
          `    path in ${ALLOWLIST_FILE} with a reason if the page has no successor.`,
      );
    } else if (!currentPaths.has(match.target)) {
      errors.push(
        `${removedPath} redirects to ${match.target}, which is not a page.\n` +
          `    The rule ['${match.from}', '${match.to}'] in redirects.js sends readers to a 404.`,
      );
    }
  }

  return errors;
}

function checkRedirectTargets(currentPaths) {
  const errors = [];

  for (const [from, to] of legacyPathRedirects) {
    if (to.includes(':splat')) {
      // A wildcard target can only be resolved against a concrete request, so check the fixed
      // part of it still names somewhere pages live.
      const prefix = to.slice(0, to.indexOf(':splat'));
      if (![...currentPaths].some((p) => p.startsWith(prefix))) {
        errors.push(`The rule ['${from}', '${to}'] in redirects.js targets ${prefix}, where no page lives.`);
      }
    } else if (!currentPaths.has(to)) {
      errors.push(`The rule ['${from}', '${to}'] in redirects.js targets a page that does not exist.`);
    }
  }

  return errors;
}

/** Every problem with `paths` against the committed snapshot, one message per problem. */
function checkRoutes(paths) {
  const snapshot = readSnapshot();
  if (!snapshot) {
    return [`${SNAPSHOT_FILE} is missing. Run \`yarn routes:snapshot\` in docs/ and commit it.`];
  }

  const currentPaths = new Set(paths);
  const removed = snapshot.filter((p) => !currentPaths.has(p));
  const added = paths.filter((p) => !snapshot.includes(p));

  const errors = [
    ...new Set([...checkRemovedPaths(removed, currentPaths, readAllowlist()), ...checkRedirectTargets(currentPaths)]),
  ];

  if (removed.length > 0 || added.length > 0) {
    errors.push(
      `${SNAPSHOT_FILE} is out of date: ${added.length} page(s) added, ${removed.length} removed.\n` +
        'Run `yarn routes:snapshot` in docs/ and commit the result.\n' +
        [...added.map((p) => `  + ${p}`), ...removed.map((p) => `  - ${p}`)].join('\n'),
    );
  }

  return errors;
}

/** @type {import('@docusaurus/types').PluginModule} */
function routesSnapshotPlugin() {
  return {
    name: 'routes-snapshot',
    // Runs once every plugin has loaded its content and before bundling, so a missing redirect
    // fails the build in seconds rather than after the site has been compiled.
    async allContentLoaded({ allContent }) {
      const docsContent = allContent['docusaurus-plugin-content-docs'].default;
      const paths = currentVersionPaths(docsContent);

      if (process.env.ROUTES_SNAPSHOT === 'update') {
        writeSnapshot(paths);
        console.log(`Wrote ${paths.length} paths to ${SNAPSHOT_FILE}.`);
        return;
      }

      const errors = checkRoutes(paths);
      if (errors.length > 0) {
        for (const error of errors) console.error(`::error::${error}`);
        throw new Error(`${errors.length} docs route problem(s); see above.`);
      }
      console.log(`${SNAPSHOT_FILE} is up to date (${paths.length} paths).`);
    },
  };
}

module.exports = routesSnapshotPlugin;

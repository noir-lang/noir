// Browser-safe route math for the agent-readable docs, shared with the build-time
// wiring in llmsTxt.js.
//
// This module is imported by src/theme/Root.js, so it ships in the client bundle and
// MUST NOT reference `process.env` or any other Node-only API — Docusaurus does not
// define arbitrary `process.env.*` in the browser, and a top-level access throws
// `process is not defined` during hydration. Keep this to pure path string math; the
// Docusaurus plugin/remark wiring (which does read `process.env.ENV`) lives in
// llmsTxt.js, which only the build/config imports.

const versions = require('../versions.json');

const olderVersions = versions.slice(1);

// First path segments (relative to baseUrl) for which the llms-txt plugin emits no
// markdown sibling, so a page under them must not advertise a (nonexistent) `.md`:
// the leading segments of the routes dropped from the index (the `llmExcludeRoutes`
// set in llmsTxt.js — search/tags/dev/older stable versions) plus the built `404.html`
// page (never indexed; unlike the extensionless doc routes its pathname carries a file
// extension, so it is listed explicitly).
const EXCLUDED_SEGMENTS = new Set(['search', 'tags', 'dev', '404', '404.html', ...olderVersions]);

/**
 * Map a served page pathname (which already carries the baseUrl, e.g. `/docs/foo`) to
 * the absolute path of its markdown sibling (`/docs/foo.md`), or null when the page has
 * no sibling (the homepage maps to the root `index.md`; excluded routes map to null).
 * `baseUrl` is passed in (not read from the environment) so the result is identical
 * during server rendering and client hydration.
 */
function markdownSiblingForPathname(pathname, baseUrl) {
  if (!pathname) return null;
  const base = (baseUrl || '/').replace(/\/+$/, ''); // '/docs' or ''
  let p = pathname;
  if (p.length > 1 && p.endsWith('/')) p = p.slice(0, -1);
  if (p === base || p === '' || p === '/') return `${base}/index.md`;
  const rel = base && (p === base || p.startsWith(`${base}/`)) ? p.slice(base.length) : p;
  const firstSegment = rel.split('/').filter(Boolean)[0];
  if (firstSegment && EXCLUDED_SEGMENTS.has(firstSegment)) return null;
  return `${p}.md`;
}

module.exports = {
  olderVersions,
  markdownSiblingForPathname,
};

// Build-time wiring for agent-readable docs (llms.txt + a markdown sibling per page).
//
// Imported only by docusaurus.config.ts (a Node/build context), so it may read
// `process.env.ENV`. The browser-safe path helper used by the theme `Root` lives in
// llmsPaths.js instead, so the client bundle never pulls this module (and its
// `process.env` access) in. Everything here plugs into a native Docusaurus extension
// point rather than walking the built site after the fact:
//
//   - `llmExcludeRoutes`        -> sitemap `ignorePatterns` + the llms-txt plugin's
//                                  `content.excludeRoutes` (coverage denominator).
//   - `remarkLlmsTweaks`        -> the llms-txt plugin's `content.remarkPlugins`, run on
//                                  each generated page's markdown AST.
//   - `llmsDiscoveryPlugin`     -> a plugin whose `injectHtmlTags()` adds the hidden
//                                  documentation-index link to every page.

const { olderVersions } = require('./llmsPaths');

// baseUrl the site is served under: '/docs/' for prod/staging, '/' for local dev.
const BASE_URL = process.env.ENV === 'dev' ? '/' : '/docs/';
const BASE = BASE_URL.replace(/\/+$/, ''); // '/docs' or '' (root)

// Routes kept OUT of both the llms.txt/markdown index and the sitemap, so the
// agent-readiness coverage denominator equals exactly the pages we index: the latest
// stable version (`versions[0]`, served at the site root) plus the reference. Older
// stable snapshots, the unreleased `dev` version, and utility routes are dropped. Both
// base-aware (`/docs/...`) and base-relative forms are listed because Docusaurus route
// matching (plugin) and sitemap path matching differ; a form that doesn't apply simply
// matches nothing.
const llmExcludeRoutes = [
  '/search',
  '/docs/search',
  '/tags',
  '/tags/**',
  '/**/tags',
  '/**/tags/**',
  '/dev',
  '/dev/**',
  '/docs/dev',
  '/docs/dev/**',
  ...olderVersions.flatMap((v) => [`/${v}`, `/${v}/**`, `/docs/${v}`, `/docs/${v}/**`]),
];

// Pointer to the documentation index, used in both the in-page markdown directive and
// the hidden HTML body link.
const LLMS_TXT_HREF = `${BASE}/llms.txt`;

/**
 * remark plugin run after the llms-txt plugin's built-in processing on each generated
 * page's markdown AST. It:
 *   1. collapses the doubled baseUrl the plugin emits on in-page links under a non-root
 *      baseUrl (`/docs/docs/...` -> `/docs/...`); with baseUrl `/docs/` and
 *      routeBasePath `/`, every route is served at `/docs/<page>` and none under
 *      `/docs/docs/`, so the collapse only ever fixes the doubled links; and
 *   2. prepends a blockquote pointing at the documentation index.
 */
function remarkLlmsTweaks() {
  const doubled = BASE ? new RegExp(`${BASE}${BASE}/`, 'g') : null;
  const collapse = (node) => {
    if (!node || typeof node !== 'object') return;
    if (doubled && typeof node.url === 'string') {
      node.url = node.url.replace(doubled, `${BASE}/`);
    }
    if (Array.isArray(node.children)) node.children.forEach(collapse);
  };
  return (tree) => {
    collapse(tree);
    tree.children.unshift({
      type: 'blockquote',
      children: [
        {
          type: 'paragraph',
          children: [
            { type: 'text', value: 'For the complete documentation index, see ' },
            { type: 'link', url: LLMS_TXT_HREF, children: [{ type: 'text', value: 'llms.txt' }] },
          ],
        },
      ],
    });
  };
}

// Visually-hidden (standard clip-rect pattern) so the directive is in the DOM body for
// agents but invisible to human readers.
const BODY_DIRECTIVE =
  `<div data-llms-txt-directive style="position:absolute;width:1px;height:1px;padding:0;` +
  `margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0;">` +
  `For the complete documentation index, see ` +
  `<a href="${LLMS_TXT_HREF}" tabindex="-1">llms.txt</a>.</div>`;

/**
 * Docusaurus plugin that injects the hidden documentation-index link right after the
 * opening <body> tag of every built page, via the native `injectHtmlTags` lifecycle.
 */
function llmsDiscoveryPlugin() {
  return {
    name: 'llms-txt-discovery',
    injectHtmlTags() {
      return { preBodyTags: [BODY_DIRECTIVE] };
    },
  };
}

module.exports = {
  llmExcludeRoutes,
  remarkLlmsTweaks,
  llmsDiscoveryPlugin,
};

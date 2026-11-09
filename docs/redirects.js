// Documentation URLs that external sites, blog posts and search results still point at, each
// mapped to the page that now holds the content. `docusaurus.config.ts` renders these into the
// Netlify `_redirects` file as 301s, and `plugins/routes-snapshot.js` reads them to verify
// that no page can be removed without one.
//
// Netlify applies rules in order and stops at the first match, so the generated rules are
// emitted before the `/docs/*` catch-all rewrite, and more specific sources come first — a
// wildcard placed above a literal would shadow it. Every source is anchored at a top-level
// segment, so none of them can swallow a versioned path, which always carries its version as the
// first segment (`/docs/<version>/...`).
//
// Paths are version-agnostic and site-relative: `/guides/oracles`, not `/docs/guides/oracles`.

/** @type {[from: string, to: string][]} */
const legacyPathRedirects = [
  ['/explainers/explainer-writing-noir', '/guides/thinking_in_circuits'],
  ['/explainers/explainer-oracle', '/guides/oracles'],
  ['/how_to/how-to-oracles', '/guides/how_to_use_oracles'],
  ['/how_to/debugger/*', '/guides/debugging/:splat'],
  ['/tutorials/noirjs_app', '/guides/building_a_web_app'],
  ['/noir/concepts/data_types/*', '/language/data_types/:splat'],
  ['/noir/concepts/*', '/language/:splat'],
  ['/noir/modules_packages_crates/*', '/project_structure/:splat'],
  ['/noir/standard_library/containers', '/libraries/standard_library/containers/boundedvec'],
  [
    '/noir/standard_library/cryptographic_primitives/ecdsa_sig_verification',
    '/libraries/standard_library/cryptographic_primitives/signatures',
  ],
  ['/noir/standard_library/*', '/libraries/standard_library/:splat'],
  ['/reference/debugger/*', '/tooling/debugger/:splat'],
  ['/reference/noir_codegen', '/tooling/noir_codegen'],
  ['/getting_started/noir_installation', '/installation'],
  ['/getting_started/*', '/getting_started_manually'],
];

module.exports = { legacyPathRedirects };

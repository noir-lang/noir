# Noir Docs

This is the source code for the Noir documentation site at [noir-lang.org](https://noir-lang.org).

This website is built using [Docusaurus 3](https://docusaurus.io/), a modern static website
generator.

> **Note**: The docs folder is now a standalone project and no longer part of the main Noir yarn workspace.
> This change was made to resolve conflicts with Netlify deployments.

## Contributing

Interested in contributing to the docs?

Check out the contributing guide [here](../CONTRIBUTING.md).

## Development

### Prerequisites

- Node.js (tested and working with v21.6.1)
- Yarn (tested and working with v4.5.2)

### Installation

This project requires recent versions of Rust and Cargo to be installed.
Any build errors should indicate dependencies that need installing, and at what version.

Navigate to the docs directory and install dependencies:

```sh
cd docs
yarn install
```

### Local Development

From the _docs_ directory:

1. Fetch and generate the list of recent stable documentation versions to build:

```sh
yarn version::stables
```

2. Start a development server serving docs preview:

```sh
yarn dev
```

This command starts a local development server and opens up a browser window. Most changes are
reflected live without having to restart the server.

### Build

From the _docs_ directory:

1. Fetch and generate the list of recent stable documentation versions to build:

```sh
yarn version::stables
```

2. Build the docs:

```sh
yarn build
```

This command generates static content into the _build_ directory and can be served using any static
contents hosting service.

3. Verify build by serving a preview locally:

```sh
yarn serve
```

## Adding, Moving or Removing a Page

Every page URL is recorded in `routes.snapshot.json`, and `yarn build` fails if one of them stops
being served without somewhere for its readers to go. External sites, blog posts and search results
link straight at these URLs, and Docusaurus' own broken-link check cannot see them — it only
validates links inside this site.

When you rename, move or delete a page:

1. Add the old URL to `redirects.js`, mapped to the page that now holds the content.
   `docusaurus.config.ts` renders that table into the Netlify `_redirects` file as 301s. Paths
   there are site-relative and version-agnostic: `/guides/oracles`, not `/docs/guides/oracles`.
   If the content has no successor anywhere, record the URL in `removed-urls.json` with a reason
   instead, and it will be allowed to 404.
2. Refresh the snapshot, which runs a full build with the snapshot rewritten from it:

```sh
yarn routes:snapshot
```

The build also fails when a redirect points at a page that no longer exists, so the table cannot
rot into a chain of 301s ending in a 404.

Adding a page needs step 2 as well — the snapshot lists every page, so a new one makes it stale
and `yarn build` fails until it is regenerated. A new page needs no redirect. `yarn dev` does not
check the snapshot.

Versioned snapshots under `versioned_docs/` are not covered: their URLs are frozen at release and
keep working on their own.

## Production Testing

The site will be deployed at `noir-lang.org/docs/`. Test production configuration locally:

### Simple Test
```sh
yarn production:serve
```
Access at: `http://localhost:3000/docs/`

## Cutting a New Version

When a new Noir version is released, a versioned snapshot of the docs needs to be created. This is
normally done automatically by the release workflow (`.github/workflows/release.yml`), but can also
be done manually.

From the _docs_ directory, run:

```sh
yarn cut_version <VERSION>
```

For example: `yarn cut_version v1.0.0-beta.20`

This script does four things:

1. Removes the new version from `versions.json` (since `yarn version::stables` will have added it
   from the GitHub release, but the versioned docs snapshot doesn't exist yet).
2. Builds the docs (running preprocessing to resolve `#include_code` directives and generate the
   Nargo CLI reference).
3. Runs `yarn docusaurus docs:version <VERSION>` to snapshot the current docs into
   `versioned_docs/version-<VERSION>/` and create a matching sidebar in `versioned_sidebars/`.
4. Deletes any snapshot older than the newest `VERSIONS_TO_KEEP` (see `scripts/cut_version.sh`),
   keeping `versioned_docs/` bounded. Only the versions listed in `versions.json` are ever built
   and served — `scripts/setStable.ts` caps that list at `NUMBER_OF_VERSIONS_TO_SHOW` — so older
   snapshots are dead weight in the repository. `VERSIONS_TO_KEEP` is deliberately larger than
   `NUMBER_OF_VERSIONS_TO_SHOW`, so raising the number of served versions does not immediately
   require re-cutting a snapshot that has already been deleted. Docs for deleted versions remain
   available at their published URLs and in git history.

After this, the new version will appear in the version dropdown on the site.

> **Important**: The `#include_code` directives must be resolved _before_ the snapshot is taken.
> The `versioned_docs/` directory should never contain raw `#include_code` directives — CI will
> reject PRs that introduce them.

## Quick Commands Reference

All commands should be run from the `docs` directory:

| Command | Description |
|---------|-------------|
| `yarn install` | Install dependencies |
| `yarn dev` | Start development server |
| `yarn build` | Build production site |
| `yarn serve` | Serve built site locally |
| `yarn version::stables` | Update stable versions list |
| `yarn cut_version <VERSION>` | Cut a new versioned docs snapshot |
| `yarn routes:snapshot` | Build the site and refresh the page-URL snapshot |
| `yarn clean` | Clean build artifacts |
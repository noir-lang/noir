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

- Node.js (tested working with v25)
- Yarn (tested working with v4)
- Rust (tested working with v1)
- GNU ld (tested working with v2)
- jq (tested working with v1)

### Navigation

If you are in the main noir directory, navigate into this docs directory:

```sh
cd docs
```

### Dependency Installation

Install Node.js dependencies:

```sh
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

### Version Cutting

The cutting of new versioned docs is automatically managed by the [release GitHub Action](../.github/workflows/release.yml).

To manually cut new versions for testing or patching purposes:

1. Checkout the desired content to cut with:

```sh
git checkout <version_tag>
```

2. Cut a new version based on the checked out content:

```sh
yarn cut_version <version_name>
```

3. Start a development server serving docs preview:

```sh
yarn dev
```

> **Note**: Only the latest stable versions are expected to show by default. To test cutting a custom version, manually edit the `versions.json` file after running step (2).

## Production Testing

The site will be deployed at `noir-lang.org/docs/`. Test production configuration locally:

### Simple Test
```sh
yarn production:serve
```
Access at: `http://localhost:3000/docs/`

## Quick Commands Reference

All commands should be run from the `docs` directory:

| Command | Description |
|---------|-------------|
| `yarn install` | Install dependencies |
| `yarn dev` | Start development server |
| `yarn build` | Build production site |
| `yarn serve` | Serve built site locally |
| `yarn version::stables` | Update stable versions list |
| `yarn clean` | Clean build artifacts |
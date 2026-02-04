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
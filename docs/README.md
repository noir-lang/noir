# Noir Docs

This is the source code for the Noir documentation site at [noir-lang.org](https://noir-lang.org).

This website is built using [Docusaurus 3](https://docusaurus.io/), a modern static website
generator.

## Contributing

Interested in contributing to the docs?

Check out the contributing guide [here](../CONTRIBUTING.md).

## Development

### Installation

This project requires recent versions of Rust and Cargo to be installed.
Any build errors should indicate dependencies that need installing, and at what version.

From the _noir_ root directory, run:

```sh
yarn
```

### Local Development

From the _noir_ root directory:

1. Fetch and generate the list of recent stable documentation versions to build:

```sh
yarn workspace docs version::stables
```

2. Start a development server serving docs preview:

```sh
yarn workspace docs dev
```

This command starts a local development server and opens up a browser window. Most changes are
reflected live without having to restart the server.

### Build

From the _noir_ root directory:

1. Fetch and generate the list of recent stable documentation versions to build:

```sh
yarn workspace docs version::stables
```

2. Build the docs:

```sh
yarn workspace docs build
```

This command generates static content into the _build_ directory and can be served using any static
contents hosting service.

3. Verify build by serve a preview of the docs locally:

```sh
yarn workspace docs serve
```

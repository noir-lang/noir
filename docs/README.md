# Noir Docs

This is the source code for the Noir documentation site at [noir-lang.org](https://noir-lang.org).

This website is built using [Docusaurus 3](https://docusaurus.io/), a modern static website
generator.

## Contributing

Interested in contributing to the docs?

Check out the contributing guide [here](../CONTRIBUTING.md).

## Development

### Installation

This project requires recent versions of rust and cargo to be installed.
Any build errors should indicate dependencies that need installing, and at what version.

On the root folder of the repository, run:

```
yarn
yarn build
```

### Local Development

```
yarn workspace docs version
```

This command fetches and compiles the list of documentation versions to build with.

```
yarn workspace docs dev
```

This command starts a local development server and opens up a browser window. Most changes are
reflected live without having to restart the server.

### Build

```
yarn workspace docs build
```

This command generates static content into the `build` directory and can be served using any static
contents hosting service. You can see a preview by running:

```
yarn workspace docs serve
```

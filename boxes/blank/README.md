# Aztec Box

This box is a one-stop-shop for Aztec that will deploy a minimal barebones HTML+JS page. You can use it as a boilerplate to start developing your own Aztec app in seconds!

## Prerequisites

- You should have Docker installed. If you don't, follow [this guide](https://docs.aztec.network/dev_docs/getting_started/quickstart#install-docker).

## Installation

To start, run the Aztec install script:

```bash
bash -i <(curl -s install.aztec.network)`
```

After a few minutes, you should have all the Aztec CLI commands ready to run.

### 1. Launching the sandbox

Run:

```bash
aztec-sandbox
```

This will install all the dependencies and run the sandbox on port 8080 together with a anvil node.

### 2. Unboxing the box

Unbox the box with:

```bash
aztec-cli unbox blank
```

and install dependencies:

```bash
yarn
```

This sandbox requires [Docker](https://www.docker.com/) to be installed _and running_ locally. In the event the image needs updating, you can run `yarn install:sandbox` (see [sandbox docs](https://docs.aztec.network/developers/cli/main) for more information.)

Time to build. Run:

```bash
yarn dev
```

Your app is waiting for you on port 5176. Time to make it your own!

In the `src/contracts` folder, you'll find the default contract being deployed. Don't forget to recompile with `aztec-nargo compile`! Read the [aztec.nr documentation](https://docs.aztec.network/dev_docs/contracts/main) to get started with the `aztec.nr` framework.

[Read the full Sandbox reference](https://docs.aztec.network/dev_docs/cli/sandbox-reference) for more info on what exactly is happening on your machine!

## More info

Here's what is inside your `src` folder:

- `contracts` - The Aztec Contracts you just deployed!
- `config.ts` - A file exporting environment, and other configurations you need
- `index.html` and `.ts` - The actual website you're deploying

There's also a `test` folder with minimal testing you can expand on, and an `artifacts` folder should pop up once you run your app, these are the artifacts from your contract's compilation.

Visit the [Aztec Docs](https://docs.aztec.network) for more information on how Aztec works, and the [Awesome Aztec Repository](https://github.com/AztecProtocol/awesome-aztec) for more cool projects, boilerplates and tooling.

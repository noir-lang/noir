---
title: Quickstart
---

The easiest way to start developing on Aztec locally is through `npx aztec-app`. This is a convenient way of installing the development environment (A.K.A. Sandbox) and starting new projects from a boilerplate.

To locally install the Sandbox without other tools, see [here](./getting_started/manual_install.md).

## Prerequisites

- Node.js >= v18 (recommend installing with [nvm](https://github.com/nvm-sh/nvm))
- Docker (visit [this page of the Docker docs](https://docs.docker.com/get-docker/) on how to install it)

### Run the `npx` script

Thanks to Node, you can run the recommended `npx script`:

```bash
npx aztec-app
```

This script gives you some options to bootstrap a new project, start/stop the sandbox, or see the logs. Run `npx aztec-app -h` for a list of options.

## Install Noir LSP (recommended)

Install the [Noir Language Support extension](https://marketplace.visualstudio.com/items?itemName=noir-lang.vscode-noir) to get syntax highlighting, syntax error detection and go-to definitions for your Aztec contracts.

Once the extension is installed, check your nargo binary by hovering over `Nargo` in the status bar on the bottom right of the application window. Click to choose the path to `aztec-nargo` (or regular `nargo`, if you have that installed).

You can print the path of your `aztec-nargo` executable by running:

```bash
which aztec-nargo
```

To specify a custom nargo executable, go to the VSCode settings and search for "noir", or click extension settings on the `noir-lang` LSP plugin.
Update the `Noir: Nargo Path` field to point to your desired `aztec-nargo` executable.

## What's next?

Now you have a development network running, so you're ready to start coding your first app with Aztec.nr and Aztec.js!

To follow the series of tutorials, start with the private voting contract [here](./tutorials/contract_tutorials/private_voting_contract.md).

If you want to just keep learning, you can read about the high level architecture on the [Core Components page](./aztec/concepts/state_model/index.md) and [the lifecycle of a transaction](./aztec/concepts/transactions.md).


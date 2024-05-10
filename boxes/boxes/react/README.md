# Aztec Box

This box is a one-stop-shop for Aztec that will deploy a minimal React page. You can use it as a boilerplate to start developing your own Aztec app in seconds!

## Getting Started

The easiest way to start is with a Github Codespaces, which has a generous free tier. Just click on this button:

[![One-Click React Starter](.devcontainer/assets/react_cta_badge.svg)](https://codespaces.new/AztecProtocol/aztec-packages?devcontainer_path=.devcontainer%2Freact%2Fdevcontainer.json)

## Using the `npx` command

The above method just uses the `npx` command, AKA "unboxing the box". This is a CLI command to quickly start developing on your own machine.

### Prerequisites

- Node >v18
- Docker

### Usage

Just open a terminal and write:

```bash
npx create-aztec-app
```

It should ask you some questions about your project, install and run the Sandbox (local developer network). You can also start, stop, update, and do other things on the sandbox through this script. Just run:

```bash
npx create-aztec-app sandbox --help
```

## More information

Visit the [Aztec Docs](https://docs.aztec.network) for more information on how Aztec works, and the [Awesome Aztec Repository](https://github.com/AztecProtocol/awesome-aztec) for more cool projects, boilerplates and tooling.

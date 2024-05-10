# Contract-Only Box

This box is a one-stop-shop for Aztec with the %%contract_name%% example contract. You can use it as a boilerplate to start developing your own Aztec app in seconds!

## Getting Started

The easiest way to start is with a Github Codespaces, which has a generous free tier. Just click on this button:

[![One-Click Token Starter](.devcontainer/assets/token_cta_badge.svg)](https://codespaces.new/AztecProtocol/aztec-packages?devcontainer_path=.devcontainer%2Ftoken%2Fdevcontainer.json)

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

## What's in the box

The script copied one of the example contracts and put it into a one-size-fits-all "box". With it, you can run commands such as:

- `yarn test` -> Runs the built-in tests
- `yarn compile` -> Compiles your contract so it can be deployed
- `yarn codegen` -> Generates a handy TS interface file with all your contract's methods so they're easy to interact with
- `yarn clean` -> Removes artifacts and other things you may not want to have lying around
- `yarn formatting` -> Formats your code with prettier
-  

## Testing

Contract-only boxes give you basic test functionality through `jest`, and check for existence and correct working of the sandbox.

If you want some ideas to test the contract you just bootstrapped, check out [our own e2e test suite!](%%e2e_test_url%%)

## More information

Visit the [Aztec Docs](https://docs.aztec.network) for more information on how Aztec works, and the [Awesome Aztec Repository](https://github.com/AztecProtocol/awesome-aztec) for more cool projects, boilerplates and tooling.

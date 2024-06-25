---
title: Manual install
sidebar_position: 1
---

You can have some more control over the sandbox by installing it manually through the underlying script used by [`npx aztec-app`](../getting_started.md).

This involves some knowledge on Docker if you want to stop, restart, or detach from logs. But it also gives you better control over things such as environment variables.

### Prerequisites

- Node.js >= v18 (recommend installing with [nvm](https://github.com/nvm-sh/nvm))
- Docker (visit [this page of the Docker docs](https://docs.docker.com/get-docker/) on how to install it)

### Install the sandbox

To install the latest Sandbox version, run:

```bash
bash -i <(curl -s install.aztec.network)
```

This will install the following tools:

- **aztec** - launches various infrastructure subsystems (sequencer, prover, pxe, etc).
- **aztec-nargo** - aztec's build of nargo, the noir compiler toolchain.
- **aztec-sandbox** - a wrapper around docker-compose that launches services needed for sandbox testing.
- **aztec-up** - a tool to upgrade the aztec toolchain to the latest, or specific versions.
- **aztec-builder** - A useful tool for projects to generate ABIs and update their dependencies.

Once these have been installed, to start the sandbox, run:

```bash
aztec-sandbox
```

### Have fun

**Congratulations, you have just installed and run the Aztec Sandbox!**

```bash
     /\        | |
    /  \    ___| |_ ___  ___
   / /\ \  |_  / __/ _ \/ __|
  / ____ \  / /| ||  __/ (__
 /_/___ \_\/___|\__\___|\___|

```

In the terminal, you will see some logs:

1. Sandbox version
2. Contract addresses of rollup contracts
3. PXE (private execution environment) setup logs
4. Initial accounts that are shipped with the sandbox and can be used in tests

## Running Aztec PXE / Node / P2P-Bootstrap node

If you wish to run components of the Aztec network stack separately, you can use the `aztec start` command with various options for enabling components.

```bash
aztec start --node [nodeOptions] --pxe [pxeOptions] --archiver [archiverOptions] --sequencer [sequencerOptions] --prover [proverOptions] ----p2p-bootstrap [p2pOptions]
```

Starting the aztec node alongside a PXE, sequencer or archiver, will attach the components to the node.Eg if you want to run a PXE separately to a node, you can [read this guide](../aztec/concepts/pxe/index.md)/

## Update the sandbox

To update the sandbox, you can just run:

```bash
aztec-up
```

## Next steps

Visit the [sandbox reference](../reference/sandbox_reference/index.md) for more info on which environment variables you can set, which cheat codes you can use, and learn about what exactly is the Aztec Sandbox.

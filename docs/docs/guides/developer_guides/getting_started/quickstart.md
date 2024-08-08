---
title: Quickstart
sidebar_position: 0
tags: [sandbox]
---

You can get started with an Aztec development environment (A.K.A. Sandbox) in less than 5 minutes.

The Sandbox is an Aztec network running fully on your machine, and interacting with a development Ethereum node. You can develop and deploy on it just like on a testnet or mainnet.

### Prerequisites

You need two global dependencies in your machine:

- Node.js >= v18 (recommend installing with [nvm](https://github.com/nvm-sh/nvm))
- Docker (visit [this page of the Docker docs](https://docs.docker.com/get-docker/) on how to install it)

### Install the sandbox

Run:

```bash
bash -i <(curl -s install.aztec.network)
```

This will install the following tools:

- **aztec** - launches various infrastructure subsystems (full sandbox, sequencer, prover, pxe, etc) and provides utility commands to interact with the network
- **aztec-nargo** - aztec's build of nargo, the noir compiler toolchain.
- **aztec-up** - a tool to upgrade the aztec toolchain to the latest, or specific versions.

Once these have been installed, to start the sandbox, run:

```bash
aztec start --sandbox
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

Starting the aztec node alongside a PXE, sequencer or archiver, will attach the components to the node. Eg if you want to run a PXE separately to a node, you can [read this guide](../local_env/run_more_than_one_pxe_sandbox.md).

## Update the sandbox

To update the sandbox, you can just run:

```bash
aztec-up
```


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

If you want to start coding, it is recommended to go through the tutorials in order. Click on the link below to dive in!

<div className="card-container full-width">
  <Card shadow='tl' link='/tutorials/codealong/contract_tutorials/counter_contract'>
    <CardHeader>
      <h3>Write your first contract</h3>
    </CardHeader>
    <CardBody>
     Write and deploy a simple private counter smart contract on your local sandbox
    </CardBody>
  </Card>
</div>

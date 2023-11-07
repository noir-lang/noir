---
title: Sandbox Reference
---

Here you will find a reference to everything available within the Sandbox.

## Installation

You can run the Sandbox using either Docker or npm.

### With Docker

```bash
/bin/bash -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

This will attempt to run the Sandbox on ` localhost:8080`. You can change the port defined in `./.aztec/docker-compose.yml`. Running the command again will overwrite any changes made to the `docker-compose.yml`.

If you don't have the CLI installed via a node package manager, this command will also install or update the CLI.

To install a specific version of the sandbox, you can set the environment variable `SANDBOX_VERSION`

```bash
SANDBOX_VERSION=<version> /bin/bash -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

NOTE: The sandbox version should be the same as your `@aztec/cli` package to ensure compatibility.

### With npm

You can download and run the Sandbox package directly if you have nodejs 18 or higher installed.

You will also need an Ethereum node like Anvil or Hardhat running locally on port 8545.

```bash
npx @aztec/aztec-sandbox @aztec/aztec-cli
```

You can read [this tutorial on how to use the npm package](../tutorials/testing.md#running-sandbox-in-the-nodejs-process)

## Running

The installation command will run the sandbox, and once installed you can run like so:

```bash
cd ~/.aztec && docker-compose up
```

## Otterscan

If you have set up the Sandbox with Docker, you will also have Otterscan.

You can see Ethereum Layer 1 activity through the local Otterscan on `http://localhost:5100`. This is especially useful for dapps that use L1-L2 messaging through [portal contracts](../contracts/portals/main.md).

## Cheat Codes

To help with testing, the sandbox is shipped with a set of cheatcodes.

Cheatcodes allow you to change the time of the Aztec block, load certain state or more easily manipulate Ethereum instead of having to write dedicated RPC calls to anvil or hardhat.

You can find the cheat code reference [here](../testing/cheat_codes.md).

## Contracts

We have shipped a number of example contracts in the `@aztec/noir-contracts` [npm package](https://www.npmjs.com/package/@aztec/noir-contracts). This is included with the cli by default so you are able to use these contracts to test with. To get a list of the names of the contracts run:

```bash title="example-contracts" showLineNumbers
% aztec-cli example-contracts
BenchmarkingContractArtifact
CardGameContractArtifact
ChildContractArtifact
DocsExampleContractArtifact
EasyPrivateTokenContractArtifact
EcdsaAccountContractArtifact
EscrowContractArtifact
ImportTestContractArtifact
LendingContractArtifact
ParentContractArtifact
PendingCommitmentsContractArtifact
PokeableTokenContractArtifact
PriceFeedContractArtifact
SchnorrAccountContractArtifact
SchnorrHardcodedAccountContractArtifact
SchnorrSingleKeyAccountContractArtifact
StatefulTestContractArtifact
TestContractArtifact
TokenBridgeContractArtifact
TokenContractArtifact
UniswapContractArtifact
```

> <sup><sub><a href="https://github.com/AztecProtocol/aztec-packages/blob/master//yarn-project/end-to-end/src/cli_docs_sandbox.test.ts#L95-L118" target="_blank" rel="noopener noreferrer">Source code: /yarn-project/end-to-end/src/cli_docs_sandbox.test.ts#L95-L118</a></sub></sup>

You can see all of our example contracts in the monorepo [here](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts/src/contracts).

## Boxes

The sandbox is shipped with full-stack Aztec project templates, with example Aztec.nr contracts, testing scripts, and web interfaces.

You can read more information about how to use boxes [here](./blank_box.md)

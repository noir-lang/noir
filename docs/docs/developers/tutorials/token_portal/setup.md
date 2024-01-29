---
title: Setup and Installation
---

In this step, we’re going to

1. Install prerequisites
2. Create a yarn project to house everything
3. Create a noir project for our Aztec contract
4. Create a hardhat project for our Ethereum contract(s)
5. Import all the Ethereum contracts we need
6. Create a yarn project that will interact with our contracts on L1 and the sandbox

We recommend going through this setup to fully understand where things live.

However if you’d rather skip this part, our dev-rels repo contains the starter code here.

# Prerequisites

- [node v18+](https://github.com/tj/n)
- [docker](https://docs.docker.com/)
- [Aztec sandbox](https://docs.aztec.network/developers/getting_started/sandbox) - you should have this running before starting the tutorial
- [Aztec CLI](../../getting_started/quickstart.md)

```bash
/bin/sh -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

# Create the root project and packages

Our root project will house everything ✨

```bash
mkdir aztec-token-bridge
cd aztec-token-bridge && mkdir packages
```

We will hold our projects inside of `packages` to follow the design of the project in the [repo](https://github.com/AztecProtocol/dev-rel/tree/main/tutorials/token-bridge-e2e).

# Create a noir project

Now inside `packages` create a new directory called `aztec-contracts`

Inside `aztec-contracts`, create the following file structure:

```tree
aztec-contracts
└── token_bridge
    ├── Nargo.toml
    ├── src
       ├── main.nr
```

Inside `Nargo.toml` add the following content:

```toml
[package]
name = "token_bridge"
authors = [""]
compiler_version = ">=0.18.0"
type = "contract"

[dependencies]
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
token_portal_content_hash_lib = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/noir-contracts/contracts/token_portal_content_hash_lib" }
protocol_types = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/noir-protocol-circuits/src/crates/types"}
```

We will also be writing some helper functions that should exist elsewhere so we don't overcomplicated our contract. In `src` create two more files - one called `util.nr` and one called `token_interface` - so your dir structure should now look like this:

```tree
aztec-contracts
└── token_bridge
    ├── Nargo.toml
    ├── src
      ├── main.nr
      ├── token_interface.nr
      ├── util.nr
```

# Create a JS hardhat project

In the `packages` dir, create a new directory called `l1-contracts` and run `yarn init -yp &&
npx hardhat init` inside of it. Keep hitting enter so you get the default setup (Javascript project)

```bash
mkdir l1-contracts
cd l1-contracts
yarn init -yp
npx hardhat init
```

Once you have a hardhat project set up, delete the existing contracts, tests, and scripts, and create a `TokenPortal.sol`:

```bash
rm -rf contracts test scripts
mkdir contracts && cd contracts
touch TokenPortal.sol
```

Now add dependencies that are required. These include interfaces to Aztec Inbox, Outbox and Registry smart contracts, OpenZeppelin contracts, and NomicFoundation.

```bash
yarn add @aztec/foundation @aztec/l1-contracts @openzeppelin/contracts && yarn add --dev @nomicfoundation/hardhat-network-helpers @nomicfoundation/hardhat-chai-matchers @nomiclabs/hardhat-ethers @nomiclabs/hardhat-etherscan @types/chai @types/mocha @typechain/ethers-v5 @typechain/hardhat chai@4.0.0 hardhat-gas-reporter solidity-coverage ts-node typechain typescript

```

This is what your `l1-contracts` should look like:

```tree
├── README.md
├── contracts
├── hardhat.config.js
├── node_modules
└── package.json
```

We will need to ensure we are using the correct Solidity version. Inside your `hardhat.config.js` update `solidity` version to this:

```json
  solidity: "0.8.20",
```

# Create src yarn project

In this directory, we will write TS code that will interact with our L1 and L2 contracts and run them against the sandbox.

We will use `viem` in this tutorial and `jest` for testing.

Inside the `packages` directory, run

```bash
mkdir src && cd src && yarn init -yp
yarn add typescript @aztec/aztec.js @aztec/accounts @aztec/noir-contracts @aztec/types @aztec/foundation @aztec/l1-artifacts viem@1.21.4 "@types/node@^20.8.2"
yarn add -D jest @jest/globals ts-jest
```

If you are going to track this repo using git, consider adding a `.gitignore` file to your `src` directory and adding `node_modules` to it.

In `package.json`, add:

```json
"type": "module",
"scripts": {
  "test": "NODE_NO_WARNINGS=1 node --experimental-vm-modules $(yarn bin jest)"
}
```

Your `package.json` should look something like this (do not copy and paste):

```json
{
  "name": "src",
  "version": "1.0.0",
  "main": "index.js",
  "license": "MIT",
  "private": true,
  "type": "module",
  "dependencies": {    
    "dep": "version",
  },
  "devDependencies": {
    "dep": "version",
  },
  "scripts": {
    "test": "NODE_NO_WARNINGS=1 node --experimental-vm-modules $(yarn bin jest)"
  }
}
```

Create a `tsconfig.json` and paste this:

```json
{
  "compilerOptions": {
    "rootDir": "../",
    "outDir": "./dest",
    "target": "es2020",
    "lib": ["dom", "esnext", "es2017.object"],
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "declaration": true,
    "allowSyntheticDefaultImports": true,
    "esModuleInterop": true,
    "downlevelIteration": true,
    "inlineSourceMap": true,
    "declarationMap": true,
    "importHelpers": true,
    "resolveJsonModule": true,
    "composite": true,
    "skipLibCheck": true
  },
  "include": [
    "packages/src/**/*",
    "contracts/**/*.json",
    "packages/src/**/*",
    "packages/aztec-contracts/token_bridge/target/*.json"
  ],
  "exclude": ["node_modules", "**/*.spec.ts", "contracts/**/*.ts"],
  "references": []
}
```

The main thing this will allow us to do is to access TS artifacts that we generate later from our test.

Then create a jest config file: `jest.config.json`

```json
{
  "preset": "ts-jest/presets/default-esm",
  "globals": {
    "ts-jest": {
      "useESM": true
    }
  },
  "moduleNameMapper": {
    "^(\\.{1,2}/.*)\\.js$": "$1"
  },
  "testRegex": "./test/.*\\.test\\.ts$",
  "rootDir": "./test"
}
```

Finally, we will create a test file. Run this in the `src` directory.:

```bash
mkdir test && cd test
touch cross_chain_messaging.test.ts
```

Your `src` dir should look like:

```tree
src
├── node_modules
└── test
    └── cross_chain_messaging.test.ts
├── jest.config.json
├── package.json
├── tsconfig.json
```

In the next step, we’ll start writing our L1 smart contract with some logic to deposit tokens to Aztec from L1.

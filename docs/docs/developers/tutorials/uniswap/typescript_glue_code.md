---
title: Deploy & Call Contracts with Typescript
---

In this step, we We will now write a Typescript to interact with the sandbox and see our Solidity and Aztec.nr contracts in action.

In the `packages` directory, go to `src` dir we created in [the token bridge tutorial](../token_portal/setup.md).

```bash
cd src/test
touch uniswap.test.ts
```

Open `uniswap.test.ts` in your editor.

We will write two tests:

1. Test the private flow (i.e. mint tokens on L1, deposit them to L2, give your intention to swap L2 asset on L1, swap on L1, bridge swapped assets back to L2)
2. Do the same in the public flow

## Compile our contracts

To compile the Solidity contracts, run this:

```bash
cd l1-contracts
npx hardhat compile
```

and the each of the Aztec.nr contracts by going into each folder and running:

```bash
aztec-nargo compile
```

And then generate the typescript interface:

```bash
aztec-cli codegen ./target/ -o ../../../src/test/fixtures uniswap --ts
```

This will create a TS interface in our `src/test` folder that will help us write our test.

## Test imports and setup

We will use the same `utils.ts` and `cross_chain_test_harness.ts` we created in the tutorial [here](../token_portal/typescript_glue_code.md#test-imports-and-setup).

In `utils.ts`, add:

```typescript
export const [UniswapPortalAbi, UniswapPortalBytecode] =
  getL1ContractABIAndBytecode("UniswapPortal");
```

### Setup the fork

Since we want to use L1 Uniswap, we need the sandbox to execute against a fork of L1. This has be easily done:
in your terminal add the following variables:

```
export FORK_BLOCK_NUMBER=17514288
export FORK_URL=<YOUR_RPC_URL e.g. https://mainnet.infura.io/v3/API_KEY>
```

Now rerun the sandbox:

```bash
/bin/sh -c "$(curl -fsSL 'https://sandbox.aztec.network')"
```

### Back to test setup

Okay now we are ready to write our tests:

open `uniswap.test.ts` and lets do the initial description of the test:

```typescript
import {
  AccountWallet,
  AztecAddress,
  DebugLogger,
  EthAddress,
  Fr,
  PXE,
  TxStatus,
  computeAuthWitMessageHash,
  createDebugLogger,
  createPXEClient,
  waitForPXE,
} from "@aztec/aztec.js";
import { getInitialTestAccountsWallets } from '@aztec/accounts/testing';
import {
  Chain,
  HttpTransport,
  PublicClient,
  createPublicClient,
  createWalletClient,
  getContract,
  http,
  parseEther,
} from "viem";
import { foundry } from "viem/chains";
import { CrossChainTestHarness } from "./shared/cross_chain_test_harness.js";
import { UniswapContract } from "./fixtures/Uniswap.js";
import { beforeAll, expect, jest } from "@jest/globals";
import {
  UniswapPortalAbi,
  UniswapPortalBytecode,
  delay,
  deployL1Contract,
} from "./fixtures/utils.js";
import { mnemonicToAccount } from "viem/accounts";

const {
  PXE_URL = "http://localhost:8080",
  ETHEREUM_HOST = "http://localhost:8545",
} = process.env;
const MNEMONIC = "test test test test test test test test test test test junk";
const hdAccount = mnemonicToAccount(MNEMONIC);
const expectedForkBlockNumber = 17514288;

#include_code uniswap_l1_l2_test_setup_const yarn-project/end-to-end/src/shared/uniswap_l1_l2.ts raw
#include_code uniswap_setup yarn-project/end-to-end/src/uniswap_trade_on_l1_from_l2.test.ts raw
#include_code uniswap_l1_l2_test_beforeAll yarn-project/end-to-end/src/shared/uniswap_l1_l2.ts raw
```

## Private flow test

#include_code uniswap_private yarn-project/end-to-end/src/shared/uniswap_l1_l2.ts typescript

## Public flow test

#include_code uniswap_public yarn-project/end-to-end/src/shared/uniswap_l1_l2.ts typescript

## Running the test

Make sure your sandbox is running.

```bash
cd ~/.aztec && docker-compose up
```

Then run this in the root directory.

```bash
cd packages/src
yarn test uniswap
```

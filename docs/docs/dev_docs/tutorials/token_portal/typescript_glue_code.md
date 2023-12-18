---
title: Deploy & Call Contracts with Typescript
---

In this step we will write a Typescript test to interact with the sandbox and call our contracts!

Go to the `src/test` directory in your `packages` dir and create a new file called `cross_chain_messaging.test.ts`:

```bash
cd src/test
touch cross_chain_messaging.test.ts
```

Open `cross_chain_messaging.test.ts`.

We will write two tests:

1. Test the deposit and withdraw in the private flow
2. Do the same in the public flow

## Test imports and setup

We need some helper files that can keep our code clean. Inside your `src/test` directory:

```bash
cd fixtures
touch utils.ts
touch cross_chain_test_harness.ts
```

In `utils.ts`, put:

```typescript
import * as fs from "fs";
import { AztecAddress, EthAddress, TxStatus, Wallet } from "@aztec/aztec.js";
import { TokenContract } from "@aztec/noir-contracts/types";
import {
  Account,
  Chain,
  Hex,
  HttpTransport,
  PublicClient,
  WalletClient,
  getContract,
} from "viem";
import type { Abi, Narrow } from "abitype";

import { TokenBridgeContract } from "./TokenBridge.js";

const PATH = "../../packages/l1-contracts/artifacts/contracts";
const EXT = ".sol";
function getL1ContractABIAndBytecode(contractName: string) {
  const pathToArtifact = `${PATH}/${contractName}${EXT}/${contractName}.json`;
  const artifacts = JSON.parse(fs.readFileSync(pathToArtifact, "utf-8"));
  return [artifacts.abi, artifacts.bytecode];
}

const [PortalERC20Abi, PortalERC20Bytecode] =
  getL1ContractABIAndBytecode("PortalERC20");
const [TokenPortalAbi, TokenPortalBytecode] =
  getL1ContractABIAndBytecode("TokenPortal");

#include_code deployL1Contract /yarn-project/ethereum/src/deploy_l1_contracts.ts raw

#include_code deployAndInitializeTokenAndBridgeContracts /yarn-project/end-to-end/src/shared/cross_chain_test_harness.ts raw

#include_code delay /yarn-project/end-to-end/src/fixtures/utils.ts raw
```

This code

- gets your Solidity contract ABIs
- uses viem to deploy them to Ethereum
- uses Aztec.js to deploy the token and token bridge contract on L2, sets the bridge's portal address to `tokenPortalAddress` and initializes all the contracts

Now let's create another util file to can handle interaction with these contracts to mint/deposit the functions:

In `cross_chain_test_harness.ts`, add:

#include_code cross_chain_test_harness /yarn-project/end-to-end/src/shared/cross_chain_test_harness.ts typescript

This is a class that holds all contracts as objects and exposes easy to use helper methods to interact with our contracts.

Now let's write our tests.

Open `cross_chain_messaging.test.ts` and paste the initial description of the test:

```typescript
import { expect, jest} from '@jest/globals'
import { AccountWallet, AztecAddress, DebugLogger, EthAddress, Fr, computeAuthWitMessageHash, createDebugLogger, createPXEClient, getSandboxAccountsWallets, waitForSandbox } from '@aztec/aztec.js';
import { TokenBridgeContract, TokenContract } from '@aztec/noir-contracts/types';

import { CrossChainTestHarness } from './shared/cross_chain_test_harness.js';
import { delay } from './fixtures/utils.js';
import { mnemonicToAccount } from 'viem/accounts';
import { createPublicClient, createWalletClient, http } from 'viem';
import { foundry } from 'viem/chains';

const { PXE_URL = 'http://localhost:8080', ETHEREUM_HOST = 'http://localhost:8545' } = process.env;
const MNEMONIC = 'test test test test test test test test test test test junk';
const hdAccount = mnemonicToAccount(MNEMONIC);

describe('e2e_cross_chain_messaging', () => {
  jest.setTimeout(90_000);

  let logger: DebugLogger;
  // include code:
  let user1Wallet: AccountWallet;
  let user2Wallet: AccountWallet;
  let ethAccount: EthAddress;
  let ownerAddress: AztecAddress;

  let crossChainTestHarness: CrossChainTestHarness;
  let l2Token: TokenContract;
  let l2Bridge: TokenBridgeContract;
  let outbox: any;

  beforeEach(async () => {
    logger = createDebugLogger('aztec:e2e_uniswap');
    const pxe = createPXEClient(PXE_URL);
    await waitForSandbox(pxe);
    const wallets = await getSandboxAccountsWallets(pxe);

    const walletClient = createWalletClient({
      account: hdAccount,
      chain: foundry,
      transport: http(ETHEREUM_HOST),
    });
    const publicClient = createPublicClient({
      chain: foundry,
      transport: http(ETHEREUM_HOST),
    });

    crossChainTestHarness = await CrossChainTestHarness.new(
      pxe,
      publicClient,
      walletClient,
      wallets[0],
      logger,
    );

    l2Token = crossChainTestHarness.l2Token;
    l2Bridge = crossChainTestHarness.l2Bridge;
    ethAccount = crossChainTestHarness.ethAccount;
    ownerAddress = crossChainTestHarness.ownerAddress;
    outbox = crossChainTestHarness.outbox;
    user1Wallet = wallets[0];
    user2Wallet = wallets[1];
    logger = logger;
    logger('Successfully deployed contracts and initialized portal');
  });
```

This fetches the wallets from the sandbox and deploys our cross chain harness on the sandbox!

## Private flow test

#include_code e2e_private_cross_chain /yarn-project/end-to-end/src/e2e_cross_chain_messaging.test.ts typescript

## Public flow test

#include_code e2e_public_cross_chain /yarn-project/end-to-end/src/e2e_public_cross_chain_messaging.test.ts typescript

## Running the test

```bash
cd packages/src
DEBUG='aztec:e2e_uniswap' yarn test
```

### Error handling

Note - you might have a jest error at the end of each test saying "expected 1-2 arguments but got 3". In case case simply remove the "120_000" at the end of each test. We have already set the timeout at the top so this shouldn't be a problem.

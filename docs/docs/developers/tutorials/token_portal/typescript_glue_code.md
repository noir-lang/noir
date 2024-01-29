---
title: Deploy & Call Contracts with Typescript
---

In this step we will write a Typescript test to interact with the sandbox and call our contracts!

## Test imports and setup

We need some helper files that can keep our code clean. Inside your `src/test` directory:

```bash
cd fixtures
touch utils.ts
cd .. && mkdir shared && cd shared
touch cross_chain_test_harness.ts
```

In `utils.ts`, we need a delay function. Put this:

#include_code delay yarn-project/end-to-end/src/fixtures/utils.ts typescript

In `cross_chain_test_harness.ts`, add:

#include_code cross_chain_test_harness /yarn-project/end-to-end/src/shared/cross_chain_test_harness.ts typescript

This
- gets your Solidity contract ABIs
- uses Aztec.js to deploy them to Ethereum
- uses Aztec.js to deploy the token and token bridge contract on L2, sets the bridge's portal address to `tokenPortalAddress` and initializes all the contracts
- exposes easy to use helper methods to interact with our contracts.

Now let's write our tests.

We will write two tests:

1. Test the deposit and withdraw in the private flow
2. Do the same in the public flow

Open `cross_chain_messaging.test.ts` and paste the initial description of the test:

```typescript
import { expect, jest} from '@jest/globals'
import { AccountWallet, AztecAddress, DebugLogger, EthAddress, Fr, computeAuthWitMessageHash, createDebugLogger, createPXEClient, waitForSandbox } from '@aztec/aztec.js';
import { getSandboxAccountsWallets } from '@aztec/accounts/testing';
import { TokenContract } from '@aztec/noir-contracts/Token';
import { TokenBridgeContract } from '@aztec/noir-contracts/TokenBridge';

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

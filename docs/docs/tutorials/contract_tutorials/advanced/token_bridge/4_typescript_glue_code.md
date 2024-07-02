---
title: Deploy & Call Contracts with Typescript
---

In this step we will write a Typescript test to interact with the sandbox and call our contracts!

## Test imports and setup

We need some helper files that can keep our code clean. Inside your `src/test` directory:

```bash
cd fixtures
cd .. && mkdir shared && cd shared
touch cross_chain_test_harness.ts
```

In `cross_chain_test_harness.ts`, add:

```ts
import { expect } from '@jest/globals'
#include_code cross_chain_test_harness /yarn-project/end-to-end/src/shared/cross_chain_test_harness.ts raw
```

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
import { beforeAll, describe, beforeEach, expect, jest, it} from '@jest/globals'
import { AccountWallet, AztecAddress, BatchCall, type DebugLogger, EthAddress, Fr, computeAuthWitMessageHash, createDebugLogger, createPXEClient, waitForPXE, L1ToL2Message, L1Actor, L2Actor, type PXE, type Wallet } from '@aztec/aztec.js';
import { getInitialTestAccountsWallets } from '@aztec/accounts/testing';
import { TokenContract } from '@aztec/noir-contracts.js/Token';
import { sha256ToField } from '@aztec/foundation/crypto';
import { TokenBridgeContract } from './fixtures/TokenBridge.js';
import { createAztecNodeClient } from '@aztec/circuit-types';
import { deployInstance, registerContractClass } from '@aztec/aztec.js/deployment';
import { SchnorrAccountContractArtifact } from '@aztec/accounts/schnorr';

import { CrossChainTestHarness } from './shared/cross_chain_test_harness.js';
import { mnemonicToAccount } from 'viem/accounts';
import { createPublicClient, createWalletClient, http, toFunctionSelector } from 'viem';
import { foundry } from 'viem/chains';

const { PXE_URL = 'http://localhost:8080', ETHEREUM_HOST = 'http://localhost:8545' } = process.env;
const MNEMONIC = 'test test test test test test test test test test test junk';
const hdAccount = mnemonicToAccount(MNEMONIC);
const aztecNode = createAztecNodeClient(PXE_URL);
export const NO_L1_TO_L2_MSG_ERROR =
  /No non-nullified L1 to L2 message found for message hash|Tried to consume nonexistent L1-to-L2 message/;

async function publicDeployAccounts(sender: Wallet, accountsToDeploy: Wallet[], pxe: PXE) {
    const accountAddressesToDeploy = await Promise.all(
        accountsToDeploy.map(async a => {
            const address = await a.getAddress();
            const isDeployed = await pxe.isContractPubliclyDeployed(address);
            return { address, isDeployed };
        })
    ).then(results => results.filter(result => !result.isDeployed).map(result => result.address));
    if (accountAddressesToDeploy.length === 0) return
    const instances = await Promise.all(accountAddressesToDeploy.map(account => sender.getContractInstance(account)));
    const batch = new BatchCall(sender, [
        (await registerContractClass(sender, SchnorrAccountContractArtifact)).request(),
        ...instances.map(instance => deployInstance(sender, instance!).request()),
    ]);
    await batch.send().wait();
}

describe('e2e_cross_chain_messaging', () => {
  jest.setTimeout(90_000);

  let logger: DebugLogger;
  let wallets: AccountWallet[];
  let user1Wallet: AccountWallet;
  let user2Wallet: AccountWallet;
  let ethAccount: EthAddress;
  let ownerAddress: AztecAddress;

  let crossChainTestHarness: CrossChainTestHarness;
  let l2Token: TokenContract;
  let l2Bridge: TokenBridgeContract;
  let outbox: any;

  beforeAll(async () => {
      logger = createDebugLogger('aztec:e2e_uniswap');
      const pxe = createPXEClient(PXE_URL);
      await waitForPXE(pxe);
      wallets = await getInitialTestAccountsWallets(pxe);

      // deploy the accounts publicly to use public authwits
      await publicDeployAccounts(wallets[0], wallets, pxe);
  })

  beforeEach(async () => {
    logger = createDebugLogger('aztec:e2e_uniswap');
    const pxe = createPXEClient(PXE_URL);
    await waitForPXE(pxe);

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
      aztecNode,
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
  });
```

This fetches the wallets from the sandbox and deploys our cross chain harness on the sandbox!

## Private flow test

Paste the private flow test below the setup:

#include_code e2e_private_cross_chain /yarn-project/end-to-end/src/e2e_cross_chain_messaging.test.ts typescript

## Public flow test

Paste the public flow below the private flow:

```ts
#include_code e2e_public_cross_chain /yarn-project/end-to-end/src/e2e_public_cross_chain_messaging/deposits.test.ts raw
})
```

## Running the test

```bash
cd packages/src
DEBUG='aztec:e2e_uniswap' yarn test
```

### Error handling

Note - you might have a jest error at the end of each test saying "expected 1-2 arguments but got 3". In case case simply remove the "120_000" at the end of each test. We have already set the timeout at the top so this shouldn't be a problem.

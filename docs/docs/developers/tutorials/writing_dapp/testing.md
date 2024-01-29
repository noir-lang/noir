---
title: Testing
---

To wrap up this tutorial, we'll set up a simple automated test for our dapp contracts. We will be using [jest](https://jestjs.io/), but any nodejs test runner works fine.

Here we'll only test the happy path for a `transfer` on our private token contract, but in a real application you should be testing both happy and unhappy paths, as well as both your contracts and application logic. Refer to the full [testing guide](../testing.md) for more info on testing and assertions.

## Dependencies

Start by installing our test runner, in this case jest:

```sh
yarn add -D jest
```

We'll need to [install and run the Sandbox](../../cli/sandbox-reference.md#installation).

## Test setup

Create a new file `src/index.test.mjs` with the imports we'll be using and an empty test suite to begin with:

```js
import {
  Contract,
  ExtendedNote,
  Fr,
  Note,
  computeMessageSecretHash,
  createPXEClient,
  waitForPXE,
} from "@aztec/aztec.js";
import { createAccount } from '@aztec/accounts/testing';
import { TokenContractArtifact } from "@aztec/noir-contracts/Token";

const {
  PXE_URL = "http://localhost:8080",
  ETHEREUM_HOST = "http://localhost:8545",
} = process.env;

describe("token contract", () => {});
```

Let's set up our test suite. We'll make sure the Sandbox is running, create two fresh accounts to test with, and deploy an instance of our contract. `aztec.js` provides the helper functions we need to do this:

#include_code setup yarn-project/end-to-end/src/sample-dapp/index.test.mjs javascript

:::tip
Instead of creating new accounts in our test suite, we can use the ones already initialized by the Sandbox upon startup. This can provide a speed boost to your tests setup. However, bear in mind that you may accidentally introduce an interdependency across test suites by reusing the same accounts. Read more [here](../testing.md#using-sandbox-initial-accounts).
:::

## Writing our test

Now that we have a working test environment, we can write our first test for exercising the `transfer` function on the token contract. We will use the same `aztec.js` methods we used when building our dapp:

#include_code test yarn-project/end-to-end/src/sample-dapp/index.test.mjs javascript

In this example, we assert that the `recipient`'s balance is increased by the amount transferred. We could also test that the `owner`'s funds are decremented by the same amount, or that a transaction that attempts to send more funds than those available would fail. Check out the [testing guide](../testing.md) for more ideas.

## Running our tests

We can run our `jest` tests using `yarn`. The quirky syntax is due to [jest limitations in ESM support](https://jestjs.io/docs/ecmascript-modules), as well as not picking up `mjs` file by default:

```sh
yarn node --experimental-vm-modules $(yarn bin jest) --testRegex '.*\.test\.mjs$'
```

## Next steps

Now that you have finished the tutorial, you can learn more about [writing contracts with Noir](../../contracts/main.md) or read about the [fundamental concepts behind Aztec Network](../../../learn/about_aztec/technical_overview.md).

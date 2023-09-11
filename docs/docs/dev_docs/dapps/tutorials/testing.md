# Testing

To wrap up this tutorial, we'll set up a simple automated test for our dapp contracts. We will be using [jest](https://jestjs.io/), but any nodejs test runner works fine. 

Here we'll only test the happy path for a `transfer` on our private token contract, but in a real application you should be testing both happy and unhappy paths, as well as both your contracts and application logic. Refer to the full [testing guide](../testing.md) for more info on testing and assertions.

## Dependencies

Start by installing our test runner, in this case jest:

```sh
yarn add -D jest
```

We'll also be running our Sandbox within the test suite, to avoid polluting a global instance, so we'll need to install the Sandbox itself as a dependency as well:

```sh
yarn add -D @aztec/aztec-sandbox
```

## Test setup

Create a new file `src/index.test.mjs` with the imports we'll be using and an empty test suite to begin with:

```js
import { createSandbox } from '@aztec/aztec-sandbox';
import { Contract, createAccount } from '@aztec/aztec.js';
import PrivateTokenArtifact from '../contracts/private_token/target/PrivateToken.json' assert { type: 'json' };

describe('private token', () => {

});
```

Let's set up our test suite. We'll start [a new Sandbox instance within the test](../testing.md#running-sandbox-in-the-nodejs-process), create two fresh accounts to test with, and deploy an instance of our contract. The `aztec-sandbox` and `aztec.js` provide the helper functions we need to do this:

#include_code setup yarn-project/end-to-end/src/sample-dapp/index.test.mjs javascript

Note that, since we are starting a new Sandbox instance, we need to `stop` it in the test suite teardown. Also, even though the Sandbox runs within our tests, we still need a running Ethereum development node. Make sure you are running [Anvil](https://book.getfoundry.sh/anvil/), [Hardhat Network](https://hardhat.org/hardhat-network/docs/overview), or [Ganache](https://trufflesuite.com/ganache/) along with your tests.

## Writing our test

Now that we have a working test environment, we can write our first test for exercising the `transfer` function on the private token contract. We will use the same `aztec.js` methods we used when building our dapp:

#include_code test yarn-project/end-to-end/src/sample-dapp/index.test.mjs javascript

In this example, we assert that the `recipient`'s balance is increased by the amount transferred. We could also test that the `owner`'s funds are decremented by the same amount, or that a transaction that attempts to send more funds than those available would fail. Check out the [testing guide](../testing.md) for more ideas.

## Running our tests

With a local Ethereum development node running in port 8545, we can run our `jest` tests using `yarn`. The quirky syntax is due to [jest limitations in ESM support](https://jestjs.io/docs/ecmascript-modules), as well as not picking up `mjs` file by default:

```sh
yarn node --experimental-vm-modules $(yarn bin jest) --testRegex '.*\.test\.mjs$'
```

## Next steps

Now that you have finished the tutorial, you can dig deeper on the [APIs for dapp development](../api/main.md), learn more about [writing contracts with Noir](../../contracts/main.md), check out the [Sandbox's architecture](../../sandbox/main.md), or read about the [fundamental concepts behind Aztec Network](../../../concepts/foundation/main.md).
# Testing

Testing is an integral part of any piece of software, and especially important for any blockchain application. In this page we will cover how to interact with your Noir contracts in a testing environment to write automated tests for your apps. 

We will be using typescript to write our tests, and rely on the [`aztec.js`](https://www.npmjs.com/package/@aztec/aztec.js) client library to interact with a local Aztec network. We will use [`jest`](https://jestjs.io/) as a testing library, though feel free to use whatever you work with. Configuring the nodejs testing framework is out of scope for this guide.

## A simple example

Let's start with a simple example for a test using the [Sandbox](../sandbox/main.md). We will create two accounts and deploy a token contract in a setup step, and then issue a transfer from one user to another.

#include_code sandbox-example /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

This test sets up the environment by creating a client to the Aztec RPC server running on the Sandbox on port 8080. It then creates two new accounts, dubbed `owner` and `recipient`. Last, it deploys an instance of the [`PrivateTokenContract`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-contracts/src/contracts/private_token_contract/src/main.nr), minting an initial 100 tokens to the owner.

Once we have this setup, the test itself is simple. We check the balance of the `recipient` user to ensure it has no tokens, send and await a deployment transaction, and then check the balance again to ensure it was increased. Note that all numeric values are represented as [native bigints](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt) to avoid loss of precision.

:::info
We are using the `PrivateTokenContract` [typescript interface](../contracts/compiling.md#typescript-interfaces) to get type-safe methods for deploying and interacting with the token contract.
:::

To run the test, first make sure the Sandbox is running on port 8080, and then [run your tests using jest](https://jestjs.io/docs/getting-started#running-from-command-line). Your test should pass, and you should see the following output in the Sandbox logs, where each chunk corresponds to a transaction. Note how this test run has a total of four transactions: two for deploying the account contracts for the `owner` and `recipient`, another for deploying the token contract, and a last one for actually executing the transfer.

```text
rpc_server Registered account 0x2efa51d2e67581aef4578e8cc647a1af2e3f40e9872deeda0919e5f77cb8b2d2
rpc_server Added contract SchnorrAccount at 0x2efa51d2e67581aef4578e8cc647a1af2e3f40e9872deeda0919e5f77cb8b2d2
node Simulating tx 19bfe4fb2569be2168f01eefe5e5a4284d6c1678f17ab5e94c6ba9c811bcb214
node Simulated tx 19bfe4fb2569be2168f01eefe5e5a4284d6c1678f17ab5e94c6ba9c811bcb214 succeeds
rpc_server Executed local simulation for 19bfe4fb2569be2168f01eefe5e5a4284d6c1678f17ab5e94c6ba9c811bcb214
rpc_server Sending transaction 19bfe4fb2569be2168f01eefe5e5a4284d6c1678f17ab5e94c6ba9c811bcb214
node Received tx 19bfe4fb2569be2168f01eefe5e5a4284d6c1678f17ab5e94c6ba9c811bcb214
sequencer Submitted rollup block 2 with 1 transactions

rpc_server Registered account 0x12ef7ceb5064da3a729f598a6a50585059794fdcf347a6fc9bb317002162e3db
rpc_server Added contract SchnorrAccount at 0x12ef7ceb5064da3a729f598a6a50585059794fdcf347a6fc9bb317002162e3db
node Simulating tx 0f195f8f6fb8fe29cf8159c5c664c1288788f1151a5413ec0e35cf378de74794
node Simulated tx 0f195f8f6fb8fe29cf8159c5c664c1288788f1151a5413ec0e35cf378de74794 succeeds
rpc_server Executed local simulation for 0f195f8f6fb8fe29cf8159c5c664c1288788f1151a5413ec0e35cf378de74794
rpc_server Sending transaction 0f195f8f6fb8fe29cf8159c5c664c1288788f1151a5413ec0e35cf378de74794
node Received tx 0f195f8f6fb8fe29cf8159c5c664c1288788f1151a5413ec0e35cf378de74794
sequencer Submitted rollup block 3 with 1 transactions

rpc_server Added contract PrivateToken at 0x24e691d8bde970ab9e84fe669ea5ac8019c32c199a55aaa8a3e704db763af88f
node Simulating tx 0101e1a3d73c3a112a18b7e4954edfe611d74ae0dc59e1688221ecda982ba943
node Simulated tx 0101e1a3d73c3a112a18b7e4954edfe611d74ae0dc59e1688221ecda982ba943 succeeds
rpc_server Executed local simulation for 0101e1a3d73c3a112a18b7e4954edfe611d74ae0dc59e1688221ecda982ba943
rpc_server Sending transaction 0101e1a3d73c3a112a18b7e4954edfe611d74ae0dc59e1688221ecda982ba943
node Received tx 0101e1a3d73c3a112a18b7e4954edfe611d74ae0dc59e1688221ecda982ba943
sequencer Submitted rollup block 4 with 1 transactions

node Simulating tx 2132767911fbbe67e24a3e51bc769ba2ae874eb1ba56e69cef8fc9e2c5eba04c
node Simulated tx 2132767911fbbe67e24a3e51bc769ba2ae874eb1ba56e69cef8fc9e2c5eba04c succeeds
rpc_server Executed local simulation for 2132767911fbbe67e24a3e51bc769ba2ae874eb1ba56e69cef8fc9e2c5eba04c
rpc_server Sending transaction 2132767911fbbe67e24a3e51bc769ba2ae874eb1ba56e69cef8fc9e2c5eba04c
node Received tx 2132767911fbbe67e24a3e51bc769ba2ae874eb1ba56e69cef8fc9e2c5eba04c
sequencer Submitted rollup block 5 with 1 transactions
```

### Using Sandbox initial accounts

Instead of creating new accounts in our test suite, we can use the ones already initialized by the Sandbox upon startup. This can provide a speed boost to your tests setup. However, bear in mind that you may accidentally introduce an interdependency across test suites by reusing the same accounts.

#include_code use-existing-wallets /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

### Running Sandbox in the nodejs process

Instead of connecting to a local running Sandbox instance, you can also start your own Sandbox within the nodejs process running your tests, for an easier setup. To do this, import the `@aztec/aztec-sandbox` package in your project, and run `createSandbox` during setup. Note that this will still require you to run a local Ethereum development node like [Anvil](https://book.getfoundry.sh/anvil/), [Hardhat Network](https://hardhat.org/hardhat-network/docs/overview), or [Ganache](https://trufflesuite.com/ganache/).

#include_code in-proc-sandbox /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

The `createSandbox` returns a `stop` callback that you should run once your test suite is over to stop all Sandbox services.

#include_code stop-in-proc-sandbox /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

## Assertions

We will now see how to use `aztec.js` to write assertions about transaction statuses, about chain state both public and private, and about logs.

### Transactions

In the example above we used `contract.methods.method().send().wait()` to create a function call for a contract, send it, and await it to be mined successfully. But what if we want to assert failing scenarios?

#### A private call fails

We can check that a call to a private function would fail by simulating it locally and expecting a rejection. Remember that all private function calls are only executed locally in order to preserve privacy. As an example, we can try transferring more tokens than we have, which will fail an assertion with the `Balance too low` error message.

#include_code local-tx-fails /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

Under the hood, the `send()` method executes a simulation, so we can just call the usual `send().wait()` to catch the same failure.

#include_code local-tx-fails-send /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

#### A transaction is dropped

We can have private transactions that work fine locally, but are dropped by the sequencer when tried to be included due to a double-spend. In this example, we simulate two different transfers that would succeed individually, but not when both are tried to be mined. Here we need to `send()` the transaction and `wait()` for it to be mined.

#include_code tx-dropped /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

#### A public call fails locally

Public function calls can be caught failing locally similar to how we catch private function calls. For this example, we use a [`TokenContract`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr) instead of a private one.

:::info
Keep in mind that public function calls behave as in EVM blockchains, in that they are executed by the sequencer and not locally. Local simulation helps alert the user of a potential failure, but the actual execution path of a public function call will depend on when it gets mined.
:::

#include_code local-pub-fails /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

#### A public call fails on the sequencer

We can ignore a local simulation error for a public function via the `skipPublicSimulation`. This will submit a failing call to the sequencer, who will then reject it and drop the transaction.

#include_code pub-dropped /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

If you run the snippet above, you'll see the following error in the Sandbox logs:

```
WARN Error processing tx 06dc87c4d64462916ea58426ffcfaf20017880b353c9ec3e0f0ee5fab3ea923f: Assertion failed: Balance too low.
```

:::info
In the near future, transactions where a public function call fails will get mined but flagged as reverted, instead of dropped.
:::

### State

We can check private or public state directly rather than going through view-only methods, as we did in the initial example by calling `token.methods.balance().view()`. Bear in mind that directly accessing contract storage will break any kind of encapsulation.

To query storage directly, you'll need to know the slot you want to access. This can be checked in the [contract's `Storage` definition](../contracts/storage.md) directly for most data types. However, when it comes to mapping types, as in most EVM languages, we'll need to calculate the slot for a given key. To do this, we'll use the `CheatCodes` utility class:

#include_code calc-slot /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

#### Querying private state

Private state in the Aztec Network is represented via sets of [private notes](../../concepts/foundation/state_model.md#private-state). In our token contract example, the balance of a user is represented as a set of unspent value notes, each with their own corresponding numeric value.

#include_code value-note-def yarn-project/aztec-nr/value-note/src/value_note.nr rust

We can query the RPC server for all notes encrypted for a given user in a contract slot. For this example, we'll get all notes encrypted for the `owner` user that are stored on the token contract address and on the slot we calculated earlier. To calculate the actual balance, we extract the `value` of each note, which is the first element, and sum them up.

#include_code private-storage /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

#### Querying public state

[Public state](../../concepts/foundation/state_model.md#public-state) behaves as a key-value store, much like in the EVM. This scenario is much more straightforward, in that we can directly query the target slot and get the result back as a buffer. Note that we use the [`TokenContract`](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-contracts/src/contracts/token_contract/src/main.nr) in this example, which defines a mapping of public balances on slot 6.

#include_code public-storage /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

### Logs

Last but not least, we can check the logs of [events](../contracts/events.md) emitted by our contracts. Contracts in Aztec can emit both [encrypted](../contracts/events.md#encrypted-events) and [unencrypted](../contracts/events.md#unencrypted-events) events.

:::info
At the time of this writing, only unencrypted events can be queried directly. Encrypted events are always assumed to be encrypted notes.
:::

#### Querying unencrypted logs

We can query the RPC server for the unencrypted logs emitted in the block where our transaction is mined. Note that logs need to be unrolled and formatted as strings for consumption.

#include_code unencrypted-logs /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

## Cheats

The `CheatCodes` class, which we used for [calculating the storage slot above](#state), also includes a set of cheat methods for modifying the chain state that can be handy for testing.

### Set next block timestamp

The `warp` method sets the time for next execution, both on L1 and L2. We can test this using an `isTimeEqual` function in a `Test` contract defined like the following:

#include_code is-time-equal yarn-project/noir-contracts/src/contracts/test_contract/src/main.nr rust

We can then call `warp` and rely on the `isTimeEqual` function to check that the timestamp was properly modified.

#include_code warp /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

:::info
The `warp` method calls `evm_setNextBlockTimestamp` under the hood on L1.
:::

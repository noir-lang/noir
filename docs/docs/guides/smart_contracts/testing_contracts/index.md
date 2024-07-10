---
title: Testing Contracts
---

Aztec contracts can be tested in a variety of ways depending on the needs of a particular application and the complexity of the interactions they must support.

To test individual contract functions, you can use the Testing eXecution Environment (TXE) described below. For more complex interactions that require checking that the protocol rules are enforced, you should [write end-to-end tests using TypeScript](../../js_apps/test.md).

## Pure Noir tests

Noir supports the `#[test]` annotation which can be used to write simple logic tests on isolated utility functions. These tests only make assertions on algorithms and cannot interact with protocol-specific constructs such as `storage` or `context`, but are extremely fast and can be useful in certain scenarios.

#include_code pure_noir_testing /noir-projects/noir-contracts/contracts/card_game_contract/src/cards.nr rust

To learn more about Noir testing, please refer to [the Noir docs](https://Noir-lang.org/docs/tooling/testing/).

## TXE (pronounced "trixie")

In order to interact with the protocol, Aztec contracts leverage the power of oracles: functions that reach out to the outside world and are able to query and manipulate data outside of itself. The values returned by oracles are then constrained inside Noir and modifications to the blockchain state are later verified to adhere to the protocol rules by our kernel circuits.

However, all of this is often not necessary to ensure the contract logic itself is sound. All that we need is an entity to provide values consistent with real execution. This is where our TXE (Testing eXecution Environment, pronounced "trixie") comes in!

TXE is a JSON RPC server much like PXE, but provides an extra set of oracle functions called `cheatcodes` that allow developers to manipulate the state of the chain and simulate contract execution. Since TXE skips most of the checks, block building and other intricacies of the Aztec protocol, it is much faster to run than simulating everything in the sandbox.

## TXE vs End-to-end tests

End-to-end tests are written in typescripts and use compiled Aztec contracts and generated Typescript interfaces, a private execution environment (PXE) and a simulated execution environment to process transactions, create blocks and apply state updates. This allows for advanced checks on state updates like generation the of logs, cross-chain messages and checking transaction status and also enforce the rules of the protocol (e.g. checks in our rollup circuits). If you need the rules of the protocol to be enforced or require complex interactions (such as with L1 contracts), please refer to [Testing Aztec.nr contracts with Typescript](../../js_apps/test.md).

The TXE is a super fast framework in Noir to quickly test your smart contract code. 

So to summarize:
* End-to-end tests are written in Typescript. TXE in Noir.
* End-to-end tests are most similar to using mocha + ethers.js to test Solidity Contracts. TXE is like foundry (fast tests in solidity) 

### Running TXE

In order to use the TXE, it must be running on a known address. 

:::tip
If you have [the sandbox](../../../getting_started.md) installed, you can quickly deploy a TXE by running:

`docker run --workdir /usr/src/yarn-project --entrypoint bash --name txe -p 8080:8080 --rm -it aztecprotocol/aztec -c "yarn workspaces focus @aztec/txe && cd txe && yarn build && yarn start"`

This will be improved in the future with a dedicated command.
:::

By default, TXE runs at `http://localhost:8080`. Using `aztec-nargo`, contract tests can be run with:

`aztec-nargo test --use-legacy --silence-warnings --oracle-resolver http://host.docker.internal:8080`

:::warning
Since TXE tests are written in Noir and executed with `aztec-nargo`, they all run in parallel. This also means every test creates their own isolated environment, so state modifications are local to each one of them.

Executing many tests in parallel might slow processing of the RPC calls down to the point of making them timeout. To control this timeout the `NARGO_FOREIGN_CALL_TIMEOUT` env variable is used.
:::

### Writing TXE tests

`aztec-nr` provides an utility class called `TestEnvironment`, that should take care of the most common operations needed to setup contract testing. Setting up a new test environment with `TestEnvironment::new()` **will reset the current test's TXE state**.

:::tip
You can find all of the methods available in the `TestEnvironment` [here](https://github.com/AztecProtocol/aztec-packages/blob/#include_aztec_version/noir-projects/aztec-nr/aztec/src/test/helpers/test_environment.nr).
:::

#include_code txe_test_increment /noir-projects/noir-contracts/contracts/counter_contract/src/main.nr rust

:::warning
Tests run significantly faster as `unconstrained` functions. This means we generate bytecode (Brillig) and not circuits (ACIR), which _should_ yield exactly the same results. Any other behavior is considered a bug.
:::

### Imports

Writing tests in contracts requires importing additional modules from Aztec.nr. Here are the modules that are needed for testing the increment function in the counter contract.

#include_code test_imports /noir-projects/noir-contracts/contracts/counter_contract/src/main.nr rust

### Deploying contracts

```rust
let deployer = env.deploy("path_to_contract_ts_interface");

// Now one of these can be called, depending on the contract and their possible initialization options.
// Remember a contract can only be initialized once.

let my_private_initializer_call_interface = MyContract::interface().private_constructor(...);
let my_contract_instance = deployer.with_private_initializer(my_private_initializer_call_interface);

// or

let my_public_initializer_call_interface = MyContract::interface().public_constructor(...);
let my_contract_instance = deployer.with_public_initializer(my_public_initializer_call_interface);

// or

let my_contract_instance = deployer.without_initializer();
```

:::warning
At the moment, TXE uses the generated contract TypeScript interfaces to perform deployments, and they must be provided as either an absolute path, a relative path to TXE's location or a module in an npm direct dependency such as `@aztec/noir-contracts.js`. It is not always necessary to deploy a contract in order to test it, but sometimes it's inevitable (when testing functions that depend on the contract being initialized, or contracts that call others for example) **It is important to keep them up to date**, as TXE cannot recompile them on changes. This will be improved in the future.
:::

### Calling functions

Our test environment is capable of utilizing the autogenerated contract interfaces to abstract calls, but without going through the usual external call flow (meaning much faster execution).

#### Private

For example, to call the private `transfer` function on the token contract:

#include_code txe_test_transfer_private /noir-projects/noir-contracts/contracts/token_contract/src/test/transfer_private.nr rust

#### Public

To call the public `transfer_public` function:

#include_code call_public /noir-projects/noir-contracts/contracts/token_contract/src/test/transfer_public.nr rust

#### Unconstrained

Unconstrained functions can be directly called from the contract interface. Notice that we need to set the contract address to the specific token contract that we are calling before making the call. This is to ensure that `view_notes` works properly.

#include_code txe_test_call_unconstrained /noir-projects/noir-contracts/contracts/token_contract/src/test/utils.nr rust

### Creating accounts

The test environment provides two different ways of creating accounts, depending on the testing needs. For most cases, it is only necessary to obtain a valid `AztecAddress` that represents the user's account contract. For this, is is enough to do:

```rust
let mocked_account_address = env.create_account();
```

These accounts also create the necessary keys to ensure notes can be created/nullified, etc.

For more advanced flows, such as authwits, it is necessary to create a real `AccountContract`, with valid signing keys that gets actually deployed to TXE. For that you can use:

```rust
let real_account_address = env.create_account_contract(secret);
```

Besides deploying a complete `SchnorrAccountContract`, key derivation is performed so that authwits can be signed. It is slightly slower than the mocked version.

Once accounts have been created, you can impersonate them in your test by calling:

```rust
env.impersonate(account_address);
```

### Checking state

It is possible to use the regular oracles in tests in order to retrieve public and private state and make assertions about them.

:::warning
Remember to switch to the current contract's address in order to be able to read it's siloed state!
:::

Reading public state:
#include_code txe_test_read_public /noir-projects/noir-contracts/contracts/token_contract/src/test/utils.nr rust

Reading notes:
#include_code txe_test_read_notes /noir-projects/noir-contracts/contracts/counter_contract/src/main.nr rust

### Authwits

#### Private

You can add [authwits](../writing_contracts/authwit.md) to the TXE. Here is an example of testing a private token transfer using authwits:

#include_code private_authwit /noir-projects/noir-contracts/contracts/token_contract/src/test/transfer_private.nr rust

#### Public

#include_code public_authwit /noir-projects/noir-contracts/contracts/token_contract/src/test/transfer_public.nr rust

### Storing notes in cache

Sometimes we have to tell TXE about notes that are not generated by ourselves, but someone else. This allows us to check if we are able to decrypt them:

#include_code txe_test_store_note /noir-projects/noir-contracts/contracts/token_contract/src/test/utils.nr rust

### Time traveling

TXE can force the generation of "new blocks" very quickly using:

```rust
env.advance_block_by(n_blocks);
```

This will effectively consolidate state transitions into TXE's internal trees, allowing things such as reading "historical state" from private, generating inclusion proofs, etc.

### Failing cases

You can test functions that you expect to fail generically, with the `#[test(should_fail)]` annotation, or that it should fail with a specific message with `#[test(should_fail_with = "Failure message")]`.

For example:

#include_code fail_with_message /noir-projects/noir-contracts/contracts/token_contract/src/test/transfer_private.nr rust

You can also use the `assert_public_call_fails` or `assert_private_call_fails` methods on the `TestEnvironment` to check that a call fails.

#include_code assert_public_fail /noir-projects/noir-contracts/contracts/token_contract/src/test/transfer_public.nr rust

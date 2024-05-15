---
title: Call Types
---

## What is a Call

We say that a smart contract is called when one of its functions is invoked and its code is run. This means there'll be:

- a caller
- arguments
- return values
- a call status (successful or failed)

There are multiple types of calls, and some of the naming can make things **very** confusing. This page lists the different call types and execution modes, pointing out key differences between them.

## Ethereum Call Types

Even though we're discussing Aztec, its design is heavily influenced by Ethereum and many of the APIs and concepts are quite similar. It is therefore worthwhile to briefly review how things work there and what naming conventions are used to provide context to the Aztec-specific concepts.

Broadly speaking, Ethereum contracts can be thought of as executing as a result of three different things: running certain EVM opcodes, running Solidity code (which compiles to EVM opcodes), or via the node JSON-RPC interface (e.g. when executing transactions).

### EVM

Certain opcodes allow contracts to make calls to other contracts, each with different semantics. We're particularly interested in `CALL` and `STATICCALL`, and how those relate to contract programming languages and client APIs.

#### `CALL`

This is the most common and basic type of call. It grants execution control to the caller until it eventually returns. No special semantics are in play here. Most Ethereum transactions spend the majority of their time in `CALL` contexts.

#### `STATICCALL`

This behaves almost exactly the same as `CALL`, with one key difference: any state-changing operations are forbidden and will immediately cause the call to fail. This includes writing to storage, emitting logs, or deploying new contracts. This call is used to query state on an external contract, e.g. to get data from a price oracle, check for access control permissions, etc.

#### Others

The `CREATE` and `CREATE2` opcodes (for contract deployment) also result in something similar to a `CALL` context, but all that's special about them has to do with how deployments work. `DELEGATECALL` (and `CALLCODE`) are somewhat complicated to understand but don't have any Aztec equivalents, so they are not worth covering.

### Solidity

Solidity (and other contract programming languages such as Vyper) compile down to EVM opcodes, but it is useful to understand how they map language concepts to the different call types.

#### Mutating External Functions

These are functions marked `payable` (which can receive ETH, which is a state change) or with no mutability declaration (sometimes called `nonpayable`). When one of these functions is called on a contract, the `CALL` opcode is emitted, meaning the callee can perform state changes, make further `CALL`s, etc.

It is also possible to call such a function with `STATICCALL` manually (e.g. using assembly), but the execution will fail as soon as a state-changing opcode is executed.

#### `view`

An external function marked `view` will not be able to mutate state (write to storage, etc.), it can only _view_ the state. Solidity will emit the `STATICCALL` opcode when calling these functions, since its restrictions provide added safety to the caller (e.g. no risk of reentrancy).

Note that it is entirely possible to use `CALL` to call a `view` function, and the result will be the exact same as if `STATICCALL` had been used. The reason why `STATICCALL` exists is so that _untrusted or unknown_ contracts can be called while still being able to reason about correctness. From the [EIP](https://eips.ethereum.org/EIPS/eip-214):

> '`STATICCALL` adds a way to call other contracts and restrict what they can do in the simplest way. It can be safely assumed that the state of all accounts is the same before and after a static call.'

### JSON-RPC

From outside the EVM, calls to contracts are made via [JSON-RPC](https://ethereum.org/en/developers/docs/apis/json-rpc/) methods, typically from some client library that is aware of contract ABIs, such as [ethers.js](https://docs.ethers.org/v5) or [viem](https://viem.sh/).

#### `eth_sendTransaction`

This method is how transactions are sent to a node to get them to be broadcast and eventually included in a block. The specified `to` address will be called in a `CALL` context, with some notable properties:

- there are no return values, even if the contract function invoked does return some data
- there is no explicit caller: it is instead derived from a provided signature

Some client libraries choose to automatically issue `eth_sendTransaction` when calling functions from a contract ABI that are not marked as `view` - [ethers is a good example](https://docs.ethers.org/v5/getting-started/#getting-started--writing). Notably, this means that any return value is lost and not available to the calling client - the library typically returns a transaction receipt instead. If the return value is required, then the only option is to simulate the call `eth_call`.

Note that it is possible to call non state-changing functions (i.e. `view`) with `eth_sendTransaction` - this is always meaningless. What transactions do is change the blockchain state, so all calling such a function achieves is for the caller to lose funds by paying for gas fees. The sole purpose of a `view` function is to return data, and `eth_sendTransaction` does not make the return value available.

#### `eth_call`

This method is the largest culprit of confusion around calls, but unfortunately requires understanding of all previous concepts in order to be explained. Its name is also quite unhelpful.

What `eth_call` does is simulate a transaction (a call to a contract) given the current blockchain state. The behavior will be the exact same as `eth_sendTransaction`, except:

- no actual transaction will be created
- while gas _will_ be measured, there'll be no transaction fees of any kind
- no signature is required: the `from` address is passed directly, and can be set to any value (even if the private key is unknown, or if they are contract addresses!)
- the return value of the called contract is available

`eth_call` is typically used for one of the following:

- query blockchain data, e.g. read token balances
- preview the state changes produced by a transaction, e.g. the transaction cost, token balance changes, etc

Because some libraries ([such as ethers](https://docs.ethers.org/v5/getting-started/#getting-started--reading)) automatically use `eth_call` for `view` functions (which when called via Solidity result in the `STATICCALL` opcode), these concepts can be hard to tell apart. The following bears repeating: **an `eth_call`'s call context is the same as `eth_sendTransaction`, and it is a `CALL` context, not `STATICCALL`.**

## Aztec Call Types

Large parts of the Aztec Network's design are still not finalized, and the nitty-gritty of contract calls is no exception. This section won't therefore contain a thorough review of these, but rather list some of the main ways contracts can currently be interacted with, with analogies to Ethereum call types when applicable.

While Ethereum contracts are defined by bytecode that runs on the EVM, Aztec contracts have multiple modes of execution depending on the function that is invoked.

### Private Execution

Contract functions marked with `#[aztec(private)]` can only be called privately, and as such 'run' in the user's device. Since they're circuits, their 'execution' is actually the generation of a zk-SNARK proof that'll later be sent to the sequencer for verification.

#### Private Calls

Private functions from other contracts can be called either regularly or statically by using the `.call()` and `.static_call` functions. They will also be 'executed' (i.e. proved) in the user's device, and `static_call` will fail if any state changes are attempted (like the EVM's `STATICCALL`).

#include_code private_call /noir-projects/noir-contracts/contracts/lending_contract/src/main.nr rust

Unlike the EVM however, private execution doesn't revert in the traditional way: in case of error (e.g. a failed assertion, a state changing operation in a static context, etc.) the proof generation simply fails and no transaction request is generated, spending no network gas or user funds.

#### Public Calls

Since public execution can only be performed by the sequencer, public functions cannot be executed in a private context. It is possible however to _enqueue_ a public function call during private execution, requesting the sequencer to run it during inclusion of the transaction. It will be [executed in public](#public-execution) normally, including the possibility to enqueue static public calls.

Since the public call is made asynchronously, any return values or side effects are not available during private execution. If the public function fails once executed, the entire transaction is reverted inncluding state changes caused by the private part, such as new notes or nullifiers. Note that this does result in gas being spent, like in the case of the EVM.

#include_code enqueue_public /noir-projects/noir-contracts/contracts/lending_contract/src/main.nr rust

It is also possible to create public functions that can _only_ be invoked by privately enqueing a call from the same contract, which can very useful to update public state after private exection (e.g. update a token's supply after privately minting). This is achieved by annotating functions with `#[aztec(internal)]`.

A common pattern is to enqueue public calls to check some validity condition on public state, e.g. that a deadline has not expired or that some public value is set.

#include_code call-check-deadline /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

#include_code deadline /noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

:::warning
Calling public functions privately leaks some privacy! The caller of the function and all arguments will be revelead, so exercise care when mixing the private and public domains. To learn about alternative ways to access public state privately, look into [Shared State](/reference/reference/smart_contract_reference/storage/shared_state.md).
:::

### Public Execution

Contract functions marked with `#[aztec(public)]` can only be called publicly, and are executed by the sequencer. The computation model is very similar to the EVM: all state, parameters, etc. are known to the entire network, and no data is private. Static execution like the EVM's `STATICCALL` is possible too, with similar semantics (state can be accessed but not modified, etc.).

Since private calls are always run in a user's device, it is not possible to perform any private execution from a public context. A reasonably good mental model for public execution is that of an EVM in which some work has already been done privately, and all that is know about it is its correctness and side-effects (new notes and nullifiers, enqueued public calls, etc.). A reverted public execution will also revert the private side-effects.

Public functions in other contracts can be called both regularly and statically, just like on the EVM.

#include_code public_call /noir-projects/noir-contracts/contracts/fpc_contract/src/main.nr rust

:::note
This is the same function that was called by privately enqueuing a call to it! Public functions can be called either directly in a public context, or asynchronously by enqueuing in a private context.
:::

### Top-level Unconstrained

Contract functions with the `unconstrained` Noir keyword are a special type of function still under development, and their semantics will likely change in the near future. They are used to perform state queries from an off-chain client, and are never included in any transaction. No guarantees are made on the correctness of the result since they rely exclusively on unconstrained oracle calls.

A reasonable mental model for them is that of a `view` Solidity function that is never called in any transaction, and is only ever invoked via `eth_call`. Note that in these the caller assumes that the node is acting honestly by exectuing the true contract bytecode with correct blockchain state, the same way the Aztec version assumes the oracles are returning legitimate data.

### aztec.js

There are three different ways to execute an Aztec contract function using the `aztec.js` library, with close similarities to their [JSON-RPC counterparts](#json-rpc).

#### `simulate`

This is used to get a result out of an execution, either private or public. It creates no transaction and spends no gas. The mental model is fairly close to that of [`eth_call`](#eth_call), in that it can be used to call any type of function, simulate its execution and get a result out of it. `simulate` is also the only way to run [top-level unconstrained functions](#top-level-unconstrained).

#include_code public_getter /noir-projects/noir-contracts/contracts/auth_contract/src/main.nr rust

#include_code simulate_public_getter yarn-project/end-to-end/src/e2e_auth_contract.test.ts typescript

:::warning
No correctness is guaranteed on the result of `simulate`! Correct execution is entirely optional and left up to the client that handles this request.
:::

#### `prove`

This creates and returns a transaction request, which includes proof of correct private execution and side-efects. The request is not broadcast however, and no gas is spent. It is typically used in testing contexts to inspect transaction parameters or to check for execution failure.

#include_code local-tx-fails /yarn-project/end-to-end/src/guides/dapp_testing.test.ts typescript

Like most Ethereum libraries, `prove` also simulates public execution to try to detect runtime errors that would only occur once the transaction is picked up by the sequencer. This makes `prove` very useful in testing environments, but users shuld be wary of both false positives and negatives in production environments, particularly if the node's data is stale. Public simulation can be skipped by setting the `skipPublicSimulation` flag.

#### `send`

This is the same as [`prove`](#prove) except it also broadcasts the transaction and returns a receipt. This is how transactions are sent, getting them to be included in blocks and spending gas. It is similar to [`eth_sendTransaction`](#eth_sendtransaction), except it also performs some work on the user's device, namely the production of the proof for the private part of the transaction.

#include_code send_tx yarn-project/end-to-end/src/e2e_card_game.test.ts typescript

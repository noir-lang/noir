---
title: Components
---

<!-- TODO: This is a copy of milestone1-1, which actually contained a comprehensive explanation of components (if we remove the json rpc command suggestions) -->

:::TODO Outdated
This page needs to be updated.
:::

import Disclaimer from '../../misc/common/\_disclaimer.mdx';

<Disclaimer/>

You can track the development of these components in the [aztec3-packages repo](https://github.com/AztecProtocol/aztec3-packages/tree/master).

### KeyStore

Responsibilities:

- Holds and never exposes private keys.
- Provides sign/decrypt functionality.

### PXEClient

Responsibilities:

- Shields any upstream entities (dapp) from private information.
- Shields any downstream entities (Public Client) from private information.
- Maintains an up-to-date view of required parts of the global state from state streams to generate a tx.
- Maintains an up-to-date view of state for accounts in a `KeyStore`.

### PublicClient

Responsibilities:

- Acts largely as a facade to sub-systems.
- Provide methods for access Merkle Tree data (`SiblingPathSource`).
- Provide methods for requesting storage slot data for public state.
- Provide methods for requesting published rollup data (`RollupSource`).
- Receives txs and broadcasts them to the network via the `P2PClient`.

### P2PClient

Responsibilities:

- Receives txs from the `PublicClient`.
- Receives txs from other `P2PClient` instances in the network.
- Verifies received txs meet criteria to be added to the pool.
- Forwards all valid txs to other `P2PClient` instances in the network.
- Receives rollups and purges settled or now invalid txs from local pool.
- Purges txs that are older than a certain threshold from the local pool.

### SiblingPathSource

Responsibilities:

- Returns the sibling path for the given `tree_id` at the given leaf `index`.
- Can be injected into any context that requires path queries for a particular tree. Might be backed by implementations that call out to a server (where privacy doesn't matter, or during early development), or leverages some privacy tech (query via Nym), or could just point to a local `WorldStateSynchroniser`.

### RollupSource

Responsibilities:

- Can be queried for `Rollup` data.

### RollupReceiver

Responsibilities:

- Given the necessary rollup data, verifies it, and updates the underlying state accordingly to advance the state of the system.

### TxReceiver

Responsibilities:

- Receives a tx, performs necessary validation, and makes it available to `SequencerClient` instances for rolling up.

### TxPool

Responsibilities:

- Gives a view of the pending transaction pool.

## Module Implementation Notes

### Noir

These tasks are lower priority than providing a handcrafted ABI.

- The ability for a dev to enclose a collection of Aztec.nr functions in a 'contract scope'.
- The ability to create an Aztec.nr contract abi from the above.

Design an Aztec.nr Contract ABI, similar to a Solidity ABI which is output by Solc (see [here](https://docs.soliditylang.org/en/v0.8.13/abi-spec.html#json)). It might include for each function:

- ACIR opcodes (akin to Solidity bytecode).
- Function name and parameter names & types.
- Public input/output witness indices?
- Sourcemap information, allowing building of stack traces in simulator error cases.

### aztec-cli

Provides the `aztec-cli` binary for interacting with network from the command line. It's a thin wrapper around `aztec.js` to make calls out to the client services. It's stateless.

### aztec.js

![](https://hackmd.io/_uploads/ryRmLLL12.png)

Should provide sensible API, that provides the following functionalities. Start by writing the single e2e test that will check for the successful contract deployment. We don't need to get this perfect, but think hard about making the process feel very natural to a user of the library.

aztec.js should always be stateless. It offers the ability to interact with stateful systems such as the public and private clients.

The analogous AC component would be the AztecSdk (wraps the CoreSdk which is more analogous to the private client).

- Allows a user to create an Aztec keypair. Call `create_account` on Wallet.
- Create a `Contract` instance (similar to web3.js), given a path to an Aztec.nr Contract ABI.
- Construct `tx_request` by calling e.g. `contract.get_deployment_request(constructor_args)`.
- Call wallet `sign_tx_request(tx_request)` to get signature.
- Call `simulate_tx(signed_tx_request)` on the Private Client. In future this would help compute gas, for now we won't actually return gas (it's hard). Returns success or failure, so client knows if it should proceed, and computed kernel circuit public outputs.
- Call `create_tx(signed_tx_request)` on Private Client to produce kernel proof. Can be skipped for this milestone. In future should be able to generate either mock (few constraints) or real proof.
- Call `send_tx(tx)` on Private Client to send kernel proof to Public Client. Get back a receipt.
- Wait until the receipt is settled with repeated calls to `get_tx_receipt` on the Private Client.

### ethereum.js

L1 client library. No ethers.js. Uses our existing and new L1 client code.

### TestKeyStore

Interfaces Implemented:

- `KeyStore`

Implementation notes for this milestone:

- Exposes a single account, created by a private key given in the ctor, that defaults to some test key.
- Can be used as both the spending and decryption key for early development.

### Private Execution Environment (PXE)
![](https://hackmd.io/_uploads/ryS0sOLyh.png)

Implements:

- `PXE` (The server is a client, when used directly)

Injected:

- `KeyStore`

Implementation notes for this milestone:

- Avoid saving any state for now. Just resync from the `PublicClient` on startup.
- Avoid needing to sync the merkle trees for now. Just use `SiblingPathSource` that queries the paths from the `PublicClient`.
- `simulate_tx`
  - Can compute the contract's leaf (which will be inserted into the contract tree). See diagram linked at top of page.
  - Can spin-up a simulator instance for the constructor function.
    - (Note, executing an _interesting_ constructor function is not until the next milestone, but we still need to execute a no-op constructor function for a contract deployment, since the public inputs of the constructor carry some contract deployment info, and need to passed into the kernel circuit).
  - Simulator will check whether the tx will succeed. It will return all public inputs/outputs.
  - Can collect/generate additional data required for kernel circuit execution.
  - Can pass the fake constructor proof, public inputs, and other kernel circuit data into a kernel simulator.
  - Returns if the simulation succeeded, and the public inputs/outputs of the kernel circuit.
- `create_tx`
  - For this milestone, will just do the same as `simulate_tx`.
  - Returns a `tx` that can be sent to `send_tx` with everything but actual proof data.
- `send_tx`
  - Forwards the given tx to the Public Client.

### PublicClientImpl

![](https://hackmd.io/_uploads/Sksg3u8kh.png)

Implements:

- `PublicClient`

Injected:

- `P2PClient`
- `RollupSource`
- `MerkleTreeDb`

Implementation notes for this milestone:

- Mostly acting as a facade for other components, forwards requests as necessary.
- A `WorldStateSynchroniser` will ingest rollups from the `RollupSource` and maintain an up-to-date `MerkleTreeDb`.

<!--
Mikes bullets. Some of these in wrong place (sequencer does publishing)
- Can simulate & generate a rollup proof (1x1). (See below for section on rollup logic).
- Can submit the rollup proof to the rollup contract.
- Can sync from the rollup contract.
    - Note: syncing will only mean syncing the contract tree at this milestone.
- Can maintain a contract tree which reflects what's happened on the rollup contract.
    - The contract tree is an append-only tree.
- Can maintain “tree of contract tree roots”, much like we had for the data tree in Aztec Connect. This means clients can reference “old” contract tree roots when generating proofs, in the event the contract tree updates asynchronously. The tree of roots only needs to be of size e.g. 128, to ensure there is a large enough “window of time”.
- Can maintain a DB of all code (ACIR) which has been deployed to the contract tree, along with the contract's constructor args, and the contract's address and contract leaf's index.
- Can receive json-rpc requests from the Private Client and/or from aztec.js:
    - `get_code` - to get details about the contract code deployed at a particular address (or, differently, a particular leaf of the contract tree).
-->

### MemoryP2PClient

Implements:

- `P2PClient`

Injected:

- `RollupSource`
- `MerkleTreeDb`

Implementation notes for this milestone:

- For this milestone there won't be any p2p network.
- Perform relevant validations possible in this milestone (e.g. check no tree conflicts, skip proof verification).
- Purge relevant txs from pool on rollup receipt.
- Store txs in an in memory map of txId -> tx.

### Sequencer Client

![](https://hackmd.io/_uploads/Hk823d81h.png)

Responsibilities:

- Wins a period of time to become the sequencer (depending on finalized protocol).
- Chooses a set of txs from the tx pool to be in the rollup.
- Simulate the rollup of txs.
- Adds proof requests to the request pool (not for this milestone).
- Receives results to those proofs from the network (repeats as necessary) (not for this milestone).
- Publishes L1 tx(s) to the rollup contract via `RollupPublisher`.

For this milestone, the sequencer will just simulate and publish a 1x1 rollup and publish it to L1.

### MerkleTreeDb

Implements:

- `SiblingPathSource`

Implementation notes for this milestone:

Closest analogous component in AC is the `WorldStateDb` in bb.js. We can configure the backing store (probably leveldb) to be an in-memory only store. We don't need persistence, we will rebuild the tree at startup. This will ensure we have appropriate sync-from-zero behaviour.

Responsibilities:

- "Persists" the various merkle trees (configurable).
  - For this milestone 1.1, we'll need the following trees:
    - Contract Tree
    - Contract Tree Roots Tree (the tree whose leaves are the roots of historic rollups' contract trees)
    - Nullifier Tree (so that the contract address can never be re-registered in a future deployment)
      - Note: Suyash has implemented C++ for the 'new' kind of nullifier tree.
- Provides methods for updating the trees with commit, rollback semantics.
- Provides methods for getting hash paths to leafs in the trees.

Interface:

- `get_root`
- `get_num_leaves`
- `get_sibling_path`
- `append_leaves`

### WorldStateSynchroniser

Injected:

- `RollupSource`
- `MerkleTreeDb`

Responsibilities:

- Receives new rollups from the `RollupSource` and updates trees in the `MerkleTreeDb`.

### Rollup Archiver

Implements:

- `RollupSource`

Responsibilities:

- Pulls data in from whatever sources are needed, to fully describe a rollup e.g.
  - L1 calldata (`provessRollup` and `offchainData` for this milestone.)
  - ETH blobs (not before they are released).
  - Other sources that have archived historical ETH blobs (not before they are released).
- Combines these sources of data to describe a `Rollup`.
- Can be queried for the rollup data.

Interface:

- `get_latest_rollup_id`
- `get_rollups(from, take)`

### Rollup Publisher

Implements:

- `RollupReceiver`

Implementation notes for this milestone:

- We can leverage most of the logic from Falafels old `RollupPublisher`.
- On receipt of a rollup, will publish to L1 and respond with success or failure.

### Rollup Smart Contract

Interface:

- `processRollup(proofData, l1Data)`
- `offchainData(data)`

Implementation notes for this milestone:

The rollup contract in AC holds data in two places that are reconciled when processing the rollup. The calldata passed into the `processRollup` function, and the calldata passed into the `offchainData` function. They were separated, as only the data given to `processRollup` is needed to ensure rollup liveness (i.e. if the "offchain data" were not published, we could still produce rollups). Ultimately the plan was to move the offchain data off of L1 altogether to reduce costs.

For this milestone, we will want to leverage a similar separation of data. The data to `processData` will consist of:

- `proofData` - The proof data to be fed to the verifier. Likely will have a public inputs that represent:
  - The hash of the data in the offchain data (should use whatever commitment scheme Eth Blobs will use).
  - Recursion point fields.
- `l1Data` - Data that is needed for making L2 - L1 function calls.

For logic:

- Can receive a rollup proof and verify it (verifier just returns true for this milestone).
- Can update the state hash (which would only represent an update to the contract tree, at this milestone).

For `offchainData`:

- Receives relevant calldata that needs to be broadcast. For contract deployment, this might be:
  - Acir opcodes.
  - Constructor arguments.
  - Contract address & contract leaf.

### Kernel Circuit functionality

- Can validate the signature of a signed tx object.
- Can verify a 'previous' mock kernel circuit (this code is already in there).
- Can verify the constructor function's proof (this functionality is already in there).
- Can check contract deployment logic. (Note: it won't contain checks for private circuit execution logic, as that's for a future milestone).

### Rollup Circuit functionality

- TODO: more discussion needed to understand exactly what's needed here.
- Can verify a kernel circuit proof
- Can compute the contract's address
- Can compute a 'nullifier' for the contract's address, to prevent it being used again (possibly defer this, because we don't want to care about a nullifier tree in this milestone).
- Can compute the contract leaf for the contract (see diagrams).
- Can insert the contract's leaf into the contract tree.
- Can insert the contract tree's root into a "tree of contract tree roots".
- ???? Verifies the ACIR Verification proof (see Contract Creation doc by Zac)

## Glossary

- `witness`: Any value within the circuit, by default these are private, and thus not visible on the proof. Some witnesses may be made public (visible on the proof), at which point they also become `public_inputs`.
- `[circuit_]input`: Data that is computed outside the circuit and is fed in at the beginning as a `witness`. Note: if the context of this relating to a 'circuit' is already clear, we can omit `circuit_`, as is done in the `aztec3_circuits` repo.
- `oracle_input`: Data that, during execution, is fetched from the oracle, and made a `witness`.
- `computed_public_input`: Data that is computed within the circuit, and set to be a `public_input`.
- `public_input`: Data that is set to be public by the circuit. This could include some `circuit_input` data, and/or some `computed_public_input` data, and/or some `oracle_input` data.

## Participate

Keep up with the latest discussion and join the conversation in the [Aztec forum](https://discourse.aztec.network).

---
title: Milestones Overview
---

:::caution
We are building Aztec 3 as transparently as we can. The documents published here are merely an entry point to understanding. These documents are largely complete, but unpolished.

If you would like to help us build Aztec 3, consider reviewing our [GitHub](https://github.com/AztecProtocol) to contribute code and joining our [forum](https://discourse.aztec.network/) to participate in discussions.
:::

The milestones are written as sets of user stories to make it clear what functionality will be available as we work through each one.

## Development Strategy

The goal is that all software components of Aztec3 can be developed as independent modules (e.g. private client, public client, simulators, data sources/sinks, circuit architecture, RPC, cryptography).

When modules need to communicate across module boundaries, they do so via clearly defined APIs.

When developing a module, these external API calls can be mocked. Each module's suite of unit tests ideally are completely independent of other module implementations (i.e. all external API calls are mocked).

The goal is to enable parallel development of these milestones, where teams are not bottlenecked by the progress of other teams.

For a milestone to be complete, full integration tests must be implemented that interact across module boundaries.

:::note
The commands in all of these milestone descriptions are illustrative only. More thinking will be needed on the naming, and the actual parameters than need to be passed to each command.
:::

---

### 1.0 - Repo reformat

Ongoing.

---

### 1.1 - Deploy a contract

See granular tasks [here](milestones/milestone1-1).

As a developer, I can write a Noir++ `contract` scope, which contains a collection of pure functions (circuits) and a noop `constructor` function (no state variables yet).

I can compile my contract with Noir, and receive a contract ABI json.

I can type `aztec3-cli deploy_contract path/to/abi.json` and have a contract deployed to my local network.

I can type `aztec3-cli get_code address` to verify my contract has been deployed succesfully.

Specialisms: Noir, aztec.js, Private Client, Simulator, World State, Sequencer Client, State Gobbler, Rollup Contract, Private Kernel Circuit, Rollup Circuits.

---

### 1.2 - Nullifier functionality

We need nullifier infrastructure, and it's complex enough to be its own milestone. It makes most sense to develop it at this stage. Here's an attempt to shoe-horn it into a user story:

A user can only deploy to an aztec contract address once. Attempts to duplicate a contract address will be rejected by the network.

Specialisms: Private client, Private Kernel Circuit, State Gobbler, World State, Rollup Contract.

---

### 1.3 - Private Constructor

As a developer, I can declare `private` state variables (`UTXO` and `UTXOSet`) in the `contract` scope of my Noir++ contract.

I can write a `constructor` function in Noir, which initialises private states (**initially only owned by the Deployer**) within my contract (i.e. which pushes new commitments to the network).

I can type `aztec3-cli deploy_contract path/to/abi.json` to deploy a contract which calls a constructor function to initialise private state.

> Note: the constructor arguments could be included in the abi.json, similar to a Solidity ABI.

I can type `aztec3-cli get_storage_at contract_address storage_slot` and verify the state was set correctly for private state. (Note: obviously a user can only do this if they own the state at that storage slot).

Specialisms: Noir, aztec.js, Private Client, Simulator, World State, Sequencer Client, State Gobbler, Rollup Contract, Private Kernel Circuit, Rollup Circuits.

---

### 1.4 - A single Private Function Call

As a developer, I can write a `private` `function` in Noir. This function can read private states, modify (nullify) private states, and store updates to private states.

I can type `aztec3-cli send_tx contract_address function_signature arg1 arg2` and have a transaction which calls a single private function be included in the next block. The private function can **initially only create commitments owned by the caller**.

I can type `aztec3-cli get_transaction_receipt tx_hash` and view the status of my transaction on the local network.

I can share the pre-images of private state edits with the owners of those private states (e.g. by broadcasting the encrypted pre-image on-chain).

As a recipient of an edited state, my Private Client can update it's DB with this state update (e.g. via trial-decryption).

Specialisms: Noir, aztec.js, Private Client, Simulator, World State, State Gobbler, Rollup Contract, Private Kernel Circuit.

---

### 1.5 - A single Private Function Call (editing others' states)

As per the previous milestone, but with the ability to call a function which **edits other people's private states**.

I can share the preimages of private state edits with the owners of those private states (e.g. by broadcasting the encrypted preimage on-chain).

As a recipient of an edited state, my Private Client can update it's DB with this state update (e.g. via trial-decryption).

As a recipient of an edited state, I can type `aztec3-cli get_storage_at contract_address storage_slot` and see that my state variable has been edited.

Specialisms: Noir, aztec.js, Private Client, Simulator.

---

### :rocket: Release of basic Local Developer Testnet

Can deploy & execute isolated, private-function-only contracts.

---

## 2: L1-L2 Communication

### 2.1 - Deploy a Portal Contract

As a developer, I can write a Portal Contract to accompany my Noir++ contract.

I can deploy the Portal Contract to Ethereum, and then deploy and Aztec L2 contract which links with the Portal Contract.

I can make a query to the Private Client to learn the Portal Contract address of an L2 contract.

I can make a query to L1 to learn the L2 address for a particular Portal Contract.

Components: l1-contracts, kernel-circuit, aztec-rpc,

---

### 2.2 - L1->L2 Calls

As a developer, I can write Solidity code in my Portal Contract which can 'send messages' to a particular function of the linked L2 contract.

I can write Noir code which can 'read' messages from the L1->L2 message box.

I can send an Ethereum tx which calls a function of the Portal Contract, which in-turn sends an L1->L2 message.

I can then send an L2 tx which calls an L2 function, which 'reads' the message.

Specialisms: Noir, client, contracts, RPC

---

### 2.3 - L2->L1 Calls

As a developer, I can "`import`" the functions of the Portal Contract, so that my Noir++ code can make L1 function calls.

I can type `aztec3-cli send_tx contract_address function_signature arg1 arg2` and have a transaction which calls a private or public function, which itself then makes an L1 call.

I can type `aztec3-cli get_transaction_receipt tx_hash` and view the status of my transaction on the local network.

Q: We might need to submit an encryption of L2->L1 messages to the `unverifiedData` on-chain (so that a user can re-sync and still find the messages they sent!)

Specialisms: Noir, client, contracts, RPC

---

### :rocket: New release of local developer testnet

Can deploy & execute isolated, private-function-only contracts, which can interact with L1 for public state changes.

---

## 3: Private call stacks

### 3.1 - Inter-contract Private -> Private Calls

(Note: intRA private->private calls can just be inlined).

As a developer, I can deploy 2 Noir++ contracts. One contract can `import` the interface of the other.

A `private` function of the importing contract can call a `private` function of the imported contract, passing arguments, and receive return values.

I can type `aztec3-cli send_tx contract_address function_signature arg1 arg2` and have a transaction which calls the function described.

I can type `aztec3-cli get_transaction_receipt tx_hash` and view the status of my transaction on the local network.

TODO: pause the tree updates.

Specialisms: client, Noir, RPC

### 3.2 - Recursive private calls

(A private function calling itself)

Specialisms: client, Noir, RPC

---

:rocket: New release of local developer testnet

---

## 4: Public Functions

**NOTE: the circuits team will have a huge chunk of work for this milestone; implementing a VM within a circuit. So they'll likely need to work separately, behind an interface, for many sprints. The rest of the engineering stack can stub out the public VM**.

### 4.1 - Public Constructor

As a developer, I can declare `public` state variablesin the `contract` scope of my Noir++ contract.

I can write a `public` function in Noir, which can push new `state_transitions` to the public data tree.

I can write a `constructor` function in Noir, which can _make a call_ to a `public` function _in the same_ `contract` scope, which can be used to initialise `public` state.

I can type `aztec3-cli deploy_contract path/to/abi.json` to deploy a contract which calls a constructor function (which in-turn calls a public function) to initialise _public_ state.

> Note: the constructor arguments could be included in the abi.json, similar to a Solidity ABI.

I can type `aztec3-cli get_storage_at contract_address storage_slot` and verify the state was set correctly for public state.

Specialisms: Noir, client, RPC

---

### 4.2 - A single Public Function Call

As a developer, I can write a `public` function in Noir. This function can do public `state_reads`, and push new `state_transitions` to the public data tree.

I can type `aztec3-cli send_tx contract_address function_signature arg1 arg2` and have a transaction which calls a single public function, to be included in the next block.

I can type `aztec3-cli get_transaction_receipt tx_hash` and view the status of my transaction on the local network.

Specialisms: Noir, client, RPC

---

:rocket: New release of local developer testnet

---

## 5: More Composability

### 5.1 - IntRA-contract Private->Public Calls

A `private` function can call a `public` function of the same contract, passing arguments, but NOT receiving return values.

I can type `aztec3-cli send_tx contract_address function_signature arg1 arg2` and have a transaction which calls the function described.

I can type `aztec3-cli get_transaction_receipt tx_hash` and view the status of my transaction on the local network.

Specialisms: client, Noir, RPC

---

### 5.2 - IntER-contract Private->Public Calls

A `private` function of the importing contract can call a `public` function of the imported contract, passing arguments, but NOT receiving return values.

I can type `aztec3-cli send_tx contract_address function_signature arg1 arg2` and have a transaction which calls the function described.

I can type `aztec3-cli get_transaction_receipt tx_hash` and view the status of my transaction on the local network.

Specialisms: client, Noir, RPC

---

### 5.3 - IntER-contract Public->Public Calls

A `public` function of the importing contract can call a `public` function of the imported contract, passing arguments, and receiving return values.

I can type `aztec3-cli send_tx contract_address function_signature arg1 arg2` and have a transaction which calls the function described.

I can type `aztec3-cli get_transaction_receipt tx_hash` and view the status of my transaction on the local network.

Specialisms: client, Noir, RPC

---

### 5.4 - IntRA-contract Public->Private calls

(Note: intRA private->private calls can just be inlined).

As a developer, I can write Noir++ code to call a `private` function from a `public` function; within the same contract scope.

I can write a `private` function which can 'read' a message from a `public` function.

I can call a public function, which adds a message to some message box, for a particular private function to consume.

I can call a private function (in a later rollup) which consumes that message.

Specialisms: client, Noir, RPC

---

:rocket: Specialisms: everything

---

## 6: Introducing fees

**NOTE: these milestones might change, as we think more about fees, and the best things to tackle first.**

### 6.1 - Estimating Gas - L1->L2 message

As a developer, I can estimate the L2 gas costs associated with posting a message to the L1->L2 message box.

> Note: This might need further discussion. The L1 component is forcing the Sequencer to add data to the message tree in the next rollup, so ought to cover that L2 cost somehow by providing a payment to the Sequencer (rather than the L1 validator). Tricky.

Specialisms: client, RPC

---

### 6.2 - Estimating Gas - Private Kernel

As a developer, I can estimate the L2 gas costs associated with a private kernel snark's submission to the local test blockchain.

Specialisms: client, RPC

---

### 6.3 - Estimating Gas - Public Function

As a developer, I can estimate the L2 gas costs associated with executing a public circuit.

Specialisms: client, RPC

---

### 6.4 - Fees from L1

As a developer, I must now pay for the 'L2 component' of an L1->L2 tx.

I can pay for the 'L2 component' of an L1->L2 tx using L1 ETH.

I can pay for the 'L2 component' of an L1->L2 tx using any ERC20 token.

Specialisms: client, contracts, RPC

---

### 6.5 - Fees from Public L2

As a developer, I can write a public L2 token contract.

As a user, I can pay for L2 txs using some public L2 token.

Specialisms: client, RPC

---

### 6.6 - Fees from Private L2

As a developer, I can write a private L2 token contract.

As a user, I can pay for L2 txs using some private L2 token.

Specialisms: client, RPC

---

### :rocket: Launch local developer testnet WITH GAS & FEES (a la Ganache)

Specialisms: everything

---

## 7: Public testnet via a centralised Sequencer

> These aren't currently written as developer stories, as the developer's experience shouldn't change much from interacting with the local network. Instead, they're mostly Sequencer stories here.

### 7.1 - A centralised tx pool

As a Sequencer, I should be able to see all incoming tx requests in a 'stubbed' tx pool (can just be an http endpoint, perhaps).

Specialisms: sequencer, RPC

---

### 7.2 - Rollups done by a Sequencer

As a Sequencer, I can rollup a set of private kernel proofs into a single proof, and submit that to L1.

Specialisms: client, sequencer, circuit architecture, RPC

---

### 7.3 - Simulating and submitting a public function

As a sequencer, I can simulate a public function's opcodes.

I can then prove execution of that function, run it through a kernel snark, and add it to the rollup.

Specialisms: sequencer, circuit architecture

---

### 7.4 - Collecting L1->L2 message fees

As a Sequencer, I can collect a fee for adding an L1->L2 message to the message tree.

Specialisms: sequencer, contracts, circuit architecture

---

### 7.5 - Collecting L1 fees

As a Sequencer, I can identify when an L2 tx is being paid-for via L1 (e.g. in the case of the L2 tx component of an L1->L2 tx).

I can see the L1 fees being offered for such a tx.

I can convert the L1 currency into ETH.

I can simulate a single-function L2 tx.

I can process the L2 tx, and receive L1 tokens for doing so.

Specialisms: sequencer, contracts, circuit architecture

---

### 7.6 - Collecting Public L2 fees

As a Sequencer, I can identify when an L2 tx is being paid-for via a public L2 tx.

I can see the L2 fees being offered for such a tx.

I can interpret those fees, based on some Aztec L2 Public fungible token standard.

I can simulate the fee-paying tx, to validate that I'll be paid.

I can simulate the accompanying tx.

I can process the L2 tx, and receive public L2 tokens from the fee-paying tx.

Specialisms: sequencer, contracts, circuit architecture

---

### 7.7 - Collecting Private L2 fees

As a Sequencer, I can identify when an L2 tx is being paid-for via a private L2 tx.

I can see the L2 fees being offered for such a tx.

I can interpret those fees, based on some Aztec L2 Private fungible token standard.

I can simulate the fee-paying tx, to validate that I'll be paid.

I can simulate the accompanying tx.

I can process the L2 tx, and receive public L2 tokens from the fee-paying tx.

Specialisms: sequencer, contracts, circuit architecture

---

### 7.8 - Prover client

As a Sequencer, I can delegate proof generation to my own Prover Client (descendant of Halloumi).

Specialisms: sequencer, prover, circuit architecture

---

### 7.9 - Actual proof generation!

We actually construct circuits and generate zk-snarks at all stages.

Specialisms: sequencer, prover, circuit architecture, cryptographer

---

### :rocket: Launch centralised sequencer testnet

Specialisms: everything

---

## 8: 1st Sequencer testnet

### 8.0 - More Sequencers

> Note: depending on Honk progress, this might be UltraPlonk initially.

As a Sequencer (amongst many Sequencers), I can query whether I'm the current Sequencer.

> Note: this can be coordinated via some central endpoint at this stage.

I can access a centralised pool of txs, in order to generate rollups when it's my turn.

---

### 8.1 - P2P tx pool

As a user, I can connect to a p2p tx pool.

As a user, I can submit a tx to the pool.

As a Sequencer, I can read txs from the tx pool.

As a network participant, my local copy of the tx pool can be maintained: adding new txs, and removing already-processed txs.

### :rocket: Launch 1st Sequencer testnet

---

## 9: 2nd Sequencer testnet

### 9.0 - Sequencer Selection Protocol

As a Sequencer, I can determine whether I'm the Sequencer in a decentralised way.

I'll know some time in advance that I'll be the sequencer for a particular rollup, so I can prepare in advance.

I can submit a rollup only when I'm the chosen Sequencer.

If I fail to submit a rollup I might (TBD) be penalised.

> Other milestones are hazy, until we decide on the sequencer selection protocol.

### :rocket: Launch 2nd Sequencer testnet

---

## 10: Prover testnet

We introduce a new type of network participant: a Prover: someone other than the Sequencer who runs a Prover Client.

### 10.0 - Prover Selection Protocol

> TBD: selection criteria.

### 10.1 - Proof delegation

As a Sequencer I can delegate proofs to Provers.

As a Prover I can be paid for generating proofs for the current Sequencer.

### :rocket: Launch Prover testnet

---

## 11: Refactoring / Optimisations

- Nullifier Epochs
- Commitment (UTXO) Epochs
- More Efficient Kernel Recursion Topology (in a binary tree)
- Flexible Rollup Topology / Streaming new txs into the rollup
- Decrypting Notes
  - OMR? PIR? FMD?
- Plug Honk into the circuits
- Account Abstraction
- More efficient, newer hashes for trees

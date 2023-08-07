---
title: Milestones Overview
---

The milestones are written as sets of user stories to make it clear what functionality will be available as we work through each one.

## Development Strategy

The goal is that all software components of Aztec can be developed as independent modules (e.g. private client, public client, simulators, data sources/sinks, circuit architecture, RPC, cryptography).

When modules need to communicate across module boundaries, they do so via clearly defined APIs.

When developing a module, these external API calls can be mocked. Each module's suite of unit tests ideally are completely independent of other module implementations (i.e. all external API calls are mocked).

The goal is to enable parallel development of these milestones, where teams are not bottlenecked by the progress of other teams.

For a milestone to be complete, full integration tests must be implemented that interact across module boundaries.

---

### 1.0 - Repo setup ✅

Separate Barretenberg into its own repository.

Create a monorepo (`aztec3-packages`) for the Local Developer Testnet codebase.

---

### 1.1 - Deploy a contract ✅

See granular tasks [here](./milestone1_1).

As a developer, I can write a Noir `contract` scope, which contains a collection of pure functions (circuits) and a noop `constructor` function (no state variables yet).

I can compile my contract with Noir, and receive a contract ABI JSON.

I can deploy my new Noir Contract via `aztec.js`.

I can verify, via `aztec.js`, that my Noir Contract has been deployed successfully to the Local Developer Testnet.

---

### 1.2 - Nullifier functionality ✅

We need nullifier infrastructure, and it's complex enough to be its own milestone. It makes most sense to develop it at this stage. Here's an attempt to shoe-horn it into a user story:

As a developer, I can only deploy to an Aztec contract address once. Attempts to duplicate a contract address will be rejected by the network.

Each tx request contains a nonce. For private functions, such a nonce will be emitted as a nullifier (to prevent re-use of that nonce).

---

### 1.3 - Private Constructor ✅

As a developer, I can declare private state variables in the global `contract` scope of my Noir Contract.

I can write a `constructor` function in Noir, which initialises private states within my contract (i.e. which pushes new commitments to the network). For this milestone, these private states will only be owned by the deployer.

I can deploy my new contract, which calls a constructor function to initialise private state.

> Note: the constructor arguments could be included in the abi.json, similar to a Solidity ABI.

I can verify, via `aztec.js`, that my private state(s) were initialised correctly. (Note: obviously a user can only do this if they own the state at that storage slot).

---

### 1.4 - A single Private Function Call ✅

As a developer, I can write a `secret` function in Noir. This function can read private states, modify (nullify) private states, and store updates to private states.

I can deploy a contract containing such functions to the network.

As a user, I can create a tx request to execute any external, private function (by name, via `aztec.js`). I can generate a zk-proof of having executed the private function, then a private kernel proof that the execution adheres to network rules, and send that kernel proof to the network. In the Local Developer Testnet, that tx will be included in the next block. The private function can initially only create commitments owned by the caller.

I can view the status of my transaction on the local network, via `aztec.js`.

---

### 1.5 - A single Private Function Call (editing others' states) ⚙️

As per the previous milestone, but with the ability to call a function which edits other people's private states.

As a developer, I can write `unconstrained` functions in Noir, which allow me to query Aztec state variables without broadcasting a tx to the network (much like the `getBalance()` function of an ERC20 contract).

As a user, I can share the preimages of private state edits with the owners of those private states (e.g. by broadcasting the encrypted preimage on-chain).

As a recipient of an edited state, my RPC Client can update it's DB with this state update (e.g. via trial-decryption).

As a recipient of an edited state, I can query the current value of my state variable(s) via `unconstrained` function(s), to verify that they've been edited.

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

As a developer, I can deploy a 'Portal Contract' to Ethereum (L1), which allows me to write logic across L1 and L2 for my dapp.

I can write Solidity code in my Portal Contract which can 'send messages' to a particular function of the linked L2 contract.

I can write Noir code which can 'read' messages from the L1->L2 message box.

I can send an Ethereum tx which calls a function of the Portal Contract, which in-turn sends an L1->L2 message.

I can then send an L2 tx which calls an L2 function, which 'reads' (consumes) the message.

---

### 2.3 - L2->L1 Calls

As a developer, I can write Noir code (in my Noir Contract) which can 'send messages' to my Noir Contract's corresponding Portal Contract (on L1).

I can send an Aztec tx which calls a private (or public) function, which can send the intended message to L1, via the Rollup Contract.

I can then send an Ethereum tx to my Portal Contract (or some other contract), which can 'read' (consume) the message.

> Note: We might need to submit an encryption of L2->L1 messages to the `unverifiedData` on-chain (so that a user can re-sync and still find the messages they sent!)

---

## 3: Private call stacks

### 3.1 - Inter-contract Private -> Private Calls ⚙️

> Note: intRA private->private calls can be inlined, in most cases.

As a developer, I can deploy 2 Noir Contracts. One contract can `import` the interface of the other.

A `private` function of the importing contract can call a `private` function of the imported contract, passing arguments, and receive return values.

As a user, I send a tx which calls the function described.

I can view the status of my transaction on the local network.

> Note: both functions which are executed in the one tx will need to use the same snapshot of the network's state, so pause/ignore tree updates whilst the tx is being generated.

### 3.2 - Recursive private calls

As a developer, I can write a private function which calls itself.

> Note: this requires changes to the private kernel circuit, to allow notes to be 'read' even before they've been finalised on L1.

---

:rocket: First release candidate for the local developer testnet

---

## 4: Public Functions

### 4.1 - Public Constructor

As a developer, I can declare `public` state variablesin the global `contract` scope of my Noir Contract.

I can write a `public` function in Noir, which can read and write state from/to the public data tree.

I can write a `constructor` function in Noir, which can _make a call_ to a `public` function _in the same_ `contract` scope, which can be used to initialise `public` state.

I can deploy a contract (via `aztec.js`) which calls a constructor function (which in-turn calls a public function) to initialise _public_ state.

> Note: the constructor arguments could be included in the abi.json, similar to a Solidity ABI.

I can call and `unconstrained` function to verify that the state was set correctly for public state.

---

### 4.2 - A single Public Function Call ⚙️

As a developer, I can write a `public` function in Noir. This function can do public `state_reads`, and push new `state_transitions` to the public data tree.

I can send a tx request to the network, which calls a single public function, to be included in the next block.

I can view the status of my transaction on the local network.

---

:rocket: New release of the local developer testnet

---

## 5: More Composability

### 5.1 - IntRA-contract Private->Public Calls

A `private` function can call a `public` function of the same contract, passing arguments, but NOT receiving return values.

---

### 5.2 - IntER-contract Private->Public Calls

A `private` function of the importing contract can call a `public` function of the imported contract, passing arguments, but NOT receiving return values.

---

### 5.3 - IntER-contract Public->Public Calls

A `public` function of the importing contract can call a `public` function of the imported contract, passing arguments, and receiving return values.

---

### 5.4 - IntRA-contract Public->Private calls

(Note: intRA private->private calls can just be inlined).

As a developer, I can write Noir++ code to call a `private` function from a `public` function; within the same contract scope.

I can write a `private` function which can 'read' a message from a `public` function.

I can call a public function, which adds a message to some message box, for a particular private function to consume.

I can call a private function (in a later rollup) which consumes that message.

---

:rocket: New release of the local developer testnet

---

## 6: Introducing fees

**NOTE: these milestones might change, as we think more about fees, and the best things to tackle first.**

### 6.1 - Estimating Gas - L1->L2 message

As a developer, I can estimate the L2 gas costs associated with posting a message to the L1->L2 message box.

> Note: This might need further discussion. The L1 component is forcing the Sequencer to add data to the message tree in the next rollup, so ought to cover that L2 cost somehow by providing a payment to the Sequencer (rather than the L1 validator). Tricky.

---

### 6.2 - Estimating Gas - Private Kernel

As a developer, I can estimate the L2 gas costs associated with a private kernel snark's submission to the local test blockchain.

---

### 6.3 - Estimating Gas - Public Function

As a developer, I can estimate the L2 gas costs associated with executing a public circuit.

---

### 6.4 - Fees from L1

As a developer, I must now pay for the 'L2 component' of an L1->L2 tx.

I can pay for the 'L2 component' of an L1->L2 tx using L1 ETH.

I can pay for the 'L2 component' of an L1->L2 tx using any ERC20 token.

---

### 6.5 - Fees from Public L2

As a developer, I can write a public L2 token contract.

As a user, I can pay for L2 txs using some public L2 token.

---

### 6.6 - Fees from Private L2

As a developer, I can write a private L2 token contract.

As a user, I can pay for L2 txs using some private L2 token.

---

:rocket: New release of the local developer testnet

---

## 7: Introduce actual circuits, proofs and verifiers

Up until now, milestones will have been using "simulated circuits", which contain the logic (checks and calculations) we'll need from our circuits, but without any of the computational overhead of actual circuits.
This decision was intentional, to make the Local Developer Testnet as fast as possible, so that users (devs) can play and iterate as quickly as possible.

But ultimately, we need all these transactions to be actual proofs which can be verified. We have most of the code and expertise (from Aztec Connect), we just need to plug it in!

### 7.1 Actual Kernel & Rollup circuits

Write circuit versions of the Private and Public Kernel circuits, and the Base, Merge and Root Rollup circuits.

Turn on proof validation within each of these circuits, and within the Rollup Contract on L1.

The proving scheme might initially be UltraPlonk, which might be a problem in WASM. Benchmarking needed.

### 7.2 Swap UltraPlonk for Honk

Once ready, we can swap-in the Honk proving scheme, for much faster recursive proofs.

### 7.3 Public VM Circuit

The opcode-trace of a public function will ultimately need to be verified via a Public VM Circuit. This will be quite a complicated circuit to design and build.

### 7.4 Public Function bytecode-validation circuit

The opcodes of a public function need to be provably 'linked' with a commitment to those opcodes.

## 8: Public testnet via a centralised Sequencer

> These aren't currently written as developer stories, as the developer's experience shouldn't change much from interacting with the local network. Instead, they're mostly Sequencer stories here.

### 8.1 - A centralised tx pool

As a Sequencer, I should be able to see all incoming tx requests in a 'stubbed' tx pool (can just be an http endpoint, perhaps).

---

### 8.2 - Rollups done by a Sequencer

As a Sequencer, I can rollup a set of private kernel proofs into a single proof, and submit that to L1.

---

### 8.3 - Simulating and submitting a public function

As a sequencer, I can simulate a public function's opcodes.

I can then prove execution of that function, run it through a kernel snark, and add it to the rollup.

---

### 8.4 - Collecting L1->L2 message fees

As a Sequencer, I can collect a fee for adding an L1->L2 message to the message tree.

---

### 8.5 - Collecting L1 fees

As a Sequencer, I can identify when an L2 tx is being paid-for via L1 (e.g. in the case of the L2 tx component of an L1->L2 tx).

I can see the L1 fees being offered for such a tx.

I can convert the L1 currency into ETH.

I can simulate a single-function L2 tx.

I can process the L2 tx, and receive L1 tokens for doing so.

---

### 8.6 - Collecting Public L2 fees

As a Sequencer, I can identify when an L2 tx is being paid-for via a public L2 tx.

I can see the L2 fees being offered for such a tx.

I can interpret those fees, based on some Aztec L2 Public fungible token standard.

I can simulate the fee-paying tx, to validate that I'll be paid.

I can simulate the accompanying tx.

I can process the L2 tx, and receive public L2 tokens from the fee-paying tx.

---

### 8.8 - Collecting Private L2 fees

As a Sequencer, I can identify when an L2 tx is being paid-for via a private L2 tx.

I can see the L2 fees being offered for such a tx.

I can interpret those fees, based on some Aztec L2 Private fungible token standard.

I can simulate the fee-paying tx, to validate that I'll be paid.

I can simulate the accompanying tx.

I can process the L2 tx, and receive public L2 tokens from the fee-paying tx.

---

### 8.8 - Prover client

As a Sequencer, I can delegate proof generation to my own Prover Client (descendant of Halloumi).

---

### 8.9 - Actual proof generation!

We actually construct circuits and generate zk-snarks at all stages.

---

:rocket: Launch centralised sequencer testnet

---

## 9: 1st Sequencer testnet

### 9.0 - More Sequencers

> Note: depending on Honk progress, this might be UltraPlonk initially.

As a Sequencer (amongst many Sequencers), I can query whether I'm the current Sequencer.

> Note: this can be coordinated via some central endpoint at this stage.

I can access a centralised pool of txs, in order to generate rollups when it's my turn.

---

### 9.1 - P2P tx pool

As a user, I can connect to a p2p tx pool.

As a user, I can submit a tx to the pool.

As a Sequencer, I can read txs from the tx pool.

As a network participant, my local copy of the tx pool can be maintained: adding new txs, and removing already-processed txs.

---

:rocket: Launch 1st Sequencer testnet

---

## 10: 2nd Sequencer testnet

### 10.0 - Sequencer Selection Protocol

As a Sequencer, I can determine whether I'm the Sequencer in a decentralised way.

I'll know some time in advance that I'll be the sequencer for a particular rollup, so I can prepare in advance.

I can submit a rollup only when I'm the chosen Sequencer.

If I fail to submit a rollup I might (TBD) be penalised.

> Other milestones are hazy, until we decide on the sequencer selection protocol.

---

:rocket: Launch 2nd Sequencer testnet

---

## 11: Prover testnet

We introduce a new type of network participant: a Prover: someone other than the Sequencer who runs a Prover Client.

### 11.0 - Prover Selection Protocol

> TBD: selection criteria.

### 11.1 - Proof delegation

As a Sequencer I can delegate proofs to Provers.

As a Prover I can be paid for generating proofs for the current Sequencer.

---

:rocket: Launch Prover testnet

---

## 12: Refactoring / Optimisations

- Nullifier Epochs
- Commitment (UTXO) Epochs
- More Efficient Kernel Recursion Topology (in a binary tree)
- Flexible Rollup Topology / Streaming new txs into the rollup
- Decrypting Notes
  - OMR? PIR? FMD?
- Plug Honk into the circuits
- Account Abstraction
- More efficient, newer hashes for trees

## Participate

Keep up with the latest discussion and join the conversation in the [Aztec forum](https://discourse.aztec.network).

import Disclaimer from "../../misc/common/\_disclaimer.mdx";
<Disclaimer/>

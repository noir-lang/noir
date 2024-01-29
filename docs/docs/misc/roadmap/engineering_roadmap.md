# Engineering Wishlist

The engineering roadmap is long. There are no timings assigned here. In a loose priority order:

## Sandbox Community Support

- Triage on discord / discourse / github.
- Iterating on the docs and code, given people's issues.
- Encouraging contributions to 'good first issue' issues.
- Release notes.
- Versioning.
- Aztec Improvement Proposals (AZIPs)
- Aztec Requests for Comment (AZRCs)

## Benchmarking

- Gather metrics about everything, to guide future decisions.

## Standardization efforts

- Recommended Aztec smart contract coding patterns
- Access Control (whitelists/blacklists) - probably needs the Slow Updates tree (or something similar).
- Basic _example_ private tokens
    - Including recursive calls to 'get_notes'.
- Compliant private tokens
- Private NFTs
- Public tokens
- Depositing and withdrawing tokens
    - L1<\>L2
    - public<\>private
- The Aztec Connect bridging pattern
- Using Keys (the fully-featured version of keys that we want to build)
- Plume nullifiers
- Negative reputation example
- Anti-denial-of-service
- Combining Aztec with MPC

## Polishing what we have

- Refactoring sprints.
    - Reduce tech debt.
    - More tests.

## Enforcing correct ordering Public & Private State Transitions

## Enforcing correct ordering of other 'side effects'
- Log ordering
- Enqueued public function calls

## What data actually needs to be submitted on-chain?
- For Public Functions:
    - Just emit the initially-enqueued public function request data? (The 'inputs' of the tx);
        - I.e. contract address, function selector, args, call_context.
    - OR, Just emit the final state transitions? (The 'outputs' of the tx)
        - I.e. the leaf indices and new values of the public data tree; and the new commitments/nullifiers of the note hash tree; and logs; and l2->L1 messages.

## Proper specs

- Write detailed specs, given recent protocol changes.
- Review the code to ensure it matches what we _think_ the protocol is.
- Open issues to ensure the code matches the spec.
- (Note: bringing cryptographers into the fold (to review specs) is a separate section, later in this doc).

## Iterate on the Sandbox

Based on community feedback, we'll need some teams to iterate on Sandbox features and fix bugs.

## Iterate on the Aztec Smart Contract Library

## Iterating on CI

CI takes up a significant amount of time. It gets its own section here, so we remember it exists.

## Sequencer & Prover Selection protocols

- Decide on protocol
- Spec
- Build.

## Upgradeability

- Decide on protocol
- Spec
- Build.

## Fees

- Design the Protocol
    - Interdependence on the Sequencer & Upgradeability protocols.
    - Pay fees in 1 currency, or fee abstraction?
    - Escrowing fees
    - Rebates
    - L1->L2 message fees.
    - L2->L1 fees
    - Etc.
- Build it.
    - Gas metering
    - Etc.

## Note Discovery

- Note Discovery RFP
- Decide on the protocol
- Spec
- Build it.

## Privacy-preserving queries to public nodes

- Explore PIR
- Explore alternatives
- Implement

## Keys

- Write up keys spec
- Get internal comments
- Do a RFC from the external community
- Implement

## Slow Updates tree?

We _need_ a way to read mutable public data from a private function. 

Note: we just published the [Slow Updates Tree](../../learn/concepts/communication/public_private_calls/slow_updates_tree.md).

## Contract classes and instances?

- There's a suggestion to introduce contract classes.

## Delegatecalls vs SetCode
- Which? (if at all)

## Remove the contracts tree? 🤯

- There's a suggestion to remove the notion of a contracts tree. What do we actually need the tree for? To check that a vk hash exists for a particular contract address?
- If the contract address contains the function tree, and it also contains data about the constructor args to that contract, then perhaps we don't need the contract tree to exist.
- We might still need the notion of a 'deployment':
    -  to broadcast a contract's bytecode;
    -  to 'reserve' a contract address;
    -  and possibly to prevent a constructor from being executed twice (although an app could do this ("constructor abstraction")).

## Cryptography review checkpoint

- Once we have specs (see above), we should review the rigour and secureness to our protocol.
    - Choice of hashes
    - Domain separation
    - Choice of encryption scheme
    - Keys
    - A security review of the protocol as a whole

## Testing UX team

A team focussed on testing and debugging UX.
This team should have free rein to design and add any features they see fit.

Some example features:
- Writing contract tests in Noir.
    - Mocking oracles.
- Taking inspiration from other testing frameworks.
- Much more detailed logging if things go wrong.
- Errors which only spit out opaque 254-bit hex numbers are bad.
    - Ensure all circuits are fed the human-readable information underlying all of these hex numbers.
    - If tree root comparisons (expected vs actual) fail, human-readable details about the leaves in trees should be provided.

## Tooling

## Proper Circuits

### Redesign
- The Bus
    - The bus will have an impact on the way we design the circuit logic.
    - We can hopefully avoid hashing in circuit 1 and unpacking that hash in circuit 2.
    - Understand the 'bus' and how we can use it to pass variable amounts of data between recursive circuit iterations.
    - Redesign all circuit logic to allow for the variable-sized arrays that the 'bus' enables.
- Enable 'dynamic/variable-sized **loops**'
    - allow each `for` loop (eg read requests, insertions, commitment squashing, call stack processing, bip32 key derivation, etc.) to vary in size, by deferring each loop to its own final circuit. This would require complex folding stuff.
    - This would give much more flexibility over the sizes of various arrays that a circuit can output. Without it, if one array of an app circuit needs to be size 2000, but other arrays aren't used, we'd use a kernel where every array is size 2048, meaning a huge amount of unnecessary loops of computation for those empty arrays.
- Improvements
    - We can definitely change how call stacks are processed within a kernel, to reduce hashing.
    - Squash pending commitments/nullifiers in every kernel iteration, to enable a deeper nested call depth.
- Topology of a rollup
    - Revisit the current topology:
        - We can make the rollup trees 'wonky' (rather than balanced), meaning a sequencer doesn't need to prove a load of pointless 'padding' proofs?
            - This would also enable new txs (entering the tx pool) to be added to a rollup block 'on-the-fly' (mid way through a rollup being proven) - but of course, the sequencer selection protocol might require an up-front commitment, so this might not be possible for that reason (sad face).
        - We can definitely redesign the public kernel circuit to be a '2x2' topology (i.e. a tree of public kernel proofs), to get a logarithmic speed-up (parallelism). The question is, with folding schemes, do we need that optimization?

#### Refactor of packing & unpacking data in circuits

We often pack data in circuit A, and then unpack it again in circuit B.
- args_hash
- return_values_hash
- call stacks
- read requests
- etc.

Also, for logs in particular, we allow arbitrary-sized logs. But this requires sha256 packing inside an app circuit (which is slow) (and sha256 unpacking in Solidity (which is relatively cheap)). Perhaps we also use the bus ideas for logs, to give _some_ variability in log length, but up to an upper bound. 

Also, we do a lot of sha256-compressing in our kernel and rollup circuits for data which must be checked on-chain, but grows exponentially with every round of iteration. E.g.: new contract deployment data, new nullifiers, new commitments, public state transition data, etc. This might be unavoidable. Maybe all we can do is use polynomial commitments when the EIP-4844 work is done. But maybe we can use the bus for this stuff too.

### Write proper circuits

### The Public VM Circuit

- Design it
- Build it

### The Brillig Bytecode Commitment Circuit

- A circuit which proves the brillig bytecode being emitted matches the polynomial commitment to that bytecode.

### Decide on constants!

### Honk -> Ultra Squisher Circuit

### Ultra -> Standard Squisher Circuit

## Authentication: access to private data
- Private data must not be returned to an app, unless the user authorizes it.

## Validation: preventing execution of malicious bytecode
- A node should check that the bytecode provided by an application for a given app matches the leaf in the contract tree to ensure that user doesn't execute unplanned code which might access their notes.

## Fuzz Testing

## Formal Verification

An investigation into how formal verification techniques might improve the security of Aztec software.

## P2P network

- A robust p2p network for the tx pool and the proof pool.

## Hashes

- An improved, standardized Pedersen hash in barretenberg.
- Poseidon hashing in barretenberg.

## Tree epochs
- Nullifier tree epochs
- Maybe other tree epochs.

## Chaining txs
- We have the ability to spend pending notes (which haven't-yet been added to the tree) _within the context of a single tx_.
- We need the ability to spend pending notes (which haven't yet been added to the tree) across different txs, within the context of a single rollup.
    - This happens if Alice generates two txs X & Y, where tx Y spends notes from tx X. If we want Alice to be able to generate these txs in parallel (without interacting with the network at all), we need a way for tx Y to spend notes before they've been added to the tree. The 'chaining tx' concepts from Aztec Connect can enable this.
- This added _a lot_ of complexity to Aztec Connect. Especially around fees, iirc. Caution needed.

## EIP-4844

- Understand it. Spec it. Build it.
- Includes:
    - Smart Contract changes
    - Circuit changes
    - A circuit to prove equivalence vs a BLS12-381 polynomial commitment.

## Make it all work in a browser

## Code Freeze

## Internal Audit

## External Audits






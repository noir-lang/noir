# Honk targets and win conditions

## Introduction & context

Aztec's cryptography tech stack and its associated implementation is an open-ended project with potential for many enhancements, optimisations and scope-creep.

This document is designed to definitively answer the following questions:

1. What are the metrics we care about when measuring our cryptography components?
1. What are minimum satisfiable values for these metrics?
1. What are the aspirational values for these metrics?

# Important Metrics

The following is a list of the relevant properties that affect the performance of the Aztec network:

- Size of a user transaction (in kb)
- Time to generate a user transaction proof
- Memory required to generate a user transaction proof
- Time to generate an Aztec Virtual Machine proof
- Memory required to generate an Aztec Virtual Machine proof
- Time to compute a 2-to-1 rollup proof
- Memory required to compute a 2-to-1 rollup proof

"MVP" = minimum standards that we can go to main-net with.

Note: gb = gigabytes (not gigabits, gigibits or gigibytes)

| metric | how to measure | MVP (10tps) | ideal (100tps) |
| --- | --- | --- | --- |
| proof size | total size of a user tx incl. goblin plonk proofs | 80kb | 8kb |
| prover time | A baseline "medium complexity" transaction (in web browser). Full description further down | 1 min | 10 seconds |
| verifier time | how long does it take the verifier to check a proof (incl. grumpkin IPA MSMs) | 20ms | 1ms |
| client memory consumption | fold 2^19 circuits into an accumulator an arbitrary number of times | 4gb | 1gb |
| size of the kernel circuit | number of gates | 2^17 | 2^15 |
| Aztec Virtual Machine prover time | 10,000 step VM circuit | 15 seconds | 1.5 seconds |
| Aztec Virtual Machine memory consumption | 1 million VM step circuit | 128gb | 16gb |
| 2-to-1 rollup proving time | 1 2-to-1 rollup proof | 7.4 seconds | 0.74 seconds |
| 2-to-1 rollup memory consumption | 1 2-to-1 rollup proof | 128gb | 16gb |

To come up with the above estimates, we are targeting 10 transactions per second for the MVP and 100 tps for the "ideal" case. We are assuming both block producers and rollup Provers have access to 128-core machines with 128gb of RAM. Additionally, we assume that the various process required to produce a block consume the following:

<!-- prettier-ignore -->
| process | percent of block production time allocated to process |
| --- | --- |
| transaction validation | 10% |
| block building (tx simulation) | 20% |
| public VM proof construction time | 20% |
| rollup prover time | 40% |
| UltraPlonk proof compression time | 10% |

These are very rough estimates that could use further evaluation and validation!

### Proof size

The MVP wishes to target a tx throughput of 10 txs per second.

Each Aztec node (not sequencer/prover, just a regular node that is sending transactions) needs to download `10*proof_size` bytes of data to keep track of the mempool. However, this is the _best case_ scenario.

More practically, the data throughput of a p2p network will be less than the bandwidth of participants due to network coordination costs.
As a rough heuristic, we assume that network bandwidth will be 10% of p2p user bandwidth.
NOTE: can we find some high-quality information about p2p network throughput relative to the data consumed by p2p node operators?

As a result, the MVP data throughput could scale up to `100 * proof_size` bytes of data per second.

For an MVP we wish to target a maximum bandwidth of 8MB per second (i.e. a good broadband connection). This gives us a network bandwidth of 0.8MB/s.

This sets the proof size limit to 819.2 kb per second per 100 transactions => 82 kilobytes of data per transaction.

As a rough estimate, we can assume the non-proof tx data will be irrelevant compared to 82kb, so we target a proof size of $80$ kilobytes for the MVV.

To support 100 transactions per second we would require a proof size of $8$ kilobytes.

### Prover time

The critical UX factor. To measure prover time for a transaction, we must first define a baseline transaction we wish to measure and the execution environment of the Prover.

As we build+refine our MVP, we want to avoid optimising the best-case scenario (i.e. the most basic tx type, a token transfer). Instead we want to ensure that transactions of a "moderate" complexity are possible with consumer hardware.

As a north star, we consider a private swap, and transpose it into an Aztec contract.

To perform a private swap, the following must occur:

1. Validate the user's account contract (1 kernel call)
2. Call a swap contract (1 kernel call)
3. The swap contract will initiate `transfer` calls on two token contracts (2 kernel calls)
4. A fee must be paid via our fee abstraction spec (1 kernel call)
5. A final "cleanup" proof is generated that evaluates state reads and processes the queues that have been constructed by previous kernel circuits (1 kernel call + 1 function call; the cleanup proof)

In total we have 6 kernel calls and 6 function calls.

We can further abstract the above by making the following assumption:

1. The kernel circuit is $2^{17}$ constraints
2. The average number of constraints per function call is $2^{17}$ constraints, but the first function called has $2^{19}$ constraints

Defining the first function to cost $2^{19}$ constraints is a conservative assumption due to the fact that the kernel circuit can support functions that have a max of $2^{19}$ constraints. We want to ensure that our benchmarks (and possible optimisations) capture the "heavy function" case and we don't just optimise for lightweight functions.

#### Summary of what we are measuring to capture Prover time

1. A mock kernel circuit has a size of $2^{17}$ constraints and folds _two_ Honk instances into an accumulator (the prev. kernel and the function being called)
2. The Prover must prove 5 mock function circuit proofs of size $2^{17}$ and one mock function proof of size $2^{19}$
3. The Prover must iteratively prove 6 mock kernel circuit proofs

#### Execution environment

For the MVP we can assume the user has reasonable hardware. For the purpose we use a 2-year-old macbook with 16gb RAM. The proof must be generated in a web browser

#### Performance targets

For an MVP, we target a 1 minute proof generation time. This is a substantial amount of time to ask a user to wait and we are measuring on good hardware.

In an ideal world, a 10 second proof generation time would be much better for UX.

### Verifier time

This matters because verifying a transaction is effectively free work being performed by sequencers and network nodes that propagate txns to the mempool. If verification time becomes too large it opens up potential DDOS attacks.

If we reserve 10% of the block production time for verifying user proofs, at 10 transaction per seconds this gives us 0.01s per transaction. i.e. 10ms per proof.

If the block producer has access to more than one physical machine that they can use to parallelise verification, we can extend the maximum tolerable verification time. For an MVP that requires 20ms to verify each proof, each block producer would require at least 2 physical machines to successfully build blocks.

100tps with one physical machine would require a verifiation time of 1ms per proof.

### Memory consumption

This is _critical_. Users can tolerate slow proofs, but if Honk consumes too much memory, a user cannot make a proof at all.

safari on iPhone will purge tabs that consume more than 1gb of RAM. The WASM memory cap is 4gb which defines the upper limit for an MVP.

### Kernel circuit size

Not a critical metric, but the prover time + prover memory metrics are predicated on a kernel circuit costing about 2^17 constraints!

### AVM Prover time

Our goal is to hit main-net with a network that can support 10 transactions per second. We need to estimate how many VM computation steps will be needed per transaction to determine the required speed of the VM Prover. The following uses very conservative estimations due to the difficulty of estimating this.

An Ethereum block consists of approximately 1,000 transactions, with a block gas limit of roughly 10 million gas. Basic computational steps in the Ethereum Virtual Machine consume 3 gas. If the entire block gas limit is consumed with basic computation steps (not true but let's assume for a moment), this implies that 1,000 transactions consume 3.33 million computation steps. i.e. 10 transactions per second would require roughly 33,000 steps per second and 3,330 steps per transaction.

As a conservative estimate, let us assume that every tx in a block will consume 10,000 AVM steps.

Our AVM model is currently to evaluate a transaction's public function calls within a single AVM circuit.
This means that a block of `n` transactions will require `n` pulic kernel proofs and `n` AVM proofs to be generated (assuming all txns have a public component).

If public VM proof construction consumes 20% of block time, we must generate 10 AVM proofs and 10 public kernel proofs in 2 seconds.

When measuring 2-to-1 rollup prover time, we assume we have access to a Prover network with 500 physical devices available for computation.

i.e. 10 proofs in 2 seconds across 500 devices => 1 AVM + public kernel proof in 25 seconds per physical device.

If we assume that ~10 seconds is budgeted to the public kernel proof, this would give a 15 second prover time target for a 10,000 step AVM circuit.

100 tps requires 1.5 seconds per proof.

### AVM Memory consumption

A large AWS instance can consume 128Gb of memory which puts an upper limit for AVM RAM consumption. Ideally consumer-grade hardware can be used to generate AVM proofs i.e. 16 Gb.

### 2-to-1 rollup proving time

For a rollup block containing $2^d$ transactions, we need to compute 2-to-1 rollup proofs across $d$ layers (i.e. $2^{d-1}$ 2-to-1 proofs, followed by $2^{d-2}$ proofs, followed by... etc down to requiring 1 2-to-1 proof). To hit 10tps, we must produce 1 block in $\frac{2^d}{10}$ seconds.

Note: this excludes network coordination costs, latency costs, block construction costs, public VM proof construction costs (must be computed before the 2-to-1 rollup proofs), cost to compute the final UltraPlonk proof.

To accomodate the above costs, we assume that we can budget 40% of block production time towards making proofs. Given these constraints, the following table describes maximum allowable proof construction times for a selection of block sizes.

| block size | number of successive 2-to-1 rollup proofs | number of parallel Prover machines required for base layer proofs | time required to construct a rollup proof |
| --- | --- | --- | --- |
| $1,024$ | $10$ | $512$ | 4.1s |
| $2,048$ | $11$ | $1,024$ | 7.4s |
| $4,096$ | $12$ | $2,048$ | 13.6s |
| $8,192$ | $13$ | $4,096$ | 25.2s |
| $16,384$ | $14$ | $8,192$ | 46.8s |

We must also define the maximum number of physical machines we can reasonably expect to be constructing proofs across the Prover network. If we can assume we can expect $1,024$ machines available, this caps the MVP proof construction time at 7.4 seconds.

Supporting a proof construction time of 4.1s would enable us to reduce minimum hardware requirements for the Prover network to 512 physical machines.

### 2-to-1 rollup memory consumption

Same rationale as the public VM proof construction time.

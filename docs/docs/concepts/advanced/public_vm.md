---
title: Public VM
---

## Objectives

We need to ensure that the execution environment for public functions has the following properties:

1. The Prover is fairly compensated for their work (gas metering)
1. A dishonest Prover cannot convince the kernel circuit that a public function failed, when an honest Prover would claim it succeeded
1. A public function cannot produce unsatisfiable constraints that would prevent the creation of a valid block proof
1. The published verification key for a public function is accurately represented by the opcode stream Provers will use to construct a function proof

### Why we need a VM

Point 2 requires a Virtual Machine architecture. The older model of having ACIR directly produce a circuit + verification key is flawed. For any witness that is _not_ a public input, a dishonest Prover could assign an incorrect value and create a failing proof.

The kernel circuit cannot prevent this by _requiring_ the public function proof to succeed, as it is trivial for a malicious contract writer to create unsatisfiable programs (e.g. `assert(true == false)`).

We cannot require that a Prover/Sequencer simulates the transaction to determine whether it is possible to include it in the block proof, as this amounts to unpaid compute that can be used as a DoS attack vector.

### VM Solution

How a VM solves DoS attacks + dishonest Prover/contract writer attacks.

Every VM opcode operates on some _input state_ and executes 1 of 2 functions:

1. A pure function that produces output state from input state
2. A function that produces no output state and applies assertion checks on input state

VM simulator asserts that functions of type 1 always succeed.

VM simulator allows functions of type 2 to fail while still producing a satisfiable VM proof. (i.e. the VM circuit contains a `failed` public input flag, that will be set to `true` if any assertion checks fail.)

The protocol mandates the VM proof must be valid, ensuring that functions of type 1 are correctly evaluated by the Prover.

The VM architecture must finally ensure that _all_ witness values are either public inputs or derived from public inputs. This implies that failing assertion checks are due to bad inputs and not a malicious Prover.

## Gas metering the VM

Public functions have the following L2 costs associated with them:

1. The compute cost of generating a VM proof for a given program
2. The compute cost of generating a public kernel snark proof
3. The latency cost of reading L2 state
4. The storage cost of writing L2 state

The following L1 costs also apply:

1. Data cost of broadcasting L2 state writes to L1

Under a VM model, the VM simulator can perform opcode-by-opcode gas metering.

(need to define VM architecture for more detail)

## VM ABI

Like private functions, public functions do not make state updates; they make state update _requests_ that are kicked out as public inputs.

This ensures that all VM witnesses are derived from public inputs (Merkle hash paths have lots of aux data not contained in public inputs)

It also makes gas metering easier as all L1 metering happens in the Kernel circuit, not the VM simulator circuit

# MVP Roadmap

The public kernel circuit does not directly verify a public function proof.

It verifies a _verifier circuit_ that verifies a public function proof!

Why? Modularity, ease of development, backwards compatibility support.

Proceed with the following development phases:

#### Phase 0: Full Proverless

There are no proofs. The rollup verifier smart contract returns `yes`

#### Phase 1: Kernel Proverless

Rollup proofs are generated, but public kernel snark verification logic is absent: kernel verifier returns `yes`

#### Phase 2: VM Proverless

Kernel proof is generated, but public function verifier circuit always returns `yes`

#### Phase 3: Honest prover assumption

The verifier circuit verifies a circuit whose verification key is directly computed from ACIR (like the private functions)

In theory we could go into production with this model, but without full decentralization.

#### Phase 4: Full VM

The verifier circuit verifies that a VM program has been successfully executed.

---

Additional benefit of having a VM verifier circuit is that we can upgrade the VM while supporting backwards compatibility (Kernel circuit picks one of multiple VM verification keys depending on the version used when contract was deployed).

Also means we can adopt completely different VM architectures without changing existing circuits (e.g. eWasm?)

# Draft Aztec VM Architecture

This VM arch design makes the following assumptions:

1. We want the public function developer experience to track private function developer experience as closely as possible
2. Public function efficiency is not a protocol bottleneck (i.e. if pub fns are similar to private fns, and private fns proofs can be computed on consumer hardware, we can afford a significant prover slowdown factor when making public fn proofs).

On the tradeoff space between public function prover efficiency and reduced protocol complexity, I think it's better to bias towards reduced protocol complexity.

Given that private functions compile to ACIR opcodes, it is natural to define a VM architecture that directly _simulates_ ACIR opcodes.

Consider a crypto backend that produces circuit constraints from ACIR opcodes. Minimal complexity solution requires a VM architecture that will execute _identical constraints_ within the context of a VM simulator.

i.e. we can repurpose our existing crypto backend and ACIR constraint generation modules to build the VM.

## VM architecture overview

ACIR opcodes do not directly map to constraints; they map to subroutines composed of constraints (however the subroutine size can be 1 for a simple opcode e.g. ADD)

The subroutine constraints for a given opcode is described by _microcode_.

<!-- The ACIR opcode specification is fixed for a given version of ACIR.

The VM microcode specification is fixed for a given VM version.

i.e. the same opcode stream can produce different microcode constraint depending on the VM version. This decouples the VM from ACIR and allows for VM optimizations w/o changing the ACIR spec.

TODO: this would be nice but I don't think it's possible?!
-->

A **program instruction** is a tuple of:

1. _instruction value_
2. _instruction witness context_
3. _instruction selector context_

The _instruction value_ is represents a combination of opcode value and microcode value.

The _witness context_ describes up to four _witness indices_ that are used to look up the witness values used in the microcode constraint.

The _selector context_ describes custom arithmetic selector values, when the values are not derived directly from the opcode/microcode (e.g. mul gates, add gates, linear combination gates)

An instruction value is used to look up the UltraPlonk constraint selectors (from a fixed lookup table that is shared by all VM programs) required to evaluate a microcode constraint.

The microcode constraint selector values are looked up from opcode/microcode instruction table. For arithmetic selectors, the program-specific selector context values are added in.

> We assume that if opcode selector context is nonzero, relevant selector values in the microcode lookup table are zero. Vice-versa applies. This can be validated when publishing a contract.

> Selector contexts cannot fully define an arithmetic constraint. There must be 1 degree of freedom remaining to prevent VM programs from producing unsatisfiable constraints (e.g. $q_m, q_1, q_2, q_3, q_4 = 0, q_c = 1$). TODO: how do we enforce this? We might need to transpile some ACIR opcodes that run into this issue into sequences of simpler ones that don't.

## VM program columns

Witness commitments:

| $PC$            | $I$               | $\vec{C_q}$      | $\vec{C}_{w}$               | $\vec{Q}$       | $\vec{W}$                  | $E$        | $G$         |
| --------------- | ----------------- | ---------------- | --------------------------- | --------------- | -------------------------- | ---------- | ----------- |
| Program counter | Instruction value | selector context | witness indices (4 columns) | selector values | witness values (4 columns) | Error flag | Gas counter |

Prover-computed lookup tables:

| $\vec{T}_{W}$                                   | Lookup Relation          |
| ----------------------------------------------- | ------------------------ |
| Maps witness index to witness value (4 columns) | $W_i = T_{W_i}[C_{W_i}]$ |

Program-specific precomputed lookup tables:

| $\vec{T}_I$                                                                                   | Lookup Relation                            |
| --------------------------------------------------------------------------------------------- | ------------------------------------------ |
| Maps program counter value to tuple of `instruction value, witness context, selector context` | $\{I, \vec{C}_q, \vec{C}_w \} = T_{I}[PC]$ |

The $[\vec{T}_I]$ commitments form the program-specific components of the VM verification key.

VM-specific precomputed lookup tables

| $\vec{T}_{Q}$                             | Lookup Relation                                                                                                             |
| ----------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Maps instruction value to selector values | $\{ q_1, q_2, q_3, q_4, q_m, q_c, q_{sort}, \\ q_{plookup}, q_{plookup\_index}, q_{arith}, q_{ecc}, q_{aux} \} \\ = T_Q[I]$ |

| $T_{G}$                           | Lookup Relation                |
| --------------------------------- | ------------------------------ |
| Defines gas value per instruction | $G_{i} = G_{i - 1} + T_G[I_i]$ |

## VM Verifier Logic

New arithmetic checks are required in the verifier algorithm.

1. $PC_i = PC_{i-1} + 1$ (if we add JUMP/JUMPI instructions this logic will change)
2. $\{ I, \vec{C}_q, \vec{C}_w \} = T_i[PC]$
3. $\vec{W} = T_w[\vec{C}_w]$
4. $\vec{Q} = T_q[I] + \vec{C}_q$
5. $G_i = G_{i-1} + T_G[I_i]$

Standard UltraPlonk arithmetic checks are applied to $\vec{W}$ and $\vec{Q}$, with some changes to the standard arithmetic constraint to accommodate for the error flag $E$

e.g. For some assertion check $P(W, Q)$, we do not require $P(W, Q) == 0$ but instead:

- Check $E$ is boolean
- If $P == 0, E == 0$
- If $P \ne 0, E == 1$

i.e:

If $P \ne 0$ let $X = \frac{1}{P}$, else $X == 0$

- $P \cdot (1 - E) == 0$
- $(P \cdot X - 1) \cdot E == 0$
- $E^2 - E == 0$

### Constant selectors

In the VM model, what used to be precomputed selectors are now part of the Prover's witness commitment.

The only selectors where this does not apply are:

1. Existing plookup selectors
2. Permutation selectors

#### Permutation Arguments

The original Plonk permutation argument is no longer required as all witnesses are looked up via plookup tables.

Additional grand product arguments are required for the multiple new lookups

#### Set equivalence checks

Current Composers rely on set equivalence checks to perform range checks, RAM read/writes and ROM reads.

This isn't possible in the VM model as set equivalence checks require additional constraints that depend on the size of the set. i.e. if two programs require different numbers of range checks (or different range sizes) they will not produce two uniform circuits.

In the VM arch we need to find another way to evaluate range checks, ROM and RAM. Will leave as a TODO for now, doubt it's too tricky. Range checks also fall under the assertion category i.e. failing a range check doesn't create an unsatisfiable circuit; the proof is valid but the output error flag is set to $1$.

## VM Overheads

Why this is a public-function-only abstraction.

The following UltraPlonk/Honk selectors are now witness commitments! (The $\vec{Q}$ columns)

$$q_1, q_2, q_3, q_4, q_m, q_c, q_{sort}, q_{plookup}, q_{plookup\_index}, q_{arith}, q_{ecc}, q_{aux}$$

Additional witness commitments required that are not part of UltraPlonk/Honk:

$PC, I, C_{w1}, C_{w2}, C_{w3}, C_{w4}, E, G, T_{w1}, T_{w2}, T_{w3}, T_{w4}$

In addition, selector context polynomials $C_{q}$ will likely represent at least 4 selectors in order to handle ACIR's linear combination instruction

That's 28 additional witness polynomial commitments on top of the usual Honk commitments.

The additional 4 plookup arguments will also likely require at least 2 additional grand product + sorted list commitments, pushing the number of extra commitments to 32.

To contrast, we are expecting Honk to have 6 Prover commitments:

$w1, w2, w3, w4, Z, S$

(Z = combined grand product of permutation argument and plookup argument)
(S = sorted list for plookup argument)

Plus 2 opening proofs for KZG.

i.e. the overall number of Prover multi-exponentiations has grown from $8n$ to $40n$

TLDR: a VM simulator circuit of an ACIR program is approx. 6 times more expensive to prove than a proof of a natively-compiled ACIR circuit.

We can afford to pay this for public functions, but this is far too great an overhead for locally-generated private function proofs.

---

## Building the VM simulator circuit

The plan is that the existing crypto backend can do the following by re-using the Plonk standard library with a custom composer:

1. build the VM circuit lookup tables
2. build VM verification keys for a given program _and_ generate program-specific VM proofs

i.e. stdlib code is re-used and only composer code changes.

### Computing $T_Q, T_G$ using the stdlib

A program is written that executes every ACIR opcode in sequence.

Composer will track an 'instruction counter'. If the opcode makes a standard library call, the stdlib function is executed as normal, but the Composer will do the following:

- Whenever a constraint is added, selector values are written into $T_Q$ at the current `instruction_counter` value
- Default value written into $T_G$. We'll need to pass special context booleans into a stdlib function if it is performing a state read/write in order to add special values into $T_G$
- `instruction_counter` is incremented

### Computing VM circuits + proofs using the stdlib

In this context, the Composer already possesses $T_Q$.

Consider the execution of an ACIR opcode stream. If the opcode makes a standard library call, the stdlib function is executed as normal, but the Composer will do the following when a constraint is added:

- Current program counter value added into $PC$
- Composer validates the selector values are present in $T_Q$ and looks up the instruction value
- instruction value added to $I$
- If not already present, program counter/instruction value mapping is written into $T_I$, along with required witness indices
- Witness index values are written into $C_w$
- Depending on context, some constraints may write values into $C_q$ (e.g.basic arithmetic gates)
- Gas values looked up from $T_G$ and $G$ updated
- Similarly $E$ updated if assertion fails

We assume the circuit will evaluate an opcode stream of a fixed size. If the opcode stream is smaller than this limit, the remaining instructions are evaluated as NOP instructions

The Composer will be able to produce the program-specific components of the verification key $[\vec{T}_I]$ as well as the Prover-specific commitments for a proof given the public inputs used.

## Validating VM programs

When adding a contract to Aztec, we want to validate that the $T_I$ commitments are commitments to a published opcode stream.

We can do this efficiently in a custom circuit.

1. Compute Fiat-Shamir challenges $\alpha, \zeta$ by hashing $[\vec{T}_I]$
2. Evaluate $\vec{T}_I(\zeta)$ via KZG, using $\alpha$ to create linear combination of all commitments within $[\vec{T}_I]$
3. Iterate over the opcode stream and manually compute $\vec{T}_I(\zeta)$ in linear-time using field operations. Validate this matches the evaluation of the commitments

In addition, we can perform checks on each opcode to validate conditions on selector contexts within $T_I$ and ensure unsatisfiable constraints are not being generated.

---

## Open Questions

No doubt many, but first one on my mind is:

Q: Do we want to support JUMP/JUMPI instructions for public ACIR functions? Would be _very_ nice to have but creates discontinuity between public/private functions.

# Proving System Components

# Interactive Proving Systems

## Ultra Plonk

UltraPlonk is a variant of the [PLONK](https://eprint.iacr.org/2019/953) protocol - a zkSNARK with a universal trusted setup.

UltraPlonk utilizes the "Ultra" circuit arithmetisation. This is a configuration with four wires per-gate, and the following set of gate types:

- arithmetic gate
- elliptic curve point addition/doubling gate
- range-check gate
- plookup table gate
- memory-checking gates
- non-native field arithmetic gates

## Honk

Honk is a variant of the PLONK protocol. Plonk performs polynomial testing via checking a polynomial relation is zero modulo the vanishing polynomial of a multiplicative subgroup. Honk performs the polynomial testing via checking, using a sumcheck protocol, that a relation over multilinear polynomials vanishes when summed over a boolean hypercube.

The first protocol to combine Plonk and the sumcheck protocol was [HyperPlonk](https://eprint.iacr.org/2022/1355)

Honk uses a custom arithmetisation that extends the Ultra circuit arithmetisation (not yet finalized, but includes efficient Poseidon2 hashing)

# Incrementally Verifiable Computation Subprotocols

An Incrementally Verifiable Computation (IVC) scheme describes a protocol that defines some concept of persistent state, and enables multiple successive proofs to evolve the state over time.

IVC schemes are used by Aztec in two capacities:

1. to compute a client-side proof of one transaction execution.
2. to compute a proof of a "rollup" circuit, that updates rollup state based on a block of user transactions

Both use IVC schemes. Client-side, each function call in a transaction is a "step" in the IVC scheme. Rollup-side, aggregating two transaction proofs is a "step" in the IVC scheme.

The client-side IVC scheme is substantially more complex than the rollup-side scheme due to performance requiremenmts.

Rollup-side, each "step" in the IVC scheme is a Honk proof, which are recursively verified. As a result, no protoocols other than Honk are required to execute rollup-side IVC.

We perform one layer of ["proof-system compression"](https://medium.com/aztec-protocol/proof-compression-a318f478d575) in the rollup. The final proof of block-correctness is constructed as a Honk proof. An UltraPlonk circuit is used to verify the correctness of the Honk proof, so that the proof that is verified on-chain is an UltraPlonk proof.
Verification gas costs are lower for UltraPlonk vs Honk due to the following factors:

1. Fewer precomputed selector polynomials, reducing Verifier G1 scalar multiplications
2. UltraPlonk does not use multilinear polynomials, which removes 1 pairing from the Verifier, as well as O(logn) G1 scalar multiplications.

The following sections list the protocol components required to implement client-side IVC. We make heavy use of folding schemes to build an IVC scheme. A folding scheme enables instances of a relation to be folded into a single instance of the original relation, but in a "relaxed" form. Depending on the scheme, restrictions may be placed on the instances that can be folded.

The main two families of folding schemes are derived from the [Nova](https://eprint.iacr.org/2021/370) protocol and the [Protostar](https://eprint.iacr.org/2023/620) protocol respectively.

## Protogalaxy

The [Protogalaxy](https://eprint.iacr.org/2023/1106) protocol efficiently supports the ability to fold multiple Honk instances (describing different circuits) into the same accumulator. To constrast, the Nova/Supernova/Hypernova family of folding schemes assume that a single circuit is being repeatedly folded (each Aztec function circuit is a distinct circuit, which breaks this assumption).

It is a variant of [Protostar](https://eprint.iacr.org/2023/620). Unlike Protostar, Protogalaxy enables multiple instances to be efficiently folded into the same accumulator instance.

The Protogalaxy protocol is split into two subprotocols, each modelled as interactive protocols between a Prover and a Verifier.

#### Protogalaxy Fold

The "Fold" Prover/Verifier validates that `k` instances of a defined relation (in our case the Honk relation) have been correctly folded into an accumulator instance.

#### Protogalaxy Decider

The "Decider" Prover/Verifier validate whether an accumulator instance correctly satisfies the accumulator relation. The accumulator being satisfiable inductively shows that all instances that have been folded were satisfied as well. (additional protocol checks are required to reason about _which_ instances have been folded into the accumulator. See the [IVC specification](https://hackmd.io/h0yTcOHiQWeeTXnxTQhTNQ?view) for more information. (note to zac: put this in the protocol specs!)

## Goblin Plonk

[Goblin Plonk](https://hackmd.io/@aztec-network/BkGNaHUJn/%2FGfNR5SE5ShyXXmLxNCsg3g) is a computation delegation scheme that improves Prover performance when evaluating complex algorithms.

In the context of an IVC scheme, Goblin Plonk enables a Prover to defer non-native group operations required by a Verifier algorithm, across multiple recursive proofs, to a single step evaluated at the conclusion of the IVC Prover algorithm.

Goblin Plonk is composed of three subcomponents:

#### Transcript Aggregation Subprotocol

This subprotocol aggregates deferred computations from two independent instances, into a single instance

#### Elliptic Curve Virtual Machine (ECCVM) Subprotocol

The ECCVM is a Honk circuit with a custom circuit arithmetisation, designed to optimally evaluate elliptic curve arithmetic computations that have been deferred. It is defined over the Grumpkin elliptic curve.

#### Translator Subprotocol

The Translator is a Honk circuit, defined over BN254, with a custom circuit arithmetisation, designed to validate that the input commitments of an ECCVM circuit align with the delegated computations described by a Goblin Plonk transcript commitment.

## Plonk Data Bus

When passing data between successive IVC steps, the canonical method is to do so via public inputs. This adds significant costs to an IVC folding verifier (or recursive verifier when not using a folding scheme). Public inputs must be hashed prior to generating Fiat-Shamir challenges. When this is performed in-circuit, this adds a cost linear in the number of public inputs (with unpleasant constants ~30 constraints per field element).

The Data Bus protocol eliminates this cost by representing cross-step data via succinct commitments instead of raw field elements.

The Plonk Data Bus protocol enables efficient data transfer between two Honk instances within a larger IVC protocol.

# Polynomial Commitment Schemes

The UltraPlonk, Honk, Goblin Plonk and Plonk Data Bus protocols utilize Polynomial Interactive Oracle Proofs as a core component, thus requiring the use of polynomial commitment schemes (PCS).

UltraPlonk and Honk utilize multilinear PCS. The Plonk Data Bus and Goblin Plonk also utilize univariate PCS.

For multilinear polynomial commitment schemes, we use the [ZeroMorph](https://eprint.iacr.org/2023/917) protocol, which itself uses a univariate PCS as a core component.

Depending on context we use the following two univariate schemes within our cryptography stack.

## KZG Commitments

The [KZG](https://www.iacr.org/archive/asiacrypt2010/6477178/6477178.pdf) polynomial commitment scheme requires a universal setup and is instantiated over a pairing-friendly elliptic curve.

Computing an opening proof of a degree-$n$ polynomial requires $n$ scalar multiplications, with a constant proof size and a constant verifier time.

## Inner Product Argument

The [IPA](https://eprint.iacr.org/2019/1177.pdf) PCS has worse asymptotics than KZG but can be instantiated over non-pairing friendly curves.

We utilize the Grumpkin elliptic curve as part of the Goblin Plonk protocol, where we utilize the curve cycle formed between BN254 and Grumpkin to translate expensive non-native BN254 group operations in a BN254 circuit, into native group operations in a Grumpkin circuit.

Computing an opening proof of a degree-$n$ polynomial requires $2n$ scalar multiplications, with a $O(logn)$ proof size and an $O(n)$ verifier time.

To batch-verify multiple opening proofs, we use the technique articulated in the [Halo](https://eprint.iacr.org/2019/1021) protocol. To compute a proof of a single rollup block, only one linear-time PCS opening proof is verified despite multiple IPA proofs being generated as part of constructing the rollup proof.

# Combined IVC + Proving System Protocol

The following block diagrams describe the components used by the client-side and server-side Provers when computing client proofs and rollup proofs respectively.

![proof-system-components](/img/protocol-specs/cryptography/proof-system-components.png)

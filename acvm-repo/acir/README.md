# ACIR documentation (draft)

## Abstract

This document describes the purpose of ACIR, what it is and how ACIR programs
can be used by compilers and proving systems. It is intended to be a reference
documentation for ACIR.

## Introduction

The purpose of ACIR is to make the link between a generic proving system, such
as Aztec's Barretenberg, and a frontend, such as Noir, which describes user
specific computations.

More precisely, Noir is a programming language for zero-knowledge proofs (ZKP)
which allows users to write programs in an intuitive way using a high-level
language close to Rust syntax. Noir is able to generate a proof of execution of
a Noir program, using an external proving system. However, proving systems use
specific low-level constrain-based languages. Similarly, frontends have their
own internal representation in order to represent user programs.

The goal of ACIR is to provide a generic open-source intermediate
representation close to proving system 'languages', but agnostic to a specific
proving system, that can be used both by proving system as well as a target for
frontends. So, at the end of the day, an ACIR program is just another
representation of a program, dedicated to proving systems.

## Abstract Circuit Intermediate Representation
ACIR stands for abstract circuit intermediate representation:
- **abstract circuit**: circuits are a simple computation model where basic
    computation units, named gates, are connected with wires. Data flows
    through the wires while gates compute output wires based on their input.
    More formally, they are directed acyclic graphs (DAG) where the vertices
    are the gates and the edges are the wires. Due to the immutability nature
    of the wires (their value does not change during an execution), they are
    well suited for describing computations for ZKPs. Furthermore, we do not
    lose any expressiveness when using a circuit as it is well known that any
    bounded computation can be translated into an arithmetic circuit (i.e a
    circuit with only addition and multiplication gates).
The term abstract here simply means that we do not refer to an actual physical
circuit (such as an electronic circuit). Furthermore, we will not exactly use
the circuit model, but another model even better suited to ZKPs, the constraint
model (see below).
- **intermediate representation**: The ACIR representation is intermediate
because it lies between a frontend and its proving system. ACIR bytecode makes
the link between noir compiler output and the proving system backend input.

## The constraint model

The first step for generating a proof that a specific program was executed, is
to execute this program. Since the proving system is going to handle ACIR
programs, we need in fact to execute an ACIR program, using the user-supplied
inputs.

In ACIR terminology, the gates are called opcodes and the wires are called
partial witnesses. However, instead of connecting the opcodes together through
wires, we create constraints: an opcode constraints together a set of wires.
This constraint model trivially supersedes the circuit model. For instance, an
addition gate `output_wire = input_wire_1 + input_wire_2` can be expressed with
the following arithmetic constraint:
`output_wire - (input_wire_1 + input_wire_2) = 0`


## Solving

Because of these constraints, executing an ACIR program is called solving the
witnesses. From the witnesses representing the inputs of the program, whose
values are supplied by the user, we find out what the other witnesses should be
by executing/solving the constraints one-by-one in the order they were defined.

For instance, if `input_wire_1` and `input_wire_2` values are supplied as `3` and
`8`, then we can solve the opcode
`output_wire - (input_wire_1 + input_wire_2) = 0` by saying that `output_wire` is
`11`.

In summary, the workflow is the following:
1. user program -> (compilation) ACIR, a list of opcodes which constrain
    (partial) witnesses
2. user inputs + ACIR -> (execution/solving) assign values to all the
    (partial) witnesses
3. witness assignment + ACIR -> (proving system) proof

Although the ordering of opcode does not matter in theory, since a system of
equations is not dependent on its ordering, in practice it matters a lot for the
solving (i.e the performance of the execution). ACIR opcodes **must be ordered**
so that each opcode can be resolved one after the other.

The values of the witnesses lie in the scalar field of the proving system. We
will refer to it as `FieldElement` or ACIR field. The proving system needs the
values of all the partial witnesses and all the constraints in order to generate
a proof.

*Remark*: The value of a partial witness is unique and fixed throughout a program
    execution, although in some rare cases, multiple values are possible for a
    same execution and witness (when there are several valid solutions to the
    constraints). Having multiple possible values for a witness may indicate that
    the circuit is not safe.

*Remark*: Why do we use the term partial witnesses? It is because the proving
    system may create other constraints and witnesses (especially with
    `BlackBoxFuncCall`, see below). A proof refers to a full witness assignments
    and their constraints. ACIR opcodes and their partial witnesses are still an
    intermediate representation before getting the full list of constraints and
    witnesses. For the sake of simplicity, we will refer to witness instead of
    partial witness from now on.


## ACIR Reference

We assume here that the proving system is Barretenberg. Some parameters may
slightly change with another proving system, in particular the bit size of
`FieldElement`, which is 254 for Barretenberg.

Some opcodes have inputs and outputs, which means that the output is constrained
to be the result of the opcode computation from the inputs. The solver expects
that all inputs are known when solving such opcodes.

Some opcodes are not constrained, which means they will not be used by the
proving system and are only used by the solver.

Finally, some opcodes will have a predicate, whose value is `0` or `1`. Its
purpose is to nullify the opcode when the value is `0`, so that it has no
effect. Note that removing the opcode is not a solution because this modifies
the circuit (the circuit being mainly the list of the opcodes).

*Remark*: Opcodes operate on witnesses, but we will see that some opcode work on
    expressions of witnesses. We call an expression a linear combination of
    witnesses and/or products of two witnesses (and also a constant term). A
    single witness is a (simple) expression, and conversely, an expression can
    be turned into a single witness using an assert-zero opcode (see below). So
    basically, using witnesses or expressions is equivalent, but the latter can
    avoid the creation of witness in some cases.

### AssertZero opcode

An `AssertZero` opcode adds the constraint that `P(w) = 0`, where
`w=(w_1,..w_n)` is a tuple of `n` witnesses, and `P` is a multi-variate
polynomial of total degree at most `2`.

### BlackBoxFuncCall opcode

These opcodes represent a specific computation. Even if any computation can be
done using only assert-zero opcodes, it is not always efficient. Some proving
systems, and in particular the proving system from Aztec, can implement several
computations more efficiently using for instance look-up tables. The
`BlackBoxFuncCall` opcode is used to ask the proving system to handle the
computation by itself.

All black box functions take as input a tuple `(witness, num_bits)`, where
`num_bits` is a constant representing the bit size of the input witness, and they
have one or several witnesses as output.

Some more advanced computations assume that the proving system has an
"embedded curve". It is a curve that cycles with the main curve of the proving
system, i.e the scalar field of the embedded curve is the base field of the main
one, and vice-versa. The curves used by the proving system are dependent on the
proving system (and/or its configuration). Aztec's Barretenberg uses BN254 as the
main curve and Grumpkin as the embedded curve.

NOTE: see the [black_box_functions](src/circuit/black_box_functions.rs) file for
the most up-to-date documentation on these opcodes.

The black box functions supported by ACIR are:

**AES128Encrypt**: ciphers the provided plaintext using AES128 in CBC mode, padding the input using PKCS#7.

**AND**: performs the bitwise AND of `lhs` and `rhs`. `bit_size` must be the same for both inputs.

**XOR**: performs the bitwise XOR of `lhs` and `rhs`. `bit_size` must be the same for both inputs.

**RANGE**: constraint the input to be of the provided bit size

**SHA256**: computes sha256 of the inputs

**Blake2s**: computes the Blake2s hash of the inputs, as specified in https://tools.ietf.org/html/rfc7693

**Blake3**: computes the Blake3 hash of the inputs

**SchnorrVerify**: Verify a Schnorr signature over the embedded curve

**PedersenCommitment**: Computes a Pedersen commitments of the inputs using generators of the embedded curve

**PedersenHash**: Computes a Pedersen commitments of the inputs and their number, using generators of the embedded curve

**EcdsaSecp256k1**: Verify an ECDSA signature over Secp256k1

**EcdsaSecp256r1**: Same as EcdsaSecp256k1, but done over another curve.

**MultiScalarMul**: scalar multiplication with a variable base/input point (`P`) of the embedded curve

**Keccak256**: Computes the Keccak-256 (Ethereum version) of the inputs.

**Keccakf1600**: Keccak Permutation function of width 1600

**EmbeddedCurveAdd**: Embedded curve addition

**BigIntAdd**: BigInt addition

**BigIntSub**: BigInt subtraction

**BigIntMul**: BigInt multiplication

**BigIntDiv**: BigInt division

**BigIntFromLeBytes**: BigInt from le bytes

**BigIntToLeBytes**: BigInt to le bytes

**Poseidon2Permutation**: Permutation function of Poseidon2

**Sha256Compression**: SHA256 compression function

**RecursiveAggregation**: verify a proof inside the circuit.
**Warning: this opcode is subject to change.**

This black box function does not fully verify a proof, what it does is verify
that the provided `key_hash` is indeed a hash of `verification_key`, allowing
the user to use the verification key as private inputs and only have the
`key_hash` as public input, which is more performant.

Another thing that it does is preparing the verification of the proof. In order
to fully verify a proof, some operations may still be required to be done by the
final verifier. This is why this black box function does not say if verification
is passing or not.

If you have several proofs to verify in one ACIR program, you would call
`RecursiveAggregation()` multiple times while passing the
`output_aggregation_object` as `input_aggregation_object` to the next
`RecursiveAggregation()` call, except for the first call where you do not have
any `input_aggregation_object`.

If one of the proof you verify with the black box function does not verify, then
the verification of the proof of the main ACIR program will ultimately fail.


### Brillig

This opcode is used as a hint for the solver when executing (solving) the
circuit. The opcode does not generate any constraint and is usually the result
of the compilation of an unconstrained noir function.

Let's see an example with euclidean division.
The normal way to compute `a/b`, where `a` and `b` are 8-bits integers, is to
implement the Euclidean algorithm which computes in a loop (or recursively)
modulus of the kind 'a mod b'. Doing this computation requires a lot of steps to
be properly implemented in ACIR, especially the loop with a condition. However,
euclidean division can be easily constrained with one assert-zero opcode:
`a = bq+r`, assuming `q` is 8 bits and `r<b`. Since these assumptions can easily
written with a few range opcodes, euclidean division can in fact be implemented
with a small number of opcodes.

However, in order to write these opcodes we need the result of the division
which are the witness `q` and `r`. But from the constraint `a=bq+r`, how can the
solver figure out how to solve `q` and `r` with only one equation? This is where
brillig/unconstrained function come into action. We simply define a function that
performs the usual Euclid algorithm to compute `q` and `r` from `a` and `b`.
Since Brillig opcode does not generate constraint, it won't be provided to the
proving system but simply used by the solver to compute the values of `q` and
`r`.

In summary, solving a Brillig opcode performs the computation defined by its
bytecode, on the provided inputs, and assign the result to the outputs witnesses,
without adding any constraint.

NOTE: see the [circuit/opcodes.rs](src/circuit/opcodes.rs) file for the most
up-to-date documentation on these opcodes.

#### MemoryOp: memory abstraction for ACIR

ACIR is able to address any array of witnesses. Each array is assigned an ID
(`BlockId`) and needs to be initialized with the `MemoryInit` opcode. Then it is
possible to read and write from/to an array by providing the index and the value
we read/write, as arithmetic expression. Note that ACIR arrays all have a known
fixed length (given in the `MemoryInit` opcode below).

#### MemoryInit

Initialize an ACIR array from a vector of witnesses.

There must be only one MemoryInit per block_id, and MemoryOp opcodes must
come after the MemoryInit.

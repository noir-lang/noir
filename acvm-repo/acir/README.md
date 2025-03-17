# ACIR documentation (draft)

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

Computes a recursive aggregation object internally when verifying a proof inside
another circuit.
The outputted aggregation object will then be either checked in a
top-level verifier or aggregated upon again.
The aggregation object should be maintained by the backend implementer.

This opcode prepares the verification of the final proof.
In order to fully verify a recursive proof, some operations may still be required
to be done by the final verifier (e.g. a pairing check).
This is why this black box function does not say if verification is passing or not.
It delays the expensive part of verification out of the SNARK
and leaves it to the final verifier outside of the SNARK circuit.

This opcode also verifies that the key_hash is indeed a hash of verification_key,
allowing the user to use the verification key as private inputs and only
have the key_hash as public input, which is more performant.

**Warning: the key hash logic does not need to be part of the black box and subject to be removed.**

If one of the recursive proofs you verify with the black box function fails to
verify, then the verification of the final proof of the main ACIR program will
ultimately fail.

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

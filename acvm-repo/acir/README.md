# ACIR documentation (draft)

## Abstract
This document describes the purpose of ACIR, what it is and how ACIR programs can be used by compilers and proving systems. It is intended to be a reference documentation for ACIR.

## Introduction
The purpose of ACIR is to make the link between a generic proving system, such as Aztec's Barretenberg, and a frontend, such as Noir, which describes user-specific computations.

More precisely, Noir is a programming language for zero-knowledge proofs (ZKP) which allows users to write programs in an intuitive way using a high-level language close to Rust syntax. Noir is able to generate a proof of execution of a Noir program, using an external proving system. However, proving systems use specific low-level constrain-based languages. Similarly, frontends have their own internal representation in order to represent user programs.

The goal of ACIR is to provide a generic open-source intermediate representation close to proving system 'languages', but agnostic to a specific proving system, that can be used both by proving system as well as a target for frontends. So, at the end of the day, an ACIR program is just another representation of a program, dedicated to proving systems.

## Abstract Circuit Intermediate Representation
ACIR stands for abstract circuit intermediate representation:
- **abstract circuit**: circuits are a simple computation model where basic computation units, named gates, are connected with wires. Data flows through the wires while gates compute output wires based on their input. More formally, they are directed acyclic graphs (DAG) where the vertices are the gates and the edges are the wires. Due to the immutability nature of the wires (their value does not change during an execution), they are well suited for describing computations for ZKPs. Furthermore, we do not lose any expressiveness when using a circuit as it is well known that any bounded computation can be translated into an arithmetic circuit (i.e a circuit with only addition and multiplication gates).
The term abstract here simply means that we do not refer to an actual physical circuit (such as an electronic circuit). Furthermore, we will not exactly use the circuit model, but another model even better suited to ZKPs, the constraint model (see below).
- **intermediate representation**: The ACIR representation is intermediate because it lies between a frontend and its proving system. ACIR bytecode makes the link between noir compiler output and the proving system backend input.

## The constraint model
The first step for generating a proof that a specific program was executed, is to execute this program. Since the proving system is going to handle ACIR programs, we need in fact to execute an ACIR program, using the user-supplied inputs.

In ACIR terminology, the gates are called opcodes and the wires are called partial witnesses. However, instead of connecting the opcodes together through wires, we create constraints: an opcode constraints together a set of wires.
This constraint model trivially supersedes the circuit model. For instance, an addition gate output_wire = input_wire_1 + input_wire_2 can be expressed with the following arithmetic constraint: output_wire - (input_wire_1 + input_wire_2) = 0



## Solving
Because of these constraints, executing an ACIR program is called solving the witnesses. From the witnesses representing the inputs of the program, whose values are supplied by the user, we find out what the other witnesses should be by executing/solving the constraints one-by-one in the order they were defined.

For instance, if input_wire_1 and input_wire_2 values are supplied as 3 and 8, then we can solve the opcode output_wire - (input_wire_1 + input_wire_2) = 0 by saying that output_wire is 11.

In summary, the workflow is the following:
1. user program -> (compilation) ACIR, a list of opcodes which constrain (partial) witnesses
2. user inputs + ACIR -> (execution/solving) assign values to all the (partial) witnesses 
3. witness assignment + ACIR -> (proving system) proof


Although the ordering of opcode does not matter in theory, since a system of equations is not dependent on its ordering, in practice it matters a lot for the solving (i.e the performance of the execution). ACIR opcodes **must be ordered** so that each opcode can be resolved one after the other.


The values of the witnesses lie in the scalar field of the proving system. We will refer to it as FieldElement or ACIR field. The proving system needs the values of all the partial witnesses and all the constraints in order to generate a proof.


*Remark*: The value of a partial witness is unique and fixed throughout a program execution, although in some rare cases, multiple values are possible for a same execution and witness (when there are several valid solutions to the constraints). Having multiple possible values for a witness may indicate that the circuit is not safe.

*Remark*: Why do we use the term partial witnesses? It is because the proving system may create other constraints and witnesses (especially with BlackBoxFuncCall, see below). A proof refers to a full witness assignments and their constraints. ACIR opcodes and their partial witnesses are still an intermediate representation before getting the full list of constraints and witnesses. For the sake of simplicity, we will refer to witness instead of partial witness from now on.


## ACIR Reference
We assume here that the proving system is Barretenberg. Some parameters may slightly change with another proving system, in particular the bit size of FieldElement, which is 254 for Barretenberg.

Some opcodes have inputs and outputs, which means that the output is constrained to be the result of the opcode computation from the inputs. The solver expects that all inputs are known when solving such opcodes.

Some opcodes are not constrained, which means they will not be used by the proving system and are only used by the solver.

Finally, some opcodes will have a predicate, whose value is 0 or 1. Its purpose is to nullify the opcode when the value is 0, so that it has no effect. Note that removing the opcode is not a solution because this modifies the circuit (the circuit being mainly the list of the opcodes). 

*Remark*: Opcodes operate on witnesses, but we will see that some opcode work on expressions of witnesses. We call an expression a linear combination of witnesses and/or products of two witnesses (and also a constant term). A single witness is a (simple) expression, and conversely, an expression can be turned into a single witness using an assert-zero opcode (see below). So basically, using witnesses or expressions is equivalent, but the latter can avoid the creation of witness in some cases.

### AssertZero opcode
An AssertZero opcode adds the constraint that P(w) = 0, where w=(w_1,..w_n) is a tuple of n witnesses, and P is a multi-variate polynomial of total degree at most 2.
The coefficients ${q_M}_{i,j}, q_i,q_c$ of the polynomial are known values which define the opcode.
A general expression of assert-zero opcode is the following: $\sum_{i,j} {q_M}_{i,j}w_iw_j + \sum_i q_iw_i +q_c = 0$

An assert-zero opcode can be used to:
- **express a constraint** on witnesses; for instance to express that a witness $w$ is a boolean, you can add the opcode:  $w*w-w=0$
- or, to **compute the value** of an arithmetic operation of some inputs. For instance, to multiply two witnesses $x$ and $y$, you would use the opcode $z-x*y=0$, which would constraint $z$ to be $x*y$.


The solver expects that at most one witness is not known when executing the opcode.

### BlackBoxFuncCall opcode
These opcodes represent a specific computation. Even if any computation can be done using only assert-zero opcodes, it is not always efficient. Some proving systems, and in particular the proving system from Aztec, can implement several computations more efficiently using for instance look-up tables. The BlackBoxFuncCall opcode is used to ask the proving system to handle the computation by itself.
All black box functions take as input a tuple (witness, num_bits), where num_bits is a constant representing the bit size of the input witness, and they have one or several witnesses as output.
Some more advanced computations assume that the proving system has an 'embedded curve'. It is a curve that cycle with the main curve of the proving system, i.e the scalar field of the embedded curve is the base field of the main one, and vice-versa. The curves used by the proving system are dependent on the proving system (and/or its configuration). Aztec's Barretenberg uses BN254 as the main curve and Grumpkin as the embedded curve.

The black box functions supported by ACIR are:

**AES128Encrypt**: ciphers the provided plaintext using AES128 in CBC mode, padding the input using PKCS#7.
- inputs: byte array [u8; N]
- iv: initialization vector [u8; 16]
- key: user key [u8; 16]
- outputs: byte vector [u8] of length `input.len() + (16 - input.len() % 16)``

**AND**: performs the bitwise AND of lhs and rhs. bit_size must be the same for both inputs.
- lhs: (witness, bit_size)
- rhs: (witness, bit_size)
- output: a witness whose value is constrained to be lhs AND rhs, as bit_size bit integers

**XOR**: performs the bitwise XOR of lhs and rhs. bit_size must be the same for both inputs.
- lhs: (witness, bit_size)
- rhs: (witness, bit_size)
- output: a witness whose value is constrained to be lhs XOR rhs, as bit_size bit integers

**RANGE**: constraint the input to be of the provided bit size
input: (witness, bit_size)

**SHA256**: computes sha256 of the inputs
- inputs are a byte array, i.e a vector of (FieldElement, 8)
- output is a byte array of len 32, i.e a vector of 32 (FieldElement, 8), constrained to be the sha256 of the inputs.

**Blake2s**: computes the Blake2s hash of the inputs, as specified in https://tools.ietf.org/html/rfc7693
- inputs are a byte array, i.e a vector of (FieldElement, 8)
- output is a byte array of length 32, i.e a vector of 32 (FieldElement, 8), constrained to be the blake2s of the inputs.


**SchnorrVerify**: Verify a Schnorr signature over the embedded curve
- inputs are:
    - Public key as 2 (FieldElement, 254)
    - signature as a vector of 64 bytes (FieldElement, 8)
    - message as a vector of (FieldElement, 8)
- output: A witness representing the result of the signature verification; 0 for failure and 1 for success.

Since the scalar field of the embedded curve is NOT the ACIR field, the (r,s) signature is represented as a 64 bytes array for the two field elements. On the other hand, the public key coordinates are ACIR fields.
The proving system decides how the message is to be hashed. Barretenberg uses Blake2s.


**PedersenCommitment**: Computes a Pedersen commitments of the inputs using generators of the embedded curve
- input: vector of (FieldElement, 254)
- output: 2 witnesses representing the x,y coordinates of the resulting Grumpkin point
- domain separator: a constant public value (a field element) that you can use so that the commitment also depends on the domain separator. Noir uses 0 as domain separator.

The backend should handle proper conversion between the inputs being ACIR field elements and the scalar field of the embedded curve. In the case of Aztec's Barretenberg, the latter is bigger than the ACIR field so it is straightforward. The Pedersen generators are managed by the proving system.


**PedersenHash**: Computes a Pedersen commitments of the inputs and their number, using generators of the embedded curve
- input: vector of (FieldElement, 254)
- output: the x-coordinate of the pedersen commitment of the 'prepended input' (see below)
- domain separator: a constant public value (a field element) that you can use so that the hash also depends on the domain separator. Noir uses 0 as domain separator.

In Barretenberg, PedersenHash is doing the same as PedersenCommitment, except that it prepends the inputs with their length.


**HashToField128Security**: This opcode is deprecated and will be removed.

**EcdsaSecp256k1**: Verify an ECDSA signature over Secp256k1
- inputs:
    - x coordinate of public key as 32 bytes
    - y coordinate of public key as 32 bytes
    - the signature, as a 64 bytes array
    - the hash of the message, as a vector of bytes
- output: 0 for failure and 1 for success

Inputs and outputs are similar to SchnorrVerify, except that because we use a different curve (secp256k1), the field elements involved in the signature and the public key are defined as an array of 32 bytes. Another difference is that we assume the message is already hashed.

**EcdsaSecp256r1**: Same as EcdsaSecp256k1, but done over another curve.

**MultiScalarMul**: scalar multiplication with a variable base/input point (P) of the embedded curve
- input:
    points (FieldElement, N) a vector of x and y coordinates of input points [x1, y1, x2, y2,...].
    scalars (FieldElement, N) a vector of low and high limbs of input scalars [s1_low, s1_high, s2_low, s2_high, ...]. (FieldElement, N) For Barretenberg, they must both be less than 128 bits.
- output: (FieldElement, N) a vector of x and y coordinates of output points [op1_x, op1_y, op2_x, op2_y, ...]. Points computed as $s_low*P+s_high*2^{128}*P$

Because the Grumpkin scalar field is bigger than the ACIR field, we provide 2 ACIR fields representing the low and high parts of the Grumpkin scalar $a$:
$a=low+high*2^{128},$ with $low, high < 2^{128}$

**Keccak256**: Computes the Keccak-256 (Ethereum version) of the inputs.
- inputs: Vector of bytes (FieldElement, 8)
- outputs: Vector of 32 bytes (FieldElement, 8)


**Keccak256VariableLength**: Computes the Keccak-256 (Ethereum version) of the inputs, restricted to the given length.
- inputs: Vector of bytes (FieldElement, 8)
- var_message_size: number of inputs to hash; it must be less (or equal) than the inputs length
- outputs: a vector of 32 bytes (FieldElement, 8)



**RecursiveAggregation**: verify a proof inside the circuit.
**Warning: this opcode is subject to change.**
- verification_key: Vector of (FieldElement, 254) representing the verification key of the circuit being verified
- public_inputs: Vector of (FieldElement, 254)  representing the public inputs corresponding to the proof being verified
- key_hash: one (FieldElement, 254). It should be the hash of the verification key. Barretenberg expects the Pedersen hash of the verification key
- input_aggregation_object: an optional vector of (FieldElement, 254). It is a blob of data specific to the proving system.
- output_aggregation_object: Some witnesses returned by the function, representing some data internal to the proving system.

This black box function does not fully verify a proof, what it does is verifying that the key_hash is indeed a hash of verification_key, allowing the user to use the verification key as private inputs and only have the key_hash as public input, which is more performant.
Another thing that it does is preparing the verification of the proof. In order to fully verify a proof, some operations may still be required to be done by the final verifier. This is why this black box function does not say if verification is passing or not.
If you have several proofs to verify in one ACIR program, you would call RecursiveAggregation() multiple times and passing the output_aggregation_object as input_aggregation_object to the next RecursiveAggregation() call, except for the first call where you do not have any input_aggregation_object.
If one of the proof you verify with the black box function does not verify, then the verification of the proof of the main ACIR program will ultimately fail.


### Brillig
This opcode is used as a hint for the solver when executing (solving) the circuit. The opcode does not generate any constraint and is usually the result of the compilation of an unconstrained noir function.
- inputs: inputs to the opcode, as 'arithmetic expressions'.
- outputs: opcode outputs, as witnesses
- bytecode: assembly code representing the computation to perform within this opcode. The noir assembly specification is not part of this document.
- predicate: an arithmetic expression that disable the opcode when it is null.

Let's see an example with euclidean division.
The normal way to compute a/b, where a and b are 8-bits integers, is to implement Euclid algorithm which computes in a loop (or recursively) modulus of the kind 'a mod b'. Doing this computation requires a lot of steps to be properly implemented in ACIR, especially the loop with a condition. However, euclidean division can be easily constrained with one assert-zero opcode: a = bq+r, assuming q is 8 bits and r<b. Since these assumptions can easily written with a few range opcodes, euclidean division can in fact be implemented with a small number of opcodes.

However, in order to write these opcodes we need the result of the division which are the witness q and r. But from the constraint a=bq+r, how can the solver figure out how to solve q and r with only one equation? This is where brillig/unconstrained function come into action. We simply define a function that performs the usual Euclid algorithm to compute q and r from a and b. Since Brillig opcode does not generate constraint, it won't be provided to the proving system but simply used by the solver to compute the values of q and r.

In summary, solving a Brillig opcode performs the computation defined by its bytecode, on the provided inputs, and assign the result to the outputs witnesses, without adding any constraint.

### Directive
This opcode is a specialization of Brillig opcode. Instead of having some generic assembly code like Brillig, a directive has a hardcoded name which tells the solver which computation to do: with Brillig, the computation refers to the compiled bytecode of an unconstrained Noir function, but with a directive, the computation is hardcoded inside the compiler. Directives will be replaced by Brillig opcodes in the future.

### MemoryOp: memory abstraction for ACIR
ACIR is able to address any array of witnesses. Each array is assigned an id (BlockId) and needs to be initialized with the MemoryInit opcode. Then it is possible to read and write from/to an array by providing the index and the value we read/write, as arithmetic expression. Note that ACIR arrays all have a known fixed length (given in the MemoryInit opcode below)
- block_id: identifier of the array
- op: describe the memory operation to perform
	- operation: constant expression having value 1 for a write to memory and 0 for a read
	- index: array index, it must be less than the array length
	- value: the value we are reading, when operation is 0, or the value we write at the specified index, when operation is 1
- predicate: an arithmetic expression that disable the opcode when it is null.

### MemoryInit
Initialize an ACIR array from a vector of witnesses.
- block_id: identifier of the array
- init: Vector of witnesses specifying the initial value of the arrays

There must be only one MemoryInit per block_id, and MemoryOp opcodes must come after the MemoryInit.

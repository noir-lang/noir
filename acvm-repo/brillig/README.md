# Brillig

## Intro to Brillig

Brillig is a general virtual machine architecture for usage with an NP complete circuit language that aims to incorporate unconstrained or non-deterministic functionality. For example, a language which compiles down to ACIR, can integrate unconstrained functions into its circuits by also compiling down to Brillig bytecode.

Why we need Brillig
---

Zero-knowledge (ZK) domain-specific languages (DSL) enable developers to generate ZK proofs from their programs by compiling code down to the constraints of an NP complete language (such as R1CS or PLONKish languages). However, the hard bounds of a constraint system can be very limiting to the functionality of a ZK DSL, and integrating a general VM is very useful for the following reasons:

1) Unconstrained execution

Enabling a circuit language to perform unconstrained execution is a powerful tool. Said another way, unconstrained execution lets developers generate witnesses from code that does not generate any constraints. Being able to execute logic outside of a circuit is critical for both circuit performance and constructing proofs on information that is external to a circuit.

For example, multiple encrypted notes are a common way to represent single private values.
When notes need to be presented for a proof, the notes selected need to be verified, but the exact choice is not fully constrained.
The note selection algorithm does not need a proof of execution, just the note outputs suffice to constrain the system.
We refer to these discretionary choice functions as unconstrained functions.
```
// Say we need to reveal and nullify certain notes to pass this check
// How can we formalize this partially constrained choice?
if get_balance() > min_amount {
    // ... perform some action ...
}
```

Fetching information from somewhere external to a circuit can also be used to enable developers to improve circuit efficiency.

For example, we may have a finite field which we want to transform to a byte array for use somewhere else in our circuit. To convert the field to a byte array in our circuit would require looping over the field's size in bytes and performing multiple bit operations. Such as in the pseudocode below where `x` is a finite field:

```
for i in 0..FIELD_SIZE_IN_BYTES {
    byte_array[i] = (x >> (8 * i)) & 0xff;
}
```
There are a couple problems with this approach.
1. Bit operations in circuits are very inefficient and require lots of constraints.
2. Finite fields are not inherently ordered, so any finite field `x` that we would want to perform a bit operation upon would have to be range constrained to some integer value. This ultimately defeats the purpose of being able to decompose any Field element to a byte array.

Instead we can write out arithmetic constraints inside of our circuit that maintains we have a valid byte array. For a 254 bit finite field `x` we can write out the following pseudocode:
```
assert(
    (byte_array[0]*2^0 + byte_array[1]*2^8 + ... + byte_array[31]*2^248) - x == 0
)
```
However, there is a problem with this pseudocode as it has been laid out so far. The statements above makes sense from the point of view of the verifier, but the prover does not know what the values of `byte_array` are implicitly. `byte_array[0]` could be full value of `x` while the rest of the byte array values are 0, or the array could be the valid byte array that we want. The prover must inject the correct values into the arithmetic constraint above. The prover can inject this information by performing the byte decomposition in an unconstrained environment.

A ZK DSL does not just prove computation but proves that some computation was handled correctly. Thus, it is necessary that when we switch from performing some operation directly inside of a circuit to inside of an unconstrained environment that the appropriate constraints are still laid down elsewhere in the circuit. The note selection algorithm example at the top of this section follows the same methodology. We are not constraining the unconstrained execution, but rather its outputs.

Brillig provides a way to evaluate such unconstrained functions in a safe, general VM.

2) Attributing incorrectness

Brillig's bytecode when compiling a ZK DSL into a program commitment must differentiate between runtime errors and simply invalid proofs. This proves useful in environments that rely on hedged trust and are distributed, such as blockchains.

In such systems, sequenced proofs of shared public state have unique trust assumptions. Proofs of correctness are not constructed by the user/transactor, creating complications. Two scenarios must be distinguished when executing functions on shared public state:

1. When a user attempts to call a public function via a sequencer but inputs data that causes the function to revert and the contract to fail.
2. When a transaction is valid, but the prover deliberately assigns incorrect values to the witnesses, leading to a failing proof for a legitimate transaction.

In private functions, where the user is also the prover, distinguishing between these scenarios isn't necessary. However, for public functions, it becomes vital to separate them to accurately identify potential malicious actors in hedged trust scenarios. In the first scenario, the responsibility lies with the user, whereas, in the second scenario, it's with the prover.

To ensure the protocol always demands valid proofs of knowledge, a virtual machine (VM) becomes necessary. Here, a VM operates as a zero-knowledge circuit that interprets a program. The VM sequentially executes program instructions and utilizes a public failure flag. In case of VM execution reverting, the failure flag is set to one, and on success, it is set to zero. Regardless of the outcome, an honest user will always have a means to create a valid proof.

Any general VM could technically be used to accomplish this task such as WASM or the EVM. However, most other general VMs were designed with other execution environments in mind. Proving time is expected to take up the majority of any ZK DSL program's execution, and Brillig's architecture was specifically designed with SNARK proving efficiency in mind.

## Brillig Architecture

Design principles
---

The focus of Brillig is on simplicity, safety, correctness and SNARK proving efficiency.

Finite Field VM
---

The Brillig VM operates over finite fields and supports using up to a 128 bit integer representation. The finite field Brillig supports is generalizable and based upon the field ACIR supports (where [Arkworks](https://github.com/arkworks-rs/algebra/tree/master/ff) is used for the finite field interface). The ACIR fields currently supported are the prime fields of the bn254 curve and the bls12-381 curve.

All integers ultimately translate to the finite field which Brillig is based upon. For example when a BinaryIntOp is used, the field is first cast to a fixed bitsize integer that can be accommodated within the field, prior to performing operations. Certain operations, such as ordered comparison or signed division, only make sense in the context of BinaryIntOp. The exact maximum integer value the field can store should not be relied upon, however, it is assured to be capable of packing 3 32-bit integers.

The decision to have Brillig operate over finite fields simplifies SNARK proving and optimizes for efficiency, as ultimately all data types translate to a finite field which is the native data type for SNARKs.

Bytecode Structure and Function
---
Brillig bytecode acts as an alternate compilation target for a domain specific circuit language. It will be explored in more detail in another section of this document. The bytecode is available both standalone and as part of ACIR in the form of the Brillig opcode. It primarily operates on fixed register indices but also includes dedicated memory access operations. Designed as a minimal register machine, it supports conditional jumps, a lightweight callstack, and can access a flat memory array of field element cells, thereby fulfilling the core requirements of a low-level VM.

The `MAX_REGISTERS` is 2**16.

Interfacing with Provers
---
During execution, when a backend integrated with ACVM encounters the Brillig opcode, it is expected to trust the outputs of the opcode. In the ACIR context, the Brillig opcode is intended for unconstrained functions, hence the assumption of not needing to prove execution. However, the Brillig opcode might be processed by a prover backend for reasons outlined in the "Attributing incorrectness" section. The specifics of this interaction reflect the inherent trust requirements and complexity of operating within a blockchain environment. In this case, a zkVM approach would be needed to prove execution.

Execution Model
---
The Brillig VM is a low-level register-based machine. It uses a program counter to step through its readonly bytecode data and consists of a callstack with just return addresses, in addition to register and memory spaces filled with field elements. Control flow operations are limited to a non-conditional jump, conditional jumps (checking if a register is zero), and a call operation that manipulates the callstack. There's also a foreign call instruction that allows any non-Brillig input to enter the system.

The VM accepts four initialization parameters.
1. An initial list of registers, represented by a list of field elements
2. The initial memory, represented by a list of field elements
3. The bytecode to be executed
4. A list of foreign call results. Each result is a list of field elements

The VM then processes each opcode according to their [specification](02_opcode_listing.md).

If the VM reaches a foreign call instruction it will first fetch the call's input according to the register information inside of the instruction. Through an internal counter and the foreign call results supplied to the VM, the VM will determine whether the outputs have been resolved. If they have not been resolved, the VM will then pause and return a status containing the foreign call inputs. The caller of the VM should interpret the call information returned and update the Brillig process. Execution can then continue as normal. While technically the foreign call result is considered part of the VM's input along with bytecode and initial registers, it is practically an input to the program.

Error and Exception Handling
---
Failed asserts, represented by the TRAP opcode, during the execution of an unconstrained function, result in an error in the Brillig opcode, accompanied by data detailing the failure. In a hedged trust blockchain environment, a prover might still want to generate a 'valid' proof of an error result so that incorrectness can be correctly attributed. This emphasizes the importance of handling errors and exceptions within the context of a blockchain-based VM.

Conclusion
---
The Brillig VM provides a flexible and efficient environment for executing unconstrained code in a circuit environment. By thoughtfully integrating with snark provers and blockchain technology, and with careful attention to error handling, Brillig fulfills a key role within the Noir programming ecosystem.

## Brillig Example

For the following Noir:

```
fn main() {
    let mut a = 10;
    let mut b = 5;
    let mut c = 0;

    c = a + b;

    if c <= 15 {
        a = a * b;
    } else {
        b = a + b;
    }

    print(a, b, c);
}
```

One possible Brillig output would be:

```
[
    { // location = 0
        "Const": {
            "destination": 0,
            "value": { "inner": "10" }
        }
    },
    { // location = 1
        "Const": {
            "destination": 1,
            "value": { "inner": "5" }
        }
    },
    { // location = 2
        "Const": {
            "destination": 2,
            "value": { "inner": "0" }
        }
    },
    { // location = 3
        "Const": {
            "destination": 3,
            "value": { "inner": "15" }
        }
    },
    { // location = 4
        "BinaryIntOp": {
            "destination": 2,
            "op": "Add",
            "bit_size": 32,
            "lhs": 0,
            "rhs": 1
        }
    },
    { // location = 5
        "BinaryIntOp": {
            "destination": 4,
            "op": "LessThanEquals",
            "bit_size": 32,
            "lhs": 2,
            "rhs": 3
        }
    },
    { // location = 6
        "JumpIfNot": {
            "condition": 4,
            "location": 9
        }
    },
    { // location = 7
        "BinaryFieldOp": {
            "destination": 0,
            "op": "Multiply",
            "lhs": 0,
            "rhs": 1
        }
    },
    { // location = 8
        "Jump": {
            "location": 10
        }
    },
    { // location = 9
        "BinaryFieldOp": {
            "destination": 1,
            "op": "Add",
            "lhs": 0,
            "rhs": 1
        }
    },
    { // location = 10
        "ForeignCall": {
            "function": "print",
            "destination": [], // No output
            "input": [
                {"RegisterIndex": 0},
                {"RegisterIndex": 1},
                {"RegisterIndex": 2}
            ]
        }
    },
    { // location = 11
        "Stop": {}
    }
]
```

The execution and interpretation of the program would be as follows:
1. The three `Const` instructions load values into reg0, reg1, reg2, reg3. These are variables a, b and c, and a temporary equal to 15
2. At location 4, let reg2 (c) equal reg0 plus reg1 (a + b)
3. Let reg4 (a temporary value) equal reg2 LessThanEquals reg3 (c <= 15)
4. If reg4 is 0 (so c > 15), jump to location 9 where we set b = a + b then go to location 10. Otherwise if c <= 15, we fall through to location 7, set a = a * b, and then go to location 10.
5. At location 10, we queue up inputs to a foreign call from reg0, reg1, reg2 (variables a, b, and c). This interrupts execution, calls to the outer system, and then returns to Brillig execution. If this had outputs, they might be written to registers and memory.
6. We finally reach the final location where we `Stop`. If this were a function to be called by another Brillig function, we would `Return`.
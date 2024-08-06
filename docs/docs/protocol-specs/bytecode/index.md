---
title: Bytecode
---

<!-- Mike review:
- Does Noir have detailed opcode specs for ACIR/Brillig? Where should we write specs for the bytecode, and the encodings of the bytecode, and the compression algorithms for the bytecode, and for the artifacts?

(Alvaro) Maybe we should write a spec of Acir and Brillig? I think noir doesn't have one. Currently the encoding is shared between the acvm_repo and barretenberg, it's using bincode + gzip and serde-reflect to make it cross-language. It feels weird that aztec specs out Acir and Brillig.

- We mention compressing bytecode. We should specify how compression is encoded (because all nodes will need to follow a compression and decompression standard). Is there a prefix to convey the type & version of compression that's been used? How is the length of a blob of bytecode conveyed?

(Alvaro) Palla will add how the length and other properties of the bytecode are broadcasted through the registrar contract but basically it's putting the length as the first field before the serialized bytecode starts. The serialized bytecode is the one outputted by Circuit::serialize in the acvm_repo.


- This section needs to align with some descriptions in the `contract-deployment/*` section. It might be worth liaising with Palla, to see which section should describe the following: hashing of bytecode (how is it hashed), committing to bytecode (how is it committed-to (although perhaps this belongs in the avm section)), the desired contents of an artifact, how is an artifact hashed? And for the artifact, for a particular function, what information is included?

(Alvaro)
 - Hashing of bytecode => It's done in the deployment section, I have linked to it in the relevant section.
 - Commiting to bytecode => Will follow up on this, I see a KZG commitment in the AVM section but I'm not sure that will be verified in the registrar contract.
 - Desired contents of an artifact => I have added a section with the desired contents of an artifact.
 - How is an artifact hashed? => I have added a link to the deployment section that specifies it.
 - For the artifact, for a particular function, what information is included? => I have added a section with the desired contents of an artifact that includes this information.


-->

This section describes how contracts are represented within the protocol for execution.

In the context of Aztec, a contract is a set of functions which can be of one of three types:

- Private functions: The functions that run on user's machines. They are circuits that must be individually executed by the [ACVM](https://github.com/noir-lang/noir/blob/master/acvm-repo/acvm/src/pwg/mod.rs#L132) and proved by barretenberg.
- Public functions: The functions that are run by sequencers. They are aggregated in a bytecode block that must be executed and proven by the AVM.
- Unconstrained functions: Helper functions that are run on users' machines but are not constrained. They are represented individually as bytecode that is executed by the ACVM.
  - Unconstrained functions are often used to fetch and serialize private data, for use as witnesses to a circuit.
  - They can also be used to convey how dapps should handle a particular contract's data.

When a contract is compiled, private and unconstrained functions are compiled individually. Public functions are compiled together to a single bytecode with an initial dispatch table based on function selectors. Since public functions are run in a VM, we do not incur a huge extra proving cost for the branching that is required to execute different functions.

If a private function needs unconstrained hints, the bytecode that generates the unconstrained hints is embedded in the private circuit. This allows the ACVM to compute the hints during witness generation.

There are three different (but related) bytecode standards that are used in Aztec: AVM bytecode, Brillig bytecode and ACIR bytecode.

# AVM Bytecode

The AVM bytecode is the compilation target of the public functions of a contract. It's specified in the [AVM section](../public-vm/instruction-set.mdx). It allows control flow and uses a flat memory model which tracks bit sizes of values stored in memory via tagging of memory indexes. Sequencers run the AVM bytecode of the public functions of a contract using the AVM and prove the correct execution of it.

# Brillig Bytecode

Brillig bytecode is the compilation target of all the unconstrained code in a contract. Any unconstrained hint used by a private function is compiled to Brillig bytecode. Also, contracts' top level unconstrained functions are entirely compiled to Brillig bytecode. In the case of Noir, it compiles public functions entirely to a single block of brillig bytecode that is then converted to AVM bytecode. Similarly to AVM bytecode, Brillig bytecode allows control flow.

Brillig bytecode will be very similar to AVM bytecode. While AVM bytecode is specifically designed to be executed by the AVM, brillig bytecode is meant to be more general and allow the use of arbitrary oracles.

Oracles allow nondeterminism during the execution of a given function, allowing the simulator entity to choose the value that an oracle will return during the simulation process. Oracles are heavily used by aztec.nr to fetch data during simulation of private and unconstrained functions, such as fetching notes. They are also used to notify the simulator about events arising during execution, such as a nullified note so that it's not offered again during the simulation.

However, AVM bytecode doesn't allow arbitrary oracles, any nondeterminism introduced is done in a way that the protocol can ensure that the simulator entity (the sequencer) cannot manipulate the result of an oracle. As such, when transforming brillig bytecode to AVM bytecode, all the oracles are replaced by the specific opcodes that the AVM supports for nondeterminism, like [TIMESTAMP](../public-vm/instruction-set.mdx#isa-section-timestamp), [ADDRESS](../public-vm/instruction-set.mdx#isa-section-address), etc. Any opcode that requires the simulator entity to provide data external to the AVM memory is non-deterministic.

The current implementation of Brillig can be found [in the noir repository](https://github.com/noir-lang/noir/blob/master/acvm-repo/brillig/src/opcodes.rs#L60). It's actively being changed to become "AVM bytecode without arbitrary oracles" and right now the differences are handled by a transpiler.

# ACIR Bytecode

ACIR bytecode is the compilation target of contract private functions. ACIR expresses arithmetic circuits and thus has no control flow. Control flow in regular functions is either unrolled (for loops) or flattened (by inlining and adding predicates), resulting in a single function with no control flow to be transformed to ACIR.

The types of opcodes that can appear in ACIR are:

- Arithmetic: They can express any degree-2 multivariate relation between witness indices. They are the most common opcodes in ACIR.
- BlackBoxFuncCall: They assign the witnesses of the parameters and the witnesses of the return values of black box functions. Black box functions are commonly used operations that are treated as a black box, meaning that the underlying backend chooses how to prove them efficiently.
- Brillig: This opcode contains a block of brillig bytecode, witness indices of the parameters and witness indices of the return values. When ACIR bytecode needs an unconstrained hint, the bytecode that is able to generate the hint at runtime is embedded in a Brillig opcode, and the result of running the hint is assigned to the return witnesses specified in the opcode. The simulator entity is the one responsible for executing the brillig bytecode. The results of the execution of the function are assigned to the witnesses of the return values and they should be constrained to be correct by the ACIR bytecode.
- MemoryOp: They handle memory operations. When accessing arrays with indices unknown at compile time, the compiler cannot know which witness index is being read. The memory abstraction allows acir to read and write to dynamic positions in arrays in an efficient manner, offloading the responsibility of proving the correct access to the underlying backend.

This implies that a block of ACIR bytecode can represent more than one program, since it can contain any number of Brillig opcodes each one containing a full Brillig program that computes a hint that the circuit needs at runtime.

# Usage of the bytecode

## Compiling a contract

When a contract is compiled, an artifact will be generated. This artifact needs to be hashed in a specific manner [detailed in the deployment section](../contract-deployment/classes.md#artifact-hash) for publishing.

The exact form of the artifact is not specified by the protocol, but it needs at least the following information:

### Contract artifact

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `name` | `string` | The name of the contract. |
| `compilerVersion` | `string` | Version of the compiler that generated the bytecode. This is a string to convey extra information like the version of Aztec.nr used. |
| `functions` | [FunctionEntry[]](#function-entry) | The functions of the contract. |
| `publicBytecode` | `string` | The AVM bytecode of the public functions, converted to base64. |
| `events` | [EventAbi[]](#event-abi) | The events of the contract. |

### Event ABI

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `name` | `string` | The event name. |
| `fields` | [ABIVariable](#abi-variable) | The fields of the event. |

### Function entry

If the function is public, the entry will be its ABI. If the function is private or unconstrained, the entry will be the ABI + the artifact.

### Function artifact

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `bytecode` | `string` | The ACIR bytecode of the function, converted to base64. |

### Function ABI

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `name` | `string` | The name of the function. |
| `functionType` | `string` | `private`, `public` or `unconstrained`. |
| `parameters` | [ABIParameter[]](#abi-parameter) | Function parameters. |
| `returnTypes` | `AbiType[]` | The types of the return values. |

### ABI Variable

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `name` | `string` | The name of the variable. |
| `type` | [AbiType](#abi-type) | The type of the variable. |

### ABI Parameter

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `name` | `string` | The name of the variable. |
| `type` | [AbiType](#abi-type) | The type of the variable. |
| `visibility` | `string` | `public` or `secret`. |

### ABI Type

<!-- prettier-ignore -->
| Field | Type | Description |
|----------|----------|----------|
| `kind` | `string` | `field`, `boolean`, `integer`, `array`, `string` or `struct` |
| `sign?` | `string` | The sign of the integer. Applies to integers only. |
| `width?` | `number` | The width of the integer in bits. Applies to integers only. |
| `length?` | `number` | The length of the array or string. Applies to arrays and strings only. |
| `type?` | [AbiType](#abi-type) | The types of the array elements. Applies to arrays only. |
| `fields?` | [ABIVariable[]](#abi-variable) | the fields of the struct. Applies to structs only. |

### Bytecode in the artifact

The protocol mandates that public bytecode needs to be published to a data availability solution, since the sequencers need to have the data available to run the public functions. Also, it needs to use an encoding that is friendly to the public VM, such as the one specified in the [AVM section](../public-vm/bytecode-validation-circuit.md).

The bytecode of private and unconstrained functions doesn't need to be published, instead, users that desire to use a given contract can add the artifact to their PXE before interacting with it. Publishing it is [supported but not required](../contract-deployment/classes.md#broadcast) by the protocol. However, the verification key of a private function is hashed into the function's leaf of the contract's function tree, so the user can prove to the protocol that he executed the function correctly. Also, contract classes contain an [artifact hash](../contract-deployment/classes.md#artifact-hash) so the PXE can verify that the artifact corresponds with the contract class.

The encoding of private and unconstrained functions is not specified by the protocol, but it's recommended to follow [the encoding](https://github.com/noir-lang/noir/blob/master/acvm-repo/acir/src/circuit/mod.rs#L157) that Barretenberg and the ACVM share that is serialization using bincode and gzip for compression.

This implies that the encoding of private and unconstrained functions does not need to be friendly to circuits, since when publishing it the protocol only sees a [generic array of field elements](../contract-deployment/classes.md#broadcast).

## Executing a private function

When executing a private function, its ACIR bytecode will be executed by the PXE using the ACVM. The ACVM will generate the witness of the execution. The proving system can be used to generate a proof of the correctness of the witness.

The fact that the correct function was executed is checked by the protocol by verifying that the [contract class ID](../contract-deployment/classes.md#class-identifier) contains one leaf in the function tree with this selector and the verification key of the function.

## Executing an unconstrained function

When executing an unconstrained function, its Brillig bytecode will be executed by the PXE using the ACVM, similarly to private functions, but the PXE will not prove the execution. Instead, the PXE will return the result of the execution of the function to the user.

## Executing a public function

When executing a public function, its AVM bytecode will be executed by the sequencer with the specified selector and arguments. The sequencer will generate a public VM proof of the correct execution of the AVM bytecode.

The fact that the correct bytecode was executed is checked by the protocol by verifying that the [contract class ID](../contract-deployment/classes.md#class-identifier) contains the [commitment](../public-vm/bytecode-validation-circuit.md#committed-representation) to the bytecode used.

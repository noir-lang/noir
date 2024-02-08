---
title: Bytecode
---

<!-- Mike review:
- Do we need to mention Noir? Is the bytecode inextricably tethered to Noir?
- Does Noir have detailed opcode specs for ACIR/Brillig? Where should we write specs for the bytecode, and the encodings of the bytecode, and the compression algorithms for the bytecode, and for the artifacts?
- Please can we write a struct which formally details the data that must be contained within an "artifact". This doc should specify the data we want, and the codebase(s) should work towards this spec.
- We mention compressing bytecode. We should specify how compression is encoded (because all nodes will need to follow a compression and decompression standard). Is there a prefix to convey the type & version of compression that's been used? How is the length of a blob of bytecode conveyed?
- I mention a couple of times below: If a function contains both private and unconstrained logic, what does the bytecode look like? Is it a concatenation of all the logic? How is it encoded? Similar question for a public function making an unconstrained call.

- This section needs to align with some descriptions in the `contract-deployment/*` section. It might be worth liaising with Palla, to see which section should describe the following: broadcasting bytecode (how is it encoded), hashing of bytecode (how is it hashed), committing to bytecode (how is it committed-to (although perhaps this belongs in the avm section)), the desired contents of an artifact, how is an artifact hashed? And for the artifact, for a particular function, what information is included?
-->

This section describes how contracts are represented within the protocol for execution.

In the context of Aztec, a contract is a set of functions which can be of one of three types:

- Private functions: The functions that run on user's machines. At the noir level, they are regular programs.
- Public functions: The functions that are run by sequencers. At the noir level, they are unconstrained functions, that are later proven by the public VM.
- Unconstrained functions: Helper functions that are run on users' machines but are not constrained. At the noir level, they are top level unconstrained functions.
  - Unconstrained functions are often used to fetch and serialize private data, for use as witnesses to a circuit.
  - They can also be used to convey how dapps should handle a particular contract's data.

When a contract is compiled, private and unconstrained functions are compiled individually. Public functions are compiled together to a single bytecode with an initial dispatch table based on function selectors. Since public functions are run in a VM, we do not incur a huge extra proving cost for the branching that is required to execute different functions.

<!-- What if a private function makes an unconstrained call - is the entire function not just compiled "as one"? The bytecode of a private function will need to convey the unconstrained computations that the PXE must perform. -->
<!-- Cam public functions make unconstrained calls? Is the public and unconstrained bytecode amalgamated? -->

There are three different (but related) bytecode standards that are used in Aztec: AVM bytecode, Brillig bytecode and ACIR bytecode.

# AVM Bytecode

The AVM bytecode is the compilation target of the public functions of a contract. It's specified in the [AVM section](../public-vm/instruction-set). It allows control flow and uses a flat memory model which tracks bit sizes of values stored in memory via tagging of memory indexes. Sequencers run the AVM bytecode of the public functions of a contract using the public VM and prove the correct execution of it.

# Brillig Bytecode

Brillig bytecode is the compilation target of all the unconstrained functions in noir. Any unconstrained function used by a private function is compiled to Brillig bytecode. Also, contracts' top level unconstrained functions are entirely compiled to Brillig bytecode.

Brillig bytecode will be a thin superset <!-- Is it true that it'll be a "superset", or might there be some AVM opcodes which are not matched by a Brillig opcode? --> of AVM bytecode that allows for the use of oracles. Oracles allow nondeterminism during the execution of a given function, allowing the simulator entity to choose the value that an oracle will return during the simulation process. Oracles are heavily used by aztec.nr to fetch data during simulation of private and unconstrained functions, such as fetching notes. They are also used to notify the simulator about events arising during execution, such as a nullified note so that it's not offered again during the simulation. Similarly to AVM bytecode, Brillig bytecode allows control flow.

The current implementation of Brillig can be found [in the noir repository](https://github.com/noir-lang/noir/blob/master/acvm-repo/brillig/src/opcodes.rs#L60). It'll change when the specification of AVM bytecode is finished to become a superset of it.

# ACIR Bytecode

ACIR bytecode is the compilation target of all regular noir programs, including contract private functions. ACIR expresses arithmetic circuits and thus has no control flow. Control flow in regular functions is either unrolled (for loops) or flattened (by inlining and adding predicates), resulting in a single function with no control flow to be transformed to ACIR.

The types of opcodes that can appear in ACIR are:

- Arithmetic: They can express any degree-2 multivariate relation between witness indices. They are the most common opcodes in ACIR.
- BlackBoxFuncCall: They assign the witnesses of the parameters and the witnesses of the return values of black box functions. Black box functions are commonly used operations that are treated as a black box, meaning that the underlying backend chooses how to prove them efficiently.
- Brillig: They assign the witnesses of the parameters and the witnesses of the return values of brillig functions. When an unconstrained function is called from a regular function, the bytecode for the called function gets embedded in a Brillig opcode <!-- please could you expand on what this means? -->. The simulator entity is the one responsible for executing the brillig bytecode. The results of the execution of the function are assigned to the witnesses of the return values and they should be constrained to be correct by the ACIR bytecode.
- MemoryOp: They handle memory operations. When accessing arrays with indices unknown at compile time, the compiler cannot know which witness index is being read. The memory abstraction allows noir to read and write to dynamic positions in arrays in an efficient manner, offloading the responsibility of proving the correct access to the underlying backend.

# Usage of the bytecode

### Compiling a contract

<!-- Mike: See my top-of-page review comments. Let's formally spec the artifact. -->

When a contract is compiled, an artifact will be generated containing:

- The private functions compiled to ACIR bytecode. The verification key of the private functions can be generated from the ACIR bytecode.
- The unconstrained functions compiled to Brillig bytecode.
- A public bytecode blob containing the bytecode of all the public functions compiled to AVM bytecode.

The public bytecode <!-- and sometimes private bytecode, and sometimes unconstrained bytecode - see the `contract-deployment/instances.md section. --> needs to be published to a data availability solution, since the sequencers need to have the data available to run the public functions. Also, it needs to use an encoding that is friendly to the public VM, such as the one specified in the [AVM section](../public-vm/bytecode-validation-circuit).

The bytecode of private and unconstrained functions doesn't need to be published, instead, users that desire to use a given contract can add the artifact to their PXE before interacting with it. Publishing it is convenient, but not required <!-- explain that private bytecode and unconstrained bytecode _can_ be published, though. In fact, for `compute_note_hash_and_nullifier`, this bytecode must be published. Maybe link to the `contract-deployment/instances.md` section. For some contracts, it might be important to publish private bytecode to L1. EDIT: hmmm chat with Palla about whether private bytecode actually will be publishable to L1, because he's done some more recent thinking about this. -->. However, the verification key of a private function is hashed into the function's leaf of the contract's function tree, so the user can prove to the protocol that he executed the function correctly.

This implies that the encoding of private and unconstrained functions does not need to be friendly to circuits, since the only thing the protocol tracks is the verification key, allowing to use compression techniques.

### Executing a private function

When executing a private function, its ACIR bytecode will be executed by the PXE using the ACVM. The ACVM will generate the witness of the execution. The proving system can be used to generate a proof of the correctness of the witness.

### Executing an unconstrained function

When executing an unconstrained function, its Brillig bytecode will be executed by the PXE using the ACVM, similarly to private functions, but the PXE will not prove the execution. Instead, the PXE will return the result of the execution of the function to the user.

### Executing a public function

When executing a public function, its AVM bytecode will be executed by the sequencer with the specified selector and arguments. The sequencer will generate a public VM proof of the correct execution of the AVM bytecode.

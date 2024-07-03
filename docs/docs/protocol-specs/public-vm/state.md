# State

This section describes the types of state maintained by the AVM.

## Machine State

**Machine state** is transformed on an instruction-per-instruction basis. Each execution context has its own machine state.

### _MachineState_

| Field                 | Type            | Description |
| ---                   | ---             | ---         |
| `l2GasLeft`           | `field`         | Tracks the amount of L2 gas remaining at any point during execution. Initialized from contract call arguments. |
| `daGasLeft`           | `field`         | Tracks the amount of DA gas remaining at any point during execution. Initialized from contract call arguments. |
| `pc`                  | `field`         | Index into the contract's bytecode indicating which instruction to execute. Initialized to 0 during context initialization. |
| `internalCallStack`   | `Vector<field>` | A stack of program counters pushed to and popped from by `INTERNALCALL` and `INTERNALRETURN` instructions. Initialized as empty during context initialization. |
| `memory`              | `[field; 2^32]` | A $2^{32}$ entry memory space accessible by user code (AVM instructions). All $2^{32}$ entries are assigned default value 0 during context initialization. See ["Memory Model"](./memory-model) for a complete description of AVM memory. |

<!-- TODO(4608): formally define memory's type - not just an array of fields, but tagged... -->

## World State

### AVM's access to Aztec State

[Aztec's global state](../state) is implemented as a few merkle trees. These trees are exposed to the AVM as follows:

| State             | Tree                  | Merkle Tree Type | AVM Access |
| ---               | ---                   | ---              | ---        |
| Public Storage    | Public Data Tree      | Updatable        | membership-checks (latest), reads, writes   |
| Note Hashes       | Note Hash Tree        | Append-only      | membership-checks (start-of-block), appends |
| Nullifiers        | Nullifier Tree        | Indexed          | membership-checks (latest), appends         |
| L1-to-L2 Messages | L1-to-L2 Message Tree | Append-only      | membership-checks (start-of-block)          |
| Headers           | Archive Tree          | Append-only      | membership-checks, leaf-preimage-reads      |
| Contracts\*       | -                     | -                | - |

> \* As described in ["Contract Deployment"](../contract-deployment), contracts are not stored in a dedicated tree. A [contract class](../contract-deployment/classes) is [represented](../contract-deployment/classes#registration) as an unencrypted log containing the `ContractClass` structure (which contains the bytecode) and a nullifier representing the class identifier. A [contract instance](../contract-deployment/instances) is [represented](../contract-deployment/classes#registration) as an unencrypted log containing the `ContractInstance` structure and a nullifier representing the contract address.

### AVM World State

The AVM does not directly modify Aztec state. Instead, it stages modifications to be applied later pending successful execution. As part of each execution context, the AVM maintains **world state** which is a representation of Aztec state that includes _staged_ modifications.

As the AVM executes contract code, instructions may read or modify the world state within the current context. If execution within a particular context reverts, staged world state modifications are rejected by the caller. If execution succeeds, staged world state modifications are accepted.

#### _AvmWorldState_

The following table defines an AVM context's world state interface:

| Field            | AVM Instructions & Access |
| ---              | ---                       |
| `contracts`      | [`*CALL`](./instruction-set#isa-section-call) (special case, see below\*)           |
| `publicStorage`  | [`SLOAD`](./instruction-set#isa-section-sload) (membership-checks (latest) & reads), [`SSTORE`](./instruction-set#isa-section-sstore) (writes)                                |
| `noteHashes`     | [`NOTEHASHEXISTS`](./instruction-set#isa-section-notehashexists) (membership-checks (start-of-block)), [`EMITNOTEHASH`](./instruction-set#isa-section-emitnotehash) (appends) |
| `nullifiers`     | [`NULLIFIERSEXISTS`](./instruction-set#isa-section-nullifierexists) membership-checks (latest), [`EMITNULLIFIER`](./instruction-set#isa-section-emitnullifier) (appends)      |
| `l1ToL2Messages` | [`L1TOL2MSGEXISTS`](./instruction-set#isa-section-l1tol2msgexists) (membership-checks (start-of-block))                                                                       |
| `headers`        | [`HEADERMEMBER`](./instruction-set#isa-section-headermember) (membership-checks & leaf-preimage-reads)                                                                        |

> \* `*CALL` is short for `CALL`/`STATICCALL`/`DELEGATECALL`.

> \* For the purpose of the AVM, the world state's `contracts` member is readable for [bytecode fetching](./execution#bytecode-fetch-and-decode), and it is effectively updated when a new contract class or instance is created (along with a nullifier for the contract class identifier or contract address).

### World State Access Trace

**The circuit implementation of the AVM does _not_ prove that its world state accesses are valid and properly sequenced**, and does not perform actual tree updates. Thus, _all_ world state accesses, **regardless of whether they are rejected due to a revert**, must be traced and eventually handed off to downstream circuits (public kernel and rollup circuits) for comprehensive validation and tree updates.

This trace of an AVM session's contract calls and world state accesses is named the **world state access trace**.

> The world state access trace is also important for enforcing limitations on the maximum number of allowable world state accesses.

#### _WorldStateAccessTrace_

Each entry in the world state access trace is listed below along with its type and the instructions that append to it:

| Field                 | Relevant State    | Type                               | Instructions           |
| ---                   | ---               | ---                                | ---                    |
| `accessCounter`       | all state         | `field`                            | incremented by all instructions below |
| `contractCalls`       | Contracts         | `Vector<TracedContractCall>`       | [`*CALL`](./instruction-set#isa-section-call)                       |
| `publicStorageReads`  | Public Storage    | `Vector<TracedStorageRead>`        | [`SLOAD`](./instruction-set#isa-section-sload)                      |
| `publicStorageWrites` | Public Storage    | `Vector<TracedStorageWrite>`       | [`SSTORE`](./instruction-set#isa-section-sstore)                    |
| `noteHashChecks`      | Note Hashes       | `Vector<TracedNoteHashCheck>`      | [`NOTEHASHEXISTS`](./instruction-set#isa-section-notehashexists)    |
| `noteHashes`       | Note Hashes       | `Vector<TracedNoteHash>`           | [`EMITNOTEHASH`](./instruction-set#isa-section-emitnotehash)        |
| `nullifierChecks`     | Nullifiers        | `Vector<TracedNullifierCheck>`     | [`NULLIFIERSEXISTS`](./instruction-set#isa-section-nullifierexists) |
| `nullifiers`       | Nullifiers        | `Vector<TracedNullifier>`          | [`EMITNULLIFIER`](./instruction-set#isa-section-emitnullifier)      |
| `l1ToL2MessageChecks` | L1-To-L2 Messages | `Vector<TracedL1ToL2MessageCheck>` | [`L1TOL2MSGEXISTS`](./instruction-set#isa-section-l1tol2msgexists)    |
| `archiveChecks`       | Headers           | `Vector<TracedArchiveLeafCheck>`   | [`HEADERMEMBER`](./instruction-set#isa-section-headermember)        |

> The types tracked in these trace vectors are defined [here](./type-structs).

> `*CALL` is short for `CALL`/`STATICCALL`/`DELEGATECALL`.

> Aztec tree operations like membership checks, appends, or leaf updates are performed in-circuit by downstream circuits (public kernel and rollup circuits), _after_ AVM execution. The world state access trace is a list of requests made by the AVM for later circuits to perform.

## Accrued Substate

> The term "accrued substate" is borrowed from the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf).

**Accrued substate** is accrued throughout a context's execution, but updates to it are strictly never relevant to subsequent instructions, contract calls, or transactions. An execution context is always initialized with empty accrued substate. Its vectors are append-only, and the instructions listed below append to these vectors. If a contract call's execution succeeds, its accrued substate is appended to the caller's. If a contract's execution reverts, its accrued substate is ignored.

#### _AccruedSubstate_

| Field                | Type                        | Instructions      |
| ---                  | ---                         | ---               |
| `unencryptedLogs`    | `Vector<UnencryptedLog>`    | [`EMITUNENCRYPTEDLOG`](./instruction-set#isa-section-emitunencryptedlog)          |
| `sentL2ToL1Messages` | `Vector<SentL2ToL1Message>` | [`SENDL1TOL2MSG`](./instruction-set#isa-section-sendl2tol1msg) |

> The types tracked in these vectors are defined [here](./type-structs).

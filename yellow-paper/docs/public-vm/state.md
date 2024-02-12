# State

This section describes the types of state maintained by the AVM.

## Machine State

**Machine state** is transformed on an instruction-per-instruction basis. Each execution context has its own machine state.

### _MachineState_

| Field                 | Type            | Description |
| ---                   | ---             | ---         |
| `l1GasLeft`           | `field`         | Tracks the amount of L1 gas remaining at any point during execution. |
| `l2GasLeft`           | `field`         | Tracks the amount of L2 gas remaining at any point during execution. |
| `daGasLeft`           | `field`         | Tracks the amount of DA gas remaining at any point during execution. |
| `pc`                  | `field`         | Index into the contract's bytecode indicating which instruction to execute. Initialized\* to 0. |
| `internalCallStack`   | `Vector<field>` | A stack of program counters pushed to and popped from by `INTERNALCALL` and `INTERNALRETURN` instructions. Initialized\* as empty. |
| `memory`              | `[field; 2^32]` | A $2^{32}$ entry memory space accessible by user code (bytecode instructions). All 2^32 entries are initialized\* to 0. See ["Memory Model"](./memory-model) for a complete description of AVM memory. |

## World State

### AVM's access to Aztec State
[Aztec's global state](../state) is implemented as a few merkle trees that are exposed to the AVM as follows:

| State             | Tree                  | Tree Type                             | AVM Access                                              |
| ---               | ---                   | ---                                   | ---                                                     |
| Note Hashes       | Note Hash Tree        | Append-only merkle tree               | membership-checks (start-of-block), appends             |
| Nullifiers        | Nullifier Tree        | Indexed merkle tree                   | membership-checks (latest), appends                     |
| L1-to-L2 Messages | L1-to-L2 Message Tree | Append-only (or indexed?) merkle tree | membership-checks (start-of-block), leaf-preimage-reads |
| Public Storage    | Public Data Tree      | Updatable merkle tree                 | membership-checks (latest), reads, writes       		  |
| Headers           | Archive Tree          | Append-only merkle tree               | membership-checks, leaf-preimage-reads                  |

> As described in ["Contract Deployment"](../contract-deployment), contracts are not stored in a dedicated tree. A [contract class](../contract-deployment/classes) is [represented](../contract-deployment/classes#registration) as an unencrypted log containing the `ContractClass` structure (which contains the bytecode) and a nullifier representing the class identifier. The `contractClasses` interface provides access to contract classes indexed by class identifier. A [contract instance](../contract-deployment/instances) is [represented](../contract-deployment/classes#registration) as an unencrypted log containing the `ContractInstance` structure and a nullifier representing the contract address. The `contractInstances` interface provides access to contract instances indexed by contract address.

### AVM World State

The AVM does not directly modify Aztec state. Instead, it stages modifications to be applied later pending successful execution. As part of each execution context, the AVM maintains **world state** which is a representation of Aztec state that includes _staged_ modifications.

As the AVM executes contract code, instructions may read or modify the world state within the current context. If execution within a particular context reverts, staged world state modifications are rejected by the caller. If execution succeeds, staged world state modifications are accepted.

#### _AvmWorldState_

The following table defines an AVM context's world state interface:

| Field            | AVM Instructions & Access                                                        |
| ---              | ---                                                                              |
| `noteHashes`     | `NOTEHASHEXISTS` (membership-checks (start-of-block)), `EMITNULLIFIER` (appends) |
| `nullifiers`     | `NULLIFIERSEXISTS` membership-checks (latest), `EMITNULLIFIER` (appends)         |
| `l1ToL2Messages` | `READL1TOL2MSG` (membership-checks (start-of-block) & leaf-preimage-reads)       |
| `publicStorage`  | `SLOAD` (membership-checks (latest) & reads), `SSTORE` (writes)                  |
| `headers`        | `HEADERMEMBER` (membership-checks & leaf-preimage-reads)                         |

### World State Access Trace

**The circuit implementation of the AVM does _not_ prove that its world state accesses are valid and properly sequenced**, and does not perform actual tree updates. Thus, _all_ world state accesses, **regardless of whether they are rejected due to a revert**, must be traced and eventually handed off to downstream circuits (public kernel and rollup circuits) for comprehensive validation and tree updates.

This trace of an AVM session's contract calls and world state accesses is named the **world state access trace**.

> The world state access trace is also important for enforcing limitations on the maximum number of allowable world state accesses.

#### _WorldStateAccessTrace_

Each entry in the world state access trace is listed below along with its type and the instructions that append to it:

| Trace                 | Relevant State    | Trace Vector Type                  | Instructions         |
| ---                   | ---               | ---                                | ---                  |
| `publicStorageReads`  | Public Storage    | `Vector<TracedStorageAccess>`      | `SLOAD`              |
| `publicStorageWrites` | Public Storage    | `Vector<TracedStorageAccess>`      | `SSTORE`             |
| `l1ToL2MessageReads`  | L1-To-L2 Messages | `Vector<TracedL1ToL2MessageRead>`  | `READL1TOL2MSG`      |
| `noteHashChecks`      | Note Hashes       | `Vector<TracedLeafCheck>`          | `NOTEHASHEXISTS`     |
| `newNoteHashes`       | Note Hashes       | `Vector<TracedNoteHash>`           | `EMITNOTEHASH`       |
| `nullifierChecks`     | Nullifiers        | `Vector<TracedIndexedLeafCheck>`   | `NULLIFIEREXISTS`    |
| `newNullifiers`       | Nullifiers        | `Vector<TracedNullifier>`          | `EMITNULLIFIER`      |
| `archiveChecks`       | Headers           | `Vector<TracedArchiveLeafCheck>`   | `HEADERMEMBER`       |
| `contractCalls`       | -                 | `Vector<TracedContractCall>`       | `*CALL`              |

> The types tracked in these trace vectors are defined [here](./type-structs).

> The syntax `*CALL` is short for `CALL`/`STATICCALL`/`DELEGATECALL`.

> Aztec tree operations like membership checks, appends, or leaf updates are performed in-circuit by downstream circuits (public kernel and rollup circuits), _after_ AVM execution. The world state access trace is a list of requests made by the AVM for later circuits to perform.

## Accrued Substate

> The term "accrued substate" is borrowed from the [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf).

**Accrued substate** is accrued throughout a context's execution, but updates to it are strictly never relevant to subsequent instructions, contract calls, or transactions. An execution context is always initialized with empty accrued substate, and instructions can append to its vectors which are _append-only_. If a contract call's execution succeeds, its accrued substate is appended to the caller's. If a contract's execution reverts, its accrued substate is ignored. There is no accrued substate "trace" that includes entries from reverted contexts.

#### _AccruedSubstate_
| Field                | Type                        | Instructions    | Description |
| ---                  | ---                         | ---             | ---         |
| `unencryptedLogs`    | `Vector<UnencryptedLog>`    | `ULOG`          |  |
| `sentL2ToL1Messages` | `Vector<SentL2ToL1Message>` | `SENDL1TOL2MSG` |  |

> The types tracked in these vectors are defined [here](./type-structs).

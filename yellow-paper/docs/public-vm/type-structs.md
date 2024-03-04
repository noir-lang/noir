# Type Definitions

This section lists type definitions relevant to AVM State and Circuit I/O.

#### _TracedContractCall_

| Field             | Type     | Description                  |
| ---               | ---      | ---                          |
| `callPointer`     | `field`  | The call pointer assigned to this call. |
| `address`         | `field`  | The called contract address. |
| `storageAddress`  | `field`  | The storage contract address (different from `address` for delegate calls). |
| `counter`         | `field`  | When did this occur relative to other world state accesses. |
| `endLifetime`     | `field`  | End lifetime of a call. Final `accessCounter` for reverted calls, `endLifetime` of parent for successful calls. Successful initial/top-level calls have infinite (max-value) `endLifetime`. |

#### _TracedL1ToL2MessageCheck_

| Field             | Type                                   | Description |
| ---               | ---                                    | ---         |
| `callPointer`     | `field`                                | Associates this item with a `TracedContractCall` entry in `worldStateAccessTrace.contractCalls` |
| `leafIndex`       | `field`                                |             |
| `msgHash`         | `field`                                | The message hash which is also the tree leaf value. |
| `exists`          | `field`                                |             |
| `endLifetime`     | `field`                                | Equivalent to `endLifetime` of the containing contract call. |

#### _TracedStorageRead_

| Field                | Type           | Description |
| ---                  | ---            | ---         |
| `callPointer`        | `field`        | Associates this item with a `TracedContractCall` entry in `worldStateAccessTrace.contractCalls`|
| `slot`               | `field`        |             |
| `exists`             | `field`        | Whether this slot has ever been previously written |
| `value`              | `field`        |             |
| `counter`            | `field`        |             |
| `endLifetime`        | `field`        | Equivalent to `endLifetime` of the containing contract call. The last `counter` at which this read/write should be considered to "exist" if this call or a parent reverted. |

#### _TracedStorageWrite_

| Field                | Type           | Description |
| ---                  | ---            | ---         |
| `callPointer`        | `field`        | Associates this item with a `TracedContractCall` entry in `worldStateAccessTrace.contractCalls`|
| `slot`               | `field`        |             |
| `value`              | `field`        |             |
| `counter`            | `field`        |             |
| `endLifetime`        | `field`        | Equivalent to `endLifetime` of the containing contract call. The last `counter` at which this read/write should be considered to "exist" if this call or a parent reverted. |

#### _TracedNoteHashCheck_

| Field                | Type           | Description |
| ---                  | ---            | ---         |
| `callPointer`        | `field`        | Associates this item with a `TracedContractCall` entry in `worldStateAccessTrace.contractCalls` |
| `leafIndex`          | `field`        |             |
| `noteHash`           | `field`        | unsiloed    |
| `exists`             | `field`        |             |
| `counter`            | `field`        |             |
| `endLifetime`        | `field`        | Equivalent to `endLifetime` of the containing contract call. |

#### _TracedNoteHash_

| Field                | Type           | Description |
| ---                  | ---            | ---         |
| `callPointer`        | `field`        | Associates this item with a `TracedContractCall` entry in `worldStateAccessTrace.contractCalls` |
| `noteHash`           | `field`        |             |
| `counter`            | `field`        |             |
| `endLifetime`        | `field`        | Equivalent to `endLifetime` of the containing contract call. The last `counter` at which this object should be considered to "exist" if this call or a parent reverted. |

> Note: `value` here is not siloed by contract address nor is it made unique with a nonce. Note hashes are siloed and made unique by the public kernel.

#### _TracedNullifierCheck_

| Field                | Type           | Description |
| ---                  | ---            | ---         |
| `callPointer`        | `field`        | Associates this item with a `TracedContractCall` entry in `worldStateAccessTrace.contractCalls` |
| `nullifier`          | `field`        | unsiloed    |
| `exists`             | `field`        |             |
| `counter`            | `field`        |             |
| `endLifetime`        | `field`        | Equivalent to `endLifetime` of the containing contract call. |

#### _TracedNullifier_

| Field                | Type           | Description |
| ---                  | ---            | ---         |
| `callPointer`        | `field`        | Associates this item with a `TracedContractCall` entry in `worldStateAccessTrace.contractCalls` |
| `nullifier`          | `field`        |             |
| `counter`            | `field`        |             |
| `endLifetime`        | `field`        | Equivalent to `endLifetime` of the containing contract call. The last `counter` at which this object should be considered to "exist" if this call or a parent reverted. |

#### _TracedArchiveLeafCheck_

| Field         | Type           | Description |
| ---           | ---            | ---         |
| `leafIndex`   | `field`        |             |
| `leaf`        | `field`        |             |

#### _UnencryptedLog_

| Field     | Type                                  | Description |
| ---       | ---                                   | ---         |
| `address` | `AztecAddress`                        | Contract address that emitted the log. |
| `log`     | `[field; MAX_UNENCRYPTED_LOG_LENGTH]` |             |

#### _SentL2ToL1Message_

| Field       | Type           | Description |
| ---         | ---            | ---         |
| `address`   | `AztecAddress` | L2 contract address that emitted the message. |
| `recipient` | `EthAddress`   | L1 contract address to send the message to.  |
| `content`   | `field`        | Message content. |

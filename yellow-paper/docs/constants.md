---
title: Constants
---

:::warning Draft
All of these constants are subject to change, pending benchmarking, optimizations, and general protocol changes.
:::

## Tree Constants

See also: [state](./state/index.md).

:::warning Tree Epochs
Note: we might introduce tree epochs, which will reduce the height of each epoch's tree, and means we don't need to estimate future network state growth in advance.
:::

<!-- prettier-ignore -->
| Name | Value | Description |
|---|---|---|
| `ARCHIVE_TREE_HEIGHT` | `27` | Prudent justification: 1 block/min \* 200 years ~= 2^27 blocks |
| `NOTE_HASH_TREE_HEIGHT` | `39` | Prudent justification: 10 tx/s \* 8 notes/tx \* 200 years. |
| `NULLIFIER_TREE_HEIGHT` | `42` | Prudent justification: \[Number of notes _ 2 (to allow a huge buffer for initialization nullifiers and other nullifier usage)] + \[ 2 _ Estimated number of contracts (to allow a huge buffer for contract class & instance nullifiers) ]. An estimate for the number of contracts ever to be deployed is debatable. |
| `PUBLIC_DATA_TREE_HEIGHT` | `39` | Prudent justification: 10 tx/s \* 8 storage slots/tx \* 200 years. |
| `L1_TO_L2_MESSAGE_TREE` | `33` | Prudent justification: 10 tx/s \* 10% of txs consuming a message \* 200 years. |
| `PRIVATE_FUNCTION_TREE_HEIGHT` | `5` | Note: this means a smart contract can only declare `2 ** 5 = 32` private functions. |

For all trees, an empty leaf has value `0`.

For all indexed merkle trees, the 0th leaf is the "zero predecessor leaf" with leaf preimage `{ value: 0, next_index: 0, next_value: 0}`. <!-- TODO: link to the explanation of what the preimage is in the indexed merkle tree section instead -->

## Circuit Constants

:::warning
Note: "per call" values might be much more flexible, once the data bus is introduced. These numbers are finger-in-the-air estimates of values that might be possible with the data bus. Benchmarking will be needed.
:::

The statically-sized nature the kernel & rollup circuits will restrict the quantity of 'side effects' that a single call or transaction can create.

### Per Call

<!-- prettier-ignore -->
| Name | Value | Description |
|---|---|---|
| `RETURN_VALUES_LENGTH` | 4 |
| `MAX_NEW_NOTE_HASHES_PER_CALL` | 128 |
| `MAX_NEW_NULLIFIERS_PER_CALL` | 128 |
| `MAX_NEW_L2_TO_L1_MSGS_PER_CALL` | 4 |
| `MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL` | 128 |
| `MAX_PUBLIC_DATA_READS_PER_CALL` | 128 |
| `MAX_UNENCRYPTED_LOG_HASHES_PER_CALL` | 128 |
| `MAX_ENCRYPTED_LOG_HASHES_PER_CALL` | 128 |
| `MAX_ENCRYPTED_NOTE_PREIMAGE_HASHES_PER_CALL` | 128 |
| `MAX_NOTE_HASH_READ_REQUESTS_PER_CALL` | 128 |
| `MAX_NULLIFIER_READ_REQUESTS_PER_CALL` | 128 |
| `MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_CALL` | 1 | TODO: we shouldn't need this, given the reset circuit. |
| `MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL` | 32 |
| `MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL` | 32 |

### Per Tx

<!-- prettier-ignore -->
| Name | Value | Description |
|---|---|---|
| `MAX_NEW_NOTE_HASHES_PER_TX` | 128 |
| `MAX_NEW_NULLIFIERS_PER_TX` | 128 |
| `MAX_NEW_L2_TO_L1_MSGS_PER_TX` | 16 |
| `MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX` | 16 |
| `MAX_PUBLIC_DATA_READS_PER_TX` | 16 |
| `MAX_UNENCRYPTED_LOG_HASHES_PER_TX` | 128 |
| `MAX_ENCRYPTED_LOG_HASHES_PER_TX` | 128 |
| `MAX_ENCRYPTED_NOTE_PREIMAGE_HASHES_PER_TX` | 128 |
| `MAX_OPTIONALLY_REVEALED_DATA_LENGTH_PER_TX` | 4 |
| `MAX_NOTE_HASH_READ_REQUESTS_PER_TX` | 128 | TODO: we shouldn't need this, given the reset circuit. |
| `MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX` | 4 | TODO: we shouldn't need this, given the reset circuit. |
| `MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX` | 32 |
| `MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX` | 32 |

## Block constants

## Genesis Constants

### Genesis Addresses

:::warning
TODO: consider whether these addresses should be 'nice', small values, or whether these addresses should be derived in the same way as all other addresses (but with a deployer_address of `0x00`).

TODO: some of these contracts will be baked into genesis. Some of them might need to be deployed as part of the first network 'upgrade', and so might end up being removed from this section. This is still being discussed.
:::

| Name                                    | Value   | Description                                            |
| --------------------------------------- | ------- | ------------------------------------------------------ |
| Space reserved for precompile addresses |         |                                                        |
| `CONTRACT_CLASS_REGISTERER_ADDRESS`     | 0x10000 | See [here](./contract-deployment/classes.md#genesis)   |
| `CONTRACT_INSTANCE_DEPLOYER_ADDRESS`    | 0x10001 | See [here](./contract-deployment/instances.md#genesis) |
| `FEE_JUICE_ADDRESS`                     | 0x10002 | TODO: consider at what stage this should be deployed.  |

### Genesis Archive Tree

The 0th leaf of the archive tree will be hard-coded at genesis to be an empty collection of state trees and an empty previous header.

<!-- TODO: Jan expand on this, please. -->

### Genesis Nullifiers

| Name                                                                                 | Value | Description                                           |
| ------------------------------------------------------------------------------------ | ----- | ----------------------------------------------------- |
| The zero predecessor leaf.                                                           | TODO  | Needed to make an indexed merkle tree work.           |
| The zero predecessor leaf index.                                                     | `0`   | Needed to make an indexed merkle tree work.           |
| `GENESIS_NULLIFIER_LEAF_INDEX_OF_CLASS_ID_NULLIFIER_OF_CONTRACT_CLASS_REGISTERER`    | `1`   | See [here](./contract-deployment/classes.md#genesis). |
| `GENESIS_NULLIFIER_LEAF_INDEX_OF_DEPLOYMENT_NULLIFIER_OF_CONTRACT_CLASS_REGISTERER`  | `2`   | See [here](./contract-deployment/classes.md#genesis). |
| `GENESIS_NULLIFIER_LEAF_INDEX_OF_CLASS_ID_NULLIFIER_OF_CONTRACT_INSTANCE_DEPLOYER`   | `3`   | See [here](./contract-deployment/classes.md#genesis). |
| `GENESIS_NULLIFIER_LEAF_INDEX_OF_DEPLOYMENT_NULLIFIER_OF_CONTRACT_INSTANCE_DEPLOYER` | `4`   | See [here](./contract-deployment/classes.md#genesis). |
| `GENESIS_NULLIFIER_LEAF_INDEX_OF_CLASS_ID_NULLIFIER_OF_FEE_JUICE_CONTRACT`           | `5`   | See [here](./contract-deployment/classes.md#genesis). |
| `GENESIS_NULLIFIER_LEAF_INDEX_OF_DEPLOYMENT_NULLIFIER_OF_FEE_JUICE_CONTRACT`         | `6`   | See [here](./contract-deployment/classes.md#genesis). |

:::warning
TODO: hard-code the actual nullifier values, once the code has been frozen.
:::

<!-- TODO: Palla, do we need an 'initialisation nullifier' for all of these genesis contracts too? -->

These verbose names are designed to get more specific from left to right.

## Precompile Constants

See the [precompiles](./addresses-and-keys/precompiles.md#constants) section.

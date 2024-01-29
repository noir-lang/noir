---
title: Registry
---

The registry is a contract deployed on L1, that contains addresses for the `Rollup`, `Inbox` and `Outbox`. It also keeps track of the different versions that have been deployed and let you query prior deployments easily.

**Links**: [Interface](https://github.com/AztecProtocol/aztec-packages/blob/master/l1-contracts/src/core/interfaces/messagebridge/IRegistry.sol), [Implementation](https://github.com/AztecProtocol/aztec-packages/blob/master/l1-contracts/src/core/messagebridge/Registry.sol).

## `numberOfVersions()`

Retrieves the number of versions that have been deployed.

#include_code registry_number_of_versions l1-contracts/src/core/interfaces/messagebridge/IRegistry.sol solidity

| Name           | Description |
| -------------- | ----------- |
| ReturnValue    | The number of versions that have been deployed |

## `getRollup()`
Retrieves the current rollup contract.

#include_code registry_get_rollup l1-contracts/src/core/interfaces/messagebridge/IRegistry.sol solidity

| Name           | Description |
| -------------- | ----------- |
| ReturnValue    | The current rollup |

## `getInbox()`

Retrieves the current inbox contract.

#include_code registry_get_inbox l1-contracts/src/core/interfaces/messagebridge/IRegistry.sol solidity

| Name           | Description |
| -------------- | ----------- |
| ReturnValue    | The current Inbox |

## `getOutbox()`

Retrieves the current inbox contract.

#include_code registry_get_outbox l1-contracts/src/core/interfaces/messagebridge/IRegistry.sol solidity

| Name           | Description |
| -------------- | ----------- |
| ReturnValue    | The current Outbox |

## `getVersionFor(address _rollup)`

Retrieve the version of a specific rollup contract. 

#include_code registry_get_version_for l1-contracts/src/core/interfaces/messagebridge/IRegistry.sol solidity

| Name           | Description |
| -------------- | ----------- |
| `_rollup`      | The address of the rollup to lookup |
| ReturnValue    | The version number of `_rollup` |

#### Edge cases
Will revert with `Registry__RollupNotRegistered(_rollup)` if the rollup have not been registered.

## `getSnapshot(uint256 _version)`

Retrieve the snapshot of a specific version. 

#include_code registry_snapshot l1-contracts/src/core/libraries/DataStructures.sol solidity
#include_code registry_get_snapshot l1-contracts/src/core/interfaces/messagebridge/IRegistry.sol solidity

| Name           | Description |
| -------------- | ----------- |
| `_version`     | The version number to fetch data for |
| ReturnValue.rollup      | The address of the `rollup` for the `_version` |
| ReturnValue.inbox       | The address of the `inbox` for the `_version` |
| ReturnValue.outbox      | The address of the `outbox` for the `_version` |
| ReturnValue.blockNumber | The block number of the snapshot creation |


## `getCurrentSnapshot()`

Retrieves the snapshot for the current version.

#include_code registry_get_current_snapshot l1-contracts/src/core/interfaces/messagebridge/IRegistry.sol solidity

| Name           | Description |
| -------------- | ----------- |
| ReturnValue.rollup      | The address of the `rollup` for the current `_version` |
| ReturnValue.inbox       | The address of the `inbox` for the current `_version` |
| ReturnValue.outbox      | The address of the `outbox` for the current `_version` |
| ReturnValue.blockNumber | The block number of the snapshot creation |


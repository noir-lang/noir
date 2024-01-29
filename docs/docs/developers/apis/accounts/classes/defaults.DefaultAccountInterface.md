---
id: "defaults.DefaultAccountInterface"
title: "Class: DefaultAccountInterface"
sidebar_label: "DefaultAccountInterface"
custom_edit_url: null
---

[defaults](../modules/defaults.md).DefaultAccountInterface

Default implementation for an account interface. Requires that the account uses the default
entrypoint signature, which accepts an EntrypointPayload as defined in noir-libs/aztec-noir/src/entrypoint.nr.

## Implements

- `AccountInterface`

## Constructors

### constructor

• **new DefaultAccountInterface**(`authWitnessProvider`, `address`, `nodeInfo`): [`DefaultAccountInterface`](defaults.DefaultAccountInterface.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `authWitnessProvider` | `AuthWitnessProvider` |
| `address` | `CompleteAddress` |
| `nodeInfo` | `Pick`\<`NodeInfo`, ``"chainId"`` \| ``"protocolVersion"``\> |

#### Returns

[`DefaultAccountInterface`](defaults.DefaultAccountInterface.md)

## Properties

### address

• `Private` **address**: `CompleteAddress`

___

### authWitnessProvider

• `Private` **authWitnessProvider**: `AuthWitnessProvider`

___

### entrypoint

• `Private` **entrypoint**: `EntrypointInterface`

## Methods

### createAuthWitness

▸ **createAuthWitness**(`message`): `Promise`\<`AuthWitness`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `message` | `Fr` |

#### Returns

`Promise`\<`AuthWitness`\>

#### Implementation of

AccountInterface.createAuthWitness

___

### createTxExecutionRequest

▸ **createTxExecutionRequest**(`executions`): `Promise`\<`TxExecutionRequest`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `executions` | `FunctionCall`[] |

#### Returns

`Promise`\<`TxExecutionRequest`\>

#### Implementation of

AccountInterface.createTxExecutionRequest

___

### getCompleteAddress

▸ **getCompleteAddress**(): `CompleteAddress`

#### Returns

`CompleteAddress`

#### Implementation of

AccountInterface.getCompleteAddress

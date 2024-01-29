---
id: "defaults.DefaultAccountEntrypoint"
title: "Class: DefaultAccountEntrypoint"
sidebar_label: "DefaultAccountEntrypoint"
custom_edit_url: null
---

[defaults](../modules/defaults.md).DefaultAccountEntrypoint

Implementation for an entrypoint interface that follows the default entrypoint signature
for an account, which accepts an EntrypointPayload as defined in noir-libs/aztec-noir/src/entrypoint.nr.

## Implements

- `EntrypointInterface`

## Constructors

### constructor

• **new DefaultAccountEntrypoint**(`address`, `auth`, `chainId?`, `version?`): [`DefaultAccountEntrypoint`](defaults.DefaultAccountEntrypoint.md)

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `address` | `AztecAddress` | `undefined` |
| `auth` | `AuthWitnessProvider` | `undefined` |
| `chainId` | `number` | `DEFAULT_CHAIN_ID` |
| `version` | `number` | `DEFAULT_VERSION` |

#### Returns

[`DefaultAccountEntrypoint`](defaults.DefaultAccountEntrypoint.md)

## Properties

### address

• `Private` **address**: `AztecAddress`

___

### auth

• `Private` **auth**: `AuthWitnessProvider`

___

### chainId

• `Private` **chainId**: `number` = `DEFAULT_CHAIN_ID`

___

### version

• `Private` **version**: `number` = `DEFAULT_VERSION`

## Methods

### createTxExecutionRequest

▸ **createTxExecutionRequest**(`executions`): `Promise`\<`TxExecutionRequest`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `executions` | `FunctionCall`[] |

#### Returns

`Promise`\<`TxExecutionRequest`\>

#### Implementation of

EntrypointInterface.createTxExecutionRequest

___

### getEntrypointAbi

▸ **getEntrypointAbi**(): `FunctionAbi`

#### Returns

`FunctionAbi`

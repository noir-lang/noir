---
id: "account.AccountInterface"
title: "Interface: AccountInterface"
sidebar_label: "AccountInterface"
custom_edit_url: null
---

[account](../modules/account.md).AccountInterface

Handler for interfacing with an account. Knows how to create transaction execution
requests and authorize actions for its corresponding account.

## Hierarchy

- [`AuthWitnessProvider`](account.AuthWitnessProvider.md)

- [`EntrypointInterface`](account.EntrypointInterface.md)

  ↳ **`AccountInterface`**

## Methods

### createAuthWitness

▸ **createAuthWitness**(`message`): `Promise`\<`AuthWitness`\>

Create an authorization witness for the given message.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `message` | `Fr` | Message to authorize. |

#### Returns

`Promise`\<`AuthWitness`\>

#### Inherited from

[AuthWitnessProvider](account.AuthWitnessProvider.md).[createAuthWitness](account.AuthWitnessProvider.md#createauthwitness)

___

### createTxExecutionRequest

▸ **createTxExecutionRequest**(`executions`): `Promise`\<`TxExecutionRequest`\>

Generates an authenticated request out of set of function calls.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `executions` | `FunctionCall`[] | The execution intents to be run. |

#### Returns

`Promise`\<`TxExecutionRequest`\>

The authenticated transaction execution request.

#### Inherited from

[EntrypointInterface](account.EntrypointInterface.md).[createTxExecutionRequest](account.EntrypointInterface.md#createtxexecutionrequest)

___

### getCompleteAddress

▸ **getCompleteAddress**(): `CompleteAddress`

Returns the complete address for this account.

#### Returns

`CompleteAddress`

---
id: "account.EntrypointInterface"
title: "Interface: EntrypointInterface"
sidebar_label: "EntrypointInterface"
custom_edit_url: null
---

[account](../modules/account.md).EntrypointInterface

Creates transaction execution requests out of a set of function calls.

## Hierarchy

- **`EntrypointInterface`**

  ↳ [`AccountInterface`](account.AccountInterface.md)

## Methods

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

---
id: "account.AuthWitnessProvider"
title: "Interface: AuthWitnessProvider"
sidebar_label: "AuthWitnessProvider"
custom_edit_url: null
---

[account](../modules/account.md).AuthWitnessProvider

Creates authorization witnesses.

## Hierarchy

- **`AuthWitnessProvider`**

  ↳ [`AccountInterface`](account.AccountInterface.md)

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

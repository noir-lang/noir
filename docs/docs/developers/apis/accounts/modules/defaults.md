---
id: "defaults"
title: "Module: defaults"
sidebar_label: "defaults"
sidebar_position: 0
custom_edit_url: null
---

The `@aztec/accounts/defaults` export provides the base class [DefaultAccountContract](../classes/defaults.DefaultAccountContract.md) for implementing account contracts that use the default entrypoint payload module.

Read more in [Writing an account contract](https://docs.aztec.network/developers/wallets/writing_an_account_contract).

## Classes

- [DefaultAccountContract](../classes/defaults.DefaultAccountContract.md)
- [DefaultAccountEntrypoint](../classes/defaults.DefaultAccountEntrypoint.md)
- [DefaultAccountInterface](../classes/defaults.DefaultAccountInterface.md)

## Type Aliases

### EntrypointPayload

Ƭ **EntrypointPayload**: `Object`

Encoded payload for the account contract entrypoint

#### Type declaration

| Name | Type | Description |
| :------ | :------ | :------ |
| `function_calls` | `EntrypointFunctionCall`[] | Encoded function calls to execute |
| `nonce` | `Fr` | A nonce for replay protection |

## Variables

### DEFAULT\_CHAIN\_ID

• `Const` **DEFAULT\_CHAIN\_ID**: ``31337``

Default L1 chain ID to use when constructing txs (matches hardhat and anvil's default).

___

### DEFAULT\_VERSION

• `Const` **DEFAULT\_VERSION**: ``1``

Default protocol version to use.

## Functions

### buildPayload

▸ **buildPayload**(`calls`): `Object`

Assembles an entrypoint payload from a set of private and public function calls

#### Parameters

| Name | Type |
| :------ | :------ |
| `calls` | `FunctionCall`[] |

#### Returns

`Object`

| Name | Type | Description |
| :------ | :------ | :------ |
| `packedArguments` | `PackedArguments`[] | The packed arguments of functions called |
| `payload` | [`EntrypointPayload`](defaults.md#entrypointpayload) | The payload for the entrypoint function |

___

### hashPayload

▸ **hashPayload**(`payload`): `Buffer`

Hashes an entrypoint payload to a 32-byte buffer (useful for signing)

#### Parameters

| Name | Type |
| :------ | :------ |
| `payload` | [`EntrypointPayload`](defaults.md#entrypointpayload) |

#### Returns

`Buffer`

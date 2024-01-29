---
id: "contract.BatchCall"
title: "Class: BatchCall"
sidebar_label: "BatchCall"
custom_edit_url: null
---

[contract](../modules/contract.md).BatchCall

A batch of function calls to be sent as a single transaction through a wallet.

## Hierarchy

- `BaseContractInteraction`

  ↳ **`BatchCall`**

## Constructors

### constructor

• **new BatchCall**(`wallet`, `calls`): [`BatchCall`](contract.BatchCall.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `wallet` | [`Wallet`](../modules/account.md#wallet) |
| `calls` | `FunctionCall`[] |

#### Returns

[`BatchCall`](contract.BatchCall.md)

#### Overrides

BaseContractInteraction.constructor

## Properties

### calls

• `Protected` **calls**: `FunctionCall`[]

___

### pxe

• `Protected` **pxe**: `PXE`

#### Inherited from

BaseContractInteraction.pxe

___

### tx

• `Protected` `Optional` **tx**: `Tx`

#### Inherited from

BaseContractInteraction.tx

___

### txRequest

• `Protected` `Optional` **txRequest**: `TxExecutionRequest`

#### Inherited from

BaseContractInteraction.txRequest

___

### wallet

• `Protected` **wallet**: [`Wallet`](../modules/account.md#wallet)

## Methods

### create

▸ **create**(): `Promise`\<`TxExecutionRequest`\>

Create a transaction execution request that represents this batch, encoded and authenticated by the
user's wallet, ready to be simulated.

#### Returns

`Promise`\<`TxExecutionRequest`\>

A Promise that resolves to a transaction instance.

#### Overrides

BaseContractInteraction.create

___

### send

▸ **send**(`options?`): [`SentTx`](contract.SentTx.md)

Sends a transaction to the contract function with the specified options.
This function throws an error if called on an unconstrained function.
It creates and signs the transaction if necessary, and returns a SentTx instance,
which can be used to track the transaction status, receipt, and events.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options` | [`SendMethodOptions`](../modules/contract.md#sendmethodoptions) | An optional object containing 'from' property representing the AztecAddress of the sender. If not provided, the default address is used. |

#### Returns

[`SentTx`](contract.SentTx.md)

A SentTx instance for tracking the transaction status and information.

#### Inherited from

BaseContractInteraction.send

___

### simulate

▸ **simulate**(`options?`): `Promise`\<`Tx`\>

Simulates a transaction execution request and returns a tx object ready to be sent.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options` | [`SendMethodOptions`](../modules/contract.md#sendmethodoptions) | optional arguments to be used in the creation of the transaction |

#### Returns

`Promise`\<`Tx`\>

The resulting transaction

#### Inherited from

BaseContractInteraction.simulate

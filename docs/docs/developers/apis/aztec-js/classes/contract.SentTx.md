---
id: "contract.SentTx"
title: "Class: SentTx"
sidebar_label: "SentTx"
custom_edit_url: null
---

[contract](../modules/contract.md).SentTx

The SentTx class represents a sent transaction through the PXE, providing methods to fetch
its hash, receipt, and mining status.

## Hierarchy

- **`SentTx`**

  ↳ [`DeploySentTx`](contract.DeploySentTx.md)

## Constructors

### constructor

• **new SentTx**(`pxe`, `txHashPromise`): [`SentTx`](contract.SentTx.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `pxe` | `PXE` |
| `txHashPromise` | `Promise`\<`TxHash`\> |

#### Returns

[`SentTx`](contract.SentTx.md)

## Properties

### pxe

• `Protected` **pxe**: `PXE`

___

### txHashPromise

• `Protected` **txHashPromise**: `Promise`\<`TxHash`\>

## Methods

### getReceipt

▸ **getReceipt**(): `Promise`\<`TxReceipt`\>

Retrieve the transaction receipt associated with the current SentTx instance.
The function fetches the transaction hash using 'getTxHash' and then queries
the PXE to get the corresponding transaction receipt.

#### Returns

`Promise`\<`TxReceipt`\>

A promise that resolves to a TxReceipt object representing the fetched transaction receipt.

___

### getTxHash

▸ **getTxHash**(): `Promise`\<`TxHash`\>

Retrieves the transaction hash of the SentTx instance.
The function internally awaits for the 'txHashPromise' to resolve, and then returns the resolved transaction hash.

#### Returns

`Promise`\<`TxHash`\>

A promise that resolves to the transaction hash of the SentTx instance.

___

### getUnencryptedLogs

▸ **getUnencryptedLogs**(): `Promise`\<`GetUnencryptedLogsResponse`\>

Gets unencrypted logs emitted by this tx.

#### Returns

`Promise`\<`GetUnencryptedLogsResponse`\>

The requested logs.

**`Remarks`**

This function will wait for the tx to be mined if it hasn't been already.

___

### getVisibleNotes

▸ **getVisibleNotes**(): `Promise`\<`ExtendedNote`[]\>

Get notes of accounts registered in the provided PXE/Wallet created in this tx.

#### Returns

`Promise`\<`ExtendedNote`[]\>

The requested notes.

**`Remarks`**

This function will wait for the tx to be mined if it hasn't been already.

___

### wait

▸ **wait**(`opts?`): `Promise`\<`FieldsOf`\<`TxReceipt`\>\>

Awaits for a tx to be mined and returns the receipt. Throws if tx is not mined.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `opts?` | [`WaitOpts`](../modules/contract.md#waitopts) | Options for configuring the waiting for the tx to be mined. |

#### Returns

`Promise`\<`FieldsOf`\<`TxReceipt`\>\>

The transaction receipt.

___

### waitForReceipt

▸ **waitForReceipt**(`opts?`): `Promise`\<`TxReceipt`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `opts?` | [`WaitOpts`](../modules/contract.md#waitopts) |

#### Returns

`Promise`\<`TxReceipt`\>

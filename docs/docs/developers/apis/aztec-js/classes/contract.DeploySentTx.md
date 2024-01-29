---
id: "contract.DeploySentTx"
title: "Class: DeploySentTx<TContract>"
sidebar_label: "DeploySentTx"
custom_edit_url: null
---

[contract](../modules/contract.md).DeploySentTx

A contract deployment transaction sent to the network, extending SentTx with methods to create a contract instance.

## Type parameters

| Name | Type |
| :------ | :------ |
| `TContract` | extends [`Contract`](contract.Contract.md) = [`Contract`](contract.Contract.md) |

## Hierarchy

- [`SentTx`](contract.SentTx.md)

  ↳ **`DeploySentTx`**

## Constructors

### constructor

• **new DeploySentTx**\<`TContract`\>(`wallet`, `txHashPromise`, `postDeployCtor`, `completeContractAddress?`): [`DeploySentTx`](contract.DeploySentTx.md)\<`TContract`\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `TContract` | extends [`Contract`](contract.Contract.md) = [`Contract`](contract.Contract.md) |

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `wallet` | `PXE` \| [`Wallet`](../modules/account.md#wallet) | - |
| `txHashPromise` | `Promise`\<`TxHash`\> | - |
| `postDeployCtor` | (`address`: `AztecAddress`, `wallet`: [`Wallet`](../modules/account.md#wallet)) => `Promise`\<`TContract`\> | - |
| `completeContractAddress?` | `CompleteAddress` | The complete address of the deployed contract |

#### Returns

[`DeploySentTx`](contract.DeploySentTx.md)\<`TContract`\>

#### Overrides

[SentTx](contract.SentTx.md).[constructor](contract.SentTx.md#constructor)

## Properties

### completeContractAddress

• `Optional` **completeContractAddress**: `CompleteAddress`

The complete address of the deployed contract

___

### postDeployCtor

• `Private` **postDeployCtor**: (`address`: `AztecAddress`, `wallet`: [`Wallet`](../modules/account.md#wallet)) => `Promise`\<`TContract`\>

#### Type declaration

▸ (`address`, `wallet`): `Promise`\<`TContract`\>

##### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `AztecAddress` |
| `wallet` | [`Wallet`](../modules/account.md#wallet) |

##### Returns

`Promise`\<`TContract`\>

___

### pxe

• `Protected` **pxe**: `PXE`

#### Inherited from

[SentTx](contract.SentTx.md).[pxe](contract.SentTx.md#pxe)

___

### txHashPromise

• `Protected` **txHashPromise**: `Promise`\<`TxHash`\>

#### Inherited from

[SentTx](contract.SentTx.md).[txHashPromise](contract.SentTx.md#txhashpromise)

## Methods

### deployed

▸ **deployed**(`opts?`): `Promise`\<`TContract`\>

Awaits for the tx to be mined and returns the contract instance. Throws if tx is not mined.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `opts?` | [`DeployedWaitOpts`](../modules/contract.md#deployedwaitopts) | Options for configuring the waiting for the tx to be mined. |

#### Returns

`Promise`\<`TContract`\>

The deployed contract instance.

___

### getContractInstance

▸ **getContractInstance**(`wallet?`, `address?`): `Promise`\<`TContract`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `wallet?` | [`Wallet`](../modules/account.md#wallet) |
| `address?` | `AztecAddress` |

#### Returns

`Promise`\<`TContract`\>

___

### getReceipt

▸ **getReceipt**(): `Promise`\<`TxReceipt`\>

Retrieve the transaction receipt associated with the current SentTx instance.
The function fetches the transaction hash using 'getTxHash' and then queries
the PXE to get the corresponding transaction receipt.

#### Returns

`Promise`\<`TxReceipt`\>

A promise that resolves to a TxReceipt object representing the fetched transaction receipt.

#### Inherited from

[SentTx](contract.SentTx.md).[getReceipt](contract.SentTx.md#getreceipt)

___

### getTxHash

▸ **getTxHash**(): `Promise`\<`TxHash`\>

Retrieves the transaction hash of the SentTx instance.
The function internally awaits for the 'txHashPromise' to resolve, and then returns the resolved transaction hash.

#### Returns

`Promise`\<`TxHash`\>

A promise that resolves to the transaction hash of the SentTx instance.

#### Inherited from

[SentTx](contract.SentTx.md).[getTxHash](contract.SentTx.md#gettxhash)

___

### getUnencryptedLogs

▸ **getUnencryptedLogs**(): `Promise`\<`GetUnencryptedLogsResponse`\>

Gets unencrypted logs emitted by this tx.

#### Returns

`Promise`\<`GetUnencryptedLogsResponse`\>

The requested logs.

**`Remarks`**

This function will wait for the tx to be mined if it hasn't been already.

#### Inherited from

[SentTx](contract.SentTx.md).[getUnencryptedLogs](contract.SentTx.md#getunencryptedlogs)

___

### getVisibleNotes

▸ **getVisibleNotes**(): `Promise`\<`ExtendedNote`[]\>

Get notes of accounts registered in the provided PXE/Wallet created in this tx.

#### Returns

`Promise`\<`ExtendedNote`[]\>

The requested notes.

**`Remarks`**

This function will wait for the tx to be mined if it hasn't been already.

#### Inherited from

[SentTx](contract.SentTx.md).[getVisibleNotes](contract.SentTx.md#getvisiblenotes)

___

### wait

▸ **wait**(`opts?`): `Promise`\<[`DeployTxReceipt`](../modules/contract.md#deploytxreceipt)\<`TContract`\>\>

Awaits for the tx to be mined and returns the receipt along with a contract instance. Throws if tx is not mined.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `opts?` | [`DeployedWaitOpts`](../modules/contract.md#deployedwaitopts) | Options for configuring the waiting for the tx to be mined. |

#### Returns

`Promise`\<[`DeployTxReceipt`](../modules/contract.md#deploytxreceipt)\<`TContract`\>\>

The transaction receipt with the deployed contract instance.

#### Overrides

[SentTx](contract.SentTx.md).[wait](contract.SentTx.md#wait)

___

### waitForReceipt

▸ **waitForReceipt**(`opts?`): `Promise`\<`TxReceipt`\>

#### Parameters

| Name | Type |
| :------ | :------ |
| `opts?` | [`WaitOpts`](../modules/contract.md#waitopts) |

#### Returns

`Promise`\<`TxReceipt`\>

#### Inherited from

[SentTx](contract.SentTx.md).[waitForReceipt](contract.SentTx.md#waitforreceipt)

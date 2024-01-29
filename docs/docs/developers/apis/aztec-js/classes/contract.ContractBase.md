---
id: "contract.ContractBase"
title: "Class: ContractBase"
sidebar_label: "ContractBase"
custom_edit_url: null
---

[contract](../modules/contract.md).ContractBase

Abstract implementation of a contract extended by the Contract class and generated contract types.

## Hierarchy

- **`ContractBase`**

  ↳ [`Contract`](contract.Contract.md)

## Implements

- `DeployedContract`

## Constructors

### constructor

• **new ContractBase**(`completeAddress`, `artifact`, `wallet`, `portalContract`): [`ContractBase`](contract.ContractBase.md)

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `completeAddress` | `CompleteAddress` | The deployed contract's complete address. |
| `artifact` | `ContractArtifact` | The Application Binary Interface for the contract. |
| `wallet` | [`Wallet`](../modules/account.md#wallet) | The wallet used for interacting with this contract. |
| `portalContract` | `EthAddress` | The portal contract address on L1, if any. |

#### Returns

[`ContractBase`](contract.ContractBase.md)

## Properties

### artifact

• `Readonly` **artifact**: `ContractArtifact`

The Application Binary Interface for the contract.

#### Implementation of

DeployedContract.artifact

___

### completeAddress

• `Readonly` **completeAddress**: `CompleteAddress`

The deployed contract's complete address.

#### Implementation of

DeployedContract.completeAddress

___

### methods

• **methods**: `Object` = `{}`

An object containing contract methods mapped to their respective names.

#### Index signature

▪ [name: `string`]: [`ContractMethod`](../modules/contract.md#contractmethod)

___

### portalContract

• `Readonly` **portalContract**: `EthAddress`

The portal contract address on L1, if any.

#### Implementation of

DeployedContract.portalContract

___

### wallet

• `Protected` **wallet**: [`Wallet`](../modules/account.md#wallet)

The wallet used for interacting with this contract.

## Accessors

### address

• `get` **address**(): `AztecAddress`

Address of the contract.

#### Returns

`AztecAddress`

## Methods

### withWallet

▸ **withWallet**(`wallet`): `this`

Creates a new instance of the contract wrapper attached to a different wallet.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `wallet` | [`Wallet`](../modules/account.md#wallet) | Wallet to use for sending txs. |

#### Returns

`this`

A new contract instance.

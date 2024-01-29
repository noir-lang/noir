---
id: "contract.DeployMethod"
title: "Class: DeployMethod<TContract>"
sidebar_label: "DeployMethod"
custom_edit_url: null
---

[contract](../modules/contract.md).DeployMethod

Creates a TxRequest from a contract ABI, for contract deployment.
Extends the ContractFunctionInteraction class.

## Type parameters

| Name | Type |
| :------ | :------ |
| `TContract` | extends [`ContractBase`](contract.ContractBase.md) = [`Contract`](contract.Contract.md) |

## Hierarchy

- `BaseContractInteraction`

  ↳ **`DeployMethod`**

## Constructors

### constructor

• **new DeployMethod**\<`TContract`\>(`publicKey`, `pxe`, `artifact`, `postDeployCtor`, `args?`): [`DeployMethod`](contract.DeployMethod.md)\<`TContract`\>

#### Type parameters

| Name | Type |
| :------ | :------ |
| `TContract` | extends [`ContractBase`](contract.ContractBase.md) = [`Contract`](contract.Contract.md) |

#### Parameters

| Name | Type | Default value |
| :------ | :------ | :------ |
| `publicKey` | `Point` | `undefined` |
| `pxe` | `PXE` | `undefined` |
| `artifact` | `ContractArtifact` | `undefined` |
| `postDeployCtor` | (`address`: `AztecAddress`, `wallet`: [`Wallet`](../modules/account.md#wallet)) => `Promise`\<`TContract`\> | `undefined` |
| `args` | `any`[] | `[]` |

#### Returns

[`DeployMethod`](contract.DeployMethod.md)\<`TContract`\>

#### Overrides

BaseContractInteraction.constructor

## Properties

### args

• `Private` **args**: `any`[] = `[]`

___

### artifact

• `Private` **artifact**: `ContractArtifact`

___

### completeAddress

• `Optional` **completeAddress**: `CompleteAddress` = `undefined`

The complete address of the contract.

___

### constructorArtifact

• `Private` **constructorArtifact**: `FunctionArtifact`

Constructor function to call.

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

### publicKey

• `Private` **publicKey**: `Point`

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

## Methods

### create

▸ **create**(`options?`): `Promise`\<`TxExecutionRequest`\>

Create a contract deployment transaction, given the deployment options.
This function internally calls `request()` and `sign()` methods to prepare
the transaction for deployment. The resulting signed transaction can be
later sent using the `send()` method.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options` | [`DeployOptions`](../modules/contract.md#deployoptions) | An object containing optional deployment settings, including portalContract, contractAddressSalt, and from. |

#### Returns

`Promise`\<`TxExecutionRequest`\>

A Promise resolving to an object containing the signed transaction data and other relevant information.

#### Overrides

BaseContractInteraction.create

___

### send

▸ **send**(`options?`): [`DeploySentTx`](contract.DeploySentTx.md)\<`TContract`\>

Send the contract deployment transaction using the provided options.
This function extends the 'send' method from the ContractFunctionInteraction class,
allowing us to send a transaction specifically for contract deployment.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options` | [`DeployOptions`](../modules/contract.md#deployoptions) | An object containing various deployment options such as portalContract, contractAddressSalt, and from. |

#### Returns

[`DeploySentTx`](contract.DeploySentTx.md)\<`TContract`\>

A SentTx object that returns the receipt and the deployed contract instance.

#### Overrides

BaseContractInteraction.send

___

### simulate

▸ **simulate**(`options`): `Promise`\<`Tx`\>

Simulate the request.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `options` | [`DeployOptions`](../modules/contract.md#deployoptions) | Deployment options. |

#### Returns

`Promise`\<`Tx`\>

The simulated tx.

#### Overrides

BaseContractInteraction.simulate

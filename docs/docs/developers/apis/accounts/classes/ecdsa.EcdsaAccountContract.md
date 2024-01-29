---
id: "ecdsa.EcdsaAccountContract"
title: "Class: EcdsaAccountContract"
sidebar_label: "EcdsaAccountContract"
custom_edit_url: null
---

[ecdsa](../modules/ecdsa.md).EcdsaAccountContract

Account contract that authenticates transactions using ECDSA signatures
verified against a secp256k1 public key stored in an immutable encrypted note.

## Hierarchy

- [`DefaultAccountContract`](defaults.DefaultAccountContract.md)

  ↳ **`EcdsaAccountContract`**

## Constructors

### constructor

• **new EcdsaAccountContract**(`signingPrivateKey`): [`EcdsaAccountContract`](ecdsa.EcdsaAccountContract.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signingPrivateKey` | `Buffer` |

#### Returns

[`EcdsaAccountContract`](ecdsa.EcdsaAccountContract.md)

#### Overrides

[DefaultAccountContract](defaults.DefaultAccountContract.md).[constructor](defaults.DefaultAccountContract.md#constructor)

## Properties

### signingPrivateKey

• `Private` **signingPrivateKey**: `Buffer`

## Methods

### getAuthWitnessProvider

▸ **getAuthWitnessProvider**(`_address`): `AuthWitnessProvider`

#### Parameters

| Name | Type |
| :------ | :------ |
| `_address` | `CompleteAddress` |

#### Returns

`AuthWitnessProvider`

#### Overrides

[DefaultAccountContract](defaults.DefaultAccountContract.md).[getAuthWitnessProvider](defaults.DefaultAccountContract.md#getauthwitnessprovider)

___

### getContractArtifact

▸ **getContractArtifact**(): `ContractArtifact`

#### Returns

`ContractArtifact`

#### Inherited from

[DefaultAccountContract](defaults.DefaultAccountContract.md).[getContractArtifact](defaults.DefaultAccountContract.md#getcontractartifact)

___

### getDeploymentArgs

▸ **getDeploymentArgs**(): `Buffer`[]

#### Returns

`Buffer`[]

#### Overrides

[DefaultAccountContract](defaults.DefaultAccountContract.md).[getDeploymentArgs](defaults.DefaultAccountContract.md#getdeploymentargs)

___

### getInterface

▸ **getInterface**(`address`, `nodeInfo`): `AccountInterface`

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `CompleteAddress` |
| `nodeInfo` | `NodeInfo` |

#### Returns

`AccountInterface`

#### Inherited from

[DefaultAccountContract](defaults.DefaultAccountContract.md).[getInterface](defaults.DefaultAccountContract.md#getinterface)

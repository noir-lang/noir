---
id: "schnorr.SchnorrAccountContract"
title: "Class: SchnorrAccountContract"
sidebar_label: "SchnorrAccountContract"
custom_edit_url: null
---

[schnorr](../modules/schnorr.md).SchnorrAccountContract

Account contract that authenticates transactions using Schnorr signatures
verified against a Grumpkin public key stored in an immutable encrypted note.

## Hierarchy

- [`DefaultAccountContract`](defaults.DefaultAccountContract.md)

  ↳ **`SchnorrAccountContract`**

## Constructors

### constructor

• **new SchnorrAccountContract**(`signingPrivateKey`): [`SchnorrAccountContract`](schnorr.SchnorrAccountContract.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `signingPrivateKey` | `Fq` |

#### Returns

[`SchnorrAccountContract`](schnorr.SchnorrAccountContract.md)

#### Overrides

[DefaultAccountContract](defaults.DefaultAccountContract.md).[constructor](defaults.DefaultAccountContract.md#constructor)

## Properties

### signingPrivateKey

• `Private` **signingPrivateKey**: `Fq`

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

▸ **getDeploymentArgs**(): `Fr`[]

#### Returns

`Fr`[]

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

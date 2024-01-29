---
id: "single_key.SingleKeyAccountContract"
title: "Class: SingleKeyAccountContract"
sidebar_label: "SingleKeyAccountContract"
custom_edit_url: null
---

[single\_key](../modules/single_key.md).SingleKeyAccountContract

Account contract that authenticates transactions using Schnorr signatures verified against
the note encryption key, relying on a single private key for both encryption and authentication.

## Hierarchy

- [`DefaultAccountContract`](defaults.DefaultAccountContract.md)

  ↳ **`SingleKeyAccountContract`**

## Constructors

### constructor

• **new SingleKeyAccountContract**(`encryptionPrivateKey`): [`SingleKeyAccountContract`](single_key.SingleKeyAccountContract.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `encryptionPrivateKey` | `Fq` |

#### Returns

[`SingleKeyAccountContract`](single_key.SingleKeyAccountContract.md)

#### Overrides

[DefaultAccountContract](defaults.DefaultAccountContract.md).[constructor](defaults.DefaultAccountContract.md#constructor)

## Properties

### encryptionPrivateKey

• `Private` **encryptionPrivateKey**: `Fq`

## Methods

### getAuthWitnessProvider

▸ **getAuthWitnessProvider**(`«destructured»`): `AuthWitnessProvider`

#### Parameters

| Name | Type |
| :------ | :------ |
| `«destructured»` | `CompleteAddress` |

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

▸ **getDeploymentArgs**(): `any`[]

#### Returns

`any`[]

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

---
id: "defaults.DefaultAccountContract"
title: "Class: DefaultAccountContract"
sidebar_label: "DefaultAccountContract"
custom_edit_url: null
---

[defaults](../modules/defaults.md).DefaultAccountContract

Base class for implementing an account contract. Requires that the account uses the
default entrypoint method signature.

## Hierarchy

- **`DefaultAccountContract`**

  ↳ [`EcdsaAccountContract`](ecdsa.EcdsaAccountContract.md)

  ↳ [`SchnorrAccountContract`](schnorr.SchnorrAccountContract.md)

  ↳ [`SingleKeyAccountContract`](single_key.SingleKeyAccountContract.md)

## Implements

- `AccountContract`

## Constructors

### constructor

• **new DefaultAccountContract**(`artifact`): [`DefaultAccountContract`](defaults.DefaultAccountContract.md)

#### Parameters

| Name | Type |
| :------ | :------ |
| `artifact` | `ContractArtifact` |

#### Returns

[`DefaultAccountContract`](defaults.DefaultAccountContract.md)

## Properties

### artifact

• `Private` **artifact**: `ContractArtifact`

## Methods

### getAuthWitnessProvider

▸ **getAuthWitnessProvider**(`address`): `AuthWitnessProvider`

#### Parameters

| Name | Type |
| :------ | :------ |
| `address` | `CompleteAddress` |

#### Returns

`AuthWitnessProvider`

___

### getContractArtifact

▸ **getContractArtifact**(): `ContractArtifact`

#### Returns

`ContractArtifact`

#### Implementation of

AccountContract.getContractArtifact

___

### getDeploymentArgs

▸ **getDeploymentArgs**(): `any`[]

#### Returns

`any`[]

#### Implementation of

AccountContract.getDeploymentArgs

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

#### Implementation of

AccountContract.getInterface

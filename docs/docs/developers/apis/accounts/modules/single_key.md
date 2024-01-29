---
id: "single_key"
title: "Module: single_key"
sidebar_label: "single_key"
sidebar_position: 0
custom_edit_url: null
---

The `@aztec/accounts/single_key` export provides a testing account contract implementation that uses a single Grumpkin key for both authentication and encryption.
It is not recommended to use this account type in production.

## Classes

- [SingleKeyAccountContract](../classes/single_key.SingleKeyAccountContract.md)

## References

### getUnsafeSchnorrAccount

Renames and re-exports [getSingleKeyAccount](single_key.md#getsinglekeyaccount)

___

### getUnsafeSchnorrWallet

Renames and re-exports [getSingleKeyWallet](single_key.md#getsinglekeywallet)

## Variables

### SingleKeyAccountContractArtifact

• `Const` **SingleKeyAccountContractArtifact**: `ContractArtifact`

## Functions

### getSingleKeyAccount

▸ **getSingleKeyAccount**(`pxe`, `encryptionAndSigningPrivateKey`, `saltOrAddress?`): `AccountManager`

Creates an Account that uses the same Grumpkin key for encryption and authentication.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pxe` | `PXE` | An PXE server instance. |
| `encryptionAndSigningPrivateKey` | `Fq` | Grumpkin key used for note encryption and signing transactions. |
| `saltOrAddress?` | `CompleteAddress` \| `Salt` | Deployment salt or complete address if account contract is already deployed. |

#### Returns

`AccountManager`

___

### getSingleKeyWallet

▸ **getSingleKeyWallet**(`pxe`, `address`, `signingKey`): `Promise`\<`AccountWallet`\>

Gets a wallet for an already registered account using Schnorr signatures with a single key for encryption and authentication.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pxe` | `PXE` | An PXE server instance. |
| `address` | `AztecAddress` | Address for the account. |
| `signingKey` | `Fq` | - |

#### Returns

`Promise`\<`AccountWallet`\>

A wallet for this account that can be used to interact with a contract instance.

---
id: "ecdsa"
title: "Module: ecdsa"
sidebar_label: "ecdsa"
sidebar_position: 0
custom_edit_url: null
---

The `@aztec/accounts/ecdsa` export provides an ECDSA account contract implementation, that uses an ECDSA private key for authentication, and a Grumpkin key for encryption.
Consider using this account type when working with integrations with Ethereum wallets.

## Classes

- [EcdsaAccountContract](../classes/ecdsa.EcdsaAccountContract.md)

## Variables

### EcdsaAccountContractArtifact

• `Const` **EcdsaAccountContractArtifact**: `ContractArtifact`

## Functions

### getEcdsaAccount

▸ **getEcdsaAccount**(`pxe`, `encryptionPrivateKey`, `signingPrivateKey`, `saltOrAddress?`): `AccountManager`

Creates an Account that relies on an ECDSA signing key for authentication.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pxe` | `PXE` | An PXE server instance. |
| `encryptionPrivateKey` | `Fq` | Grumpkin key used for note encryption. |
| `signingPrivateKey` | `Buffer` | Secp256k1 key used for signing transactions. |
| `saltOrAddress?` | `CompleteAddress` \| `Salt` | Deployment salt or complete address if account contract is already deployed. |

#### Returns

`AccountManager`

___

### getEcdsaWallet

▸ **getEcdsaWallet**(`pxe`, `address`, `signingPrivateKey`): `Promise`\<`AccountWallet`\>

Gets a wallet for an already registered account using ECDSA signatures.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pxe` | `PXE` | An PXE server instance. |
| `address` | `AztecAddress` | Address for the account. |
| `signingPrivateKey` | `Buffer` | ECDSA key used for signing transactions. |

#### Returns

`Promise`\<`AccountWallet`\>

A wallet for this account that can be used to interact with a contract instance.

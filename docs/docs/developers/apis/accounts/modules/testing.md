---
id: "testing"
title: "Module: testing"
sidebar_label: "testing"
sidebar_position: 0
custom_edit_url: null
---

The `@aztec/accounts/testing` export provides utility methods for testing, in particular in a Sandbox environment.

Use the [createAccount](testing.md#createaccount) and [createAccounts](testing.md#createaccounts) methods to create new sample accounts for testing,
or use [getInitialTestAccountsWallets](testing.md#getinitialtestaccountswallets) to obtain a list of wallets for the Sandbox pre-seeded accounts.

## Variables

### INITIAL\_TEST\_ACCOUNT\_SALTS

• `Const` **INITIAL\_TEST\_ACCOUNT\_SALTS**: `Fr`[]

___

### INITIAL\_TEST\_ENCRYPTION\_KEYS

• `Const` **INITIAL\_TEST\_ENCRYPTION\_KEYS**: `Fq`[]

___

### INITIAL\_TEST\_SIGNING\_KEYS

• `Const` **INITIAL\_TEST\_SIGNING\_KEYS**: `Fq`[] = `INITIAL_TEST_ENCRYPTION_KEYS`

## Functions

### createAccount

▸ **createAccount**(`pxe`): `Promise`\<`AccountWalletWithPrivateKey`\>

Deploys and registers a new account using random private keys and returns the associated Schnorr account wallet. Useful for testing.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pxe` | `PXE` | PXE. |

#### Returns

`Promise`\<`AccountWalletWithPrivateKey`\>

- A wallet for a fresh account.

___

### createAccounts

▸ **createAccounts**(`pxe`, `numberOfAccounts?`): `Promise`\<`AccountWalletWithPrivateKey`[]\>

Creates a given number of random accounts using the Schnorr account wallet.

#### Parameters

| Name | Type | Default value | Description |
| :------ | :------ | :------ | :------ |
| `pxe` | `PXE` | `undefined` | PXE. |
| `numberOfAccounts` | `number` | `1` | How many accounts to create. |

#### Returns

`Promise`\<`AccountWalletWithPrivateKey`[]\>

The created account wallets.

___

### deployInitialTestAccounts

▸ **deployInitialTestAccounts**(`pxe`): `Promise`\<\{ `account`: `AccountManager` ; `privateKey`: `Fq`  }[]\>

Deploys the initial set of schnorr signature accounts to the test environment

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pxe` | `PXE` | PXE instance. |

#### Returns

`Promise`\<\{ `account`: `AccountManager` ; `privateKey`: `Fq`  }[]\>

The set of deployed Account objects and associated private encryption keys

___

### getDeployedTestAccountsWallets

▸ **getDeployedTestAccountsWallets**(`pxe`): `Promise`\<`AccountWalletWithPrivateKey`[]\>

Queries a PXE for it's registered accounts and returns wallets for those accounts using keys in the initial test accounts.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pxe` | `PXE` | PXE instance. |

#### Returns

`Promise`\<`AccountWalletWithPrivateKey`[]\>

A set of AccountWallet implementations for each of the initial accounts.

___

### getInitialTestAccountsWallets

▸ **getInitialTestAccountsWallets**(`pxe`): `Promise`\<`AccountWalletWithPrivateKey`[]\>

Gets a collection of wallets for the Aztec accounts that are initially stored in the test environment.

#### Parameters

| Name | Type | Description |
| :------ | :------ | :------ |
| `pxe` | `PXE` | PXE instance. |

#### Returns

`Promise`\<`AccountWalletWithPrivateKey`[]\>

A set of AccountWallet implementations for each of the initial accounts.

---
id: "account"
title: "Module: account"
sidebar_label: "account"
sidebar_position: 0
custom_edit_url: null
---

The `account` module provides utilities for managing accounts. The AccountManager class
allows to deploy and register a fresh account, or to obtain a `Wallet` instance out of an account
already deployed. Use the `@aztec/accounts` package to load default account implementations that rely
on ECDSA or Schnorr signatures.

## Interfaces

- [AccountContract](../interfaces/account.AccountContract.md)
- [AccountInterface](../interfaces/account.AccountInterface.md)
- [AuthWitnessProvider](../interfaces/account.AuthWitnessProvider.md)
- [EntrypointInterface](../interfaces/account.EntrypointInterface.md)

## Type Aliases

### Salt

Ƭ **Salt**: `Fr` \| `number` \| `bigint`

A contract deployment salt.

___

### Wallet

Ƭ **Wallet**: [`AccountInterface`](../interfaces/account.AccountInterface.md) & `PXE`

The wallet interface.

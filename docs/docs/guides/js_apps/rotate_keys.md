---
title: How to Rotate Nullifier Keys
---

This guide explains how to rotate nullifer secret and public keys using Aztec.js. To learn more about key rotation, read the [concepts section](../../aztec/concepts/accounts/keys.md#key-rotation).

## Prerequisites

You should have a wallet whose keys you want to rotate. You can learn how to create wallets from [this guide](./create_account.md).

You should also have a PXE initialized.

## Relevant imports

You will need to import these from Aztec.js:

#include_code imports yarn-project/end-to-end/src/e2e_key_rotation.test.ts typescript

## Create nullifier secret and public key

`newNskM` = new master nullifier secret key

`newNpkM` = new master nullifier public key (type `PublicKey`)

#include_code create_keys yarn-project/end-to-end/src/e2e_key_rotation.test.ts typescript

## Rotate nullifier secret and public key

Call `rotateNullifierKeys` on the AccountWallet to rotate the secret key in the PXE and call the key registry with the new derived public key.

#include_code rotateNullifierKeys yarn-project/end-to-end/src/e2e_key_rotation.test.ts typescript

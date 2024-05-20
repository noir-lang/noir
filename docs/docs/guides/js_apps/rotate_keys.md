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

Call `rotateMasterNullifierKey` on the PXE to rotate the secret key. 

#include_code rotateMasterNullifierKey yarn-project/end-to-end/src/e2e_key_rotation.test.ts typescript

## Rotate public key 

Connect to the key registry contract with your wallet. 

#include_code keyRegistryWithB yarn-project/end-to-end/src/e2e_key_rotation.test.ts typescript

Then `rotate_npk_m` on the key registry contract to rotate the public key:

#include_code rotate_npk_m yarn-project/end-to-end/src/e2e_key_rotation.test.ts typescript

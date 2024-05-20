---
title: Key Rotation
---

## Prerequisite reading

- [Keys](../../../../aztec/concepts/accounts/keys.md)

## Introduction

It is possible for users to rotate their keys, which can be helpful if some of their keys are leaked.

Because of this, notes are associated with their `nullifier key` rather than any sort of 'owner' address. 

It is still possible to nullify the notes with the old nullifier key even after the key rotation.

## Things to consider

- 'Owner' is arbitrary - as long as you know the nullifier secret, you can nullify a note
- Consider how key rotation can affect account contracts, eg you can add additional security checks for who or how the key rotation is called

## Glossary

- `npk_m_hash`: master nullifying public key hash
- `nsk_app`: app nullifying secret key - the app-specific NSK (learn more about app-scoped keys [here](../../../../aztec/concepts/accounts/keys.md#scoped-keys))
- `nsk_hash`: nullifying secret key hash
- `ivpk_m`: incoming view public key (master) (learn more about IVPKs [here](../../../../aztec/concepts/accounts/keys.md#keys))
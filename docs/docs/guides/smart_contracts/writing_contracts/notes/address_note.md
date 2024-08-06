---
title: Using Address Note in Aztec.nr
---

Address notes hold one main property of the type `AztecAddress`. It also holds `npk_m_hash` and `randomness`, similar to other note types.

## AddressNote

This is the AddressNote struct:

#include_code address_note_struct noir-projects/aztec-nr/address-note/src/address_note.nr rust

## Importing AddressNote

### In Nargo.toml

```toml
address_note = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="noir-projects/aztec-nr/address-note" }
```

### In your contract

#include_code addressnote_import noir-projects/noir-contracts/contracts/escrow_contract/src/main.nr rust

## Working with AddressNote

### Creating a new note 

Creating a new `AddressNote` takes the following args:

- `address` (`AztecAddress`): the address to store in the AddressNote
- `npk_m_hash` (`Field`): the master nullifier public key hash of the user

#include_code addressnote_new noir-projects/noir-contracts/contracts/escrow_contract/src/main.nr rust

In this example, `owner` is the `address` and the `npk_m_hash` of the donor was computed earlier.

## Learn more

- [Keys, including nullifier keys and outgoing viewer](../../../../aztec/concepts/accounts/keys.md)
- [How to write a custom note](./custom_note.md)
- [AddressNote reference](https://docs.aztec.network/reference/smart_contract_reference/aztec-nr/address-note/address_note)
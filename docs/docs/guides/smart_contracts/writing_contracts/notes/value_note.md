---
title: Using Value Notes in Aztec.nr
---

ValueNotes hold one main property - a `value` - and have utils useful for manipulating this value, such as incrementing and decrementing it similarly to an integer.

## ValueNote

This is the ValueNote struct:

#include_code value-note-def noir-projects/aztec-nr/value-note/src/value_note.nr rust

## Importing ValueNote

### In Nargo.toml

```toml
value_note = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="noir-projects/aztec-nr/value-note" }
```

### In your contract

#include_code import_valuenote noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

## Working with ValueNote

### Creating a new note

Creating a new `ValueNote` takes the following args:

- `value` (`Field`): the value of the ValueNote
- `npk_m_hash` (`Field`): the master nullifier public key hash of the user

#include_code valuenote_new noir-projects/noir-contracts/contracts/crowdfunding_contract/src/main.nr rust

In this example, `amount` is the `value` and the `npk_m_hash` of the donor was computed earlier.

### Getting a balance

A user may have multiple notes in a set that all refer to the same content (e.g. a set of notes representing a single token balance). By using the `ValueNote` type to represent token balances, you do not have to manually add each of these notes and can instead use a helper function `get_balance()`.

It takes one argument - the set of notes.

#include_code get_balance noir-projects/noir-contracts/contracts/stateful_test_contract/src/main.nr rust

This can only be used in an unconstrained function.

### Incrementing and decrementing

Both `increment` and `decrement` functions take the same args:

#include_code increment_args noir-projects/aztec-nr/value-note/src/utils.nr rust

Note that this will create a new note in the set of notes passed as the first argument.
For example:
#include_code increment_valuenote noir-projects/noir-contracts/contracts/benchmarking_contract/src/main.nr rust

The `decrement` function works similarly except the `amount` is the number that the value will be decremented by, and it will fail if the sum of the selected notes is less than the amount.

## Learn more

- [Keys, including nullifier keys and outgoing viewer](../../../../aztec/concepts/accounts/keys.md)
- [How to write a custom note](./custom_note.md)
- [ValueNote reference](https://docs.aztec.network/reference/smart_contract_reference/aztec-nr/value-note/value_note)
# State Variables

## `PublicState`

Public state is persistent state which is _publicly visible_, by anyone in the world.

For developers coming from other blockchain ecosystems (such as Ethereum) this will be a familiar concept, because there, _all_ state is _publicly visible_.

Aztec public state follows an account-based model. That is, each state occupies a leaf in an account-based merkle tree; the _public state tree_ (LINK). See here (LINK) for more of the technical details.

The `PublicState<T, T_SERIALISED_LEN>` struct, provides a wrapper around conventional Noir types `T`, allowing such types to be written-to and read-from the public state tree.

#include_code PublicState /yarn-project/noir-contracts/src/contracts/public_token_contract/src/storage.nr rust

:::danger TODO
Examples which:
- initialise a `PublicState<T>` by itself (without being wrapped in a `Map`)
- initialise a `PublicState<T>` where `T` is a custom struct.
:::


## `Map<T>`


## Private State

### UTXO trees

### Notes

### Custom Notes

### `UTXO<NoteType>`

### `UTXOSet<NoteType>`
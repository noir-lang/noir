---
title: Storage
---

Smart contracts rely on storage, acting as the persistent memory on the blockchain. In Aztec, because of its hybrid, privacy-first architecture, the management of this storage is more complex than other blockchains like Ethereum.

You control this storage in Aztec using the `Storage` struct. This struct serves as the housing unit for all your smart contract's state variables - the data it needs to keep track of and maintain.

These state variables come in two forms: public and private. Public variables are visible to anyone, and private variables remain hidden within the contract.

Aztec.nr has a few abstractions to help define the type of data your contract holds. These include Singletons, ImmutableSingletons, Set, and Map.

On this page, you’ll learn:

- How to manage a smart contract's storage structure
- The distinctions and applications of public and private state variables
- How to use Singleton, ImmutableSingleton, Set, and Map
- An overview of 'notes' and the UTXO model
- Practical implications of Storage in real smart contracts
  In an Aztec.nr contract, storage is to be defined as a single struct, that contains both public and private state variables.

## Public and private state variables

Public state variables can be read by anyone, while private state variables can only be read by their owner (or people whom the owner has shared the decrypted data or note viewing key with).

Public state follows the Ethereum style account model, where each contract has its own key-value datastore. Private state follows a UTXO model, where note contents (pre-images) are only known by the sender and those able to decrypt them - see ([state model](../../../../concepts/foundation/state_model/main.md) and [private/public execution](../../../../concepts/foundation/communication/public_private_calls/main.md)) for more background.

## Storage struct

:::info
The struct **must** be called `Storage` for the Aztec.nr library to properly handle it (this will be relaxed in the future).
:::

```rust
struct Storage {
  // public state variables
  // private state variables
}
```

:::danger
If your contract works with storage (has Storage struct defined), you **MUST** include a `compute_note_hash_and_nullifier` function to allow PXE to process encrypted events. See [encrypted events](../events.md#processing-encrypted-events) for more.

If you don't yet have any private state variables defined put there a placeholder function:

#include_code compute_note_hash_and_nullifier_placeholder /yarn-project/noir-contracts/contracts/token_bridge_contract/src/main.nr rust
:::

Since Aztec.nr is written in Noir, which on its own is state-less, we need to specify how the storage struct should be initialized to read and write data correctly. This is done by specifying an `init` function that is run in functions that rely on reading or altering the state variables. This `init` function must declare the storage struct with an instantiation defining how variables are accessed and manipulated. The function MUST be called `init` for the Aztec.nr library to properly handle it (this will be relaxed in the future).

```rust
impl Storage {
  fn init(context: Context) -> Self {
    Storage {
      // (public state variables)::new()
      // (private state variables)::new()
    }
  }
}
```

If you have defined a `Storage` struct following this naming scheme, then it will be made available to you through the reserved `storage` keyword within your contract functions.

:::warning Using slot `0` is not supported!
No storage values should be initialized at slot `0` - storage slots begin at `1`. This is a known issue that will be fixed in the future.
:::

## Map

A `map` is a state variable that "maps" a key to a value. It can be used with private or public storage variables.

:::info
In Aztec.nr, keys are always `Field`s (or types that can be serialized as Fields) and values can be any type - even other maps. `Field`s are finite field elements, but you can think of them as integers for now.
:::

It includes a [`Context`](../context.mdx) to specify the private or public domain, a `storage_slot` to specify where in storage the map is stored, and a `start_var_constructor` which tells the map how it should operate on the underlying type. This includes how to serialize and deserialize the type, as well as how commitments and nullifiers are computed for the type if it's private.

You can view the implementation in the Aztec.nr library [here](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/state_vars/map.nr).

### `new`

When declaring the storage for a map, we use the `Map::new()` constructor. As seen below, this takes the `storage_slot` and the `start_var_constructor` along with the [`Context`](../context.mdx).

We will see examples of map constructors for public and private variables in later sections.

#### As private storage

When declaring a mapping in private storage, we have to specify which type of Note to use. In the example below, we are specifying that we want to use the `Singleton` note type. See the [Singleton](#singletonnotetype) section for more information.

In the Storage struct:

#include_code storage-map-singleton-declaration /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

In the `Storage::init` function:

#include_code state_vars-MapSingleton /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

#### Public Example

When declaring a public mapping in Storage, we have to specify that the type is public by declaring it as `PublicState` instead of specifying a note type like with private storage above.

In the Storage struct:

#include_code storage_minters /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

In the `Storage::init` function:

#include_code storage_minters_init /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

### `at`

When dealing with a map, we can access the value at a given key using the `::at` method. This takes the key as an argument and returns the value at that key.

This function behaves similarly for both private and public maps. An example could be if we have a map with `minters`, which is mapping addresses to a flag for whether they are allowed to mint tokens or not.

#include_code read_minter /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

Above, we are specifying that we want to get the storage in the Map `at` the `msg_sender()`, read the value stored and check that `msg_sender()` is indeed a minter. Doing a similar operation in Solidity code would look like:

```solidity
require(minters[msg.sender], "caller is not minter");
```

## Public State Variables

To define that a variable is public, it is wrapped in the `PublicState` struct.

The `PublicState` struct is generic over the variable type `T` and its serialized size `T_SERIALIZED_LEN`.
:::info
Currently, the length of the types must be specified when declaring the storage struct but the intention is that this will be inferred in the future.
:::

The struct contains a `storage_slot` which, similar to Ethereum, is used to figure out _where_ in storage the variable is located. Notice that while we don't have the exact same [state model](../../../../concepts/foundation/state_model/main.md) as EVM chains it will look similar from the contract developers point of view.

Beyond the struct, the `PublicState` also contains `serialization_methods`, which is a struct with methods that instruct the `PublicState` how to serialize and deserialize the variable. You can find the details of `PublicState` in the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/state_vars/public_state.nr).

:::info
The British haven't surrendered fully to US spelling, so serialization is currently inconsistent.
:::

The Aztec.nr library provides serialization methods for various common types.

:::info
An example using a larger struct can be found in the [lending example](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts/contracts/lending_contract)'s use of an [`Asset`](https://github.com/AztecProtocol/aztec-packages/tree/master/yarn-project/noir-contracts/contracts/lending_contract/src/asset.nr).
:::

### `new`

When declaring the storage for `T` as a persistent public storage variable, we use the `PublicState::new()` constructor. As seen below, this takes the `storage_slot` and the `serialization_methods` as arguments along with the [`Context`](../context.mdx), which in this case is used to share interface with other structures.

#include_code public_state_struct_new /yarn-project/aztec-nr/aztec/src/state_vars/public_state.nr rust

#### Single value example

Say that we wish to add `admin` public state variable into our storage struct. In the struct we can define it as:

#include_code storage_admin /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

And then when initializing it in the `Storage::init` function we can do:

#include_code storage_admin_init /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

We have specified that we are storing a `Field` that should be placed in storage slot `1`. This is just a single value, and is similar to the following in solidity:

```solidity
address internal admin;
```

:::info
We know its verbose, and are working on making it less so.
:::

#### Mapping example

Say we want to have a group of `minters` that are able to mint assets in our contract, and we want them in public storage, because [access control in private is quite cumbersome](../../../../concepts/foundation/communication/public_private_calls/main.md#a-note-on-l2-access-control). In the `Storage` struct we can add it as follows:

#include_code storage_minters /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

And then when initializing it in the `Storage::init` function we can do it as follows:

#include_code storage_minters_init /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

In this case, specifying that we are dealing with a map of Fields, and that it should be put at slot 2.

This would be similar to the following in solidity:

```solidity
mapping(address => bool) internal minters;
```

### `read`

On the `PublicState` structs we have a `read` method to read the value at the location in storage and using the specified deserialization method to deserialize it.

#### Reading from our `admin` example

For our `admin` example from earlier, this could be used as follows to check that the stored value matches the `msg_sender()`.

#include_code read_admin /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

#### Reading from our `minters` example

As we saw in the Map earlier, a very similar operation can be done to perform a lookup in a map.

#include_code read_minter /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

### `write`

We have a `write` method on the `PublicState` struct that takes the value to write as an input and saves this in storage. It uses the serialization method to serialize the value which inserts (possibly multiple) values into storage.

#### Writing to our `admin` example

#include_code write_admin /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

#### Writing to our `minters` example

#include_code write_minter /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

## Private State Variables

In contrast to public state, private state is persistent state that is **not** visible to the whole world. Depending on the logic of the smart contract, a private state variable's current value will only be known to one entity, or a closed group of entities.

The value of a private state variable can either be shared via an [encrypted log](../events.md#encrypted-events), or offchain via web2, or completely offline: it's up to the app developer.

Aztec private state follows a utxo-based model. That is, a private state's current value is represented as one or many [notes](#notes). Each note is stored as an individual leaf in a utxo-based merkle tree: the [private state tree](../../../../concepts/foundation/state_model/main.md).

To greatly simplify the experience of writing private state, Aztec.nr provides three different types of private state variable:

- [Singleton<NoteType\>](#singletonnotetype)
- [ImmutableSingleton<NoteType\>](#immutablesingletonnotetype)
- [Set<NoteType\>](#setnotetype)

These three structs abstract-away many of Aztec's protocol complexities, by providing intuitive methods to modify notes in the utxo tree in a privacy-preserving way.

:::info
An app can also choose to emit data via unencrypted log, or to define a note whose data is easy to figure out, then the information is technically not private and could be visible to anyone.
:::

### Notes

Unlike public state variables, which can be arbitrary types, private state variables operate on `NoteType`.

Notes are the fundamental elements in the private world.

A note should conform to the following interface:

#include_code NoteInterface /yarn-project/aztec-nr/aztec/src/note/note_interface.nr rust

The interplay between a private state variable and its notes can be confusing. Here's a summary to aid intuition:

A private state variable (of type `Singleton`, `ImmutableSingleton` or `Set`) may be declared in storage.

Every note contains (as a 'header') the contract address and storage slot of the state variable to which it "belongs". A note is said to "belong" to a private state if the storage slot of the private state matches the storage slot contained in the note's header. The header provides information that helps the user interpret the note's data. Without it the user would have to figure out where it belongs by brute-forcing the address and contract space.

Management of this 'header' is abstracted-away from developers who use the `ImmutableSingleton`, `Singleton` and `Set` types.

A private state variable is colloquially said to "point" to one or many notes (depending on the type), if those note(s) all "belong" to that private state, and those note(s) haven't-yet been nullified.

An `ImmutableSingleton` will point to _one_ note over the lifetime of the contract. ("One", hence "Singleton"). This note is a struct of information that is persisted forever.

A `Singleton` may point to _one_ note at a time. ("One", hence "Singleton"). But since it's not "immutable", the note that it points to may be [replaced](#replace) by functions of the contract, over time. The "current value" of a `Singleton` is interpreted as the one note which has not-yet been nullified. The act of 'replacing' a Singleton's note is how a `Singleton` state may be modified by functions.

`Singleton` is a useful type when declaring a private state which may only ever be modified by those who are privy to the current value of that state.

A `Set` may point to _multiple_ notes at a time. The "current value" of a private state variable of type `Set` is some 'accumulation' of all not-yet nullified notes which "belong" to the `Set`. The term "some accumulation" is intentionally vague. The interpretation of the "current value" of a `Set` must be expressed by the smart contract developer. A common use case for a `Set` is to represent the sum of a collection of values (in which case 'accumulation' is 'summation').

Think of a ZCash balance (or even a Bitcoin balance). The "current value" of a user's ZCash balance is the sum of all unspent (not-yet nullified) notes belonging to that user. To modify the "current value" of a `Set` state variable, is to [`insert`](#insert) new notes into the `Set`, or [`remove`](#remove) notes from that set.

Interestingly, if a developer requires a private state to be modifiable by users who _aren't_ privy to the value of that state, a `Set` is a very useful type. The `insert` method allows new notes to be added to the `Set` without knowing any of the other notes in the set! (Like posting an envelope into a post box, you don't know what else is in there!).

## `Singleton<NoteType>`

Singleton is a private state variable that is unique in a way. When a Singleton is initialized, a note is created to represent its value. And the way to update the value is to destroy the current note, and create a new one with the updated value.

Like for public state, we define the struct to have context, a storage slot and `note_interface` specifying how the note should be constructed and manipulated. You can view the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/state_vars/singleton.nr).

An example of singleton usage in the account contracts is keeping track of public keys. The `Singleton` is added to the `Storage` struct as follows:

#include_code storage-singleton-declaration /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

### `new`

As part of the initialization of the `Storage` struct, the `Singleton` is created as follows, here at the specified storage slot and with the `NoteInterface` for `CardNote`.

#include_code start_vars_singleton /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

### `initialize`

As mentioned, the Singleton is initialized to create the first note and value.

When this function is called, a nullifier of the storage slot is created, preventing this Singleton from being initialized again.

:::danger Privacy-Leak
Beware that because this nullifier is created only from the storage slot without randomness it is "leaky". This means that if the storage slot depends on the an address then it is possible to link the nullifier to the address. For example, if the singleton is part of a `map` with an `AztecAddress` as the key then the nullifier will be linked to the address.
:::

Unlike public states, which have a default initial value of `0` (or many zeros, in the case of a struct, array or map), a private state (of type `Singleton`, `ImmutableSingleton` or `Set`) does not have a default initial value. The `initialize` method (or `insert`, in the case of a `Set`) must be called.

:::info
Extend on what happens if you try to use non-initialized state.
:::

### `is_initialized`

An unconstrained method to check whether the Singleton has been initialized or not. It takes an optional owner and returns a boolean. You can view the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/aztec/src/state_vars/singleton.nr).

### `replace`

To update the value of a `Singleton`, we can use the `replace` method. The method takes a new note as input, and replaces the current note with the new one. It emits a nullifier for the old value, and inserts the new note into the data tree.

An example of this is seen in a example card game, where we create a new note (a `CardNote`) containing some new data, and replace the current note with it:

#include_code state_vars-SingletonReplace /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

If two people are trying to modify the Singleton at the same time, only one will succeed as we don't allow duplicate nullifiers! Developers should put in place appropriate access controls to avoid race conditions (unless a race is intended!).

### `get_note`

This function allows us to get the note of a Singleton, essentially reading the value.

#include_code state_vars-SingletonGet /yarn-project/noir-contracts/contracts/docs_example_contract/src/main.nr rust

#### Nullifying Note reads

It's possible that at the time this function is called, the system hasn't synced to the block where the latest note was created. Or a malicious user might feed an old state to this function, tricking the proving system into thinking that the value hasn't changed. To avoid an attack around it, this function will destroy the current note, and replace it with a duplicated note that has the same fields. Because the nullifier of the latest note will be emitted, if two people are trying to use this function against the same note, only one will succeed (no duplicate nullifiers allowed).

### `view_note`

Functionally similar to [`get_note`](#get_note), but executed in unconstrained functions and can be used by the wallet to fetch notes for use by front-ends etc.

## `ImmutableSingleton<NoteType>`

ImmutableSingleton represents a unique private state variable that, as the name suggests, is immutable. Once initialized, its value cannot be altered.

#include_code struct /yarn-project/aztec-nr/aztec/src/state_vars/immutable_singleton.nr rust

### `new`

As part of the initialization of the `Storage` struct, the `Singleton` is created as follows, here at storage slot 1 and with the `NoteInterface` for `PublicKeyNote`.

#include_code storage_init /yarn-project/noir-contracts/contracts/schnorr_account_contract/src/main.nr rust

### `initialize`

When this function is invoked, it creates a nullifier for the storage slot, ensuring that the ImmutableSingleton cannot be initialized again. 

:::danger Privacy-Leak
Beware that because this nullifier is created only from the storage slot without randomness it is "leaky". This means that if the storage slot depends on the an address then it is possible to link the nullifier to the address. For example, if the singleton is part of a `map` with an `AztecAddress` as the key then the nullifier will be linked to the address.
:::

Set the value of an ImmutableSingleton by calling the `initialize` method:

#include_code initialize /yarn-project/noir-contracts/contracts/schnorr_account_contract/src/main.nr rust

Once initialized, an ImmutableSingleton's value remains unchangeable. This method can only be called once.

### `is_initialized`

An unconstrained method to check if the ImmutableSingleton has been initialized. Takes an optional owner and returns a boolean. You can find the implementation [here](https://github.com/AztecProtocol/aztec-packages/blob/master//yarn-project/aztec-nr/aztec/src/state_vars/immutable_singleton.nr).

### `get_note`

Similar to the `Singleton`, we can use the `get_note` method to read the value of an ImmutableSingleton.

Use this method to retrieve the value of an initialized ImmutableSingleton.

#include_code get_note /yarn-project/noir-contracts/contracts/schnorr_account_contract/src/main.nr rust

Unlike a `Singleton`, the `get_note` function for an ImmutableSingleton doesn't destroy the current note in the background. This means that multiple accounts can concurrently call this function to read the value.

This function will throw if the ImmutableSingleton hasn't been initialized.

### `view_note`

Functionally similar to `get_note`, but executed unconstrained and can be used by the wallet to fetch notes for use by front-ends etc.

## `Set<NoteType>`

Set is used for managing a collection of notes. All notes in a set are of the same `NoteType`. But whether these notes all belong to one entity, or are accessible and editable by different entities, is totally up to the developer. Due to our state model, the set is a collection of notes inserted into the data-tree, but notes are never removed from the tree itself, they are only nullified.

#include_code struct /yarn-project/aztec-nr/aztec/src/state_vars/set.nr rust

And can be added to the `Storage` struct as follows. Here adding a set for a custom note, the TransparentNote (useful for [public -> private communication](../functions.md#public---private)).

#include_code storage_pending_shields /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

### `new`

The `new` method tells the contract how to operate on the underlying storage.

We can initialize the set as follows:

#include_code storage_pending_shields_init /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

### `insert`

Allows us to modify the storage by inserting a note into the set.

A commitment from the note will be generated, and inserted into data-tree, allowing us to later use in contract interactions.

A commitment from the note will be generated, and inserted into data-tree, allowing us to later use in contract interactions. Recall that the content of the note should be shared with the owner to allow them to use it, as mentioned this can be done via an [encrypted log](../events.md#encrypted-events), or offchain via web2, or completely offline.

#include_code insert /yarn-project/aztec-nr/easy-private-state/src/easy_private_state.nr rust

### `insert_from_public`

While we don't support private functions to directly alter public storage, the opposite direction is possible. The `insert_from_public` allow public function to insert notes into private storage. This is very useful when we want to support private function calls that must have been initiated in public, such as shielding or the like.

The usage is rather straight-forward and very similar to using the `insert` method with the difference that this one is called in public functions.

#include_code insert_from_public /yarn-project/noir-contracts/contracts/token_contract/src/main.nr rust

### `remove`

Will remove a note from the set if it previously has been read from storage, e.g. you have fetched it through a `get_notes` call. This is useful when you want to remove a note that you have previously read from storage and do not have to read it again.

Nullifiers are emitted when reading values to make sure that they are up to date.

An example of how to use this operation is visible in the `easy_private_state`:

#include_code remove /yarn-project/aztec-nr/easy-private-state/src/easy_private_state.nr rust

### `get_notes`

This function returns the notes the account has access to.

The kernel circuits are constrained to a maximum number of notes this function can return at a time. Check [here](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-protocol-circuits/src/crates/types/src/constants.nr) and look for `MAX_READ_REQUESTS_PER_CALL` for the up-to-date number.

Because of this limit, we should always consider using the second argument `NoteGetterOptions` to limit the number of notes we need to read and constrain in our programs. This is quite important as every extra call increases the time used to prove the program and we don't want to spend more time than necessary.

An example of such options is using the [filter_notes_min_sum](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/aztec-nr/value-note/src/filter.nr) to get "enough" notes to cover a given value. Essentially, this function will return just enough notes to cover the amount specified such that we don't need to read all our notes. For users with a lot of notes, this becomes increasingly important.

#include_code get_notes /yarn-project/aztec-nr/easy-private-state/src/easy_private_state.nr rust

### `view_notes`

Functionally similar to [`get_notes`](#get_notes), but executed unconstrained and can be used by the wallet to fetch notes for use by front-ends etc.

#include_code view_notes /yarn-project/aztec-nr/value-note/src/balance_utils.nr rust

There's also a limit on the maximum number of notes that can be returned in one go. To find the current limit, refer to [this file](https://github.com/AztecProtocol/aztec-packages/blob/master/yarn-project/noir-protocol-circuits/src/crates/types/src/constants.nr) and look for `MAX_NOTES_PER_PAGE`.

The key distinction is that this method is unconstrained. It does not perform a check to verify if the notes actually exist, which is something the [`get_notes`](#get_notes) method does under the hood. Therefore, it should only be used in an unconstrained contract function.

This function requires a `NoteViewerOptions`. The `NoteViewerOptions` is essentially similar to the [`NoteGetterOptions`](#notegetteroptions), except that it doesn't take a custom filter.

### NoteGetterOptions

`NoteGetterOptions` encapsulates a set of configurable options for filtering and retrieving a selection of notes from a [data oracle](../functions.md#oracle-functions). Developers can design instances of `NoteGetterOptions`, to determine how notes should be filtered and returned to the functions of their smart contracts.

#include_code NoteGetterOptions /yarn-project/aztec-nr/aztec/src/note/note_getter_options.nr rust

#### `selects: BoundedVec<Option<Select>, N>`

`selects` is a collection of filtering criteria, specified by `Select { field_index: u8, value: Field }` structs. It instructs the data oracle to find notes whose (`field_index`)th field matches the provided `value`.

#### `sorts: BoundedVec<Option<Sort>, N>`

`sorts` is a set of sorting instructions defined by `Sort { field_index: u8, order: u2 }` structs. This directs the data oracle to sort the matching notes based on the value of the specified field index and in the indicated order. The value of order is **1** for _DESCENDING_ and **2** for _ASCENDING_.

#### `limit: u32`

When the `limit` is set to a non-zero value, the data oracle will return a maximum of `limit` notes.

#### `offset: u32`

This setting enables us to skip the first `offset` notes. It's particularly useful for pagination.

#### `filter: fn ([Option<Note>; MAX_READ_REQUESTS_PER_CALL], FILTER_ARGS) -> [Option<Note>; MAX_READ_REQUESTS_PER_CALL]`

Developers have the option to provide a custom filter. This allows specific logic to be applied to notes that meet the criteria outlined above. The filter takes the notes returned from the oracle and `filter_args` as its parameters.

It's important to note that the process of applying the custom filter to get the final notes is not constrained. It's crucial to verify the returned notes even if certain assumptions are made in the custom filter.

#### `filter_args: FILTER_ARGS`

`filter_args` provides a means to furnish additional data or context to the custom filter.

#### Methods

Several methods are available on `NoteGetterOptions` to construct the options in a more readable manner:

#### `fn new() -> NoteGetterOptions<Note, N, Field>`

This function initializes a `NoteGetterOptions` that simply returns the maximum number of notes allowed in a call.

#### `fn with_filter(filter, filter_args) -> NoteGetterOptions<Note, N, FILTER_ARGS>`

This function initializes a `NoteGetterOptions` with a [`filter`](#filter-fn-optionnote-max_read_requests_per_call-filter_args---optionnote-max_read_requests_per_call) and [`filter_args`](#filter_args-filter_args).

#### `.select`

This method adds a [`Select`](#selects-boundedvecoptionselect-n) criterion to the options.

#### `.sort`

This method adds a [`Sort`](#sorts-boundedvecoptionsort-n) criterion to the options.

#### `.set_limit`

This method lets you set a limit for the maximum number of notes to be retrieved.

#### `.set_offset`

This method sets the offset value, which determines where to start retrieving notes.

#### Examples

The following code snippet creates an instance of `NoteGetterOptions`, which has been configured to find the cards that belong to `account_address`. The returned cards are sorted by their points in descending order, and the first `offset` cards with the highest points are skipped.

#include_code state_vars-NoteGetterOptionsSelectSortOffset /yarn-project/noir-contracts/contracts/docs_example_contract/src/options.nr rust

The first value of `.select` and `.sort` is the index of a field in a note type. For the note type `CardNote` that has the following fields:

#include_code state_vars-CardNote /yarn-project/noir-contracts/contracts/docs_example_contract/src/types/card_note.nr rust

The indices are: 0 for `points`, 1 for `secret`, and 2 for `owner`.

In the example, `.select(2, account_address)` matches the 2nd field of `CardNote`, which is `owner`, and returns the cards whose `owner` field equals `account_address`.

`.sort(0, SortOrder.DESC)` sorts the 0th field of `CardNote`, which is `points`, in descending order.

There can be as many conditions as the number of fields a note type has. The following example finds cards whose fields match the three given values:

#include_code state_vars-NoteGetterOptionsMultiSelects /yarn-project/noir-contracts/contracts/docs_example_contract/src/options.nr rust

While `selects` lets us find notes with specific values, `filter` lets us find notes in a more dynamic way. The function below picks the cards whose points are at least `min_points`:

#include_code state_vars-OptionFilter /yarn-project/noir-contracts/contracts/docs_example_contract/src/options.nr rust

We can use it as a filter to further reduce the number of the final notes:

#include_code state_vars-NoteGetterOptionsFilter /yarn-project/noir-contracts/contracts/docs_example_contract/src/options.nr rust

One thing to remember is, `filter` will be applied on the notes after they are picked from the database. Therefore, it's possible that the actual notes we end up getting are fewer than the limit.

The limit is `MAX_READ_REQUESTS_PER_CALL` by default. But we can set it to any value **smaller** than that:

#include_code state_vars-NoteGetterOptionsPickOne /yarn-project/noir-contracts/contracts/docs_example_contract/src/options.nr rust

---
title: Migration notes
description: Read about migration notes from previous versions, which could solve problems while updating
keywords: [sandbox, cli, aztec, notes, migration, updating, upgrading]
---

Aztec is in full-speed development. Literally every version breaks compatibility with the previous ones. This page attempts to target errors and difficulties you might encounter when upgrading, and how to resolve them.

## TBD

### [Aztec.nr] rand oracle is now called unsafe_rand
`oracle::rand::rand` has been renamed to `oracle::unsafe_rand::unsafe_rand`.
This change was made to communicate that we do not constrain the value in circuit and instead we just trust our PXE.

```diff
- let random_value = rand();
+ let random_value = unsafe_rand();
```

### [AztecJS] Simulate and get return values for ANY call
Historically it have been possible to "view" `unconstrained` functions to simulate them and get the return values, but not for `public` nor `private` functions.
This has lead to a lot of bad code where we have the same function implemented thrice, once in `private`, once in `public` and once in `unconstrained`. 
It is not possible to call `simulate` on any call to get the return values! 
However, beware that it currently always returns a Field array of size 4 for private and public.  
This will change to become similar to the return values of the `unconstrained` functions with proper return types.

```diff
-    #[aztec(private)]
-    fn get_shared_immutable_constrained_private() -> pub Leader {
-        storage.shared_immutable.read_private()
-    }
-
-    unconstrained fn get_shared_immutable() -> pub Leader {
-        storage.shared_immutable.read_public()
-    }

+    #[aztec(private)]
+    fn get_shared_immutable_private() -> pub Leader {
+        storage.shared_immutable.read_private()
+    }

- const returnValues = await contract.methods.get_shared_immutable().view();
+ const returnValues = await contract.methods.get_shared_immutable_private().simulate();
```

## 0.31.0

### [Aztec.nr] Public storage historical read API improvement

`history::public_value_inclusion::prove_public_value_inclusion` has been renamed to `history::public_storage::public_storage_historical_read`, and its API changed slightly. Instead of receiving a `value` parameter it now returns the historical value stored at that slot.

If you were using an oracle to get the value to pass to `prove_public_value_inclusion`, drop the oracle and use the return value from `public_storage_historical_read` instead:

```diff
- let value = read_storage();
- prove_public_value_inclusion(value, storage_slot, contract_address, context);
+ let value = public_storage_historical_read(storage_slot, contract_address, context);
```

If you were proving historical existence of a value you got via some other constrained means, perform an assertion against the return value of `public_storage_historical_read` instead:

```diff
- prove_public_value_inclusion(value, storage_slot, contract_address, context);
+ assert(public_storage_historical_read(storage_slot, contract_address, context) == value);
```

## 0.30.0

### [AztecJS] Simplify authwit syntax

```diff
- const messageHash = computeAuthWitMessageHash(accounts[1].address, action.request());
- await wallets[0].setPublicAuth(messageHash, true).send().wait();
+ await wallets[0].setPublicAuthWit({ caller: accounts[1].address, action }, true).send().wait();
```

```diff
const action = asset
    .withWallet(wallets[1])
    .methods.unshield(accounts[0].address, accounts[1].address, amount, nonce);
-const messageHash = computeAuthWitMessageHash(accounts[1].address, action.request());
-const witness = await wallets[0].createAuthWitness(messageHash);
+const witness = await wallets[0].createAuthWit({ caller: accounts[1].address, action });
await wallets[1].addAuthWitness(witness);
```

Also note some of the naming changes:
`setPublicAuth` -> `setPublicAuthWit`
`createAuthWitness` -> `createAuthWit`

### [Aztec.nr] Automatic NoteInterface implementation and selector changes

Implementing a note required a fair amount of boilerplate code, which has been substituted by the `#[aztec(note)]` attribute.

```diff
+ #[aztec(note)]
struct AddressNote {
    address: AztecAddress,
    owner: AztecAddress,
    randomness: Field,
    header: NoteHeader
}

impl NoteInterface<ADDRESS_NOTE_LEN>  for AddressNote {
-    fn serialize_content(self) -> [Field; ADDRESS_NOTE_LEN]{
-        [self.address.to_field(), self.owner.to_field(), self.randomness]
-    }
-
-    fn deserialize_content(serialized_note: [Field; ADDRESS_NOTE_LEN]) -> Self {
-        AddressNote {
-            address: AztecAddress::from_field(serialized_note[0]),
-            owner: AztecAddress::from_field(serialized_note[1]),
-            randomness: serialized_note[2],
-            header: NoteHeader::empty(),
-        }
-    }
-
-    fn compute_note_content_hash(self) -> Field {
-        pedersen_hash(self.serialize_content(), 0)
-    }
-
    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = context.request_nullifier_secret_key(self.owner);
        pedersen_hash([
            note_hash_for_nullify,
            secret.low,
            secret.high,
        ],0)
    }

    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
        let secret = get_nullifier_secret_key(self.owner);
        pedersen_hash([
            note_hash_for_nullify,
            secret.low,
            secret.high,
        ],0)
    }

-    fn set_header(&mut self, header: NoteHeader) {
-        self.header = header;
-    }
-
-    fn get_header(note: Self) -> NoteHeader {
-        note.header
-    }

    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        let encryption_pub_key = get_public_key(self.owner);
        emit_encrypted_log(
            context,
            (*context).this_address(),
            slot,
            Self::get_note_type_id(),
            encryption_pub_key,
            self.serialize_content(),
        );
    }

-    fn get_note_type_id() -> Field {
-        6510010011410111511578111116101
-    }
}
```

Automatic note (de)serialization implementation also means it is now easier to filter notes using `NoteGetterOptions.select` via the `::properties()` helper:

Before:

```rust
let options = NoteGetterOptions::new().select(0, amount, Option::none()).select(1, owner.to_field(), Option::none()).set_limit(1);
```

After:

```rust
let options = NoteGetterOptions::new().select(ValueNote::properties().value, amount, Option::none()).select(ValueNote::properties().owner, owner.to_field(), Option::none()).set_limit(1);
```

The helper returns a metadata struct that looks like this (if autogenerated)

```rust
ValueNoteProperties {
    value: PropertySelector { index: 0, offset: 0, length: 32 },
    owner: PropertySelector { index: 1, offset: 0, length: 32 },
    randomness: PropertySelector { index: 2, offset: 0, length: 32 },
}
```

It can also be used for the `.sort` method.

## 0.27.0

### `initializer` macro replaces `constructor`

Before this version, every contract was required to have exactly one `constructor` private function, that was used for deployment. We have now removed this requirement, and made `constructor` a function like any other.

To signal that a function can be used to **initialize** a contract, you must now decorate it with the `#[aztec(initializer)]` attribute. Initializers are regular functions that set an "initialized" flag (a nullifier) for the contract. A contract can only be initialized once, and contract functions can only be called after the contract has been initialized, much like a constructor. However, if a contract defines no initializers, it can be called at any time. Additionally, you can define as many initializer functions in a contract as you want, both private and public.

To migrate from current code, simply add an initializer attribute to your constructor functions.

```diff
+ #[aztec(initializer)]
#[aztec(private)]
fn constructor() { ... }
```

If your private constructor was used to just call a public internal initializer, then remove the private constructor and flag the public function as initializer. And if your private constructor was an empty one, just remove it.

## 0.25.0

### [Aztec.nr] Static calls

It is now possible to perform static calls from both public and private functions. Static calls forbid any modification to the state, including L2->L1 messages or log generation. Once a static context is set through a static all, every subsequent call will also be treated as static via context propagation.

```rust
context.static_call_private_function(targetContractAddress, targetSelector, args);

context.static_call_public_function(targetContractAddress, targetSelector, args);
```

### [Aztec.nr] Introduction to `prelude`

A new `prelude` module to include common Aztec modules and types.
This simplifies dependency syntax. For example:

```rust
use dep::aztec::protocol_types::address::AztecAddress;
use dep::aztec::{
    context::{PrivateContext, Context}, note::{note_header::NoteHeader, utils as note_utils},
    state_vars::Map
};
```

Becomes:

```rust
use dep::aztec::prelude::{AztecAddress, NoteHeader, PrivateContext, Map};
use dep::aztec::context::Context;
use dep::aztec::notes::utils as note_utils;
```

This will be further simplified in future versions (See [4496](https://github.com/AztecProtocol/aztec-packages/pull/4496) for further details).

The prelude consists of

#include_code prelude /noir-projects/aztec-nr/aztec/src/prelude.nr rust

### `internal` is now a macro

The `internal` keyword is now removed from Noir, and is replaced by an `aztec(internal)` attribute in the function. The resulting behavior is exactly the same: these functions will only be callable from within the same contract.

Before:

```rust
#[aztec(private)]
internal fn double(input: Field) -> Field {
    input * 2
}
```

After:

```rust
#[aztec(private)]
#[aztec(internal)]
fn double(input: Field) -> Field {
    input * 2
}
```

### [Aztec.nr] No SafeU120 anymore!

Noir now have overflow checks by default. So we don't need SafeU120 like libraries anymore.

You can replace it with `U128` instead

Before:

```
SafeU120::new(0)
```

Now:

```
U128::from_integer(0)
```

### [Aztec.nr] `compute_note_hash_and_nullifier` is now autogenerated

Historically developers have been required to include a `compute_note_hash_and_nullifier` function in each of their contracts. This function is now automatically generated, and all instances of it in contract code can be safely removed.

It is possible to provide a user-defined implementation, in which case auto-generation will be skipped (though there are no known use cases for this).

### [Aztec.nr] Updated naming of state variable wrappers

We have decided to change the naming of our state variable wrappers because the naming was not clear.
The changes are as follows:

1. `Singleton` -> `PrivateMutable`
2. `ImmutableSingleton` -> `PrivateImmutable`
3. `StablePublicState` -> `SharedImmutable`
4. `PublicState` -> `PublicMutable`

This is the meaning of "private", "public" and "shared":
Private: read (R) and write (W) from private, not accessible from public
Public: not accessible from private, R/W from public
Shared: R from private, R/W from public

Note: `SlowUpdates` will be renamed to `SharedMutable` once the implementation is ready.

### [Aztec.nr] Authwit updates

Authentication Witnesses have been updates such that they are now cancellable and scoped to a specific consumer.
This means that the `authwit` nullifier must be emitted from the account contract, which require changes to the interface.
Namely, the `assert_current_call_valid_authwit_public` and `assert_current_call_valid_authwit` in `auth.nr` will **NO LONGER** emit a nullifier.
Instead it will call a `spend_*_authwit` function in the account contract - which will emit the nullifier and perform a few checks.
This means that the `is_valid` functions have been removed to not confuse it for a non-mutating function (static).
Furthermore, the `caller` parameter of the "authwits" have been moved "further out" such that the account contract can use it in validation, allowing scoped approvals from the account POV.
For most contracts, this won't be changing much, but for the account contract, it will require a few changes.

Before:

```rust
#[aztec(public)]
fn is_valid_public(message_hash: Field) -> Field {
    let actions = AccountActions::public(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
    actions.is_valid_public(message_hash)
}

#[aztec(private)]
fn is_valid(message_hash: Field) -> Field {
    let actions = AccountActions::private(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
    actions.is_valid(message_hash)
}
```

After:

```rust
#[aztec(private)]
fn spend_private_authwit(inner_hash: Field) -> Field {
    let actions = AccountActions::private(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
    actions.spend_private_authwit(inner_hash)
}

#[aztec(public)]
fn spend_public_authwit(inner_hash: Field) -> Field {
    let actions = AccountActions::public(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
    actions.spend_public_authwit(inner_hash)
}
```

## 0.24.0

### Introduce Note Type IDs

Note Type IDs are a new feature which enable contracts to have multiple `Map`s with different underlying note types, something that was not possible before. This is done almost without any user intervention, though some minor changes are required.

The mandatory `compute_note_hash_and_nullifier` now has a fifth parameter `note_type_id`. Use this instead of `storage_slot` to determine which deserialization function to use.

Before:

```rust
unconstrained fn compute_note_hash_and_nullifier(
    contract_address: AztecAddress,
    nonce: Field,
    storage_slot: Field,
    preimage: [Field; TOKEN_NOTE_LEN]
) -> pub [Field; 4] {
    let note_header = NoteHeader::new(contract_address, nonce, storage_slot);

    if (storage_slot == storage.pending_shields.get_storage_slot()) {
        note_utils::compute_note_hash_and_nullifier(TransparentNote::deserialize_content, note_header, preimage)
    } else if (note_type_id == storage.slow_update.get_storage_slot()) {
        note_utils::compute_note_hash_and_nullifier(FieldNote::deserialize_content, note_header, preimage)
    } else {
        note_utils::compute_note_hash_and_nullifier(TokenNote::deserialize_content, note_header, preimage)
    }
```

Now:

```rust
unconstrained fn compute_note_hash_and_nullifier(
    contract_address: AztecAddress,
    nonce: Field,
    storage_slot: Field,
    note_type_id: Field,
    preimage: [Field; TOKEN_NOTE_LEN]
) -> pub [Field; 4] {
    let note_header = NoteHeader::new(contract_address, nonce, storage_slot);

    if (note_type_id == TransparentNote::get_note_type_id()) {
        note_utils::compute_note_hash_and_nullifier(TransparentNote::deserialize_content, note_header, preimage)
    } else if (note_type_id == FieldNote::get_note_type_id()) {
        note_utils::compute_note_hash_and_nullifier(FieldNote::deserialize_content, note_header, preimage)
    } else {
        note_utils::compute_note_hash_and_nullifier(TokenNote::deserialize_content, note_header, preimage)
    }
```

The `NoteInterface` trait now has an additional `get_note_type_id()` function. This implementation will be autogenerated in the future, but for now providing any unique ID will suffice. The suggested way to do it is by running the Python command shown in the comment below:

```rust
impl NoteInterface<N> for MyCustomNote {
    fn get_note_type_id() -> Field {
        // python -c "print(int(''.join(str(ord(c)) for c in 'MyCustomNote')))"
       771216711711511611110978111116101
    }
}
```

### [js] Importing contracts in JS

`@aztec/noir-contracts` is now `@aztec/noir-contracts.js`. You'll need to update your package.json & imports.

Before:

```js
import { TokenContract } from "@aztec/noir-contracts/Token";
```

Now:

```js
import { TokenContract } from "@aztec/noir-contracts.js/Token";
```

### [Aztec.nr] aztec-nr contracts location change in Nargo.toml

Aztec contracts are now moved outside of the `yarn-project` folder and into `noir-projects`, so you need to update your imports.

Before:

```rust
easy_private_token_contract = {git = "https://github.com/AztecProtocol/aztec-packages/", tag ="v0.23.0", directory = "yarn-project/noir-contracts/contracts/easy_private_token_contract"}
```

Now, update the `yarn-project` folder for `noir-projects`:

```rust
easy_private_token_contract = {git = "https://github.com/AztecProtocol/aztec-packages/", tag ="v0.24.0", directory = "noir-projects/noir-contracts/contracts/easy_private_token_contract"}
```

## 0.22.0

### `Note::compute_note_hash` renamed to `Note::compute_note_content_hash`

The `compute_note_hash` function in of the `Note` trait has been renamed to `compute_note_content_hash` to avoid being confused with the actual note hash.

Before:

```rust
impl NoteInterface for CardNote {
    fn compute_note_hash(self) -> Field {
        pedersen_hash([
            self.owner.to_field(),
        ], 0)
    }
```

Now:

```rust
impl NoteInterface for CardNote {
    fn compute_note_content_hash(self) -> Field {
        pedersen_hash([
            self.owner.to_field(),
        ], 0)
    }
```

### Introduce `compute_note_hash_for_consumption` and `compute_note_hash_for_insertion`

Makes a split in logic for note hash computation for consumption and insertion. This is to avoid confusion between the two, and to make it clear that the note hash for consumption is different from the note hash for insertion (sometimes).

`compute_note_hash_for_consumption` replaces `compute_note_hash_for_read_or_nullify`.
`compute_note_hash_for_insertion` is new, and mainly used in `lifecycle.nr``

### `Note::serialize_content` and `Note::deserialize_content` added to `NoteInterface

The `NoteInterface` have been extended to include `serialize_content` and `deserialize_content` functions. This is to convey the difference between serializing the full note, and just the content. This change allows you to also add a `serialize` function to support passing in a complete note to a function.

Before:

```rust
impl Serialize<ADDRESS_NOTE_LEN> for AddressNote {
    fn serialize(self) -> [Field; ADDRESS_NOTE_LEN]{
        [self.address.to_field(), self.owner.to_field(), self.randomness]
    }
}
impl Deserialize<ADDRESS_NOTE_LEN> for AddressNote {
    fn deserialize(serialized_note: [Field; ADDRESS_NOTE_LEN]) -> Self {
        AddressNote {
            address: AztecAddress::from_field(serialized_note[0]),
            owner: AztecAddress::from_field(serialized_note[1]),
            randomness: serialized_note[2],
            header: NoteHeader::empty(),
        }
    }
```

Now

```rust
impl NoteInterface<ADDRESS_NOTE_LEN>  for AddressNote {
    fn serialize_content(self) -> [Field; ADDRESS_NOTE_LEN]{
        [self.address.to_field(), self.owner.to_field(), self.randomness]
    }

    fn deserialize_content(serialized_note: [Field; ADDRESS_NOTE_LEN]) -> Self {
        AddressNote {
            address: AztecAddress::from_field(serialized_note[0]),
            owner: AztecAddress::from_field(serialized_note[1]),
            randomness: serialized_note[2],
            header: NoteHeader::empty(),
        }
    }
    ...
}
```

### [Aztec.nr] No storage.init() and `Serialize`, `Deserialize`, `NoteInterface` as Traits, removal of SerializationMethods and SERIALIZED_LEN

Storage definition and initialization has been simplified. Previously:

```rust
struct Storage {
    leader: PublicState<Leader, LEADER_SERIALIZED_LEN>,
    legendary_card: Singleton<CardNote, CARD_NOTE_LEN>,
    profiles: Map<AztecAddress, Singleton<CardNote, CARD_NOTE_LEN>>,
    test: Set<CardNote, CARD_NOTE_LEN>,
    imm_singleton: PrivateImmutable<CardNote, CARD_NOTE_LEN>,
}

impl Storage {
        fn init(context: Context) -> Self {
            Storage {
                leader: PublicMutable::new(
                    context,
                    1,
                    LeaderSerializationMethods,
                ),
                legendary_card: PrivateMutable::new(context, 2, CardNoteMethods),
                profiles: Map::new(
                    context,
                    3,
                    |context, slot| {
                        PrivateMutable::new(context, slot, CardNoteMethods)
                    },
                ),
                test: Set::new(context, 4, CardNoteMethods),
                imm_singleton: PrivateImmutable::new(context, 4, CardNoteMethods),
            }
        }
    }
```

Now:

```rust
struct Storage {
    leader: PublicMutable<Leader>,
    legendary_card: Singleton<CardNote>,
    profiles: Map<AztecAddress, Singleton<CardNote>>,
    test: Set<CardNote>,
    imm_singleton: PrivateImmutable<CardNote>,
}
```

For this to work, Notes must implement Serialize, Deserialize and NoteInterface Traits. Previously:

```rust
use dep::aztec::protocol_types::address::AztecAddress;
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    oracle::{
        nullifier_key::get_nullifier_secret_key,
        get_public_key::get_public_key,
    },
    log::emit_encrypted_log,
    hash::pedersen_hash,
    context::PrivateContext,
};

// Shows how to create a custom note

global CARD_NOTE_LEN: Field = 1;

impl CardNote {
    pub fn new(owner: AztecAddress) -> Self {
        CardNote {
            owner,
        }
    }

    pub fn serialize(self) -> [Field; CARD_NOTE_LEN] {
        [self.owner.to_field()]
    }

    pub fn deserialize(serialized_note: [Field; CARD_NOTE_LEN]) -> Self {
        CardNote {
            owner: AztecAddress::from_field(serialized_note[1]),
        }
    }

    pub fn compute_note_hash(self) -> Field {
        pedersen_hash([
            self.owner.to_field(),
        ],0)
    }

    pub fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(CardNoteMethods, self);
        let secret = context.request_nullifier_secret_key(self.owner);
        pedersen_hash([
            note_hash_for_nullify,
            secret.high,
            secret.low,
        ],0)
    }

    pub fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(CardNoteMethods, self);
        let secret = get_nullifier_secret_key(self.owner);
        pedersen_hash([
            note_hash_for_nullify,
            secret.high,
            secret.low,
        ],0)
    }

    pub fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    // Broadcasts the note as an encrypted log on L1.
    pub fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        let encryption_pub_key = get_public_key(self.owner);
        emit_encrypted_log(
            context,
            (*context).this_address(),
            slot,
            encryption_pub_key,
            self.serialize(),
        );
    }
}

fn deserialize(serialized_note: [Field; CARD_NOTE_LEN]) -> CardNote {
    CardNote::deserialize(serialized_note)
}

fn serialize(note: CardNote) -> [Field; CARD_NOTE_LEN] {
    note.serialize()
}

fn compute_note_hash(note: CardNote) -> Field {
    note.compute_note_hash()
}

fn compute_nullifier(note: CardNote, context: &mut PrivateContext) -> Field {
    note.compute_nullifier(context)
}

fn compute_nullifier_without_context(note: CardNote) -> Field {
    note.compute_nullifier_without_context()
}

fn get_header(note: CardNote) -> NoteHeader {
    note.header
}

fn set_header(note: &mut CardNote, header: NoteHeader) {
    note.set_header(header)
}

// Broadcasts the note as an encrypted log on L1.
fn broadcast(context: &mut PrivateContext, slot: Field, note: CardNote) {
    note.broadcast(context, slot);
}

global CardNoteMethods = NoteInterface {
    deserialize,
    serialize,
    compute_note_hash,
    compute_nullifier,
    compute_nullifier_without_context,
    get_header,
    set_header,
    broadcast,
};
```

Now:

```rust
use dep::aztec::{
    note::{
        note_header::NoteHeader,
        note_interface::NoteInterface,
        utils::compute_note_hash_for_read_or_nullify,
    },
    oracle::{
        nullifier_key::get_nullifier_secret_key,
        get_public_key::get_public_key,
    },
    log::emit_encrypted_log,
    hash::pedersen_hash,
    context::PrivateContext,
    protocol_types::{
        address::AztecAddress,
        traits::{Serialize, Deserialize, Empty}
    }
};

// Shows how to create a custom note

global CARD_NOTE_LEN: Field = 1;

impl CardNote {
    pub fn new(owner: AztecAddress) -> Self {
        CardNote {
            owner,
        }
    }
}

impl NoteInterface for CardNote {
    fn compute_note_content_hash(self) -> Field {
        pedersen_hash([
            self.owner.to_field(),
        ],0)
    }

    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(self);
        let secret = context.request_nullifier_secret_key(self.owner);
        pedersen_hash([
            note_hash_for_nullify,
            secret.high,
            secret.low,
        ],0)
    }

    fn compute_nullifier_without_context(self) -> Field {
        let note_hash_for_nullify = compute_note_hash_for_read_or_nullify(self);
        let secret = get_nullifier_secret_key(self.owner);
        pedersen_hash([
            note_hash_for_nullify,
            secret.high,
            secret.low,
        ],0)
    }

    fn set_header(&mut self, header: NoteHeader) {
        self.header = header;
    }

    fn get_header(note: CardNote) -> NoteHeader {
        note.header
    }

    fn serialize_content(self) -> [Field; CARD_NOTE_LEN]{
        [self.owner.to_field()]
    }

    fn deserialize_content(serialized_note: [Field; CARD_NOTE_LEN]) -> Self {
        AddressNote {
            owner: AztecAddress::from_field(serialized_note[0]),
            header: NoteHeader::empty(),
        }
    }

    // Broadcasts the note as an encrypted log on L1.
    fn broadcast(self, context: &mut PrivateContext, slot: Field) {
        let encryption_pub_key = get_public_key(self.owner);
        emit_encrypted_log(
            context,
            (*context).this_address(),
            slot,
            encryption_pub_key,
            self.serialize(),
        );
    }
}
```

Public state must implement Serialize and Deserialize traits.

It is still possible to manually implement the storage initialization (for custom storage wrappers or internal types that don't implement the required traits). For the above example, the `impl Storage` section would look like this:

```rust
impl Storage {
    fn init(context: Context) -> Self {
        Storage {
            leader: PublicMutable::new(
                context,
                1
            ),
            legendary_card: PrivateMutable::new(context, 2),
            profiles: Map::new(
                context,
                3,
                |context, slot| {
                    PrivateMutable::new(context, slot)
                },
            ),
            test: Set::new(context, 4),
            imm_singleton: PrivateImmutable::new(context, 4),
        }
    }
}
```

## 0.20.0

### [Aztec.nr] Changes to `NoteInterface`

1. Changing `compute_nullifier()` to `compute_nullifier(private_context: PrivateContext)`

   This API is invoked for nullifier generation within private functions. When using a secret key for nullifier creation, retrieve it through:

   `private_context.request_nullifier_secret_key(account_address)`

   The private context will generate a request for the kernel circuit to validate that the secret key does belong to the account.

   Before:

   ```rust
    pub fn compute_nullifier(self) -> Field {
        let secret = oracle.get_secret_key(self.owner);
        pedersen_hash([
            self.value,
            secret.low,
            secret.high,
        ])
    }
   ```

   Now:

   ```rust
    pub fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
        let secret = context.request_nullifier_secret_key(self.owner);
        pedersen_hash([
            self.value,
            secret.low,
            secret.high,
        ])
    }
   ```

2. New API `compute_nullifier_without_context()`.

   This API is used within unconstrained functions where the private context is not available, and using an unverified nullifier key won't affect the network or other users. For example, it's used in `compute_note_hash_and_nullifier()` to compute values for the user's own notes.

   ```rust
   pub fn compute_nullifier_without_context(self) -> Field {
        let secret = oracle.get_nullifier_secret_key(self.owner);
        pedersen_hash([
            self.value,
            secret.low,
            secret.high,
        ])
    }
   ```

   > Note that the `get_secret_key` oracle API has been renamed to `get_nullifier_secret_key`.

## 0.18.0

### [Aztec.nr] Remove `protocol_types` from Nargo.toml

The `protocol_types` package is now being reexported from `aztec`. It can be accessed through `dep::aztec::protocol_types`.

```toml
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
```

### [Aztec.nr] key type definition in Map

The `Map` class now requires defining the key type in its declaration which _must_ implement the `ToField` trait.

Before:

```rust
struct Storage {
    balances: Map<PublicMutable<Field, FIELD_SERIALIZED_LEN>>
}

let user_balance = balances.at(owner.to_field())
```

Now:

```rust
struct Storage {
    balances: Map<AztecAddress, PublicState<Field, FIELD_SERIALIZED_LEN>>
}

let user_balance = balances.at(owner)
```

### [js] Updated function names

- `waitForSandbox` renamed to `waitForPXE` in `@aztec/aztec.js`
- `getSandboxAccountsWallets` renamed to `getInitialTestAccountsWallets` in `@aztec/accounts/testing`

## 0.17.0

### [js] New `@aztec/accounts` package

Before:

```js
import { getSchnorrAccount } from "@aztec/aztec.js"; // previously you would get the default accounts from the `aztec.js` package:
```

Now, import them from the new package `@aztec/accounts`

```js
import { getSchnorrAccount } from "@aztec/accounts";
```

### Typed Addresses

Address fields in Aztec.nr now is of type `AztecAddress` as opposed to `Field`

Before:

```rust
unconstrained fn compute_note_hash_and_nullifier(contract_address: Field, nonce: Field, storage_slot: Field, serialized_note: [Field; VALUE_NOTE_LEN]) -> [Field; 4] {
        let note_header = NoteHeader::new(_address, nonce, storage_slot);
        ...
```

Now:

```rust
unconstrained fn compute_note_hash_and_nullifier(
        contract_address: AztecAddress,
        nonce: Field,
        storage_slot: Field,
        serialized_note: [Field; VALUE_NOTE_LEN]
    ) -> pub [Field; 4] {
        let note_header = NoteHeader::new(contract_address, nonce, storage_slot);
```

Similarly, there are changes when using aztec.js to call functions.

To parse a `AztecAddress` to BigInt, use `.inner`
Before:

```js
const tokenBigInt = await bridge.methods.token().simulate();
```

Now:

```js
const tokenBigInt = (await bridge.methods.token().simulate()).inner;
```

### [Aztec.nr] Add `protocol_types` to Nargo.toml

```toml
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
protocol_types = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/noir-protocol-circuits/crates/types"}
```

### [Aztec.nr] moving compute_address func to AztecAddress

Before:

```rust
let calculated_address = compute_address(pub_key_x, pub_key_y, partial_address);
```

Now:

```rust
let calculated_address = AztecAddress::compute(pub_key_x, pub_key_y, partial_address);
```

### [Aztec.nr] moving `compute_selector` to FunctionSelector

Before:

```rust
let selector = compute_selector("_initialize((Field))");
```

Now:

```rust
let selector = FunctionSelector::from_signature("_initialize((Field))");
```

### [js] Importing contracts in JS

Contracts are now imported from a file with the type's name.

Before:

```js
import { TokenContract } from "@aztec/noir-contracts/types";
```

Now:

```js
import { TokenContract } from "@aztec/noir-contracts/Token";
```

### [Aztec.nr] Aztec example contracts location change in Nargo.toml

Aztec contracts are now moved outside of the `src` folder, so you need to update your imports.

Before:

```rust
easy_private_token_contract = {git = "https://github.com/AztecProtocol/aztec-packages/", tag ="v0.16.9", directory = "noir-projects/noir-contracts/contracts/easy_private_token_contract"}
```

Now, just remove the `src` folder,:

```rust
easy_private_token_contract = {git = "https://github.com/AztecProtocol/aztec-packages/", tag ="v0.17.0", directory = "noir-projects/noir-contracts/contracts/easy_private_token_contract"}
```

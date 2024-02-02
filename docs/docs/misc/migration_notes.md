---
title: Migration notes
description: Read about migration notes from previous versions, which could solve problems while updating
keywords: [sandbox, cli, aztec, notes, migration, updating, upgrading]
---

Aztec is in full-speed development. Literally every version breaks compatibility with the previous ones. This page attempts to target errors and difficulties you might encounter when upgrading, and how to resolve them.

## TBD

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


## 0.22.0

### [Aztec.nr] `Serialize`, `Deserialize`, `NoteInterface` as Traits, removal of SerializationMethods and SERIALIZED_LEN

Storage definition and initialization has been simplified. Previously:

```rust 
struct Storage {
    leader: PublicState<Leader, LEADER_SERIALIZED_LEN>,
    legendary_card: Singleton<CardNote, CARD_NOTE_LEN>,
    profiles: Map<AztecAddress, Singleton<CardNote, CARD_NOTE_LEN>>,
    test: Set<CardNote, CARD_NOTE_LEN>,
    imm_singleton: ImmutableSingleton<CardNote, CARD_NOTE_LEN>,
}

impl Storage {
        fn init(context: Context) -> Self {
            Storage {
                leader: PublicState::new(
                    context,
                    1,
                    LeaderSerializationMethods,
                ),
                legendary_card: Singleton::new(context, 2, CardNoteMethods),
                profiles: Map::new(
                    context,
                    3,
                    |context, slot| {
                        Singleton::new(context, slot, CardNoteMethods)
                    },
                ),
                test: Set::new(context, 4, CardNoteMethods),
                imm_singleton: ImmutableSingleton::new(context, 4, CardNoteMethods),
            }
        }
    }
```

Now: 

```rust 
struct Storage {
    leader: PublicState<Leader>,
    legendary_card: Singleton<CardNote>,
    profiles: Map<AztecAddress, Singleton<CardNote>>,
    test: Set<CardNote>,
    imm_singleton: ImmutableSingleton<CardNote>,
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

impl Serialize<CARD_NOTE_LEN> for CardNote {
    fn serialize(self) -> [Field; CARD_NOTE_LEN] {
        [self.owner.to_field()]
    }
}

impl Deserialize<CARD_NOTE_LEN> for CardNote {
    fn deserialize(serialized_note: [Field; CARD_NOTE_LEN]) -> Self {
        CardNote {
            owner: AztecAddress::from_field(serialized_note[2]),
        }
    }
}

impl NoteInterface for CardNote {
    fn compute_note_hash(self) -> Field {
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
            leader: PublicState::new(
                context,
                1
            ),
            legendary_card: Singleton::new(context, 2),
            profiles: Map::new(
                context,
                3,
                |context, slot| {
                    Singleton::new(context, slot)
                },
            ),
            test: Set::new(context, 4),
            imm_singleton: ImmutableSingleton::new(context, 4),
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
    balances: Map<PublicState<Field, FIELD_SERIALIZED_LEN>>
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
const tokenBigInt = await bridge.methods.token().view();
```

Now:

```js
const tokenBigInt = (await bridge.methods.token().view()).inner;
```

### [Aztec.nr] Add `protocol_types` to Nargo.toml

```toml
aztec = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/aztec-nr/aztec" }
protocol_types = { git="https://github.com/AztecProtocol/aztec-packages/", tag="#include_aztec_version", directory="yarn-project/noir-protocol-circuits/src/crates/types"}
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
easy_private_token_contract = {git = "https://github.com/AztecProtocol/aztec-packages/", tag ="v0.16.9", directory = "yarn-project/noir-contracts/src/contracts/easy_private_token_contract"}
```

Now, just remove the `src` folder,:

```rust
easy_private_token_contract = {git = "https://github.com/AztecProtocol/aztec-packages/", tag ="v0.17.0", directory = "yarn-project/noir-contracts/contracts/easy_private_token_contract"}
```

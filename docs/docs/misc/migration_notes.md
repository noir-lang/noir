---
title: Migration notes
description: Read about migration notes from previous versions, which could solve problems while updating
keywords: [sandbox, cli, aztec, notes, migration, updating, upgrading]
---

Aztec is in full-speed development. Literally every version breaks compatibility with the previous ones. This page attempts to target errors and difficulties you might encounter when upgrading, and how to resolve them.

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

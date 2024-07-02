---
title: Migration notes
description: Read about migration notes from previous versions, which could solve problems while updating
keywords: [sandbox, aztec, notes, migration, updating, upgrading]
---

Aztec is in full-speed development. Literally every version breaks compatibility with the previous ones. This page attempts to target errors and difficulties you might encounter when upgrading, and how to resolve them.

## 0.45.0
### [Aztec.nr] Remove unencrypted logs from private
They leak privacy so is a footgun!

## 0.44.0
### [Aztec.nr] Autogenerate Serialize methods for events
```diff
#[aztec(event)]
struct WithdrawalProcessed {
    who: Field,
    amount: Field,
}

-impl Serialize<2> for WithdrawalProcessed {
-    fn serialize(self: Self) -> [Field; 2] {
-        [self.who.to_field(), self.amount as Field]
-    }
}
```

### [Aztec.nr] rename `encode_and_encrypt_with_keys` to `encode_and_encrypt_note_with_keys`
```diff
contract XYZ {
-   use dep::aztec::encrypted_logs::encrypted_note_emission::encode_and_encrypt_with_keys;
+   use dep::aztec::encrypted_logs::encrypted_note_emission::encode_and_encrypt_note_with_keys;    
....

-    numbers.at(owner).initialize(&mut new_number).emit(encode_and_encrypt_with_keys(&mut context, owner_ovpk_m, owner_ivpk_m));
+    numbers.at(owner).initialize(&mut new_number).emit(encode_and_encrypt_note_with_keys(&mut context, owner_ovpk_m, owner_ivpk_m));

}


### [Aztec.nr] changes to `NoteInterface`

`compute_nullifier` function was renamed to `compute_note_hash_and_nullifier` and now the function has to return not only the nullifier but also the note hash used to compute the nullifier.
The same change was done to `compute_nullifier_without_context` function.
These changes were done because having the note hash exposed allowed us to not having to re-compute it again in `destroy_note` function of Aztec.nr which led to significant decrease in gate counts (see the [optimization PR](https://github.com/AztecProtocol/aztec-packages/pull/7103) for more details).

```diff
- impl NoteInterface<VALUE_NOTE_LEN, VALUE_NOTE_BYTES_LEN> for ValueNote {
-    fn compute_nullifier(self, context: &mut PrivateContext) -> Field {
-        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
-        let secret = context.request_nsk_app(self.npk_m_hash);
-        poseidon2_hash([
-            note_hash_for_nullify,
-            secret,
-            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
-        ])
-    }
-
-    fn compute_nullifier_without_context(self) -> Field {
-        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
-        let secret = get_nsk_app(self.npk_m_hash);
-        poseidon2_hash([
-            note_hash_for_nullify,
-            secret,
-            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
-        ])
-    }
- }
+ impl NoteInterface<VALUE_NOTE_LEN, VALUE_NOTE_BYTES_LEN> for ValueNote {
+    fn compute_note_hash_and_nullifier(self, context: &mut PrivateContext) -> (Field, Field) {
+        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
+        let secret = context.request_nsk_app(self.npk_m_hash);
+        let nullifier = poseidon2_hash([
+            note_hash_for_nullify,
+            secret,
+            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
+        ]);
+        (note_hash_for_nullify, nullifier)
+    }
+
+    fn compute_note_hash_and_nullifier_without_context(self) -> (Field, Field) {
+        let note_hash_for_nullify = compute_note_hash_for_consumption(self);
+        let secret = get_nsk_app(self.npk_m_hash);
+        let nullifier = poseidon2_hash([
+            note_hash_for_nullify,
+            secret,
+            GENERATOR_INDEX__NOTE_NULLIFIER as Field,
+        ]);
+        (note_hash_for_nullify, nullifier)
+    }
+ }
```

### [Aztec.nr] `note_getter` returns `BoundedVec`

The `get_notes` and `view_notes` function no longer return an array of options (i.e. `[Option<Note>, N_NOTES]`) but instead a `BoundedVec<Note, N_NOTES>`. This better conveys the useful property the old array had of having all notes collapsed at the beginning of the array, which allows for powerful optimizations and gate count reduction when setting the `options.limit` value.

A `BoundedVec` has a `max_len()`, which equals the number of elements it can hold, and a `len()`, which equals the number of elements it currently holds. Since `len()` is typically not knwon at compile time, iterating over a `BoundedVec` looks slightly different than iterating over an array of options:

```diff
- let option_notes = get_notes(options);
- for i in 0..option_notes.len() {
-     if option_notes[i].is_some() {
-         let note = option_notes[i].unwrap_unchecked();
-     }
- }
+ let notes = get_notes(options);
+ for i in 0..notes.max_len() {
+     if i < notes.len() {
+         let note = notes.get_unchecked(i);
+     }
+ }
```

To further reduce gate count, you can iterate over `options.limit` instead of `max_len()`, since `options.limit` is guaranteed to be larger or equal to `len()`, and smaller or equal to `max_len()`:

```diff
- for i in 0..notes.max_len() {
+ for i in 0..options.limit {
```

### [Aztec.nr] static private authwit

The private authwit validation is now making a static call to the account contract instead of passing over control flow. This is to ensure that it cannot be used for re-entry.

To make this change however, we cannot allow emitting a nullifier from the account contract, since that would break the static call. Instead, we will be changing the `spend_private_authwit` to a `verify_private_authwit` and in the `auth` library emit the nullifier. This means that the "calling" contract will now be emitting the nullifier, and not the account. For example, for a token contract, the nullifier is now emitted by the token contract. However, as this is done inside the `auth` library, the token contract doesn't need to change much.

The biggest difference is related to "cancelling" an authwit. Since it is no longer in the account contract, you cannot just emit a nullifier from it anymore. Instead it must rely on the token contract providing functionality for cancelling.

There are also a few general changes to how authwits are generated, namely to more easily support the data required for a validity lookup now. Previously we could lookup the `message_hash` directly at the account contract, now we instead need to use the `inner_hash` and the contract of the consumer to figure out if it have already been emitted.

A minor extension have been made to the authwit creations to make it easier to sign a specific a hash with a specific caller, e.g., the `inner_hash` can be provided as `{consumer, inner_hash}` to the `createAuthWit` where it previously needed to do a couple of manual steps to compute the outer hash. The `computeOuterAuthWitHash` have been amde internal and the `computeAuthWitMessageHash` can instead be used to compute the values similarly to other authwit computations.

```diff
const innerHash = computeInnerAuthWitHash([Fr.ZERO, functionSelector.toField(), entrypointPackedArgs.hash]);
-const outerHash = computeOuterAuthWitHash(
-    this.dappEntrypointAddress,
-    new Fr(this.chainId),
-    new Fr(this.version),
-    innerHash,
-);
+const messageHash = computeAuthWitMessageHash(
+    { consumer: this.dappEntrypointAddress, innerHash },
+    { chainId: new Fr(this.chainId), version: new Fr(this.version) },
+);
```

If the wallet is used to compute the authwit, it will populate the chain id and version instead of requiring it to be provided by tha actor.

```diff
const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead')]);
-const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), chainId, version, innerHash);
-const witness = await wallets[0].createAuthWit(outerHash);
+ const witness = await wallets[0].createAuthWit({ comsumer: accounts[1].address, inner_hash });
```

## 0.43.0

### [Aztec.nr] break `token.transfer()` into `transfer` and `transferFrom`

Earlier we had just one function - `transfer()` which used authwits to handle the case where a contract/user wants to transfer funds on behalf of another user.
To reduce circuit sizes and proof times, we are breaking up `transfer` and introducing a dedicated `transferFrom()` function like in the ERC20 standard.

### [Aztec.nr] `options.limit` has to be constant

The `limit` parameter in `NoteGetterOptions` and `NoteViewerOptions` is now required to be a compile-time constant. This allows performing loops over this value, which leads to reduced circuit gate counts when setting a `limit` value.

### [Aztec.nr] canonical public authwit registry

The public authwits are moved into a shared registry (auth registry) to make it easier for sequencers to approve for their non-revertible (setup phase) whitelist. Previously, it was possible to DOS a sequencer by having a very expensive authwit validation that fails at the end, now the whitelist simply need the registry.

Notable, this means that consuming a public authwit will no longer emit a nullifier in the account contract but instead update STORAGE in the public domain. This means that there is a larger difference between private and public again. However, it also means that if contracts need to approve, and use the approval in the same tx, it is transient and don't need to go to DA (saving 96 bytes).

For the typescript wallets this is handled so the APIs don't change, but account contracts should get rid of their current setup with `approved_actions`.

```diff
- let actions = AccountActions::init(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
+ let actions = AccountActions::init(&mut context, is_valid_impl);
```

For contracts we have added a `set_authorized` function in the auth library that can be used to set values in the registry.

```diff
- storage.approved_action.at(message_hash).write(true);
+ set_authorized(&mut context, message_hash, true);
```

### [Aztec.nr] emit encrypted logs

Emitting or broadcasting encrypted notes are no longer done as part of the note creation, but must explicitly be either emitted or discarded instead.

```diff
+ use dep::aztec::encrypted_logs::encrypted_note_emission::{encode_and_encrypt, encode_and_encrypt_with_keys};

- storage.balances.sub(from, amount);
+ storage.balances.sub(from, amount).emit(encode_and_encrypt_with_keys(&mut context, from, from));
+ storage.balances.sub(from, amount).emit(encode_and_encrypt_with_keys(&mut context, from_ovpk, from_ivpk));
+ storage.balances.sub(from, amount).discard();
```

## 0.42.0

### [Aztec.nr] Unconstrained Context

Top-level unconstrained execution is now marked by the new `UnconstrainedContext`, which provides access to the block number and contract address being used in the simulation. Any custom state variables that provided unconstrained functions should update their specialization parameter:

```diff
+ use dep::aztec::context::UnconstrainedContext;

- impl MyStateVariable<()> {
+ impl MyStateVariable<UnconstrainedContext> {
```

### [Aztec.nr] Filtering is now constrained

The `filter` argument of `NoteGetterOptions` (typically passed via the `with_filter()` function) is now applied in a constraining environment, meaning any assertions made during the filtering are guaranteed to hold. This mirrors the behavior of the `select()` function.

### [Aztec.nr] Emitting encrypted notes and logs

The `emit_encrypted_log` context function is now `encrypt_and_emit_log` or `encrypt_and_emit_note`.

```diff
- context.emit_encrypted_log(log1);
+ context.encrypt_and_emit_log(log1);
+ context.encrypt_and_emit_note(note1);
```

Broadcasting a note will call `encrypt_and_emit_note` in the background. To broadcast a generic event, use `encrypt_and_emit_log`
with the same encryption parameters as notes require. Currently, only fields and arrays of fields are supported as events.

By default, logs emitted via `encrypt_and_emit_log` will be siloed with a _masked_ contract address. To force the contract address to be revealed, so everyone can check it rather than just the log recipient, provide `randomness = 0`.

## Public execution migrated to the Aztec Virtual Machine

**What does this mean for me?**

It should be mostly transparent, with a few caveats:

- Not all Noir blackbox functions are supported by the AVM. Only `Sha256`, `PedersenHash`, `Poseidon2Permutation`, `Keccak256`, and `ToRadix` are supported.
- For public functions, `context.nullifier_exists(...)` will now also consider pending nullifiers.
- The following methods of `PublicContext` are not supported anymore: `fee_recipient`, `fee_per_da_gas`, `fee_per_l2_gas`, `call_public_function_no_args`, `static_call_public_function_no_args`, `delegate_call_public_function_no_args`, `call_public_function_with_packed_args`, `set_return_hash`, `finish`. However, in terms of functionality, the new context's interface should be equivalent (unless otherwise specified in this list).
- Delegate calls are not yet supported in the AVM.
- If you have types with custom serialization that you use across external contracts calls, you might need to modify its serialization to match how Noir would serialize it. This is a known problem unrelated to the AVM, but triggered more often when using it.
- A few error messages might change format, so you might need to change your test assertions.

**Internal details**

Before this change, public bytecode was executed using the same simulator as in private: the ACIR simulator (and internally, the Brillig VM). On the Aztec.nr side, public functions accessed the context through `PublicContext`.

After this change, public bytecode will be run using the AVM simulator (the simulator for our upcoming zkVM). This bytecode is generated from Noir contracts in two steps: First, `nargo compile` produces an artifact which has Brillig bytecode for public functions, just as it did before. Second: the `avm-transpiler` takes that artifact, and it transpiles Brillig bytecode to AVM bytecode. This final artifact can now be deployed and used with the new public runtime.

On the Aztec.nr side, public functions keep accessing the context using `PublicContext` but the underlying implementation is switch with what formerly was the `AvmContext`.

## 0.41.0

### [Aztec.nr] State variable rework

Aztec.nr state variables have been reworked so that calling private functions in public and vice versa is detected as an error during compilation instead of at runtime. This affects users in a number of ways:

#### New compile time errors

It used to be that calling a state variable method only available in public from a private function resulted in obscure runtime errors in the form of a failed `_is_some` assertion.

Incorrect usage of the state variable methods now results in compile time errors. For example, given the following function:

```rust
#[aztec(public)]
fn get_decimals() -> pub u8 {
    storage.decimals.read_private()
}
```

The compiler will now error out with

```
Expected type SharedImmutable<_, &mut PrivateContext>, found type SharedImmutable<u8, &mut PublicContext>
```

The key component is the second generic parameter: the compiler expects a `PrivateContext` (becuse `read_private` is only available during private execution), but a `PublicContext` is being used instead (because of the `#[aztec(public)]` attribute).

#### Generic parameters in `Storage`

The `Storage` struct (the one marked with `#[aztec(storage)]`) should now be generic over a `Context` type, which matches the new generic parameter of all Aztec.nr libraries. This parameter is always the last generic parameter.

This means that, without any additional features, we'd end up with some extra boilerplate when declaring this struct:

```diff
#[aztec(storage)]
- struct Storage {
+ struct Storage<Context> {
-   nonce_for_burn_approval: PublicMutable<Field>,
+   nonce_for_burn_approval: PublicMutable<Field, Context>,
-   portal_address: SharedImmutable<EthAddress>,
+   portal_address: SharedImmutable<EthAddress, Context>,
-   approved_action: Map<Field, PublicMutable<bool>>,
+   approved_action: Map<Field, PublicMutable<bool, Context>, Context>,
}
```

Because of this, the `#[aztec(storage)]` macro has been updated to **automatically inject** this `Context` generic parameter. The storage declaration does not require any changes.

#### Removal of `Context`

The `Context` type no longer exists. End users typically didn't use it, but if imported it needs to be deleted.

### [Aztec.nr] View functions and interface navigation

It is now possible to explicitly state a function doesn't perform any state alterations (including storage, logs, nullifiers and/or messages from L2 to L1) with the `#[aztec(view)]` attribute, similarly to solidity's `view` function modifier.

```diff
    #[aztec(public)]
+   #[aztec(view)]
    fn get_price(asset_id: Field) -> Asset {
        storage.assets.at(asset_id).read()
    }
```

View functions only generate a `StaticCallInterface` that doesn't include `.call` or `.enqueue` methods. Also, the denomination `static` has been completely removed from the interfaces, in favor of the more familiar `view`

```diff
- let price = PriceFeed::at(asset.oracle).get_price(0).static_call(&mut context).price;
+ let price = PriceFeed::at(asset.oracle).get_price(0).view(&mut context).price;
```

```diff
#[aztec(private)]
fn enqueue_public_get_value_from_child(target_contract: AztecAddress, value: Field) {
-   StaticChild::at(target_contract).pub_get_value(value).static_enqueue(&mut context);
+   StaticChild::at(target_contract).pub_get_value(value).enqueue_view(&mut context);
}
```

Additionally, the Noir LSP will now honor "go to definitions" requests for contract interfaces (Ctrl+click), taking the user to the original function implementation.

### [Aztec.js] Simulate changes

- `.simulate()` now tracks closer the process performed by `.send().wait()`, specifically going through the account contract entrypoint instead of directly calling the intended function.
- `wallet.viewTx(...)` has been renamed to `wallet.simulateUnconstrained(...)` to better clarify what it does.

### [Aztec.nr] Keys: Token note now stores an owner master nullifying public key hash instead of an owner address

i.e.

```diff
struct TokenNote {
    amount: U128,
-   owner: AztecAddress,
+   npk_m_hash: Field,
    randomness: Field,
}
```

Creating a token note and adding it to storage now looks like this:

```diff
- let mut note = ValueNote::new(new_value, owner);
- storage.a_private_value.insert(&mut note, true);
+ let owner_npk_m_hash = get_npk_m_hash(&mut context, owner);
+ let owner_ivpk_m = get_ivpk_m(&mut context, owner);
+ let mut note = ValueNote::new(new_value, owner_npk_m_hash);
+ storage.a_private_value.insert(&mut note, true, owner_ivpk_m);
```

Computing the nullifier similarly changes to use this master nullifying public key hash.

## 0.40.0

### [Aztec.nr] Debug logging

The function `debug_log_array_with_prefix` has been removed. Use `debug_log_format` with `{}` instead. The special sequence `{}` will be replaced with the whole array. You can also use `{0}`, `{1}`, ... as usual with `debug_log_format`.

```diff
- debug_log_array_with_prefix("Prefix", my_array);
+ debug_log_format("Prefix {}", my_array);
```

## 0.39.0

### [Aztec.nr] Mutable delays in `SharedMutable`

The type signature for `SharedMutable` changed from `SharedMutable<T, DELAY>` to `SharedMutable<T, INITIAL_DELAY>`. The behavior is the same as before, except the delay can now be changed after deployment by calling `schedule_delay_change`.

### [Aztec.nr] get_public_key oracle replaced with get_ivpk_m

When implementing changes according to a [new key scheme](https://yp-aztec.netlify.app/docs/addresses-and-keys/keys) we had to change oracles.
What used to be called encryption public key is now master incoming viewing public key.

```diff
- use dep::aztec::oracles::get_public_key::get_public_key;
+ use dep::aztec::keys::getters::get_ivpk_m;

- let encryption_pub_key = get_public_key(self.owner);
+ let ivpk_m = get_ivpk_m(context, self.owner);
```

## 0.38.0

### [Aztec.nr] Emitting encrypted logs

The `emit_encrypted_log` function is now a context method.

```diff
- use dep::aztec::log::emit_encrypted_log;
- use dep::aztec::logs::emit_encrypted_log;

- emit_encrypted_log(context, log1);
+ context.emit_encrypted_log(log1);
```

## 0.36.0

### `FieldNote` removed

`FieldNote` only existed for testing purposes, and was not a note type that should be used in any real application. Its name unfortunately led users to think that it was a note type suitable to store a `Field` value, which it wasn't.

If using `FieldNote`, you most likely want to use `ValueNote` instead, which has both randomness for privacy and an owner for proper nullification.

### `SlowUpdatesTree` replaced for `SharedMutable`

The old `SlowUpdatesTree` contract and libraries have been removed from the codebase, use the new `SharedMutable` library instead. This will require that you add a global variable specifying a delay in blocks for updates, and replace the slow updates tree state variable with `SharedMutable` variables.

```diff
+ global CHANGE_ROLES_DELAY_BLOCKS = 5;

struct Storage {
-  slow_update: SharedImmutable<AztecAddress>,
+  roles: Map<AztecAddress, SharedMutable<UserFlags, CHANGE_ROLES_DELAY_BLOCKS>>,
}
```

Reading from `SharedMutable` is much simpler, all that's required is to call `get_current_value_in_public` or `get_current_value_in_private`, depending on the domain.

```diff
- let caller_roles = UserFlags::new(U128::from_integer(slow.read_at_pub(context.msg_sender().to_field()).call(&mut context)));
+ let caller_roles = storage.roles.at(context.msg_sender()).get_current_value_in_public();
```

Finally, you can remove all capsule usage on the client code or tests, since those are no longer required when working with `SharedMutable`.

### [Aztec.nr & js] Portal addresses

Deployments have been modified. No longer are portal addresses treated as a special class, being immutably set on creation of a contract. They are no longer passed in differently compared to the other variables and instead should be implemented using usual storage by those who require it. One should use the storage that matches the usecase - likely shared storage to support private and public.

This means that you will likely add the portal as a constructor argument

```diff
- fn constructor(token: AztecAddress) {
-    storage.token.write(token);
- }
+ struct Storage {
    ...
+   portal_address: SharedImmutable<AztecAddress>,
+ }
+ fn constructor(token: AztecAddress, portal_address: EthAddress) {
+    storage.token.write(token);
+    storage.portal_address.initialize(portal_address);
+ }
```

And read it from storage whenever needed instead of from the context.

```diff
- context.this_portal_address(),
+ storage.portal_address.read_public(),
```

### [Aztec.nr] Oracles

Oracle `get_nullifier_secret_key` was renamed to `get_app_nullifier_secret_key` and `request_nullifier_secret_key` function on PrivateContext was renamed as `request_app_nullifier_secret_key`.

```diff
- let secret = get_nullifier_secret_key(self.owner);
+ let secret = get_app_nullifier_secret_key(self.owner);
```

```diff
- let secret = context.request_nullifier_secret_key(self.owner);
+ let secret = context.request_app_nullifier_secret_key(self.owner);
```

### [Aztec.nr] Contract interfaces

It is now possible to import contracts on another contracts and use their automatic interfaces to perform calls. The interfaces have the same name as the contract, and are automatically exported. Parameters are automatically serialized (using the `Serialize<N>` trait) and return values are automatically deserialized (using the `Deserialize<N>` trait). Serialize and Deserialize methods have to conform to the standard ACVM serialization schema for the interface to work!

1. Only fixed length types are supported
2. All numeric types become Fields
3. Strings become arrays of Fields, one per char
4. Arrays become arrays of Fields following rules 2 and 3
5. Structs become arrays of Fields, with every item defined in the same order as they are in Noir code, following rules 2, 3, 4 and 5 (recursive)

```diff
- context.call_public_function(
-   storage.gas_token_address.read_private(),
-   FunctionSelector::from_signature("pay_fee(Field)"),
-   [42]
- );
-
- context.call_public_function(
-   storage.gas_token_address.read_private(),
-   FunctionSelector::from_signature("pay_fee(Field)"),
-   [42]
- );
-
- let _ = context.call_private_function(
-           storage.subscription_token_address.read_private(),
-           FunctionSelector::from_signature("transfer((Field),(Field),Field,Field)"),
-           [
-            context.msg_sender().to_field(),
-            storage.subscription_recipient_address.read_private().to_field(),
-            storage.subscription_price.read_private(),
-            nonce
-           ]
-  );
+ use dep::gas_token::GasToken;
+ use dep::token::Token;
+
+ ...
+ // Public call from public land
+ GasToken::at(storage.gas_token_address.read_private()).pay_fee(42).call(&mut context);
+ // Public call from private land
+ GasToken::at(storage.gas_token_address.read_private()).pay_fee(42).enqueue(&mut context);
+ // Private call from private land
+ Token::at(asset).transfer(context.msg_sender(), storage.subscription_recipient_address.read_private(), amount, nonce).call(&mut context);
```

It is also possible to use these automatic interfaces from the local contract, and thus enqueue public calls from private without having to rely on low level `context` calls.

### [Aztec.nr] Rename max block number setter

The `request_max_block_number` function has been renamed to `set_tx_max_block_number` to better reflect that it is not a getter, and that the setting is transaction-wide.

```diff
- context.request_max_block_number(value);
+ context.set_tx_max_block_number(value);
```

### [Aztec.nr] Get portal address

The `get_portal_address` oracle was removed. If you need to get the portal address of SomeContract, add the following methods to it

```
#[aztec(private)]
fn get_portal_address() -> EthAddress {
    context.this_portal_address()
}

#[aztec(public)]
fn get_portal_address_public() -> EthAddress {
    context.this_portal_address()
}
```

and change the call to `get_portal_address`

```diff
- let portal_address = get_portal_address(contract_address);
+ let portal_address = SomeContract::at(contract_address).get_portal_address().call(&mut context);
```

### [Aztec.nr] Required gas limits for public-to-public calls

When calling a public function from another public function using the `call_public_function` method, you must now specify how much gas you're allocating to the nested call. This will later allow you to limit the amount of gas consumed by the nested call, and handle any out of gas errors.

Note that gas limits are not yet enforced. For now, it is suggested you use `dep::aztec::context::gas::GasOpts::default()` which will forward all available gas.

```diff
+ use dep::aztec::context::gas::GasOpts;

- context.call_public_function(target_contract, target_selector, args);
+ context.call_public_function(target_contract, target_selector, args, GasOpts::default());
```

Note that this is not required when enqueuing a public function from a private one, since top-level enqueued public functions will always consume all gas available for the transaction, as it is not possible to handle any out-of-gas errors.

### [Aztec.nr] Emitting unencrypted logs

The `emit_unencrypted_logs` function is now a context method.

```diff
- use dep::aztec::log::emit_unencrypted_log;
- use dep::aztec::log::emit_unencrypted_log_from_private;

- emit_unencrypted_log(context, log1);
- emit_unencrypted_log_from_private(context, log2);
+ context.emit_unencrypted_log(log1);
+ context.emit_unencrypted_log(log2);
```

## 0.33

### [Aztec.nr] Storage struct annotation

The storage struct now identified by the annotation `#[aztec(storage)]`, instead of having to rely on it being called `Storage`.

```diff
- struct Storage {
-    ...
- }
+ #[aztec(storage)]
+ struct MyStorageStruct {
+    ...
+ }
```

### [Aztec.js] Storage layout and note info

Storage layout and note information are now exposed in the TS contract artifact

```diff
- const note = new Note([new Fr(mintAmount), secretHash]);
- const pendingShieldStorageSlot = new Fr(5n); // storage slot for pending_shields
- const noteTypeId = new Fr(84114971101151129711410111011678111116101n); // note type id for TransparentNote
- const extendedNote = new ExtendedNote(
-   note,
-   admin.address,
-   token.address,
-   pendingShieldStorageSlot,
-   noteTypeId,
-   receipt.txHash,
- );
- await pxe.addNote(extendedNote);
+ const note = new Note([new Fr(mintAmount), secretHash]);
+ const extendedNote = new ExtendedNote(
+   note,
+   admin.address,
+   token.address,
+   TokenContract.storage.pending_shields.slot,
+   TokenContract.notes.TransparentNote.id,
+   receipt.txHash,
+ );
+ await pxe.addNote(extendedNote);
```

### [Aztec.nr] rand oracle is now called unsafe_rand

`oracle::rand::rand` has been renamed to `oracle::unsafe_rand::unsafe_rand`.
This change was made to communicate that we do not constrain the value in circuit and instead we just trust our PXE.

```diff
- let random_value = rand();
+ let random_value = unsafe_rand();
```

### [AztecJS] Simulate and get return values for ANY call and introducing `prove()`

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

```diff
await expect(
-   asset.withWallet(wallets[1]).methods.update_admin(newAdminAddress).simulate()).rejects.toThrow(
+   asset.withWallet(wallets[1]).methods.update_admin(newAdminAddress).prove()).rejects.toThrow(
        "Assertion failed: caller is not admin 'caller_roles.is_admin'",
);
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
fn verify_private_authwit(inner_hash: Field) -> Field {
    let actions = AccountActions::private(&mut context, ACCOUNT_ACTIONS_STORAGE_SLOT, is_valid_impl);
    actions.verify_private_authwit(inner_hash)
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
# Account Circuit

## Background

Aztec accounts are different from Ethereum addresses, mainly because deriving an Ethereum address is expensive (constraint-wise) within a circuit. Also, Aztec accounts have several extra features:

- A human-readable name (an `alias`) can be associated with an account public key.
- Multiple (unlimited) spending keys (a.k.a. signing keys) can be associated with an `alias` and its `account_public_key`, to enable users to more-easily spend from multiple devices (for example).
- Spending keys can also be used for account recovery (e.g. with the aid of a 3rd party).
- If the account private key is compromised, a user can migrate to a new `account_public_key`. (They would also need to transfer all of their existing value notes to be owned by this new `account_public_key`).
- If a spending private key is compromised, a user can also migrate to a new `account_public_key`, and a brand new set of spending keys can be associated with this new `account_public_key`. (They would also need to transfer all of their existing value notes to be owned by this new `account_public_key`).

Keys:

- Spending/signing keys are used to _spend_ value notes.
- Account keys are used to decrypt encrypted value note data.
  - Also, initially (before any alias or signing keys are linked to the account), the 0th account key serves as a spending key for a user's value notes. Thereafter only spending keys can be used to spend notes.

_See the diagram (below) for derivations of the various keys._

## Keys

| Key Name              | Derivation |
| --------------------- | ---- |
| `eth_private_key`     | Random 256 bits |
| `eth_public_key`      | `eth_private_key * secp256k1.generator` |
| `eth_address`         | The right-most 160-bits of `keccak256(eth_public_key)` |
| `account_private_key` | The first 32-bytes of the signature:<br/>`eth_sign("\x19Ethereum Signed Message:\n" + len(message) + message, eth_address)`*<br/><br/>where `message = "Sign this message to generate your Aztec Privacy Key. This key lets the application decrypt your balance on Aztec.\n\nIMPORTANT: Only sign this message if you trust the application."`<br/><br/>*using a client which has access to your `eth_address`'s private key, for signing.|
| `account_public_key`  | `account_private_key * grumpkin.generator` |
| `spending_private_key`<br/>a.k.a. `signing_private_key`| Random 256 bits |
| `spending_public_key`<br/>a.k.a. `signing_public_key` | `spending_private_key * grumpkin.generator` |


## Account Glossary

| Name                  | Definition | Description |
| --------------------- | --- | --- |
| account               |  | An account is generally used to mean an `(alias_hash, account_public_key)` pair.|
| `alias`               | E.g. `alice` | Some unique human-readable string. |
| `alias_hash`          | The first 28-bytes of `blake2s_to_field(alias)`.<br/>QUESTION: how does the output of blake2s get mapped to a field element? | A constant-sized representation of an `alias`, for use in circuits. |
| `account_note` | <pre>{<br/>&nbsp;&nbsp;alias_hash,<br/>&nbsp;&nbsp;account_public_key<br/>&nbsp;&nbsp;spending_public_key<br/>}</pre> | Links together a user's `alias_hash`, their `account_public_key` and one of their `spending_public_key`s.<br/><br/>A user can register multiple account notes as a way of registering multiple `spending_public_keys` against their account. They might, for example, want to be able to spend from different devices without needing to share keys between them.<br/><br/>A user can also create a new account note as a way of registering a new `account_public_key` against their `alias_hash`. Ideally, a user would use just one `account_public_key` at a time (and transfer all value notes to be owned by that `account_public_key`), but this is not enforced by the protocol. |
| `account_note.commitment` | <pre>pedersen::compress(<br/>&nbsp;&nbsp;alias_hash,<br/>&nbsp;&nbsp;account_public_key.x,<br/>&nbsp;&nbsp;spending_public_key.x<br/>)</pre> | Each account note commitment is stored in the data tree, so that our circuits can check whether spending and account keys have been correctly registered and actually belong to the user executing the transaction. |
| `alias_hash_nullifier` | <pre>pedersen::compress(<br/>&nbsp;&nbsp;alias_hash<br/>)</pre> | This nullifier is added to the nullifier tree (by the rollup circuit) when executing the account circuit in `create` mode. It prevents an `alias` from ever being registered again by another user. |
| `account_public_key_nullifier` | <pre>pedersen::compress(<br/>&nbsp;&nbsp;account_public_key.x,<br/>&nbsp;&nbsp;account_public_key.y<br/>)</pre> | This nullifier is added to the nullifier tree (by the rollup circuit) when executing the account circuit in `create` or `migrate` modes. It prevents an `account_public_key` from ever being registered again by another user. |

## Modes: create, update, migrate

The account circuit can be executed in one of three 'modes':

- **Create**
    - Used to register a new `alias`.
    - A new 'account' is registered by generating nullifiers for a new `alias_hash` and a new `account_public_key`. This ensures the `alias_hash` and `account_public_key` haven't already been registered by someone else.
    - Two new `account_notes` may be created, as a way of registering the first two new `spending_public_keys` against the new account.
    - The circuit enforces that the caller knows the private key of `account_public_key`, by checking that a signature over the circuit's inputs has been signed by the `account_private_key`. We need to do this, in part, because the owner of this `account_public_key` might already have been sent value notes, even before registering it with Aztec Connect.
    - > Note: there are no protocol checks to ensure these new `spending_public_keys` (which are added to `account_notes`) are new or unique.
    - > Note: There are no protocol checks during `create`, to ensure the user knows private keys to these `spending_public_keys`.
- **Update**
  - Used to add _additional_ spending keys to an account.
  - Every account tx in `update` mode adds up-to two new spending keys to an account.
  - Two new `account_notes` are created, as a way of registering the two new `spending_public_keys` against the account.
  - No nullifiers are produced.
  - The circuit enforces that the caller knows the private key of an existing `signing_public_key` for this account, by:
    - checking that a signature over the circuit's inputs has been signed by a `signing_private_key`; and
    - checking that this `signing_public_key` is contained in an `account_note`'s commitment and that this commitment exists in the data tree.
  - > Note: There are no protocol checks during `update`, to ensure the user knows private key to the `account_public_key`.
- **Migrate**
  - Used to update a user's `account_public_key` without changing their `alias`.
  - The new 'account' is registered by generating a nullifier for the new `account_public_key`.
  - Two new `account_notes` may be created, as a way of registering the first two new `spending_public_keys` against this new account.
  - The circuit enforces that the caller knows the private key of an existing `signing_public_key` for this account, by:
    - checking that a signature over the circuit's inputs has been signed by a `signing_private_key`; and
    - checking that this `signing_public_key` is contained in an `account_note`'s commitment and that this commitment exists in the data tree.
  - > Note: There are no protocol checks during `migrate`, to ensure the user knows private key to the `account_public_key`.

### When to migrate?

If a user, Alice, suspects their `account_private_key` or `spending_private_key` have been compromised, then they should run the account circuit in `migrate` mode. As already stated, this will associate a new `account_public_key` to their `alias` and allow them to register new `spending_public_keys` against this new `account_public_key`. Two new account notes get created by the account circuit in `migrate` mode.

HOWEVER, the previous, 'old' account notes (containing the 'old' compromised key(s)), DO NOT get nullified. They are forever 'valid' notes in the data tree. Therefore, if Alice still owns _value_ notes which are owned by one of her old `account_public_keys`, an attacker (who somehow knows the _private_ key and a corresponding old `spending_private_key`) would still be able to spend such value notes. Therefore, after migrating their account, a user MUST ALSO transfer all of their existing notes to be owned by their new `account_public_key`.

## Example of account circuit modes

Each row of the table shows the data created by one execution of the account circuit. Rows are chronologically ordered.

| Mode    | alias | alias_hash | account public key | new spending keys  | signer | new `alias_hash_`<br/>`nullifier` emitted | new `account_`<br/>`public_key_`<br/>`nullifier` emitted | new account note commitments |
|---|---|---|---|---|---|---|---|---|
| create  | `alice` | `h(alice)` | `apk_1` | `spk_1a, spk_1b` | `apk_1` | `h(h(alice))` | `h(apk_1.x, apk_1.y)` | `h(h(alice), apk_1, spk_1a)`<br/><br/>`h(h(alice), apk_1, spk_1b)` | 
| update  | `alice` | `h(alice)` | `apk_1` | `spk_1c, spk_1d` | `spk_1b` (e.g.) | - | - | `h(h(alice), apk_1, spk_1c)`<br/><br/>`h(h(alice), apk_1, spk_1d)` | 
| update  | `alice` | `h(alice)` | `apk_1` | `spk_1e, spk_1f` | `spk_1a` (e.g.) | - | - | `h(h(alice), apk_1, spk_1e)`<br/><br/>`h(h(alice), apk_1, spk_1f)` | 
| migrate | `alice` | `h(alice)` | `apk_2` | `spk_2a, spk_2b` | `spk_1d` (e.g.) | - | `h(apk_2.x, apk_2.y)` | `h(h(alice), apk_2, spk_2a)`<br/><br/>`h(h(alice), apk_2, spk_2b)` |
| update  | `alice` | `h(alice)` | `apk_2` | `spk_2c, spk_2d` | `spk_2b` (e.g.) | - | - | `h(h(alice), apk_2, spk_2c)`<br/><br/>`h(h(alice), apk_2, spk_2d)` | 

> Note: `h` is lazy notation, being used interchangeably in this table for different hashes. Consult the earlier tables or the below pseudocode for clarity on which hashes specifically are used.
> Note: after an account `migrate`, all previous value notes should be transferred (via the join-split circuit) to be owned by the new account public key.

## More on Nullifiers

Unlike the join-split circuit (for example), which always produces nullifiers, the account circuit only conditionally produces nullifiers (see the different modes above). It's possible for `nullifier_1` or `nullifier_2` to be `0`:

- `nullifier_1 = create ? pedersen::compress(account_alias_hash) : 0;`

- `nullifier_2 = (create || migrate) ? pedersen::compress(account_public_key) : 0`

> Note: The rollup circuit for Aztec Connect permits unlimited `0` nullifiers to be added to the nullifier tree, because:
> - Each nullifier is added to the nullifier tree at the leaf index which is equal to the nullifier value.
> - So the rollup circuit will try to add `nullifier = 0` to `leafIndex = 0`.
> - First it checks whether the leaf is empty. Well `0` implies "empty", so this check will pass, and the value `0` will be once-again added to the 0th leaf.

## Diagram

[Here's](https://drive.google.com/file/d/1iscYm-B89I9LIB7YgM_L9cHaV6SSMjRT/view?usp=sharing) a detailed diagram of how all of Aztec's different keypairs are derived, and the flow of account creation and migration. (Edits are welcome - let Mike know if the link doesn't work).

# The circuit

## Account Circuit: Worked Example

_There's a little diagram at the diagrams link too._

1. Alice generates a grumpkin key pair `(account_private_key, account_public_key)`.
1. Alice can receive funds prior to registering an `alias` at `(account_public_key)`
   - I.e. a sender can send Alice funds by creating a value note with preimage values:
     - `owner = account_public_key`
     - `requires_account = false`
1. Alice can register the alias `alice` against her `account_public_key` using the account circuit.
   - The `alias_hash = hash('alice')` gets nullified, effectively 'reserving' the alias `alice` to prevent anyone else using it.
   - The `account_public_key` gets nullified, to prevent anyone else using it.
   - Alice's `new_account_public_key`, her `alias_hash`, and two new spending keys, are all linked together via two new account notes which get added to the data tree.
2. Alice must then transfer any previously-received funds that were sent to `(account_public_key)` (i.e. value notes where `requires_account = false`), to value notes whose primage values contain `(account_public_key, requires_account = true)`.
3. Alice can register unlimited additional spending keys to `(alice, account_public_key)`, via additional calls to the account circuit (in `upgrade` mode).
4. If a `spending_public_key` becomes compromised, Alice must do the following:

- generate a _new_ account note with a `new_account_public_key` and her existing `alice` alias (using the `migrate` flow). The new account note's spending keys SHOULD be different to the compromised key (although there are no protocol checks to enforce this).
- Use the account `update` flow to assign additional _non-comprimised_ spending keys to her new account note`.
- Alice transfers funds assigned to `(account_public_key, alice)` and sends them to `(new_account_public_key, alice)`

1. Similarly, if Alice's `account_private_key` becomes compromised, she can use the account circuit to migrate to a new `account_public_key`.

## Circuit Inputs: Summary

The inputs for the account circuit are:

$$ 
\text{Account Inputs} = (\text{Public Inputs}, \text{Private Inputs}) \in \mathbb{F}\_p^{13} \times \mathbb{F}\_p^{25}
$$

As previously, the field $\mathbb{F}_p$ is from the BN254 specification.

### Public Inputs: Detail

Recall that all inner circuits must have the **same number of public inputs** as they will be used homogenously by the rollup circuit. Hence, most of the account circuit's public inputs are 0, because they're not actually needed for the account circuit's functionality.

1. `proof_id = PublicInputs::ACCOUNT` (i.e. this is effectively a witness which can only take one valid value).
1. `output_note_commitment_1`
1. `output_note_commitment_2`
1. `nullifier_1`
1. `nullifier_2`
1. `public_value = 0`
1. `public_owner = 0`
1. `asset_id = 0`
1. `data_tree_root`
1. `tx_fee = 0`
1. `tx_fee_asset_id = 0`
1. `bridge_call_data = 0`
1. `defi_deposit_value = 0`
1. `defi_root = 0`
1. `backward_link = 0`
1. `allow_chain = 0`

### Private Inputs: Detail

1. `account_public_key`
1. `new_account_public_key`
1. `signing_public_key`
1. `signature`
1. `new_signing_public_key_1`
1. `new_signing_public_key_1`
1. `alias_hash = blake2s(alias).slice(0, 28)`
1. `account_nonce`
1. `create` (bool)
1. `migrate` (bool)
1. `account_note_index`
1. `account_note_path`

## Circuit Logic (Pseudocode)

Computed vars:

- `signer` = `signing_public_key`
- `message` = `pedersen::compress(alias_hash, account_public_key.x, new_account_public_key.x, spending_public_key_1.x, spending_public_key_2.x, nullifier_1, nullifier_2)`
- `account_note_commitment` = `pedersen::compress(alias_hash, account_public_key.x, signer.x)`

Computed public inputs:

- `output_note_commitment_1` = `pedersen::compress(alias_hash, new_account_public_key.x, spending_public_key_1.x)`
- `output_note_commitment_2` = `pedersen::compress(alias_hash, new_account_public_key.x, spending_public_key_2.x)`
- `nullifier_1` = `create ? pedersen::compress(alias_hash) : 0;`
- `nullifier_2` = `create || migrate ? pedersen::compress(new_account_public_key)`

Circuit constraints:

- `create == 1 || create == 0`
- `migrate == 1 || migrate == 0`
- `require(create && migrate == 0)`
- `require(new_account_public_key != spending_public_key_1)`
- `require(new_account_public_key != spending_public_key_2)`
- `if (migrate == 0) { require(account_public_key == new_account_public_key) }`
- `verify_signature(message, signer, signature) == true`
- `if (create == false) { require(membership_check(account_note_data, account_note_index, account_note_path, data_tree_root) == true) }`
- Assert all 'zeroed' public inputs are indeed zero.

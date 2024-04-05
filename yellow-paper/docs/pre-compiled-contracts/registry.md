# Registry

<!-- Review notes:
"Registry" needs an adjective. Public Key Directory? Public Key Registry? Address, Public Key & Precompile Preferences Registry (Pronounced "P(ic)K(u)Pp" Registry)? Key & Precompile Preferences Registry?
-->

The protocol should allow users to express their preferences in terms of encryption & tagging mechanisms, and also provably advertise their encryption & tagging public keys. A canonical registry contract provides an application-level solution to both problems.

## Overview and Usage

At the application level, a canonical singleton contract allows accounts to register their public keys and their preference for encryption & tagging methods. This data is kept in public storage for anyone to check when they need to send a note to an account.

An account can directly call the registry via a public function to set or update their public keys and their encryption & tagging preferences. New accounts should register themselves on deployment. Alternatively, anyone can create an entry for a new account (but not update) if they demonstrate that the public key and encryption & tagging method can be hashed to the new account's address. This allows third party services to register addresses to improve usability.

An app contract can provably read the registry during private execution via a merkle membership proof against a recent public state root, using the [archive tree](../state/archive.md). The rationale for not making a call to the registry to read is to reduce the number of function calls. When reading public state from private-land, apps must set a `max_block_number` for the current transaction to ensure the public state root is not more than `N = max_block_number - current_block_number` blocks old. This means that, if a user rotates their public key, for at most `N` blocks afterwards they may still receive notes encrypted using their old public key, which we consider to be acceptable.

An app contract can also prove that an address is not registered in the registry via a non-inclusion proof, since the public state tree is implemented as an indexed merkle tree. To prevent an app from proving that an address is not registered when in fact it was registered less than N blocks ago, we implement this check as a public function. This means that the transaction may leak that an undisclosed application attempted to interact with a non-registered address but failed.

Note that, if an account is not registered in the registry, a sender could choose to supply the public key along with the preimage of an address on-the-fly <!-- Supply the key and preimage to where? To the registry or to the app circuits? -->, if this preimage was shared with them off-chain. This allows a user to send notes to a recipient before the recipient has deployed their account contract.

## Pseudocode

The registry contract exposes functions for setting public keys and encryption methods, plus a public function for proving non-membership of some address. Reads are meant to be done directly via storage proofs and not via calls to save on proving times. Encryption and tagging preferences are expressed via their associated precompile address.

<!-- We should also provide a function which allows _current_ data to be read from the registry, via a public function. Some app devs might not mind the extra cost. And for a public function which wants to access this information, the cost of a call is tiny. -->
<!-- TODO: explain what precompile_address.VALIDATE_KEYS does (if not already done in the precompiles section), and link to that section -->
<!-- TODO: link to all constants ENCRYPTION_PRECOMPILE_ADDRESS_RANGE, MAX_KEYS_LENGTH, MAX_ENTRIES_PER_ADDRESS -->
<!-- Q: why is there a MAX_KEYS_LENGTH -->
<!-- See a couple of inlined comments in the code snippet below -->
<!-- Should we have functions to update keys / precompile address in isolation? -->
<!-- Might there be cases where a user would wish to specify a different precompile preference (or a different set of keys) for different apps? Perhaps that's too complicated, and the user would need to spawn separate account contracts to cope with such complexity? -->

```rust
contract Registry

    public mapping(address => { keys, precompile_address }) registry

    public fn set(keys, precompile_address)
        this.do_set(msg_sender, keys, precompile_address)

    public fn set_from_preimage(address, keys, precompile_address, ...address_preimage)
        assert address not in registry
        assert hash(keys, precompile_address, ...address_preimage) == address
        // Q: Shouldn't this be `this.do_set(address, keys, precompile_address)`?
        this.set(msg_sender, keys, precompile_address)

    public fn assert_non_membership(address)
        assert address not in registry

    internal public fn do_set(address, keys, precompile_address)
        assert precompile_address in ENCRYPTION_PRECOMPILE_ADDRESS_RANGE
        assert precompile_address.validate_keys(keys)
        assert keys.length < MAX_KEYS_LENGTH
        // Q: Shouldn't this be `registry[address] = ... ?`
        registry[msg_sender] = { keys, precompile_address }
```

## Storage Optimizations

The registry stores a struct for each user, which means that each entry requires multiple storage slots. Reading multiple storage slots requires multiple merkle membership proofs, which increase the total proving cost of any execution that needs access to the registry.

To reduce the number of merkle membership proofs, the registry keeps in storage only the hash of the data stored, and emits the preimage as an unencrypted event. <!-- @spalladino Do we still want to adopt this, given your dislike of using logs to convey public state? --> Nodes are expected to store these preimages, so they can be returned when clients query for the public keys for an address. Clients then prove that the preimage hashes to the commitment stored in the public data tree via a single merkle membership proof.

Note that this optimization may also be included natively into the protocol, [pending this discussion](https://forum.aztec.network/t/storing-data-of-arbitrary-length-in-the-public-data-tree/2669).

## Multiple Recipients per Address

While account contracts that belong to individual users have a clear set of public keys to announce, some private contracts may be shared by a group of users, like in a multisig or an escrow contract. In these scenarios, we want all messages intended for the shared contract to actually be delivered to all participants, using the encryption method selected by each.

This can be achieved by having the registry support multiple sets of keys and precompiles for each entry. Applications can then query the registry and obtain a list of recipients, rather than a single one.

The registry limits multi-recipient registrations to no more than `MAX_ENTRIES_PER_ADDRESS` to prevent abuse, since this puts an additional burden on the sender, who needs to emit the same note multiple times, increasing the cost of their transaction.

Contracts that intend to register multiple recipients should account for those recipients eventually rotating their keys. To support this, contracts should include a method to refresh the registered addresses:

```rust
contract Sample

    private address[] owners

    private fn register()
        let to_register = owners.map(owner => read_registry(owner))
        registry.set(this, to_register)
```

<!-- TODO: Decide whether we want to bake this in. It means a sender may have to pay extra because of a choice by the recipient, since they may be forced to emit multiple notes, which costs calldata. I'm leaning towards not doing it. -->
<!-- Yes, that's a tricky one. It seems unfair to burden a sender with extra gas just because of the entity they're interacting with... -->
<!-- What alternative methods are there for broadcasting private messages to multiple people? -->

## Discussion

See [_Addresses, keys, and sending notes (Dec 2023 edition)_](https://forum.aztec.network/t/addresses-keys-and-sending-notes-dec-2023-edition/2633) for relevant discussions on this topic.

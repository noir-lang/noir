---
sidebar_position: 4
---

# Registry

The protocol should allow users to express their preferences in terms of encryption and note tagging mechanisms, and also provably advertise their encryption public keys. A canonical registry contract provides an application-level solution to both problems.

## Overview and Usage

At the application level, a canonical singleton contract allows accounts register their public keys and their preference for encryption and tagging methods. This data is kept in public storage for anyone to check when they need to send a note to an account.

An account can directly call the registry via a public function to set or update their public keys and encryption method. New accounts should register themselves on deployment. Alternatively, anyone can create an entry for a new account (but not update) if they show the public key and encryption method can be hashed to the address. This allows third party services to register addresses to improve usability.

An app contract can provably read the registry during private execution via a merkle membership proof against the latest public state root. Rationale for not making a call to the registry to read is to reduce the number of function calls. When reading public state from private-land, apps must set a max-block-number for the current transaction to ensure the public state root is not more than N blocks old. This means that, if a user rotates their public key, for at most N blocks afterwards they may still receive notes encrypted using their old public key, which we consider to be acceptable.

An app contract can also prove that an address is not registered in the registry via a non-inclusion proof, since the public state tree is implemented as an indexed merkle tree. To prevent an app from proving that an address is not registered when in fact it was registered less than N blocks ago, we implement this check as a public function. This means that the transaction may leak that an undisclosed application attempted to interact with a non-registered address but failed.

Note that, if an account is not registered in the registry, a sender could choose to supply the public key along with the preimage of an address on-the-fly, if this preimage was shared with them off-chain. This allows a user to send notes to a recipient before the recipient has deployed their account contract.

## Pseudocode

The registry contract exposes functions for setting public keys and encryption methods, plus a public function for proving non-membership. Reads are meant to be done directly via storage proofs and not via calls to save on proving times. Encryption and tagging preferences are expressed via their associated precompile address.

```
contract Registry
    
    public mapping(address => { keys, precompile_address }) registry
        
    public fn set(keys, precompile_address)
        this.do_set(msg_sender, keys, precompile_address)
        
    public fn set_from_preimage(address, keys, precompile_address, ...address_preimage)
        assert address not in registry
        assert hash(keys, precompile_address, ...address_preimage) == address
        this.set(msg_sender, keys, precompile_address)    
    
    public fn assert_non_membership(address)
        assert address not in registry

    internal public fn do_set(address, keys, precompile_address)
        assert precompile_address in ENCRYPTION_PRECOMPILE_ADDRESS_RANGE
        assert precompile_address.validate_keys(keys)
        assert keys.length < MAX_KEYS_LENGTH
        registry[msg_sender] = { keys, precompile_address }
```

## Storage Optimizations

The registry stores a struct for each user, which means that each entry requires multiple storage slots. Reading multiple storage slots requires multiple merkle membership proofs, which increase the total proving cost of any execution that needs access to the registry.

To reduce the number of merkle membership proofs, the registry keeps in storage only the hash of the data stored, and emits the preimage as an unencrypted event. Nodes are expected to store these preimages, so they can be returned when clients query for the public keys for an address. Clients then prove that the preimage hashes to the commitment stored in the public data tree via a single merkle membership proof.

Note that this optimization may also be included natively into the protocol, [pending this discussion](https://forum.aztec.network/t/storing-data-of-arbitrary-length-in-the-public-data-tree/2669).

## Multiple Recipients per Address

While account contracts that belong to individual users have a clear set of public keys to announce, some private contracts may be shared by a group of users, like in a multisig or an escrow contract. In these scenarios, we want all messages intended for the shared contract to actually be delivered to all participants, using the encryption method selected by each.

This can be achieved by having the registry support multiple sets of keys and precompiles for each entry. Applications can then query the registry and obtain a list of recipients, rather than a single one. 

The registry limits multi-recipient registrations to no more than `MAX_ENTRIES_PER_ADDRESS` to prevent abuse, since this puts an additional burden on the sender, who needs to emit the same note multiple times, increasing the cost of their transaction.

Contracts that intend to register multiple recipients should account for those recipients eventually rotating their keys. To support this, contracts should include a method to refresh the registered addresses:

```
contract Sample

    private address[] owners

    private fn register()
        let to_register = owners.map(owner => read_registry(owner))
        registry.set(this, to_register)
```

<!-- TODO: Decide whether we want to bake this in. It means a sender may have to pay extra because of a choice by the recipient, since they may be forced to emit multiple notes, which costs calldata. I'm leaning towards not doing it. -->

## Discussion

See [_Addresses, keys, and sending notes (Dec 2023 edition)_](https://forum.aztec.network/t/addresses-keys-and-sending-notes-dec-2023-edition/2633) for relevant discussions on this topic.
---
title: Diversified and Stealth Accounts
---

The [keys specification](./specification.md) describes derivation mechanisms for diversified and stealth public keys. However, the protocol requires users to interact with addresses.

## Computing Addresses

To support diversified and stealth accounts, a user may compute the deterministic address for a given account contract that is deployed using the diversified or stealth public key, so a sender can interact with the resulting address even before the account contract is deployed.

When the user wants to access the notes that were sent to the diversified or stealth address, they can deploy the contract at their address, and control it privately from their main account.

## Account Contract Pseudocode

As an example implementation, account contracts for diversified and stealth accounts can be designed to require no private constructor or state, and delegate entrypoint access control to their master address.

```
contract DiversifiedAccount

    private fn entrypoint(payload: action[])
        assert msg_sender == get_owner_address()
        execute(payload)

    private fn is_valid(message_hash: Field)
        return get_owner_address().is_valid(message_hash)

    internal private get_owner_address()
        let address_preimage = pxe.get_address_preimage(this)
        assert hash(address_preimage) == this
        return address_preimage.deployer_address
```

Given the contract does not require initialization since it has no constructor, it can be used by its owner without being actually deployed, which reduces the setup cost.

## Discarded Approaches

An alternative approach was to introduce a new type of call, a diversified call, that would allow the caller to impersonate any address they can derive from their own, for an enshrined derivation mechanism. Account contracts could use this opcode, as opposed to a regular call, to issue calls on behalf on their diversified and stealth addresses. However, this approach failed to account for calls made back to the account contracts, in particular authwit checks. It also required protocol changes, introducing a new type of call which could be difficult to reason about, and increased attack surface. The only benefit over the approach chosen is that it would require one less extra function call to hop from the user's main account contract to the diversified or stealth one.

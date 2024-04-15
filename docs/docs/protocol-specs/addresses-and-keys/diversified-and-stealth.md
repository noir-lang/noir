---
title: Diversified and Stealth Accounts
---

The [keys specification](./keys.md) describes derivation mechanisms for diversified and stealth public keys. However, the protocol requires users to interact with addresses.

## Computing Addresses

To support diversified and stealth accounts, a user may compute the deterministic address for a given account contract that is deployed using a diversified or stealth public key, so a sender can interact with the resulting so-called "diversified" or "stealth" address even before the account contract is deployed.

When the user wants to access the notes that were sent to the diversified or stealth address, they can deploy the contract at their address, and control it privately from their main account.

<!--
> they can deploy the contract at their address, and control it privately from their main account.
Why would they need to deploy their main account in order to learn about new notes? Wouldn't it instead be possible to configure the PXE to trial-decrypt notes for the main account _even without deploying the account contract_?
-->

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

<!-- TODO: The above requires that we implement "using a contract without deploying it if it has no constructor", or "constructor abstraction", both of which are a bit controversial. -->
<!-- I think we're now happy with these previously-controversial things, right? So we can probably get rid of this `TODO`?-->

<!-- In addition to considering the flow of diversifying a user's account contract address, please could we also consider a flow where an app contract wishes to diversify _its_ address when making a call to the public world? The app contract would need to do some clever internal bookkeeping to track all its diversified addresses, and which of the apps users are making use of which diversified addresses, but it would be a worthwhile exploration. I suppose the app contract could simply deploy a new diversified contract whenever it wants to make a public call, and route the call through that diversified account contract. -->

## Discarded Approaches

An alternative approach was to introduce a new type of call, a diversified call, that would allow the caller to impersonate any address they can derive from their own, for an enshrined derivation mechanism. Account contracts could use this opcode, as opposed to a regular call, to issue calls on behalf on their diversified and stealth addresses. However, this approach failed to account for calls made back to the account contracts, in particular authwit checks. It also required protocol changes, introducing a new type of call which could be difficult to reason about, and increased attack surface. The only benefit over the approach chosen is that it would require one less extra function call to hop from the user's main account contract to the diversified or stealth one.

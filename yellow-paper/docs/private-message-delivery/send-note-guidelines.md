# Guidelines

Application contracts are in control of creating, encrypting, tagging, and broadcasting private notes to users. As such, each application is free to follow whatever scheme it prefers, choosing to override user preferences or use custom encryption and note tagging mechanisms. However, this may hinder composability, or not be compatible with existing wallet software.

In order to satisfy the requirements established for private message delivery, we suggest the following guidelines when building applications, which leverage the canonical [registry](../pre-compiled-contracts/registry.md) contract.

## Provably Sending a Note

To provably encrypt, tag, and send a note to a recipient, applications should first check the registry. This ensures that the latest preferences for the recipient are honored, in case they rotated their keys or updated their precompile preference. The registry should be queried via a direct storage read and not a function call, in order to save an additional recursion which incurs in extra proving time.

If the recipient is not in the registry, then the app should allow the sender to provide the recipient's public key from the recipient's address preimage. This allows users who have never interacted with the chain to receive encrypted notes, though it requires a collaborative sender.

If the user is not in the registry and the sender cannot provide the address preimage, then the application must prove that the user was not in the registry, or a malicious sender could simply not submit a correct merkle membership proof for the read and grief the recipient. In this scenario, it is strongly recommended that the application skips the note for the recipient as opposed to failing. This prevents an unregistered address from accidentally or maliciously bricking an application, if there is a note delivery to them in a critical code path in the application.

Execution of the precompile that implements the recipient's choice for encryption and tagging should be done using a batched delegated call, to amortize the cost of sending multiple notes using the same method, and to ensure the notes are broadcasted from the application contract's address.

### Pseudocode

The following pseudocode covers how to provably send a note to a recipient, given an `encryption_type` <!-- I think this should be `private_message_type`. Selecting the kind of key is separate from selecting an encryption scheme --> (incoming, outgoing, or internal incoming). Should the registry support [multiple entries for a given recipient](../pre-compiled-contracts/registry.md#multiple-recipients-per-address), this method must execute a batched call per each entry recovered from the registry.

```
fn provably_send_note(recipient, note, encryption_type)

    let block_number = context.latest_block_number
    let public_state_root = context.roots[block_number].public_state
    let storage_slot = calculate_slot(registry_address, registry_base_slot, recipient)

    let public_keys, precompile_address
    if storage_slot in public_state_root
        context.update_tx_max_valid_block_number(block_number + N)
        public_keys, precompile_address = indexed_merkle_read(public_state_root, storage_slot)
    else if recipient in pxe_oracle
        address_preimage = pxe_oracle.get_preimage(recipient)
        assert hash(address_preimage) == recipient
        public_keys, precompile_address = address_preimage
    else
        registry_address.assert_non_membership(recipient)
        return

    batch_private_delegate_call(precompile_address.encrypt_and_broadcast, { public_keys, encryption_type, recipient, note })
```

## Unconstrained Message Delivery

Applications may choose not to constrain proper message delivery, based on their requirements. In this case, the guidelines are the same as above, but without constraining correct execution, and without the need to assert non-membership when the recipient is not in the registry. Apps can achieve this by issuing a synchronous [unconstrained call](../calls//unconstrained-calls.md) to the encryption precompile `encrypt_and_tag` function, and emitting the resulting encrypted note.

This flexibility is useful in scenarios where the sender can be trusted to make its best effort so the recipient receives their private messages, since it reduces total proving time. An example is a standalone direct value transfer, where the sender wants the recipient to access the funds sent to them.

## Delivering Messages for Self

Applications may encrypt, tag, and broadcast messages for the same user who's initiating a transaction, using the outgoing or the incoming internal encryption key. This allows a user to have an on-chain backup of their private transaction history, which they can use to recover state in case they lose their private database. In this scenario, unconstrained message delivery is recommended, since the sender is incentivized to correctly encrypt message for themselves.

Applications may also choose to query the user wallet software via an oracle call, so the wallet can decide whether to broadcast the note to self on chain based on user preferences. This allows users to save on gas costs by avoiding unnecessary note broadcasts if they rely on other backup strategies.

Last, applications with strong compliance and auditability requirements may choose to enforce provable encryption, tagging, and delivery to the sender user. This ensures that all user activity within the application is stored on-chain, so the user can later provably disclose their activity or repudiate actions they did not take.

## Delivering Messages to Multiple Recipients via Shared Secrets

As an alternative to registering [multiple recipients for a given address](../pre-compiled-contracts/registry.md#multiple-recipients-per-address), multisig participants may deploy a contract using a shared secret derived among them. This makes it cheaper to broadcast messages to the group, since every note does not need to be individually encrypted for each of them. However, it forces all recipients in the group to use the same encryption and tagging method, and adds an extra address they need to monitor for note discovery.

## Discussions

See [_Addresses, keys, and sending notes (Dec 2023 edition)_](https://forum.aztec.network/t/addresses-keys-and-sending-notes-dec-2023-edition/2633) and [_Broadcasting notes in token contracts_](https://forum.aztec.network/t/broadcasting-notes-in-token-contracts/2658) for relevant discussions on this topic.

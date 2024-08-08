---
title: Storage slots
tags: [contracts, storage]
---

From the description of storage slots [in the Concepts](../../../../../aztec/concepts/storage/index.md#private-state-slots---slots-arent-real) you will get an idea around the logic of storage slots. In this section we will go into more detail and walk through an entire example of how storage slots are computed for private state to improve our storage slot intuition. Recall, that storage slots in the private domain is just a logical construct, and are not "actually" used for lookups, but rather just as a value to constrain against.

For the case of the example, we will look at what is inserted into the note hashes tree when adding a note in the Token contract. Specifically, we are looking at the last part of the `transfer` function:

#include_code increase_private_balance noir-projects/noir-contracts/contracts/token_contract/src/main.nr rust

This function is creating a new note and inserting it into the balance set of the recipient `to`. Recall that to ensure privacy, only the note hash is really inserted into the note hashes tree. To share the contents of the note with `to` the contract can emit an encrypted log (which this one does), or it can require an out-of-band data transfer sharing the information. Below, we will walk through the steps of how the note hash is computed and inserted into the tree. For this, we don't care about the encrypted log, so we are going to ignore that part of the function call for now.

Outlining it in more detail below as a sequence diagram, we can see how the calls make their way down the stack.
In the end a siloed note hash is computed in the kernel.

:::info
Some of the syntax below is a little butchered to make it easier to follow variables without the full code.
:::

```mermaid
sequenceDiagram
    alt Call
    Token->>BalanceMap: Map::new(map_slot);
    Token->>Token: to_bal = storage.balances.at(to)
    Token->>BalanceMap: BalanceMap.at(to)
    BalanceMap->>BalanceMap: derived_slot = H(map_slot, to)
    BalanceMap->>BalanceSet: BalanceSet::new(to, derived_slot)
    Token->>BalanceSet: to_bal.add(amount)
    BalanceSet->>BalanceSet: note = TokenNote::new(amount, to)
    BalanceSet->>Set: insert(note)
    Set->>LifeCycle: create_note(derived_slot, note)
    LifeCycle->>LifeCycle: note.header = NoteHeader { contract_address, <br> storage_slot: derived_slot, nonce: 0, note_hash_counter }
    Utils->>TokenNote: note_hiding_point = note.compute_note_hiding_point()
    TokenNote->>Utils: note_hash = note_hiding_point.x
    LifeCycle->>Context: push_note_hash(note_hash)
    end
    Context->>Kernel: unique_note_hash = H(nonce, note_hash)
    Context->>Kernel: siloed_note_hash = H(contract_address, unique_note_hash)
```

Notice the `siloed_note_hash` at the very end. It's a hash that will be inserted into the note hashes tree. To clarify what this really is, we "unroll" the values to their simplest components. This gives us a better idea around what is actually inserted into the tree.

```rust
siloed_note_hash = H(contract_address, unique_note_hash)
siloed_note_hash = H(contract_address, H(nonce, note_hash))
siloed_note_hash = H(contract_address, H(H(tx_hash, note_index_in_tx), note_hash))
siloed_note_hash = H(contract_address, H(H(tx_hash, note_index_in_tx), MSM([G_amt, G_to, G_rand, G_slot], [amount, to, randomness, derived_slot]).x))
```

MSM is a multi scalar multiplication on a grumpkin curve and G_* values are generators.

And `to` is the actor who receives the note, `amount` of the note and `randomness` is the randomness used to make the note hiding. Without the `randomness` the note could just as well be plaintext (computational cost of a preimage attack would be trivial in such a case).

:::info
Beware that this hash computation is what the aztec.nr library is doing, and not strictly required by the network (only the kernel computation is).
:::

With this note structure, the contract can require that only notes sitting at specific storage slots can be used by specific operations, e.g., if transferring funds from `from` to `to`, the notes to destroy should be linked to `H(map_slot, from)` and the new notes (except the change-note) should be linked to `H(map_slot, to)`.

That way, we can have logical storage slots, without them really existing. This means that knowing the storage slot for a note is not enough to actually figure out what is in there (whereas it would be for looking up public state).

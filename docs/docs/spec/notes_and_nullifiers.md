# Notes and Nullifiers

## Global Constants

See [constants.hpp](../../barretenberg/src/aztec/rollup/proofs/notes/constants.hpp) and [constants.hpp](../../barretenberg/src/aztec/rollup/constants.hpp) for constants.

## Pedersen background

A note on pedersen hashing.

- `pedersen::commit` returns a point.
- `pedersen::compress` returns the x-coordinate of `pedersen::commit`.

A different generator is used for each type of note and nullifier (including different generators for partial vs complete commitments). See the hackmd https://hackmd.io/gRsmqUGkSDOCI9O22qWXBA?view for a detailed description of pedersen hashing using turbo plonk.

Note: `pedersen::compress` is collision resistant (see the large comment above the `hash_single` function in the codebase, see the hackmd https://hackmd.io/urZOnB1gQimMqsMdf7ZBvw for a formal proof), so this can be used in place of `pedersen::commit` for note commitments & nullifiers.

## Notes and Commitments

### Account note

An **Account Note** associates a spending key with an account. It consists of the following field elements. See the dedicated [account_circuit.md](./account_circuit.md) for more details.

- `alias_hash`: the 224 bit `alias_hash`
- `account_public_key.x`: the x-coordinate of the account public key
- `spending_public_key.x`: the x-coordinate of the spending key that is been assigned to this account via this note.

An account note commitment is:

- `pedersen::compress(alias_hash, account_public_key.x, signing_pub_key.x)`
  - Pedersen GeneratorIndex: `ACCOUNT_NOTE_COMMITMENT`

### Value note

Consists of the following:

- `secret`: a random value to hide the contents of the
  commitment.
- `owner.x` and `owner.y`: the public key of the owner of the value note. This is a Grumpkin point.
- `account_required`: Is the note linked to an existing account or can the note be spent without an account, by directly signing with the owner key
- `creator_pubkey`: Optional. Allows the sender of a value note to inform the recipient who the note came from.
- `value`: the value contained in this note.
- `asset_id`: unique identifier for the 'currency' of this note. The RollupProcessor.sol maps asset_id's with either ETH or the address of some ERC-20 contract.
- `input_nullifier`: In order to create a value note, another value note must be nullified (except when depositing, where a 'gibberish' nullifier is generated). We include the `input_nullifier` here to ensure the commitment is unique (which, in turn, will ensure this note's nullifier will be unique).

**partial commitment**

- `pedersen::compress(secret, owner.x, owner.y, account_required, creator_pubkey)`
  - Pedersen GeneratorIndex: `VALUE_NOTE_PARTIAL_COMMITMENT`
  - `creator_pubkey` can be zero.

> _Note:_ The `secret` is to construct a hiding Pedersen commitment to hide the note details.

**complete commitment**

- `pedersen::compress(value_note_partial_commitment, value, asset_id, input_nullifier)`
  - Pedersen GeneratorIndex: `VALUE_NOTE_COMMITMENT`
  - `value` and `asset_id` can be zero

In other words:

$$
\begin{align}
&Comm(\text{ValueNote}) = \big( [(\text{note.secret} \cdot g_0 + \text{note.owner.x} \cdot g_1 + \text{note.owner.y} \cdot g_2 + \text{note.account-required} \cdot g_3 \\
&+ \text{note.creator-pubkey} \cdot g_4).x] \cdot h_0 + \text{note.value} \cdot h_1 + \text{note.asset-id} \cdot h_2 + \text{note.input-nullifier} \cdot h_3 \big) .x
\end{align}
$$

(The generator indexing is just for illustration. Consult the code.)

### Claim note

Claim notes are created to document the amount a user deposited in the first stage of a defi interaction. Whatever the output token values of the defi interaction, the data in the claim note will allow the correct share to be apportioned to the user. See the [claim circuit doc](./claim_circuit.md) for more details.

Consists of the following:

- `deposit_value`: The value that the user deposited in the first stage of their defi interaction.
- `bridge_call_data`: Contains an encoding of the bridge being interacted with.
- `value_note_partial_commitment`: See the above 'value note' section.
- `input_nullifier`: In order to create a claim note, a value note must be nullified as part of the 'defi deposit' join-split transaction. We include that `input_nullifier` here to ensure the claim commitment is unique (which, in turn, will ensure this note's nullifier will be unique).
- `defi_interaction_nonce`: A unique identifier for a particular defi interaction that took place. This is assigned by the RollupProcessor.sol contract, and emitted as an event.
- `fee`: The fee to be paid to the rollup processor, specified as part of the defi deposit join-split tx. Half gets paid to process the defi deposit tx, and half to process the later claim tx.

**partial commitment**

- `pedersen::compress(deposit_value, bridge_call_data, value_note_partial_commitment, input_nullifier)`
  - Pedersen GeneratorIndex: `CLAIM_NOTE_PARTIAL_COMMITMENT`
  - `bridge_call_data` can be zero.

**complete commitment**

- `pedersen::compress(claim_note_partial_commitment, defi_interaction_nonce, fee)`
  - Pedersen GeneratorIndex: `CLAIM_NOTE_COMMITMENT`
  - `fee` and `defi_interaction_nonce` could be zero.

### Defi Interaction note

A defi interaction note records the details of a particular defi interaction. It records the total deposited by all users and the totals output by the defi bridge. These totals get apportioned to each user based on the contents of each user's claim note.

Consists of the following:

- `bridge_call_data`: Contains an encoding of the bridge that was interacted with.
- `total_input_value`: The total deposited to the bridge by all users who took part in this defi interaction.
- `total_output_value_a`: The sum returned by the defi bridge denominated in 'token A'. (The details of 'token A' are contained in the `bridge_call_data`).
- `total_output_value_b`: The sum returned by the defi bridge denominated in 'token B'. (The details of 'token B' are contained in the `bridge_call_data`).
- `interaction_nonce`: (a.k.a. defi interaction nonce) A unique identifier for a particular defi interaction that took place. This is assigned by the RollupProcessor.sol contract, and emitted as an event.
- `interaction_result`: true/false - was the L1 transaction a success?

**commitment**

- `pedersen::compress(bridge_call_data, total_input_value, total_output_value_a, total_output_value_b, interaction_nonce, interaction_result)`
  - Pedersen GeneratorIndex: `DEFI_INTERACTION_NOTE_COMMITMENT`

# Note encryption and decryption

Details on this are found [here](https://hackmd.io/@aztec-network/BJKHah_4d)

# Nullifiers

## Value note nullifier

**Objectives** of this nullifier:

- Only the owner of a note may be able to produce the note's nullifier.
- No collisions. Each nullifier can only be produced for one value note commitment. Duplicate nullifiers must not be derivable from different note commitments.
- No collisions between nullifiers of other notes (i.e. claim notes or defi interaction notes).
- No double-spending. Each commitment must have one, and only one, nullifier.
- The nullifier must only be accepted and added to the nullifier tree if it is the output of a join-split circuit which 'spends' the corresponding note.

**Calculation**
We set out the computation steps below, with suggestions for changes:

- `hashed_pk = account_private_key * G` (where `G` is a generator unique to this operation).
  - This `hashed_pk` is useful to demonstrate to a 3rd party that you've nullified something without having to provide your secret key.
- `compressed_inputs = pedersen::compress(value_note_commitment, hashed_pk.x, hashed_pk.y, is_real_note)`
  - This compression step reduces the cost (constrain-wise) of the blake2s hash which is done next.
- `nullifier = blake2s(compressed_inputs);`
  - blake2s is needed, because a pedersen commitment alone can leak data (see comment in the code for more details on this).

Pedersen GeneratorIndex:

- `JOIN_SPLIT_NULLIFIER_ACCOUNT_PRIVATE_KEY` for the hashed_pk
- `JOIN_SPLIT_NULLIFIER` to compress the inputs

## Claim note nullifier

**Objectives** of this nullifier:

- Anyone (notably the rollup provider) may be able to produce this nullifier.
- No collisions. Each nullifier can only be produced for one claim note commitment. Duplicate nullifiers must not be derivable from different claim note commitments.
- No collisions between nullifiers of other notes (i.e. value notes or defi interaction notes).
- This nullifier must only be added to the nullifier tree if it is the output of a claim circuit which 'spends' the corresponding claim note.
- No double-spending. Each claim note commitment must have one, and only one, nullifier.

**Calculation**

- `nullifier = pedersen::compress(claim_note_commitment);`
  - Note: it is ok that observers can see which claim note is being nullified, since values in a defi interaction are public (only owners are private). Furthermore, the rollup priovider needs to be able to generate the claim proof and doesn't have access to any user secrets - so this nullifier allows this use case.
  - Pedersen GeneratorIndex:`CLAIM_NOTE_NULLIFIER`

## Defi Interaction nullifier

**Objectives** of this nullifier:

- This is not a 'conventional' nullifier, in the sense that it doesn't prevent others from 'referring' to the defi interaction note. It's really only needed so that _something_ unique may be fed into the `output_note_2` output of the claim circuit.
- Anyone (notably the rollup provider) may be able to produce a valid nullifier on behalf of any user who partook in the corresponding defi interaction.
- No collisions between nullifiers of other notes (i.e. value notes or claim notes).
- This nullifier must only be added to the nullifier tree if it is the output of a claim circuit which 'refers' the corresponding defi interaction note note and 'spends' a claim note which was created during that defi interaction.

**Calculation:**

- `nullifier = pedersen::compress(defi_interaction_note_commitment, claim_note_commitment);`
- Pedersen GeneratorIndex:`DEFI_INTERACTION_NULLIFIER`

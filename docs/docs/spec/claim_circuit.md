# Claim circuit

This circuit enables converting a claim note into two value notes, according to the defi interaction result.

## Diagrams

- [The entire defi process](https://drive.google.com/file/d/1rbBywUqM78RkCNcI0jmie6-N79PY0lqv/view?usp=sharing)

## Before the claim circuit

A defi interaction is a multi-step process which _ends_ with the claim circuit being verified on-chain. There are more complete explanations of the whole process for many individual dApps on hackmd under the 'Aztec Connect' tag.
Here's a very brief summary of the defi interaction process:

- A user wishes to interact with an Ethereum L1 dApp privately. They can use Aztec Connect to hide their identity from observers. The values they send will still be visible (but not traceable back to them). Let's use Uniswap as an example.
- The user wishes to convert 1 ETH to DAI tokens.
- They submit a 'defi deposit' of 1 ETH.
  - A join-split proof is generated in 'defi deposit' mode, which spends the 1 ETH and creates a partial claim note (see the diagrams or the join-split markdown file).
- The rollup provider bundles (sums) the user's request to deposit 1 ETH into uniswap with the requests of any other users who wanted to deposit ETH to uniswap. User costs are amortised this way.
- The rollup provider is able to assign a `bridge_call_data` to each 'bundle', and with knowledge of this `bridge_call_data` and the `total_input_value` being deposited, the rollup provider can 'complete' each user's partial claim note. I.e. the rollup provider creates a completed 'claim note' for each user. This claim note can be used later in the process to withdraw DAI via the claim circuit.
- This bundled (summed) deposit of X ETH is sent to a 'Defi Bridge Contract' - a contract specifically designed to perform swaps between Aztec Connect users and Uniswap.
- The Defi Bridge Contract sends the `total_input_value = X` ETH to Uniswap (along with some parameters which we won't go into here), and receives back Y DAI.
- The rollup contract emits an event which says "X ETH was swapped for Y DAI, and here is a 'defi interaction nonce' which represents this interaction".
- The rollup provider listens for this event to learn whether their interaction was successful, and to learn the new data: `total_output_value_a = Y`, `defi_interaction_nonce = defi_interaction_nonce`.
- The rollup provider is now in a position to submit a _claim_ (using the claim circuit) _on behalf of_ each user who partook in the defi interaction.
  - Take note of this. Submission of a claim needn't be done by the user; no private key is required. The rollup provider is incentivised to generate a claim proof by being offered a fee via the earlier join-split proof.

Now we can get into the details of the claim circuit.

## Claim circuit

At a high level, the claim circuit does the following:

- Spends a user's claim note;
- Refers to a particular defi interaction note (which contains uniquely-identifying details of a particular defi interaction);
- Outputs up-to two output 'value notes' whose values are proportional to the amount originally defi-deposited by this user.
  - `output_note_1.value = ( user_input_amount / total_input_amount ) * total_output_amount_a`
  - `output_note_2.value = ( user_input_amount / total_input_amount ) * total_output_amount_b`

(In our earlier example, `ouput_note_1.value = ( 1 / X ) * Y` DAI).

### Details

#### Inputs

Recall that all inner circuits must have the **same number of public inputs** as they will be used homogenously by the rollup circuit. Hence, some of the inputs to a claim circuit are unused and so set to 0.

##### Public Inputs

- `proof_id = ProofIds::DEFI_CLAIM`
- `output_note_commitment_1`
- `output_note_commitment_2`
- `nullifier1`
- `nullifier2`
- `public_value = 0`
- `public_owner = 0`
- `asset_id = 0`
- `data_root`
- `claim_note.fee`
- `claim_note_data.bridge_call_data_local.input_asset_id`
- `claim_note.bridge_call_data`
- `defi_deposit_value = 0`
- `defi_root`
- `backward_link = 0`
- `allow_chain = 0`

##### Private Inputs

- `claim_note_index`
- `claim_note_path`
- ```
  claim_note: {
      deposit_value,
      bridge_call_data, // actually a public input
      defi_interaction_nonce,
      fee,             // actually a public input
      value_note_partial_commitment,
      input_nullifier,
  }
  ```
- `defi_interaction_note_path`
- ```
  defi_interaction_note: {
      bridge_call_data,
      defi_interaction_nonce,
      total_input_value,
      total_output_value_a,
      total_output_value_b,
      defi_interaction_result,
      commitment,
  }
  ```
- `output_value_a`
- `output_value_b`

#### Circuit Logic (Pseudocode)

_Note: for Pedersen commitments, different generators are used for different types of commitment._

Computed vars:

- Extract data from the `claim_note.bridge_call_data`:

  - ```
    bridge_call_data_local = {
        bridge_address_id, // represents a defi bridge contract address
        input_asset_id_a,
        input_asset_id_b,     // if virtual, is the defi_interaction nonce from when a loan/LP position was opened
        output_asset_id_a,
        output_asset_id_b,

                           // during some earlier defi interaction by the user
        bit_config,
        aux_data,
    }
    ```

- The same data is also currently extracted from the `defi_interaction_note.bridge_call_data`. This is redundant, but we'll only need to remove these extra constraints if we ever approach the next power of 2.
- Extract config data from `bit_config`:
  - ```
    bit_config = {
      second_input_in_use,
      second_output_in_use,
    }
    ```
- ```
  claim_note.commitment = pedersen(
      pedersen(deposit_value, bridge_call_data, value_note_partial_commitment, input_nullifier),
      defi_interaction_nonce,
      fee,
  )
  ```
- ```
  defi_interaction_note.commitment = pedersen(
      bridge_call_data,
      total_input_value,
      total_output_value_a,
      total_output_value_b,
      defi_interaction_nonce,
      defi_interaction_result,
  )
  ```
- `output_value_1 = defi_interaction_result ? output_value_a : claim_note.deposit_value` (give a refund if the interaction failed).
- `output_asset_id_1 = defi_interaction_result ? output_asset_id_a : input_asset_id`
- `output_value_2 = second_output_virtual ? output_value_a : output_value_b`
  - If the second output is virtual, its value must equal that of the first output.
- `output_asset_id_2 = second_output_virtual ? concat(1, defi_interaction_nonce) : output_asset_id_b`
  - If virtual, attach a record of this 'opening defi interaction nonce' to the note, via the asset_id field.

Checks:

- Many values are range-checked. See [constants.hpp](../../barretenberg/src/aztec/rollup/proofs/notes/constants.hpp) and [constants.hpp](../../barretenberg/src/aztec/rollup/constants.hpp) for the variables whose bit-lengths are constrained.
- Check `bit_config` vars:
- Extract `second_input_in_use` and `second_output_in_use` from `claim_note_data.bridge_call_data_local.config`
// The below six constraints are exercised in bridge_call_data.hpp, see comments there for elaboration
- `!(input_asset_id_b.is_zero) must_imply config.second_input_in_use`
- `!(input_asset_id_b.is_zero) must_imply config.second_output_in_use`
- `config.second_input_in_use must_imply input_asset_id_a != input_asset_id_b`
- `config.second_output_in_use && both_outputs_real must_imply output_asset_id_a != output_asset_id_b`
- `first_output_virtual must_imply output_asset_id_a == virtual_asset_id_placeholder`
- `second_output_virtual must_imply output_asset_id_b == virtual_asset_id_placeholder`
- `require(claim_note.deposit_value != 0)`
- `require(deposit_value <= total_input_value)`
- `require(output_value_a <= total_output_value_a)`
- `require(output_value_b <= total_output_value_b)`
- `require(claim_note.bridge_call_data == defi_interaction_note.bridge_call_data)`
- `require(claim_note.defi_interaction_nonce == defi_interaction_note.defi_interaction_nonce)`
- Check claim note exists in the data tree using `data_root, claim_note_path, claim_note_index, claim_note.commitment`.
- Check defi interaction note exists in the data tree using `defi_root, defi_interaction_note_path, defi_interaction_nonce`.
  - Note: the leaf index of a defi interaction note is its `defi_interaction_nonce`. The `defi_interaction_nonce` is derived in the rollup circuit at the time the defi deposit (join split) is processed.

Ratio Checks (very complex code):

- Ensure `output_value_a == (deposit / total_input_value) * total_output_value_a`, unless `output_value_a == 0 && total_output_value_a == 0` (i.e. unless no value was returned by the bridge contract for output_a).
- Ensure `output_value_b == (deposit / total_input_value) * total_output_value_b`, unless `output_value_b == 0 && total_output_value_b == 0` (i.e. unless no value was returned by the bridge contract for output_b).
- (Also prevent zero denominators `total_input_value`, `total_output_value_a`, and `total_output_value_b`).

Computed public inputs:

- `nullifier_1 = pedersen(claim_note.commitment)`
- `nullifier_2 = pedersen(defi_interaction_note.commitment, claim_note.commitment)`
- `output_note_commitment1 = pedersen(value_note_partial_commitment, output_value_1, output_asset_id_1, nullifier_1)`
- `output_note_commitment2 = (second_output_virtual ^ second_output_real) ? pedersen(value_note_partial_commitment, output_value_2, output_asset_id_2, nullifier_2) : 0`

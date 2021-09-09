#include "root_rollup_proof_data.hpp"
#include "../inner_proof_data.hpp"
#include "../../constants.hpp"
#include <crypto/sha256/sha256.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

root_rollup_proof_data::root_rollup_proof_data(std::vector<uint8_t> const& proof_data)
{
    using serialize::read;
    auto ptr = proof_data.data();
    ptr += 60;
    read(ptr, rollup_size);

    auto num_fields =
        RootRollupProofFields::INNER_PROOFS_DATA + (rollup_size * rollup::PropagatedInnerProofFields::NUM_FIELDS);

    std::vector<fr> fields(num_fields);
    ptr = proof_data.data();
    for (size_t i = 0; i < num_fields; ++i) {
        read(ptr, fields[i]);
    }

    populate_from_fields(fields);
}

root_rollup_proof_data::root_rollup_proof_data(std::vector<field_ct> const& stdlib_fields)
{
    std::vector<fr> fields;
    for (const auto& stdlib_field : stdlib_fields) {
        fields.push_back(stdlib_field.get_value());
    }
    populate_from_fields(fields);
}

root_rollup_proof_data::root_rollup_proof_data(std::vector<fr> const& public_inputs)
{
    populate_from_fields(public_inputs);
}

/**
 * Convert a root rollup's proof data into a uint8 vector.
 * In `rollup_cli` this is prepended to the start of the actual proof output
 * This is because the proof output no longer contains the proof data as public inputs!
 * (we've hashed them all down to a single value to cut down on the number of public inputs)
 *
 * In order for the verifier smart contract to reconstruct the proof data hash, we must pass the
 * raw proof data into the smart contract as a Solidity bytes object,
 * which is analogous to a c++ std::vector<uint8_t> object.
 *
 * This method currently performs no acutal encoding or compression, but when we start that work,
 * this is where we should create the encoded proof data
 **/
std::vector<uint8_t> root_rollup_proof_data::encode_proof_data() const
{
    std::vector<uint8_t> buffer;

    const auto add_to_buffer = [&buffer](const fr& input) {
        auto input_bytes = input.to_buffer();

        std::copy(input_bytes.begin(), input_bytes.end(), std::back_inserter(buffer));
    };

    add_to_buffer(rollup_id);

    add_to_buffer(rollup_size);
    add_to_buffer(data_start_index);
    add_to_buffer(old_data_root);
    add_to_buffer(new_data_root);
    add_to_buffer(old_null_root);
    add_to_buffer(new_null_root);
    add_to_buffer(old_data_roots_root);
    add_to_buffer(new_data_roots_root);
    add_to_buffer(old_defi_root);
    add_to_buffer(new_defi_root);
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        add_to_buffer(bridge_ids[i]);
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        add_to_buffer(deposit_sums[i]);
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        add_to_buffer(asset_ids[i]);
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        add_to_buffer(total_tx_fees[i]);
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        // these are Grumpkin points so can call add_to_buffer directly on x/y coords (because they are fr elements, not
        // fq elements)
        add_to_buffer(defi_interaction_notes[i]);
    }
    add_to_buffer(previous_defi_interaction_hash);
    add_to_buffer(num_inner_txs);
    // This code funnels each inner proof value into a uint8 vector.
    for (const auto& inner_proof : inner_proofs) {
        add_to_buffer(inner_proof.proof_id);
        add_to_buffer(inner_proof.public_input);
        add_to_buffer(inner_proof.public_output);
        add_to_buffer(inner_proof.asset_id);
        add_to_buffer(inner_proof.note_commitment1);
        add_to_buffer(inner_proof.note_commitment2);
        add_to_buffer(inner_proof.nullifier1);
        add_to_buffer(inner_proof.nullifier2);
        add_to_buffer(inner_proof.input_owner);
        add_to_buffer(inner_proof.output_owner);
    }
    return buffer;
}

/**
 * Computes the sha256 hash of the broadcasted inputs, using the output from `encode_proof_data`.
 * The hash input data structure needs to mimic the data structure used directly in the root rollup circuit
 **/
std::vector<uint8_t> root_rollup_proof_data::compute_hash_from_encoded_inputs() const
{
    using serialize::read;
    std::vector<uint8_t> encoded_inputs = encode_proof_data();
    std::vector<uint8_t> hash_inputs;
    const auto add_to_buffer = [](std::vector<uint8_t>& buffer, const uint8_t*& input) {
        for (size_t i = 0; i < 32; ++i) {
            buffer.push_back(input[i]);
        }
        input += 32;
    };

    const uint8_t* ptr = &encoded_inputs[0];
    add_to_buffer(hash_inputs, ptr); // rollup_id
    // get the rollup size and then reset the ptr (`read` will advance the pointer, we don't want that here)
    uint32_t rollup_size_temp = 0;
    ptr += 28;
    read(ptr, rollup_size_temp);
    ptr -= 32;

    add_to_buffer(hash_inputs, ptr); // rollup_size_pow2
    add_to_buffer(hash_inputs, ptr); // data_start_index
    add_to_buffer(hash_inputs, ptr); // old_data_root
    add_to_buffer(hash_inputs, ptr); // new_data_root
    add_to_buffer(hash_inputs, ptr); // old_null_root
    add_to_buffer(hash_inputs, ptr); // new_null_root
    add_to_buffer(hash_inputs, ptr); // old_root_root
    add_to_buffer(hash_inputs, ptr); // new_root_root
    add_to_buffer(hash_inputs, ptr); // old_defi_root
    add_to_buffer(hash_inputs, ptr); // new_defi_root

    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        add_to_buffer(hash_inputs, ptr); // bridge_ids[i]
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        add_to_buffer(hash_inputs, ptr); // defi_deposit_sums[i]
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        add_to_buffer(hash_inputs, ptr); // asset_ids[i]
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        add_to_buffer(hash_inputs, ptr); // total_tx_fees[i]
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        add_to_buffer(hash_inputs, ptr); // note_commitment
    }
    add_to_buffer(hash_inputs, ptr); // previous_defi_interaction_hash

    // read the number of inner input transactions
    uint32_t num_inner_rollups = 0;
    ptr += 28;
    read(ptr, num_inner_rollups);
    ptr -= 32;
    add_to_buffer(hash_inputs, ptr); // num_inner_txs
    uint32_t num_txs_per_rollup = rollup_size_temp / num_inner_rollups;
    for (size_t i = 0; i < num_inner_rollups; ++i) {
        std::vector<uint8_t> inner_inputs;
        for (size_t j = 0; j < num_txs_per_rollup; ++j) {
            for (size_t k = 0; k < rollup::PropagatedInnerProofFields::NUM_FIELDS; ++k) {
                add_to_buffer(inner_inputs, ptr);
            }
        }
        std::array<uint8_t, 32> inner_inputs_hash = sha256::sha256(inner_inputs);
        std::vector<uint8_t> hash_reduced = fr::serialize_from_buffer(&inner_inputs_hash[0]).to_buffer();
        const uint8_t* hash_reduced_ptr = &hash_reduced[0];
        add_to_buffer(hash_inputs, hash_reduced_ptr);
    }

    auto hash_output = sha256::sha256(hash_inputs);
    return fr::serialize_from_buffer(&hash_output[0]).to_buffer();
}

void root_rollup_proof_data::populate_from_fields(std::vector<fr> const& fields)
{
    rollup_id = static_cast<uint32_t>(fields[RootRollupProofFields::ROLLUP_ID]);
    rollup_size = static_cast<uint32_t>(fields[RootRollupProofFields::ROLLUP_SIZE]);
    data_start_index = static_cast<uint32_t>(fields[RootRollupProofFields::DATA_START_INDEX]);
    old_data_root = fields[RootRollupProofFields::OLD_DATA_ROOT];
    new_data_root = fields[RootRollupProofFields::NEW_DATA_ROOT];
    old_null_root = fields[RootRollupProofFields::OLD_NULL_ROOT];
    new_null_root = fields[RootRollupProofFields::NEW_NULL_ROOT];
    old_data_roots_root = fields[RootRollupProofFields::OLD_DATA_ROOTS_ROOT];
    new_data_roots_root = fields[RootRollupProofFields::NEW_DATA_ROOTS_ROOT];
    old_defi_root = fields[RootRollupProofFields::OLD_DEFI_ROOT];
    new_defi_root = fields[RootRollupProofFields::NEW_DEFI_ROOT];
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        bridge_ids[i] = fields[RootRollupProofFields::DEFI_BRIDGE_IDS + i];
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        deposit_sums[i] = fields[RootRollupProofFields::DEFI_BRIDGE_DEPOSITS + i];
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        asset_ids[i] = fields[RootRollupProofFields::ASSET_IDS + i];
    }
    for (size_t i = 0; i < NUM_ASSETS; ++i) {
        total_tx_fees[i] = fields[RootRollupProofFields::TOTAL_TX_FEES + i];
    }

    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        defi_interaction_notes[i] = fields[RootRollupProofFields::DEFI_INTERACTION_NOTES + i];
    }
    previous_defi_interaction_hash = fields[RootRollupProofFields::PREVIOUS_DEFI_INTERACTION_HASH];

    num_inner_txs = static_cast<uint32_t>(fields[RootRollupProofFields::NUM_ROLLUP_TXS]);
    inner_proofs.resize(rollup_size);
    for (size_t i = 0; i < rollup_size; ++i) {
        auto offset = RootRollupProofFields::INNER_PROOFS_DATA + (i * rollup::PropagatedInnerProofFields::NUM_FIELDS);
        inner_proofs[i].proof_id = fields[offset + InnerProofFields::PROOF_ID];
        inner_proofs[i].public_input = fields[offset + InnerProofFields::PUBLIC_INPUT];
        inner_proofs[i].public_output = fields[offset + InnerProofFields::PUBLIC_OUTPUT];
        inner_proofs[i].asset_id = fields[offset + InnerProofFields::ASSET_ID];
        inner_proofs[i].note_commitment1 = fields[offset + InnerProofFields::NOTE_COMMITMENT1];
        inner_proofs[i].note_commitment2 = fields[offset + InnerProofFields::NOTE_COMMITMENT2];
        inner_proofs[i].nullifier1 = fields[offset + InnerProofFields::NULLIFIER1];
        inner_proofs[i].nullifier2 = fields[offset + InnerProofFields::NULLIFIER2];
        inner_proofs[i].input_owner = fields[offset + InnerProofFields::INPUT_OWNER];
        inner_proofs[i].output_owner = fields[offset + InnerProofFields::OUTPUT_OWNER];
    }
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup

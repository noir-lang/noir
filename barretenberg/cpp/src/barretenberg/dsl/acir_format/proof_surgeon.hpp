#pragma once
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/plonk_honk_shared/types/aggregation_object_type.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>

namespace acir_format {

// Where the public inputs start within a proof (after circuit_size, num_pub_inputs, pub_input_offset)
static constexpr size_t HONK_RECURSION_PUBLIC_INPUT_OFFSET = 3;

class ProofSurgeon {
  public:
    /**
     * @brief Reconstruct a bberg style proof from a acir style proof + public inputs
     * @details Insert the public inputs in the middle the proof fields after 'inner_public_input_offset' because this
     * is how the core barretenberg library processes proofs (with the public inputs starting at the third element and
     * not separate from the rest of the proof)
     *
     * @param proof_in A proof stripped of its public inputs
     * @param public_inputs The public inputs to be reinserted into the proof
     * @return std::vector<uint32_t> The witness indices of the complete proof
     */
    static std::vector<uint32_t> create_indices_for_reconstructed_proof(const std::vector<uint32_t>& proof_in,
                                                                        const std::vector<uint32_t>& public_inputs)
    {
        std::vector<uint32_t> proof;
        proof.reserve(proof_in.size() + public_inputs.size());

        // Construct a the complete proof as the concatenation {"initial data" | public_inputs | proof_in}
        proof.insert(proof.end(), proof_in.begin(), proof_in.begin() + HONK_RECURSION_PUBLIC_INPUT_OFFSET);
        proof.insert(proof.end(), public_inputs.begin(), public_inputs.end());
        proof.insert(proof.end(), proof_in.begin() + HONK_RECURSION_PUBLIC_INPUT_OFFSET, proof_in.end());

        return proof;
    }

    /**
     * @brief Extract then remove a given number of public inputs from a proof
     *
     * @param proof_witnesses Witness values of a bberg style proof containing public inputs
     * @param num_public_inputs The number of public inputs to extract from the proof
     * @return std::vector<bb::fr> The extracted public input witness values
     */
    static std::vector<bb::fr> cut_public_inputs_from_proof(std::vector<bb::fr>& proof_witnesses,
                                                            const size_t num_public_inputs_to_extract)
    {
        // Construct iterators pointing to the start and end of the public inputs within the proof
        auto pub_inputs_begin_itr =
            proof_witnesses.begin() + static_cast<std::ptrdiff_t>(HONK_RECURSION_PUBLIC_INPUT_OFFSET);
        auto pub_inputs_end_itr =
            proof_witnesses.begin() +
            static_cast<std::ptrdiff_t>(HONK_RECURSION_PUBLIC_INPUT_OFFSET + num_public_inputs_to_extract);

        // Construct the isolated public inputs
        std::vector<bb::fr> public_input_witnesses{ pub_inputs_begin_itr, pub_inputs_end_itr };

        // Erase the public inputs from the proof
        proof_witnesses.erase(pub_inputs_begin_itr, pub_inputs_end_itr);

        return public_input_witnesses;
    }

    struct RecursionWitnessData {
        std::vector<uint32_t> key_indices;
        std::vector<uint32_t> proof_indices;
        std::vector<uint32_t> public_inputs_indices;
    };

    /**
     * @brief Populate a witness vector with key, proof, and public inputs; track witness indices for each component
     * @details This method is used to constuct acir-style inputs to a recursion constraint. It is assumed that the
     * provided proof contains all of its public inputs (i.e. the conventional bberg format) which are extracted herein.
     * Each component is appended to the witness which may already contain data. The order in which they are added is
     * arbitrary as long as the corresponding witness indices are correct.
     *
     * @param witness
     * @param proof_witnesses
     * @param key_witnesses
     * @param num_public_inputs
     * @return RecursionWitnessData
     */
    static RecursionWitnessData populate_recursion_witness_data(bb::SlabVector<bb::fr>& witness,
                                                                std::vector<bb::fr>& proof_witnesses,
                                                                const std::vector<bb::fr>& key_witnesses,
                                                                const size_t num_public_inputs)
    {
        // Extract all public inputs except for those corresponding to the aggregation object
        const size_t num_public_inputs_to_extract = num_public_inputs - bb::AGGREGATION_OBJECT_SIZE;
        std::vector<bb::fr> public_input_witnesses =
            cut_public_inputs_from_proof(proof_witnesses, num_public_inputs_to_extract);

        // Helper to append some values to the witness vector and return their corresponding indices
        auto add_to_witness_and_track_indices = [](bb::SlabVector<bb::fr>& witness,
                                                   const std::vector<bb::fr>& input) -> std::vector<uint32_t> {
            std::vector<uint32_t> indices;
            indices.reserve(input.size());
            auto witness_idx = static_cast<uint32_t>(witness.size());
            for (const auto& value : input) {
                witness.push_back(value);
                indices.push_back(witness_idx++);
            }
            return indices;
        };

        // Append key, proof, and public inputs while storing the associated witness indices
        std::vector<uint32_t> key_indices = add_to_witness_and_track_indices(witness, key_witnesses);
        std::vector<uint32_t> proof_indices = add_to_witness_and_track_indices(witness, proof_witnesses);
        std::vector<uint32_t> public_input_indices = add_to_witness_and_track_indices(witness, public_input_witnesses);

        return { key_indices, proof_indices, public_input_indices };
    }
};

} // namespace acir_format

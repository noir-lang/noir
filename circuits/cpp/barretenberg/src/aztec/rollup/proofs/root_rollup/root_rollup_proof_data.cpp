#include "root_rollup_proof_data.hpp"
#include "../inner_proof_data.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace root_rollup {

root_rollup_proof_data::root_rollup_proof_data(std::vector<uint8_t> const& proof_data)
    : rollup::rollup_proof_data(proof_data)
{
    using serialize::read;
    auto ptr = proof_data.data();
    // Skip over the common inner/outer structure. Header + tx public inputs + recursive output.
    ptr += rollup::RollupProofOffsets::INNER_PROOFS_DATA +
           (rollup_size * rollup::PropagatedInnerProofFields::NUM_FIELDS * 32) + (16 * 32);
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        read(ptr, defi_interaction_notes[i]);
    }
    read(ptr, previous_defi_interaction_hash);
}

root_rollup_proof_data::root_rollup_proof_data(std::vector<fr> const& public_inputs)
    : rollup::rollup_proof_data(public_inputs)
{
    auto offset = rollup::RollupProofFields::INNER_PROOFS_DATA +
                  (rollup_size * rollup::PropagatedInnerProofFields::NUM_FIELDS) + (16);
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        defi_interaction_notes[i] = public_inputs[offset++];
    }
    previous_defi_interaction_hash = public_inputs[offset];
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup

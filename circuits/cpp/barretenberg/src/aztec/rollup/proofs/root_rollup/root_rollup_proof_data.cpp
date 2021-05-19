#include "root_rollup_proof_data.hpp"
#include "../inner_proof_data.hpp"
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace rollup {

root_rollup_proof_data::root_rollup_proof_data(std::vector<uint8_t> const& proof_data)
    : rollup_proof_data(proof_data)
{
    using serialize::read;
    auto ptr = proof_data.data();
    // Skip over the common inner/outer structure. Header + tx public inputs + recursive output.
    ptr += RollupProofOffsets::INNER_PROOFS_DATA + (rollup_size * InnerProofFields::NUM_PUBLISHED * 32) + (16 * 32);
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        read(ptr, bridge_ids[i]);
        read(ptr, deposit_sums[i]);
    }
    for (size_t i = 0; i < NUM_BRIDGE_CALLS_PER_BLOCK; ++i) {
        read(ptr, defi_interaction_notes[i]);
    }
    read(ptr, previous_defi_interaction_hash);
}

} // namespace rollup
} // namespace proofs
} // namespace rollup

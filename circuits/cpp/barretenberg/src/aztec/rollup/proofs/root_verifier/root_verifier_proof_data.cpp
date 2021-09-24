#include "root_verifier_proof_data.hpp"

namespace rollup {
namespace proofs {
namespace root_verifier {

root_verifier_proof_data::root_verifier_proof_data(std::vector<uint8_t> const& proof_data)
{
    size_t num_fields = RootVerifierProofFields::NUM_FIELDS + 16;
    using serialize::read;
    const uint8_t* ptr = proof_data.data();
    std::vector<fr> fields(num_fields);
    for (size_t i = 0; i < num_fields; ++i) {
        read(ptr, fields[i]);
    }

    broadcasted_inputs_hash_reduced = fields[RootVerifierProofFields::BROADCASTED_INPUTS_HASH_REDUCED];
    size_t offset = RootVerifierProofFields::NUM_FIELDS;

    for (auto& coord :
         { &recursion_output[0].x, &recursion_output[0].y, &recursion_output[1].x, &recursion_output[1].y }) {
        uint256_t limb[4];
        for (size_t li = 0; li < 4; ++li) {
            limb[li] = fields[offset++];
        }
        *coord = limb[0] + (uint256_t(1) << 68) * limb[1] + (uint256_t(1) << 136) * limb[2] +
                 (uint256_t(1) << 204) * limb[3];
    }
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup

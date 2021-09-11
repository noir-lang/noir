#include "root_rollup_proof_data.hpp"
#include "../inner_proof_data.hpp"
#include "../../constants.hpp"
#include <crypto/sha256/sha256.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

root_rollup_proof_data::root_rollup_proof_data(std::vector<uint8_t> const& proof_data)
{
    // 1 input hash and 16 recursion elements.
    size_t num_fields = 17;
    std::vector<fr> fields(num_fields);
    auto ptr = proof_data.data();
    for (size_t i = 0; i < num_fields; ++i) {
        read(ptr, fields[i]);
    }

    populate_from_fields(fields);
}

root_rollup_proof_data::root_rollup_proof_data(std::vector<fr> const& fields)
{
    populate_from_fields(fields);
}

void root_rollup_proof_data::populate_from_fields(std::vector<fr> const& fields)
{
    input_hash = fields[0];
    size_t offset = 1;
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

} // namespace root_rollup
} // namespace proofs
} // namespace rollup

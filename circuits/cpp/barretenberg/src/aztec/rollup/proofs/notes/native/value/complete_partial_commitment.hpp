#pragma once
#include <crypto/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace value {

inline auto complete_partial_commitment(grumpkin::fq const& partial_commitment,
                                        uint256_t const& value,
                                        uint32_t asset_id)
{
    return crypto::pedersen::compress_native({ partial_commitment, value, asset_id },
                                             GeneratorIndex::VALUE_NOTE_COMMITMENT);
};

} // namespace value
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup
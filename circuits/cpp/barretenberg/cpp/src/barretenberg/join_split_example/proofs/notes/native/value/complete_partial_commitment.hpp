#pragma once
#include "../../constants.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {
namespace value {

inline auto complete_partial_commitment(grumpkin::fq const& partial_commitment,
                                        uint256_t const& value,
                                        uint32_t asset_id,
                                        grumpkin::fq input_nullifier)
{
    return crypto::pedersen_commitment::compress_native({ partial_commitment, value, asset_id, input_nullifier },
                                                        GeneratorIndex::VALUE_NOTE_COMMITMENT);
};

} // namespace value
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example

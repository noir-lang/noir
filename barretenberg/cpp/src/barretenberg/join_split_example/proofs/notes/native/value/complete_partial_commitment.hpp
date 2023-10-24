#pragma once
#include "../../constants.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"

namespace join_split_example::proofs::notes::native::value {

inline auto complete_partial_commitment(grumpkin::fq const& partial_commitment,
                                        uint256_t const& value,
                                        uint32_t asset_id,
                                        grumpkin::fq input_nullifier)
{
    return crypto::pedersen_hash::hash({ partial_commitment, value, asset_id, input_nullifier },
                                       GeneratorIndex::VALUE_NOTE_COMMITMENT);
};

} // namespace join_split_example::proofs::notes::native::value

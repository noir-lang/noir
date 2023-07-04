#pragma once
#include "../../constants.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {
namespace value {

inline auto create_partial_commitment(barretenberg::fr const& secret,
                                      grumpkin::g1::affine_element const& owner,
                                      bool account_required,
                                      barretenberg::fr const& creator_pubkey)
{
    return crypto::pedersen_commitment::compress_native({ secret, owner.x, owner.y, account_required, creator_pubkey },
                                                        GeneratorIndex::VALUE_NOTE_PARTIAL_COMMITMENT);
}

} // namespace value
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example

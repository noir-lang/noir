#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace value {

inline auto create_partial_commitment(barretenberg::fr const& secret,
                                      grumpkin::g1::affine_element const& owner,
                                      bool account_required,
                                      barretenberg::fr const& creator_pubkey)
{
    return crypto::pedersen::compress_native({ secret, owner.x, owner.y, account_required, creator_pubkey },
                                             GeneratorIndex::VALUE_NOTE_PARTIAL_COMMITMENT);
}

} // namespace value
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup
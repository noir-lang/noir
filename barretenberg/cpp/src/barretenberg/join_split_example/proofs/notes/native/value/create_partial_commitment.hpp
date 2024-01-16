#pragma once
#include "../../constants.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace join_split_example::proofs::notes::native::value {

inline auto create_partial_commitment(bb::fr const& secret,
                                      grumpkin::g1::affine_element const& owner,
                                      bool account_required,
                                      bb::fr const& creator_pubkey)
{
    return crypto::pedersen_hash::hash({ secret, owner.x, owner.y, account_required, creator_pubkey },
                                       GeneratorIndex::VALUE_NOTE_PARTIAL_COMMITMENT);
}

} // namespace join_split_example::proofs::notes::native::value

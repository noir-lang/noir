#pragma once
#include "barretenberg/examples/join_split/constants.hpp"
#include "barretenberg/examples/join_split/types.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"

namespace bb::join_split_example::proofs::notes::circuit::value {

inline auto create_partial_commitment(field_ct const& secret,
                                      group_ct const& owner,
                                      bool_ct const& account_required,
                                      field_ct const& creator_pubkey)
{
    return pedersen_hash::hash({ secret, owner.x, owner.y, account_required, creator_pubkey },
                               GeneratorIndex::VALUE_NOTE_PARTIAL_COMMITMENT);
}

} // namespace bb::join_split_example::proofs::notes::circuit::value

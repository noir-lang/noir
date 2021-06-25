#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace account {

using namespace plonk::stdlib::types::turbo;

inline point_ct commit(field_ct const& account_alias_id,
                       point_ct const& account_public_key,
                       point_ct const& signing_pub_key)
{
    std::vector<field_ct> leaf_elements{
        account_alias_id,
        account_public_key.x,
        signing_pub_key.x,
    };
    return pedersen::commit(leaf_elements, GeneratorIndex::ACCOUNT_NOTE_HASH_INPUTS, true);
}

} // namespace account
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup
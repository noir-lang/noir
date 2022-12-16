#pragma once
#include <stdlib/types/types.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace account {

using namespace plonk::stdlib::types;

inline auto commit(field_ct const& account_alias_hash,
                   point_ct const& account_public_key,
                   point_ct const& signing_pub_key)
{
    return pedersen::compress(
        {
            account_alias_hash,
            account_public_key.x,
            signing_pub_key.x,
        },
        GeneratorIndex::ACCOUNT_NOTE_COMMITMENT);
}

} // namespace account
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup
#pragma once
#include <stdlib/types/turbo.hpp>
#include "../../constants.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace circuit {
namespace account {

using namespace plonk::stdlib::types::turbo;

inline auto commit(field_ct const& account_alias_id,
                   point_ct const& account_public_key,
                   point_ct const& signing_pub_key)
{
    return pedersen::compress(
        {
            account_alias_id,
            account_public_key.x,
            signing_pub_key.x,
        },
        true,
        GeneratorIndex::ACCOUNT_NOTE_COMMITMENT);
}

} // namespace account
} // namespace circuit
} // namespace notes
} // namespace proofs
} // namespace rollup
#pragma once
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace plonk::stdlib::types::turbo;

struct account_note {
    point_ct account_key;
    point_ct signing_key;
};

account_note create_account_note(Composer& composer,
                                 grumpkin::g1::affine_element const& account_key,
                                 grumpkin::g1::affine_element const& signing_key)
{
    return {
        { witness_ct(&composer, account_key.x), witness_ct(&composer, account_key.y) },
        { witness_ct(&composer, signing_key.x), witness_ct(&composer, signing_key.y) },
    };
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup
#pragma once
#include "account_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {
namespace account {

using namespace barretenberg;

inline fr compute_account_public_key_nullifier(grumpkin::g1::affine_element const& public_key)
{
    return crypto::pedersen::compress_native(std::vector<fr>{ public_key.x },
                                             notes::GeneratorIndex::ACCOUNT_PUBLIC_KEY_NULLIFIER);
}

} // namespace account
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example

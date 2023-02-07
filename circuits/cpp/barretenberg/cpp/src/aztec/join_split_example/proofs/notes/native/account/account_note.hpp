#pragma once
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include "../../constants.hpp"

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {
namespace account {

grumpkin::fq generate_account_commitment(const barretenberg::fr& alias_hash,
                                         const barretenberg::fr& owner_x,
                                         const barretenberg::fr& signing_x);

struct account_note {
    barretenberg::fr alias_hash;
    grumpkin::g1::affine_element owner_key;
    grumpkin::g1::affine_element signing_key;

    grumpkin::fq commit() const;
};

} // namespace account
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example
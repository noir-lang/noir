#pragma once
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

struct account_note {
    barretenberg::fr account_id;
    grumpkin::g1::affine_element owner_key;
    grumpkin::g1::affine_element signing_key;
};

grumpkin::g1::affine_element encrypt_account_note(account_note const& note);

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup
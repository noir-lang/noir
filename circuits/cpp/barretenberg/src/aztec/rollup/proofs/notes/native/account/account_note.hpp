#pragma once
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace account {

struct account_note {
    barretenberg::fr account_alias_id;
    grumpkin::g1::affine_element owner_key;
    grumpkin::g1::affine_element signing_key;
};

} // namespace account
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup
#pragma once
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

struct account_note {
    grumpkin::g1::affine_element owner_key;
    grumpkin::g1::affine_element signing_key;
};

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup
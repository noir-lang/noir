#pragma once
#include "claim_note.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {
namespace claim {

grumpkin::g1::affine_element encrypt(claim_note const& note);

} // namespace claim
} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup
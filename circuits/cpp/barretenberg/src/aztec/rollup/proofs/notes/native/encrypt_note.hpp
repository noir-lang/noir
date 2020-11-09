#pragma once
#include "value_note.hpp"

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

grumpkin::g1::affine_element encrypt_note(value_note const& note);

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup
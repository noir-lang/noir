#pragma once
#include "value_note.hpp"
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

using namespace crypto::schnorr;

signature sign_notes(std::array<value_note, 4> const& notes,
                     fr const& output_owner,
                     key_pair<grumpkin::fr, grumpkin::g1> const& keys,
                     numeric::random::Engine* engine = nullptr);

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup
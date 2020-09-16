#pragma once
#include "tx_note.hpp"
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace proofs {
namespace notes {

using namespace crypto::schnorr;

signature sign_notes(std::array<tx_note, 4> const& notes,
                     fr const& output_owner,
                     key_pair<grumpkin::fr, grumpkin::g1> const& keys,
                     numeric::random::Engine* engine = nullptr);

} // namespace notes
} // namespace proofs
} // namespace rollup
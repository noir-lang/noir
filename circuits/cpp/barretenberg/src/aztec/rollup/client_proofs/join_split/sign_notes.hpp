#pragma once
#include "tx_note.hpp"
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace crypto::schnorr;

signature sign_notes(std::array<tx_note, 4> const& notes,
                     key_pair<grumpkin::fr, grumpkin::g1> const& keys,
                     numeric::random::Engine* engine = nullptr);

} // namespace join_split
} // namespace client_proofs
} // namespace rollup
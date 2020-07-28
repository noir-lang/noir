#pragma once
#include "tx_note.hpp"
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace client_proofs {
namespace escape_hatch {

using namespace crypto::schnorr;

signature sign_notes(std::array<tx_note, 2> const& notes, key_pair<grumpkin::fr, grumpkin::g1> const& keys);

} // namespace escape_hatch
} // namespace client_proofs
} // namespace rollup
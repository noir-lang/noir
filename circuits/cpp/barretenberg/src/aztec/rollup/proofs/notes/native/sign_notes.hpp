#pragma once
#include "value_note.hpp"
#include "../../join_split/join_split_tx.hpp"
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

using namespace crypto::schnorr;

signature sign_notes(proofs::join_split::join_split_tx const& tx,
                     key_pair<grumpkin::fr, grumpkin::g1> const& keys,
                     numeric::random::Engine* engine = nullptr);

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup
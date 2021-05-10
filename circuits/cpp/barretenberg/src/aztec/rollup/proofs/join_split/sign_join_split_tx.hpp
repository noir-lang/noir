#pragma once
#include <crypto/schnorr/schnorr.hpp>
#include "../notes/native/value/value_note.hpp"
#include "join_split_tx.hpp"

namespace rollup {
namespace proofs {
namespace join_split {

using namespace crypto::schnorr;

signature sign_join_split_tx(proofs::join_split::join_split_tx const& tx,
                             key_pair<grumpkin::fr, grumpkin::g1> const& keys,
                             numeric::random::Engine* engine = nullptr);

} // namespace join_split
} // namespace proofs
} // namespace rollup
#pragma once
#include <crypto/schnorr/schnorr.hpp>
#include "../notes/native/value/value_note.hpp"
#include "join_split_tx.hpp"

namespace join_split_example {
namespace proofs {
namespace join_split {

using namespace crypto::schnorr;

signature sign_join_split_tx(proofs::join_split::join_split_tx const& tx,
                             key_pair<grumpkin::fr, grumpkin::g1> const& keys);

} // namespace join_split
} // namespace proofs
} // namespace join_split_example
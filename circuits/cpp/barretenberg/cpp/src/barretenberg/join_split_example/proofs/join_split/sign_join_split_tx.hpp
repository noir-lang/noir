#pragma once
#include "../notes/native/value/value_note.hpp"
#include "barretenberg/crypto/schnorr/schnorr.hpp"
#include "join_split_tx.hpp"

namespace join_split_example {
namespace proofs {
namespace join_split {

crypto::schnorr::signature sign_join_split_tx(proofs::join_split::join_split_tx const& tx,
                                              crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> const& keys);

} // namespace join_split
} // namespace proofs
} // namespace join_split_example

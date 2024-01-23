#pragma once
#include "../notes/native/value/value_note.hpp"
#include "barretenberg/crypto/schnorr/schnorr.hpp"
#include "join_split_tx.hpp"

namespace bb::join_split_example::proofs::join_split {

crypto::schnorr_signature sign_join_split_tx(proofs::join_split::join_split_tx const& tx,
                                             crypto::schnorr_key_pair<grumpkin::fr, grumpkin::g1> const& keys);

}

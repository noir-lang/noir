#pragma once
#include "join_split_tx.hpp"

namespace bb::join_split_example::proofs::join_split {

bb::fr compute_signing_data(join_split_tx const& tx);

}
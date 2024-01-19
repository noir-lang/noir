#pragma once
#include "../compute_circuit_data.hpp"
#include "join_split_tx.hpp"

namespace bb::join_split_example::proofs::join_split {

join_split_tx noop_tx();

using circuit_data = proofs::circuit_data;

circuit_data get_circuit_data(std::shared_ptr<bb::srs::factories::CrsFactory<curve::BN254>> const& srs,
                              bool mock = false);

} // namespace bb::join_split_example::proofs::join_split
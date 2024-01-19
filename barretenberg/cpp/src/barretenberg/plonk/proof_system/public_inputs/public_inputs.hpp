#pragma once
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <vector>

namespace bb::plonk {
template <typename Field>
Field compute_public_input_delta(const std::vector<Field>& inputs,
                                 const Field& beta,
                                 const Field& gamma,
                                 const Field& subgroup_generator);
}

#include "public_inputs_impl.hpp"

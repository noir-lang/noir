#pragma once

#include "poseidon2_params.hpp"
#include "poseidon2_permutation.hpp"
#include "sponge/sponge.hpp"

namespace crypto {

template <typename Params> class Poseidon2 {
  public:
    using FF = typename Params::FF;

    using Sponge = FieldSponge<FF, Params::t - 1, 1, Params::t, Poseidon2Permutation<Params>>;
    static FF hash(std::span<FF> input) { return Sponge::hash_fixed_length(input); }
};
} // namespace crypto
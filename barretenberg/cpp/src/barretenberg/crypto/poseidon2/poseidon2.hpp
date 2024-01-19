#pragma once

#include "poseidon2_params.hpp"
#include "poseidon2_permutation.hpp"
#include "sponge/sponge.hpp"

namespace bb::crypto {

template <typename Params> class Poseidon2 {
  public:
    using FF = typename Params::FF;

    // We choose our rate to be t-1 and capacity to be 1.
    using Sponge = FieldSponge<FF, Params::t - 1, 1, Params::t, Poseidon2Permutation<Params>>;

    /**
     * @brief Hashes a vector of field elements
     */
    static FF hash(const std::vector<FF>& input);
    /**
     * @brief Hashes vector of bytes by chunking it into 31 byte field elements and calling hash()
     * @details Slice function cuts out the required number of bytes from the byte vector
     */
    static FF hash_buffer(const std::vector<uint8_t>& input);
};

extern template class Poseidon2<Poseidon2Bn254ScalarFieldParams>;
} // namespace bb::crypto
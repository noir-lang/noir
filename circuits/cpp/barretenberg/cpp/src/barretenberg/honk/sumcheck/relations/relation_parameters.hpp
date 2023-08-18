#pragma once

#include <cstddef>
namespace proof_system::honk::sumcheck {

/**
 * @brief Container for parameters used by the grand product (permutation, lookup) Honk relations
 *
 * @tparam FF
 */
template <typename FF> struct RelationParameters {
    FF eta = FF(0);                        // Lookup
    FF beta = FF(0);                       // Permutation + Lookup
    FF gamma = FF(0);                      // Permutation + Lookup
    FF public_input_delta = FF(0);         // Permutation
    FF lookup_grand_product_delta = FF(0); // Lookup
};
} // namespace proof_system::honk::sumcheck

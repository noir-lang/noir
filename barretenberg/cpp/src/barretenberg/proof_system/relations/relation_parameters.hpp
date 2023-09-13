#pragma once

namespace proof_system {

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

    static RelationParameters get_random()
    {
        RelationParameters result;
        result.eta = FF::random_element();
        result.beta = FF::random_element();
        result.gamma = FF::random_element();
        result.public_input_delta = FF::random_element();
        result.lookup_grand_product_delta = FF::random_element();
        return result;
    }
};
} // namespace proof_system

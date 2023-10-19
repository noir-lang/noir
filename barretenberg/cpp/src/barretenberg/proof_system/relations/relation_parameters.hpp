#pragma once
#include <array>
namespace proof_system {

/**
 * @brief Container for parameters used by the grand product (permutation, lookup) Honk relations
 *
 * @tparam FF
 */
template <typename FF> struct RelationParameters {
    static constexpr int NUM_BINARY_LIMBS_IN_GOBLIN_TRANSLATOR = 4;
    static constexpr int NUM_NATIVE_LIMBS_IN_GOBLIN_TRANSLATOR = 1;
    static constexpr int NUM_CHALLENGE_POWERS_IN_GOBLIN_TRANSLATOR = 4;
    FF eta = FF(0);                        // Lookup
    FF beta = FF(0);                       // Permutation + Lookup
    FF gamma = FF(0);                      // Permutation + Lookup
    FF public_input_delta = FF(0);         // Permutation
    FF lookup_grand_product_delta = FF(0); // Lookup
    FF beta_sqr = 0;
    FF beta_cube = 0;
    // eccvm_set_permutation_delta is used in the set membership gadget in eccvm/ecc_set_relation.hpp
    // We can remove this by modifying the relation, but increases complexity
    FF eccvm_set_permutation_delta = 0;
    std::array<FF, NUM_BINARY_LIMBS_IN_GOBLIN_TRANSLATOR> accumulated_result = {
        FF(0), FF(0), FF(0), FF(0)
    }; // Goblin Translator
    std::array<FF, NUM_BINARY_LIMBS_IN_GOBLIN_TRANSLATOR + NUM_NATIVE_LIMBS_IN_GOBLIN_TRANSLATOR> evaluation_input_x = {
        FF(0), FF(0), FF(0), FF(0), FF(0)
    }; // Goblin Translator
    std::array<std::array<FF, NUM_BINARY_LIMBS_IN_GOBLIN_TRANSLATOR + NUM_NATIVE_LIMBS_IN_GOBLIN_TRANSLATOR>,
               NUM_CHALLENGE_POWERS_IN_GOBLIN_TRANSLATOR>
        batching_challenge_v = { { { FF(0), FF(0), FF(0), FF(0), FF(0) },
                                   { FF(0), FF(0), FF(0), FF(0), FF(0) },
                                   { FF(0), FF(0), FF(0), FF(0), FF(0) },
                                   { FF(0), FF(0), FF(0), FF(0), FF(0) } } };

    static RelationParameters get_random()
    {
        RelationParameters result;
        result.eta = FF::random_element();
        result.beta_sqr = result.beta.sqr();
        result.beta_cube = result.beta_sqr * result.beta;
        result.beta = FF::random_element();
        result.gamma = FF::random_element();
        result.public_input_delta = FF::random_element();
        result.lookup_grand_product_delta = FF::random_element();
        result.eccvm_set_permutation_delta = result.gamma * (result.gamma + result.beta_sqr) *
                                             (result.gamma + result.beta_sqr + result.beta_sqr) *
                                             (result.gamma + result.beta_sqr + result.beta_sqr + result.beta_sqr);
        result.accumulated_result = {
            FF::random_element(), FF::random_element(), FF::random_element(), FF::random_element()
        };

        result.evaluation_input_x = {
            FF::random_element(), FF::random_element(), FF::random_element(), FF::random_element(), FF::random_element()
        };
        result.batching_challenge_v = {
            std::array{ FF::random_element(),
                        FF::random_element(),
                        FF::random_element(),
                        FF::random_element(),
                        FF::random_element() },
            { FF::random_element(),
              FF::random_element(),
              FF::random_element(),
              FF::random_element(),
              FF::random_element() },
            { FF::random_element(),
              FF::random_element(),
              FF::random_element(),
              FF::random_element(),
              FF::random_element() },
            { FF::random_element(),
              FF::random_element(),
              FF::random_element(),
              FF::random_element(),
              FF::random_element() },
        };

        return result;
    }
};
} // namespace proof_system

#include "./pedersen_refactor.hpp"
#include <iostream>
#ifndef NO_OMP_MULTITHREADING
#include <omp.h>
#endif

// TODO(@zac-wiliamson #2341 rename to pedersen.cpp once we migrate to new hash standard)

namespace crypto {

using namespace generators;

/**
 * Given a vector of fields, generate a pedersen hash using the indexed generators.
 */

/**
 * @brief Given a vector of fields, generate a pedersen hash using generators from `generator_context`.
 *
 * @details `hash_index` is used to access offset elements of `generator_context` if required.
 *          e.g. if one desires to compute
 *          `inputs[0] * [generators[hash_index]] + `inputs[1] * [generators[hash_index + 1]]` + ... etc
 *          Potentially useful to ensure multiple hashes with the same domain separator cannot collide.
 *
 * TODO(@suyash67) can we change downstream code so that `hash_index` is no longer required? Now we have a proper
 * domain_separator parameter, we no longer need to specify different generator indices to ensure hashes cannot collide.
 * @param inputs what are we hashing?
 * @param hash_index Describes an offset into the list of generators, if required
 * @param generator_context
 * @return Fq (i.e. SNARK circuit scalar field, when hashing using a curve defined over the SNARK circuit scalar field)
 */
template <typename Curve>
typename Curve::BaseField pedersen_hash_refactor<Curve>::hash_multiple(const std::vector<Fq>& inputs,
                                                                       const size_t hash_index,
                                                                       const generator_data* const generator_context)
{
    const auto generators = generator_context->conditional_extend(inputs.size() + hash_index);

    Element result = get_length_generator() * Fr(inputs.size());

    for (size_t i = 0; i < inputs.size(); ++i) {
        result += generators.get(i, hash_index) * Fr(static_cast<uint256_t>(inputs[i]));
    }
    result = result.normalize();
    return result.x;
}

template <typename Curve>
typename Curve::BaseField pedersen_hash_refactor<Curve>::hash(const std::vector<Fq>& inputs,
                                                              size_t hash_index,
                                                              const generator_data* const generator_context)
{
    return hash_multiple(inputs, hash_index, generator_context);
}

template class pedersen_hash_refactor<curve::Grumpkin>;
} // namespace crypto
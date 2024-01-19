#include "./pedersen.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include <iostream>
#ifndef NO_OMP_MULTITHREADING
#include <omp.h>
#endif

namespace bb::crypto {

/**
 * @brief Given a vector of fields, generate a pedersen commitment using the indexed generators.
 *
 * @details This method uses `Curve::BaseField` members as inputs. This aligns with what we expect when creating
 * grumpkin commitments to field elements inside a BN254 SNARK circuit.
 * @param inputs
 * @param context
 * @return Curve::AffineElement
 */
template <typename Curve>
typename Curve::AffineElement pedersen_commitment_base<Curve>::commit_native(const std::vector<Fq>& inputs,
                                                                             const GeneratorContext context)
{
    const auto generators = context.generators->get(inputs.size(), context.offset, context.domain_separator);
    Element result = Group::point_at_infinity;

    for (size_t i = 0; i < inputs.size(); ++i) {
        result += Element(generators[i]) * static_cast<uint256_t>(inputs[i]);
    }
    return result.normalize();
}
template class pedersen_commitment_base<curve::Grumpkin>;
} // namespace bb::crypto

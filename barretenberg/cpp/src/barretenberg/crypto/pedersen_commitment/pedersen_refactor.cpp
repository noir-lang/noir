// TODO(@zac-wiliamson #2341 rename to pedersen.cpp once we migrate to new hash standard)

#include "./pedersen_refactor.hpp"
#include "./convert_buffer_to_field.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include <iostream>
#ifndef NO_OMP_MULTITHREADING
#include <omp.h>
#endif

namespace crypto {

/**
 * @brief Given a vector of fields, generate a pedersen commitment using the indexed generators.
 *
 * @details This method uses `Curve::BaseField` members as inputs. This aligns with what we expect when creating
 * grumpkin commitments to field elements inside a BN254 SNARK circuit.
 *
 * @note Fq is the *coordinate field* of Curve. Curve itself is a SNARK-friendly curve,
 * i.e. Fq represents the native field type of the SNARK circuit.
 * @param inputs
 * @param hash_index
 * @param generator_context
 * @return Curve::AffineElement
 */
template <typename Curve>
typename Curve::AffineElement pedersen_commitment_refactor<Curve>::commit_native(
    const std::vector<Fq>& inputs, const size_t hash_index, const generator_data<Curve>* const generator_context)
{
    const auto generators = generator_context->conditional_extend(inputs.size() + hash_index);
    Element result = Group::point_at_infinity;

    // `Curve::Fq` represents the field that `Curve` is defined over (i.e. x/y coordinate field) and `Curve::Fr` is the
    // field whose modulus = the group order of `Curve`.
    // The `Curve` we're working over here is a generic SNARK-friendly curve. i.e. the SNARK circuit is defined over a
    // field equivalent to `Curve::Fq`. This adds complexity when we wish to commit to SNARK circuit field elements, as
    // these are members of `Fq` and *not* `Fr`. We cast to `uint256_t` in order to convert an element of `Fq` into an
    // `Fr` element, which is the required type when performing scalar multiplications.
    static_assert(Fr::modulus > Fq::modulus,
                  "pedersen_commitment::commit_native Curve subgroup field is smaller than coordinate field. Cannot "
                  "perform injective conversion");
    for (size_t i = 0; i < inputs.size(); ++i) {
        Fr scalar_multiplier(static_cast<uint256_t>(inputs[i]));
        result += Element(generators.get(i, hash_index)) * scalar_multiplier;
    }
    return result;
}

template class pedersen_commitment_refactor<curve::Grumpkin>;
} // namespace crypto

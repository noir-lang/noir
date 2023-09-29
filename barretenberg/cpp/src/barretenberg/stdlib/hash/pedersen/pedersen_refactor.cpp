#include "pedersen_refactor.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
namespace proof_system::plonk::stdlib {

using namespace barretenberg;
using namespace crypto::generators;
using namespace proof_system;

template <typename C>
field_t<C> pedersen_hash_refactor<C>::hash_multiple(const std::vector<field_t>& inputs,
                                                    const size_t hash_index,
                                                    const generator_data* generator_context,
                                                    const bool /*unused*/)
{

    using cycle_group = cycle_group<C>;
    using cycle_scalar = typename cycle_group::cycle_scalar;
    using Curve = EmbeddedCurve;

    auto base_points = generator_context->conditional_extend(inputs.size() + hash_index).generators;

    std::vector<cycle_scalar> scalars;
    std::vector<cycle_group> points;
    scalars.emplace_back(cycle_scalar::create_from_bn254_scalar(field_t(inputs.size())));
    points.emplace_back(crypto::pedersen_hash_refactor<Curve>::get_length_generator());
    for (size_t i = 0; i < inputs.size(); ++i) {
        scalars.emplace_back(cycle_scalar::create_from_bn254_scalar(inputs[i]));
        // constructs constant cycle_group objects (non-witness)
        points.emplace_back(base_points[i + hash_index]);
    }

    auto result = cycle_group::batch_mul(scalars, points);
    return result.x;
}

template <typename C>
field_t<C> pedersen_hash_refactor<C>::hash(const std::vector<field_t>& in,
                                           size_t hash_index,
                                           const generator_data* generator_context,
                                           bool validate_inputs_in_field)
{
    return hash_multiple(in, hash_index, generator_context, validate_inputs_in_field);
}
INSTANTIATE_STDLIB_TYPE(pedersen_hash_refactor);

} // namespace proof_system::plonk::stdlib

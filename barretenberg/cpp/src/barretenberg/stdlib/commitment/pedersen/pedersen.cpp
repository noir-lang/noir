#include "pedersen.hpp"
#include "../../hash/pedersen/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace bb::stdlib {

template <typename C>
cycle_group<C> pedersen_commitment<C>::commit(const std::vector<field_t>& inputs, const GeneratorContext context)
{

    using cycle_scalar = typename cycle_group::cycle_scalar;

    const auto base_points = context.generators->get(inputs.size(), context.offset, context.domain_separator);

    std::vector<cycle_scalar> scalars;
    std::vector<cycle_group> points;
    for (size_t i = 0; i < inputs.size(); ++i) {
        scalars.emplace_back(cycle_scalar::create_from_bn254_scalar(inputs[i]));
        // constructs circuit-constant cycle_group objects (non-witness)
        points.emplace_back(base_points[i]);
    }

    return cycle_group::batch_mul(scalars, points);
}

template <typename C>
cycle_group<C> pedersen_commitment<C>::commit(const std::vector<std::pair<field_t, GeneratorContext>>& input_pairs)
{

    std::vector<cycle_scalar> scalars;
    std::vector<cycle_group> points;
    for (auto& [scalar, context] : input_pairs) {
        scalars.emplace_back(cycle_scalar::create_from_bn254_scalar(scalar));
        // constructs constant cycle_group objects (non-witness)
        points.emplace_back(context.generators->get(1, context.offset, context.domain_separator)[0]);
    }

    return cycle_group::batch_mul(scalars, points);
}

template class pedersen_commitment<bb::StandardCircuitBuilder>;
template class pedersen_commitment<bb::UltraCircuitBuilder>;
template class pedersen_commitment<bb::GoblinUltraCircuitBuilder>;

} // namespace bb::stdlib

#include "pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
namespace bb::stdlib {

using namespace bb;

template <typename C>
field_t<C> pedersen_hash<C>::hash(const std::vector<field_ct>& inputs, const GeneratorContext context)
{
    using cycle_scalar = typename cycle_group::cycle_scalar;
    using Curve = EmbeddedCurve;

    const auto base_points = context.generators->get(inputs.size(), context.offset, context.domain_separator);

    std::vector<cycle_scalar> scalars;
    std::vector<cycle_group> points;
    scalars.emplace_back(cycle_scalar::create_from_bn254_scalar(field_ct(inputs.size())));
    points.emplace_back(crypto::pedersen_hash_base<Curve>::length_generator);
    for (size_t i = 0; i < inputs.size(); ++i) {
        scalars.emplace_back(cycle_scalar::create_from_bn254_scalar(inputs[i]));
        // constructs constant cycle_group objects (non-witness)
        points.emplace_back(base_points[i]);
    }

    auto result = cycle_group::batch_mul(scalars, points);
    return result.x;
}

template <typename C>
field_t<C> pedersen_hash<C>::hash_skip_field_validation(const std::vector<field_ct>& inputs,
                                                        const GeneratorContext context)
{
    using cycle_scalar = typename cycle_group::cycle_scalar;
    using Curve = EmbeddedCurve;

    const auto base_points = context.generators->get(inputs.size(), context.offset, context.domain_separator);

    std::vector<cycle_scalar> scalars;
    std::vector<cycle_group> points;
    scalars.emplace_back(cycle_scalar::create_from_bn254_scalar(field_ct(inputs.size())));
    points.emplace_back(crypto::pedersen_hash_base<Curve>::length_generator);
    for (size_t i = 0; i < inputs.size(); ++i) {
        // `true` param = skip primality test when performing a scalar mul
        scalars.emplace_back(cycle_scalar::create_from_bn254_scalar(inputs[i], true));
        // constructs constant cycle_group objects (non-witness)
        points.emplace_back(base_points[i]);
    }

    auto result = cycle_group::batch_mul(scalars, points);
    return result.x;
}

/**
 * @brief Hash a byte_array.
 *
 * TODO(@zac-williamson #2796) Once Poseidon is implemented, replace this method with a more canonical hash algorithm
 * (that is less efficient)
 */
template <typename C>
field_t<C> pedersen_hash<C>::hash_buffer(const stdlib::byte_array<C>& input, GeneratorContext context)
{
    const size_t num_bytes = input.size();
    const size_t bytes_per_element = 31;
    size_t num_elements = static_cast<size_t>(num_bytes % bytes_per_element != 0) + (num_bytes / bytes_per_element);

    std::vector<field_ct> elements;
    for (size_t i = 0; i < num_elements; ++i) {
        size_t bytes_to_slice = 0;
        if (i == num_elements - 1) {
            bytes_to_slice = num_bytes - (i * bytes_per_element);
        } else {
            bytes_to_slice = bytes_per_element;
        }
        auto element = static_cast<field_ct>(input.slice(i * bytes_per_element, bytes_to_slice));
        elements.emplace_back(element);
    }
    field_ct hashed;
    if (elements.size() < 2) {
        hashed = hash(elements, context);
    } else {
        hashed = hash({ elements[0], elements[1] }, context);
        for (size_t i = 2; i < elements.size(); ++i) {
            hashed = hash({ hashed, elements[i] }, context);
        }
    }
    return hashed;
}
template class pedersen_hash<bb::StandardCircuitBuilder>;
template class pedersen_hash<bb::UltraCircuitBuilder>;
template class pedersen_hash<bb::GoblinUltraCircuitBuilder>;

} // namespace bb::stdlib

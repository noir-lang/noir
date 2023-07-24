#include "pedersen.hpp"
#include "../../hash/pedersen/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "pedersen_plookup.hpp"

#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace proof_system::plonk {
namespace stdlib {

using namespace crypto::generators;
using namespace barretenberg;
using namespace crypto::pedersen_commitment;

template <typename C>
point<C> pedersen_commitment<C>::commit(const std::vector<field_t>& inputs, const size_t hash_index)
{
    if constexpr (HasPlookup<C> && C::commitment_type == pedersen::CommitmentType::LOOKUP_PEDERSEN) {
        return pedersen_plookup_commitment<C>::commit(inputs, hash_index);
    }

    std::vector<point> to_accumulate;
    for (size_t i = 0; i < inputs.size(); ++i) {
        generator_index_t index = { hash_index, i };
        to_accumulate.push_back(pedersen_hash<C>::commit_single(inputs[i], index));
    }
    return pedersen_hash<C>::accumulate(to_accumulate);
}

template <typename C>
point<C> pedersen_commitment<C>::commit(const std::vector<field_t>& inputs,
                                        const std::vector<generator_index_t>& hash_generator_indices)
{
    if (inputs.size() != hash_generator_indices.size()) {
        throw_or_abort("Vector size mismatch.");
    }

    if constexpr (HasPlookup<C> && C::commitment_type == pedersen::CommitmentType::LOOKUP_PEDERSEN) {
        return pedersen_plookup_commitment<C>::commit(inputs, hash_generator_indices);
    }

    std::vector<point> to_accumulate;
    for (size_t i = 0; i < inputs.size(); ++i) {
        to_accumulate.push_back(pedersen_hash<C>::commit_single(inputs[i], hash_generator_indices[i]));
    }

    return pedersen_hash<C>::accumulate(to_accumulate);
}

template <typename C>
point<C> pedersen_commitment<C>::commit(const std::vector<std::pair<field_t, generator_index_t>>& input_pairs)
{
    if constexpr (HasPlookup<C> && C::commitment_type == pedersen::CommitmentType::LOOKUP_PEDERSEN) {
        return pedersen_plookup_commitment<C>::commit(input_pairs);
    }

    std::vector<point> to_accumulate;
    std::vector<field_t> inputs;
    for (size_t i = 0; i < input_pairs.size(); ++i) {
        to_accumulate.push_back(pedersen_hash<C>::commit_single(input_pairs[i].first, input_pairs[i].second));
        inputs.push_back(input_pairs[i].first);
    }

    return pedersen_hash<C>::accumulate(to_accumulate);
}

/**
 * Compress the pair (in_left, in_right) with a given hash index.
 * Called unsafe because this allows the option of not validating the input elements are unique, i.e. <r
 */
template <typename C>
field_t<C> pedersen_commitment<C>::compress_unsafe(const field_t& in_left,
                                                   const field_t& in_right,
                                                   const size_t hash_index,
                                                   const bool validate_input_is_in_field)
{
    if constexpr (HasPlookup<C> && C::commitment_type == pedersen::CommitmentType::LOOKUP_PEDERSEN) {
        return pedersen_plookup_commitment<C>::compress({ in_left, in_right });
    }

    std::vector<point> accumulators;
    generator_index_t index_1 = { hash_index, 0 };
    generator_index_t index_2 = { hash_index, 1 };
    accumulators.push_back(pedersen_hash<C>::commit_single(in_left, index_1, validate_input_is_in_field));
    accumulators.push_back(pedersen_hash<C>::commit_single(in_right, index_2, validate_input_is_in_field));
    return pedersen_hash<C>::accumulate(accumulators).x;
}

/**
 * Compress a vector of scalars with a given hash index.
 */
template <typename C>
field_t<C> pedersen_commitment<C>::compress(const std::vector<field_t>& inputs, const size_t hash_index)
{
    if constexpr (HasPlookup<C> && C::commitment_type == pedersen::CommitmentType::LOOKUP_PEDERSEN) {
        return pedersen_plookup_commitment<C>::compress(inputs, hash_index);
    }

    return commit(inputs, hash_index).x;
}

/**
 * Compress a byte_array.
 *
 * If the input values are all zero, we return the array length instead of "0\"
 * This is because we require the inputs to regular pedersen compression function are nonzero (we use this method to
 * hash the base layer of our merkle trees)
 */
template <typename C> field_t<C> pedersen_commitment<C>::compress(const byte_array& input)
{
    const size_t num_bytes = input.size();
    const size_t bytes_per_element = 31;
    size_t num_elements = (num_bytes % bytes_per_element != 0) + (num_bytes / bytes_per_element);

    std::vector<field_t> elements;
    for (size_t i = 0; i < num_elements; ++i) {
        size_t bytes_to_slice = 0;
        if (i == num_elements - 1) {
            bytes_to_slice = num_bytes - (i * bytes_per_element);
        } else {
            bytes_to_slice = bytes_per_element;
        }
        field_t element = static_cast<field_t>(input.slice(i * bytes_per_element, bytes_to_slice));
        elements.emplace_back(element);
    }
    field_t compressed = compress(elements, 0);

    bool_t is_zero(true);
    for (const auto& element : elements) {
        is_zero = is_zero && element.is_zero();
    }

    field_t output = field_t::conditional_assign(is_zero, field_t(num_bytes), compressed);
    return output;
}

INSTANTIATE_STDLIB_TYPE(pedersen_commitment);

} // namespace stdlib
} // namespace proof_system::plonk
#include "barretenberg/stdlib/hash/poseidon2/poseidon2.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
namespace proof_system::plonk::stdlib {

using namespace bb;
using namespace proof_system;

/**
 * @brief Hash a vector of field_ct.
 */
template <typename C> field_t<C> poseidon2<C>::hash(C& builder, const std::vector<field_ct>& inputs)
{

    /* Run the sponge by absorbing all the input and squeezing one output.
     * This should just call the sponge variable length hash function
     *
     */
    auto input{ inputs };
    return Sponge::hash_fixed_length(builder, input);
}

/**
 * @brief Hash a byte_array.
 */
template <typename C> field_t<C> poseidon2<C>::hash_buffer(C& builder, const stdlib::byte_array<C>& input)
{
    const size_t num_bytes = input.size();
    const size_t bytes_per_element = 31; // 31 bytes in a fr element
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
    return hash(builder, elements);
}
template class poseidon2<proof_system::GoblinUltraCircuitBuilder>;

} // namespace proof_system::plonk::stdlib

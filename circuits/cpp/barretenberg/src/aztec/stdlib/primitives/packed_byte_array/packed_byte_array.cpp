#include "packed_byte_array.hpp"

#include "../composers/composers.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

namespace {
template <typename Composer> Composer* get_context_from_fields(const std::vector<field_t<Composer>>& input)
{
    for (const auto& element : input) {
        if (element.get_context()) {
            return element.get_context();
        }
    }
    return nullptr;
}
} // namespace

template <typename Composer>
packed_byte_array<Composer>::packed_byte_array(Composer* parent_context, const size_t n)
    : context(parent_context)
    , num_bytes(n)
{
    const size_t num_elements = num_bytes / BYTES_PER_ELEMENT + (num_bytes % BYTES_PER_ELEMENT != 0);
    limbs = std::vector<field_pt>(num_elements);
}

template <typename Composer>
packed_byte_array<Composer>::packed_byte_array(const std::vector<field_pt>& input, const size_t bytes_per_input)
    : context(get_context_from_fields(input))
    , num_bytes(bytes_per_input * input.size())
{
    // TODO HANDLE CASE WHERE bytes_per_input > BYTES_PER_ELEMENT
    const size_t inputs_per_limb = BYTES_PER_ELEMENT / bytes_per_input;

    const size_t num_elements = num_bytes / BYTES_PER_ELEMENT + (num_bytes % BYTES_PER_ELEMENT != 0);
    for (size_t i = 0; i < num_elements; ++i) {
        field_pt limb(context, 0);

        const size_t num_inputs = (i == num_elements - 1) ? (input.size() - (i * inputs_per_limb)) : inputs_per_limb;
        for (size_t j = 0; j < num_inputs; ++j) {
            const uint64_t limb_shift = (BYTES_PER_ELEMENT - ((j + 1) * bytes_per_input)) << 3;
            limb += input[i * inputs_per_limb + j] * field_pt(context, uint256_t(1) << limb_shift);
        }
        limbs.push_back(limb);
    }
}

template <typename Composer>
packed_byte_array<Composer>::packed_byte_array(Composer* parent_context, const std::vector<uint8_t>& input)
    : context(parent_context)
    , num_bytes(input.size())
{
    const size_t num_elements = num_bytes / BYTES_PER_ELEMENT + (num_bytes % BYTES_PER_ELEMENT != 0);
    std::vector<uint256_t> data(num_elements);
    for (size_t i = 0; i < num_elements; ++i) {
        data[i] = 0;
    }
    for (size_t i = 0; i < input.size(); ++i) {
        const size_t limb = i / BYTES_PER_ELEMENT;
        const size_t limb_byte = i % BYTES_PER_ELEMENT;
        const uint64_t limb_shift = (BYTES_PER_ELEMENT - 1U - static_cast<uint64_t>(limb_byte)) << 3;

        data[limb] += uint256_t(input[i]) << limb_shift;
    }

    for (size_t i = 0; i < num_elements; ++i) {
        limbs.push_back(witness_t(context, fr(data[i])));
    }
}

template <typename Composer>
packed_byte_array<Composer>::packed_byte_array(const byte_array& input)
    : context(input.get_context())
    , num_bytes(input.size())
{
    const size_t num_elements = num_bytes / BYTES_PER_ELEMENT + (num_bytes % BYTES_PER_ELEMENT != 0);

    const auto& bits = input.bits();

    constexpr size_t bits_per_element = BYTES_PER_ELEMENT * 8;
    for (size_t i = 0; i < num_elements; ++i) {
        const size_t bytes_in_element = (i == num_elements - 1) ? num_bytes - i * BYTES_PER_ELEMENT : BYTES_PER_ELEMENT;
        const size_t bits_in_element = bytes_in_element * 8;
        field_pt limb(context, 0);
        for (size_t j = 0; j < bits_in_element; ++j) {
            const uint64_t shift = static_cast<uint64_t>(bits_per_element - 1 - j);
            limb += field_pt(bits[i * bits_per_element + j]) * field_pt(context, uint256_t(1) << shift);
        }
        limbs.push_back(limb);
    }
}

template <typename Composer>
packed_byte_array<Composer>::packed_byte_array(Composer* parent_context, const std::string& input)
    : packed_byte_array(parent_context, std::vector<uint8_t>(input.begin(), input.end()))
{}

template <typename Composer>
packed_byte_array<Composer>::packed_byte_array(const packed_byte_array& other)
    : context(other.context)
    , num_bytes(other.num_bytes)
    , limbs(other.limbs.begin(), other.limbs.end())
{}

template <typename Composer>
packed_byte_array<Composer>::packed_byte_array(packed_byte_array&& other)
    : context(other.context)
    , num_bytes(other.num_bytes)
    , limbs(other.limbs.begin(), other.limbs.end())
{}

template <typename Composer>
packed_byte_array<Composer>& packed_byte_array<Composer>::operator=(const packed_byte_array& other)
{
    context = other.context;
    num_bytes = other.num_bytes;
    limbs = std::vector<field_pt>(other.limbs.begin(), other.limbs.end());
    return *this;
}

template <typename Composer>
packed_byte_array<Composer>& packed_byte_array<Composer>::operator=(packed_byte_array&& other)
{
    context = other.context;
    num_bytes = other.num_bytes;
    limbs = std::vector<field_pt>(other.limbs.begin(), other.limbs.end());
    return *this;
}

template <typename Composer> packed_byte_array<Composer>::operator byte_array() const
{
    std::vector<bool_pt> bits;
    const size_t num_bits = num_bytes * 8;

    constexpr size_t bits_per_element = BYTES_PER_ELEMENT * 8;
    for (size_t i = 0; i < limbs.size(); ++i) {
        const size_t bits_in_limb = (i == limbs.size() - 1) ? num_bits - (i * bits_per_element) : bits_per_element;
        field_pt accumulator(context, 0);
        uint256_t limb_value(limbs[i].get_value());
        for (size_t j = 0; j < bits_in_limb; ++j) {
            const uint64_t bit_shift = (bits_per_element - 1 - j);
            uint64_t bit_val = (limb_value >> bit_shift).data[0] & (uint64_t)(1);
            bool_pt bit(witness_t(context, fr(bit_val)));
            accumulator += (field_pt(bit) * field_pt(context, uint256_t(1) << bit_shift));
            bits.emplace_back(bit);
        }
        accumulator.assert_equal(limbs[i]);
    }
    return byte_array(context, bits);
}

template <typename Composer>
void packed_byte_array<Composer>::append(const field_pt& to_append, const size_t bytes_to_append)
{
    const size_t current_capacity = limbs.size() * BYTES_PER_ELEMENT;
    const size_t current_size = size();

    const size_t current_limb_space = current_capacity - current_size;

    const size_t num_bytes_for_current_limb = std::min(current_limb_space, bytes_to_append);

    const size_t num_bytes_for_new_limb = bytes_to_append - num_bytes_for_current_limb;

    const uint256_t append_value(to_append.get_value());

    const uint64_t start = (bytes_to_append - num_bytes_for_current_limb) * 8;
    const uint64_t end = bytes_to_append * 8;

    const uint256_t append_current = append_value.slice(start, end);
    const uint256_t append_next = append_value.slice(0, start);

    const uint64_t current_padding = (current_limb_space - num_bytes_for_current_limb) << 3;
    const uint64_t next_padding = (BYTES_PER_ELEMENT - num_bytes_for_new_limb) << 3;
    bool is_constant = to_append.witness_index == UINT32_MAX;

    field_pt to_current;
    to_current = is_constant ? field_pt(context, append_current) : witness_t(context, append_current);
    limbs[limbs.size() - 1] += (to_current * field_pt(context, uint256_t(1) << current_padding));

    field_pt reconstructed = to_current;
    if (num_bytes_for_new_limb > 0) {
        field_pt to_add;
        to_add = is_constant ? field_pt(context, append_next) : witness_t(context, append_next);
        limbs.emplace_back(to_add * field_pt(context, next_padding));

        reconstructed += to_add * field_pt(context, uint256_t(1) << uint256_t(num_bytes_for_current_limb * 8));
    }

    if (!is_constant) {
        reconstructed.assert_equal(to_append);
    }

    num_bytes += bytes_to_append;
}

template <typename Composer>
std::vector<field_t<Composer>> packed_byte_array<Composer>::to_unverified_byte_slices(
    const size_t bytes_per_slice) const
{
    std::vector<field_pt> slices;
    for (size_t i = 0; i < limbs.size(); ++i) {
        uint256_t limb_value(limbs[i].get_value());
        const size_t bytes_in_limb = (i == limbs.size() - 1) ? num_bytes - (i * BYTES_PER_ELEMENT) : BYTES_PER_ELEMENT;
        const size_t num_slices = (bytes_in_limb / bytes_per_slice) + (bytes_in_limb % bytes_per_slice != 0);

        field_pt accumulator(context, 0);
        for (size_t j = 0; j < num_slices; ++j) {
            const size_t bytes_in_slice =
                (j == num_slices - 1) ? bytes_in_limb - (j * bytes_per_slice) : bytes_per_slice;
            const size_t end = (BYTES_PER_ELEMENT - (j * bytes_in_slice)) << 3;
            const size_t start = (BYTES_PER_ELEMENT - ((j + 1) * bytes_in_slice)) << 3;

            const uint256_t slice = limb_value.slice(start, end);

            if (limbs[i].witness_index != UINT32_MAX) {
                slices.push_back(witness_t(context, fr(slice)));
            } else {
                slices.push_back(field_pt(context, fr(slice)));
            }
            accumulator += (slices.back() * field_pt(context, uint256_t(1) << start));
        }

        limbs[i].assert_equal(accumulator);
    }
    return slices;
}

template <typename Composer> std::string packed_byte_array<Composer>::get_value() const
{
    std::string bytes(num_bytes, 0);
    for (size_t i = 0; i < limbs.size(); ++i) {
        const size_t bytes_in_limb = (i == limbs.size() - 1) ? num_bytes - (i * BYTES_PER_ELEMENT) : BYTES_PER_ELEMENT;
        uint256_t limb_value(limbs[i].get_value());

        for (size_t j = 0; j < bytes_in_limb; ++j) {
            const size_t end = (BYTES_PER_ELEMENT - (j)) << 3;
            const size_t start = (BYTES_PER_ELEMENT - ((j + 1))) << 3;
            const uint8_t slice = static_cast<uint8_t>(limb_value.slice(start, end).data[0]);
            bytes[i * BYTES_PER_ELEMENT + j] = static_cast<char>(slice);
        }
    }
    return bytes;
}

INSTANTIATE_STDLIB_TYPE(packed_byte_array);

} // namespace stdlib
} // namespace plonk
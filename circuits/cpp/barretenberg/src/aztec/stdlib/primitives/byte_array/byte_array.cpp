#include "byte_array.hpp"

#include <bitset>

#include "../composers/composers.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(ComposerContext* parent_context)
    : context(parent_context)
{}

template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(ComposerContext* parent_context, const size_t n)
    : context(parent_context)
    , values(std::vector<field_t<ComposerContext>>(n))
{}

template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(ComposerContext* parent_context, const std::string& input)
    : byte_array(parent_context, std::vector<uint8_t>(input.begin(), input.end()))
{}

/**
 * Create a byte array out of a vector of uint8_t bytes.
 * This constructor will instantiate each byte as a circuit witness, NOT a circuit constant.
 * Do not use this method if the input needs to be hardcoded for a specific circuit
 **/ 
template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(ComposerContext* parent_context, std::vector<uint8_t> const& input)
    : context(parent_context)
    , values(input.size())
{
    for (size_t i = 0; i < input.size(); ++i) {
        uint8_t c = input[i];
        field_t<ComposerContext> value(witness_t(context, (uint64_t)c));
        context->create_range_constraint(value.witness_index, 8);
        values[i] = value;
    }
}

/**
 * Create a byte_array out of a field element
 * 
 * The length of the byte array will default to 32 bytes, but shorter lengths can be specified.
 * If a shorter length is used, the circuit will NOT truncate the input to fit the reduced length.
 * Instead, the circuit adds constraints that VALIDATE the input is smaller than the specified length
 * 
 * e.g. if this constructor is used on a 16-bit input witness, where `num_bytes` is 1, the resulting proof will fail
 **/ 
template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(const field_t<ComposerContext>& input, const size_t num_bytes)
{
    uint256_t value = input.get_value();
    values.resize(num_bytes);
    context = input.get_context();
    if (input.is_constant()) {
        for (size_t i = 0; i < num_bytes; ++i) {
            values[i] = barretenberg::fr(value.slice((num_bytes - i - 1) * 8, (num_bytes - i) * 8));
        }
    } else {
        constexpr barretenberg::fr byte_shift(256);
        field_t<ComposerContext> validator(context, barretenberg::fr::zero());
        for (size_t i = 0; i < num_bytes; ++i)
        {
            barretenberg::fr byte_val = value.slice((num_bytes - i - 1)* 8, (num_bytes - i) * 8);
            field_t<ComposerContext> byte = witness_t(context, byte_val);
            context->create_range_constraint(byte.witness_index, 8);
            barretenberg::fr scaling_factor_value = byte_shift.pow(static_cast<uint64_t>(num_bytes - 1 - i));
            field_t<ComposerContext> scaling_factor(context, scaling_factor_value);
            validator = validator + (scaling_factor * byte);
            values[i] = byte;
        }
        context->assert_equal(validator.witness_index, input.witness_index);
    }
}

template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(ComposerContext* parent_context, bytes_t const& input)
    : context(parent_context)
    , values(input)
{}

template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(ComposerContext* parent_context, bytes_t&& input)
    : context(parent_context)
    , values(input)
{}

template <typename ComposerContext> byte_array<ComposerContext>::byte_array(const byte_array& other)
{
    context = other.context;
    std::copy(other.values.begin(), other.values.end(), std::back_inserter(values));
}

template <typename ComposerContext> byte_array<ComposerContext>::byte_array(byte_array&& other)
{
    context = other.context;
    values = std::move(other.values);
}

template <typename ComposerContext>
byte_array<ComposerContext>& byte_array<ComposerContext>::operator=(const byte_array& other)
{
    context = other.context;
    values = std::vector<field_t<ComposerContext>>();
    std::copy(other.values.begin(), other.values.end(), std::back_inserter(values));
    return *this;
}

template <typename ComposerContext>
byte_array<ComposerContext>& byte_array<ComposerContext>::operator=(byte_array&& other)
{
    context = other.context;
    values = std::move(other.values);
    return *this;
}

/**
 * Convert a byte array into a field element.
 * The byte array is represented as a big integer, that is then converted into a field element.
 * The transformation is only injective if the byte array is < 32 bytes.
 * Larger byte arrays can still be cast to a single field element, but the value will wrap around the circuit modulus
 **/ 
template <typename ComposerContext> byte_array<ComposerContext>::operator field_t<ComposerContext>() const
{
    const size_t bytes = values.size();
    barretenberg::fr shift(256);
    field_t<ComposerContext> result(context, barretenberg::fr(0));
    for (size_t i = 0; i < values.size(); ++i) {
        field_t<ComposerContext> temp(values[i]);
        barretenberg::fr scaling_factor_value = shift.pow(static_cast<uint64_t>(bytes - 1 - i));
        field_t<ComposerContext> scaling_factor(values[i].context, scaling_factor_value);
        result = result + (scaling_factor * temp);
    }
    return result.normalize();
}

template <typename ComposerContext>
byte_array<ComposerContext>& byte_array<ComposerContext>::write(byte_array const& other)
{
    values.insert(values.end(), other.bytes().begin(), other.bytes().end());
    return *this;
}

template <typename ComposerContext> byte_array<ComposerContext> byte_array<ComposerContext>::slice(size_t offset) const
{
    ASSERT(offset < values.size());
    return byte_array(context, bytes_t(values.begin() + (long)(offset), values.end()));
}

/**
 * Slice `length` bytes from the byte array, starting at `offset`. Does not add any constraints
 **/ 
template <typename ComposerContext>
byte_array<ComposerContext> byte_array<ComposerContext>::slice(size_t offset, size_t length) const
{
    ASSERT(offset < values.size());
    ASSERT(length < values.size() - offset);
    auto start = values.begin() + (long)(offset);
    auto end = values.begin() + (long)((offset + length));
    return byte_array(context, bytes_t(start, end));
}

/**
 * Reverse the bytes in the byte array
 **/ 
template <typename ComposerContext> byte_array<ComposerContext> byte_array<ComposerContext>::reverse() const
{
    bytes_t bytes(values.size());
    size_t offset = bytes.size() - 1;
    for (size_t i = 0; i < bytes.size(); i += 1, offset -= 1) {
            bytes[offset] = values[i];
    }
    return byte_array(context, bytes);
}

template <typename ComposerContext> std::vector<uint8_t> byte_array<ComposerContext>::get_value() const
{
    size_t length = values.size();
    size_t num = (length);
    std::vector<uint8_t> bytes(num, 0);
    for (size_t i = 0; i < length; ++i) {
        bytes[i] = static_cast<uint8_t>(uint256_t(values[i].get_value()).data[0]);
    }
    return bytes;
}

template <typename ComposerContext> std::string byte_array<ComposerContext>::get_string() const
{
    auto v = get_value();
    return std::string(v.begin(), v.end());
}

/**
 * Extract a bit from the byte array.
 * 
 * get_bit treats the array as a little-endian integer
 * e.g. get_bit(1) corresponds to the second bit in the last, 'least significant' byte in the array.
 * 
 **/ 
template <typename ComposerContext>
bool_t<ComposerContext> byte_array<ComposerContext>::get_bit(size_t index_reversed) const
{
    const size_t index = (values.size() * 8) - index_reversed - 1;
    const auto slice = split_byte(index);

    return slice.bit;
}

/**
 * Set a bit in the byte array
 * 
 * set_bit treats the array as a little-endian integer
 * e.g. set_bit(0) will set the first bit in the last, 'least significant' byte in the array
 * 
 * For example, if we have a 64-byte array filled with zeroes, `set_bit(0, true)` will set `values[63]` to 1,
 *              and set_bit(511, true) will set `values[0]` to 128
 * 
 * Previously we did not reverse the bit index, but we have modified the behaviour to be consistent with `get_bit`
 * 
 * The rationale behind reversing the bit index is so that we can more naturally contain integers inside byte arrays and perform bit manipulation
 **/ 
template <typename ComposerContext>
void byte_array<ComposerContext>::set_bit(size_t index_reversed, bool_t<ComposerContext> const& new_bit)
{
    const size_t index = (values.size() * 8) - index_reversed - 1;
    const auto slice = split_byte(index);
    const size_t byte_index = index / 8UL;
    const size_t bit_index = 7UL - (index % 8UL);

    field_t<ComposerContext> scaled_new_bit = field_t<ComposerContext>(new_bit) * barretenberg::fr(uint256_t(1) << bit_index);
    const auto new_value = slice.low.add_two(slice.high, scaled_new_bit).normalize();
    values[byte_index] = new_value;
}

/**
 * Split a byte at the target bit index, return [low slice, high slice, isolated bit]
 * 
 * This is a private method used by `get_bit` and `set_bit`
 **/ 
template <typename ComposerContext>
typename byte_array<ComposerContext>::byte_slice byte_array<ComposerContext>::split_byte(const size_t index) const
{
    const size_t byte_index = index / 8UL;
    const auto byte = values[byte_index];
    const size_t bit_index = 7UL - (index % 8UL);

    const uint64_t value = uint256_t(byte.get_value()).data[0];
    const uint64_t bit_value = (value >> static_cast<uint64_t>(bit_index)) & 1ULL;

    const uint64_t num_low_bits = static_cast<uint64_t>(bit_index);
    const uint64_t num_high_bits = 7ULL - num_low_bits;
    const uint64_t low_value = value & ((1ULL << num_low_bits) - 1ULL);
    const uint64_t high_value = (bit_index == 7) ? 0ULL : (value >> (8 - num_high_bits));

    if (byte.is_constant())
    {
        field_t<ComposerContext> low(context, low_value);
        field_t<ComposerContext> high(context, high_value);
        bool_t<ComposerContext> bit(context, static_cast<bool>(bit_value));
        return { low, high, bit };
    }
    field_t<ComposerContext> low = witness_t<ComposerContext>(context, low_value);
    field_t<ComposerContext> high = witness_t<ComposerContext>(context, high_value);
    bool_t<ComposerContext> bit = witness_t<ComposerContext>(context, static_cast<bool>(bit_value));

    if (num_low_bits > 0)
    {
        context->create_range_constraint(low.witness_index, static_cast<size_t>(num_low_bits));
    }
    else
    {
        context->assert_equal(low.witness_index, context->zero_idx);
    }

    if (num_high_bits > 0)
    {
        context->create_range_constraint(high.witness_index, static_cast<size_t>(num_high_bits));
    }
    else
    {
        context->assert_equal(high.witness_index, context->zero_idx);
    }

    field_t<ComposerContext> scaled_high = high * barretenberg::fr(uint256_t(1) << (8ULL - num_high_bits));
    field_t<ComposerContext> scaled_bit = field_t<ComposerContext>(bit) * barretenberg::fr(uint256_t(1) << bit_index);
    field_t<ComposerContext> result = low.add_two(scaled_high, scaled_bit);
    result.assert_equal(byte);

    return { low, scaled_high, bit };
}

INSTANTIATE_STDLIB_TYPE(byte_array);

} // namespace stdlib
} // namespace plonk
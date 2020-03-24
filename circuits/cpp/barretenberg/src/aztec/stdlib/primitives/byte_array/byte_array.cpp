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
    , values(std::vector<bool_t<ComposerContext>>(n * 8))
{}

template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(ComposerContext* parent_context, const std::string& input)
    : byte_array(parent_context, std::vector<uint8_t>(input.begin(), input.end()))
{}

template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(ComposerContext* parent_context, std::vector<uint8_t> const& input)
    : context(parent_context)
    , values(input.size() * 8)
{
    for (size_t i = 0; i < input.size(); ++i) {
        uint8_t c = input[i];
        std::bitset<8> char_bits = std::bitset<8>(static_cast<unsigned long long>(c));
        for (size_t j = 0; j < 8; ++j) {
            bool_t<ComposerContext> value(witness_t(context, char_bits[7 - j]));
            values[(i * 8) + j] = value;
        }
    }
}

template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(const field_t<ComposerContext>& input, const size_t num_bytes)
{
    const size_t num_bits = num_bytes * 8;
    barretenberg::fr value = input.get_value().from_montgomery_form();
    values.resize(num_bits);
    context = input.get_context();
    if (input.is_constant()) {
        for (size_t i = 0; i < num_bits; ++i) {
            values[i] = value.get_bit(num_bits - 1 - i);
        }
    } else {
        barretenberg::fr two(2);
        field_t<ComposerContext> validator(context, barretenberg::fr::zero());

        for (size_t i = 0; i < num_bits; ++i) {
            bool_t bit = witness_t(context, value.get_bit(num_bits - 1 - i));
            values[i] = bit;
            barretenberg::fr scaling_factor_value = two.pow(static_cast<uint64_t>(num_bits - 1 - i));
            field_t<ComposerContext> scaling_factor(context, scaling_factor_value);
            validator = validator + (scaling_factor * field_t<ComposerContext>(bit));
        }

        context->assert_equal(validator.witness_index, input.witness_index);
    }
}

template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(ComposerContext* parent_context, bits_t const& input)
    : context(parent_context)
    , values(input)
{}

template <typename ComposerContext>
byte_array<ComposerContext>::byte_array(ComposerContext* parent_context, bits_t&& input)
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
    values = std::vector<bool_t<ComposerContext>>();
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

template <typename ComposerContext> byte_array<ComposerContext>::operator field_t<ComposerContext>() const
{
    const size_t bits = values.size();
    barretenberg::fr two(2);
    field_t<ComposerContext> result(context, barretenberg::fr(0));
    for (size_t i = 0; i < values.size(); ++i) {
        field_t<ComposerContext> temp(values[i].context);
        if (values[i].is_constant()) {
            temp.additive_constant = values[i].get_value() ? barretenberg::fr::one() : barretenberg::fr::zero();
        } else {
            temp.witness_index = values[i].witness_index;
        }
        barretenberg::fr scaling_factor_value = two.pow(static_cast<uint64_t>(bits - 1 - i));
        field_t<ComposerContext> scaling_factor(values[i].context, scaling_factor_value);
        result = result + (scaling_factor * temp);
    }
    return result;
}

template <typename ComposerContext>
byte_array<ComposerContext>& byte_array<ComposerContext>::write(byte_array const& other)
{
    values.insert(values.end(), other.bits().begin(), other.bits().end());
    return *this;
}

template <typename ComposerContext> byte_array<ComposerContext> byte_array<ComposerContext>::slice(size_t offset) const
{
    ASSERT(offset < values.size());
    return byte_array(context, bits_t(values.begin() + (long)(offset * 8), values.end()));
}

template <typename ComposerContext>
byte_array<ComposerContext> byte_array<ComposerContext>::slice_bits(size_t offset, size_t length) const
{
    ASSERT(offset < values.size());
    ASSERT(length < values.size() - offset);
    auto start = values.begin() + (long)(offset);
    auto end = values.begin() + (long)((offset + length));
    return byte_array(context, bits_t(start, end));
}

template <typename ComposerContext>
byte_array<ComposerContext> byte_array<ComposerContext>::slice(size_t offset, size_t length) const
{
    ASSERT(offset < values.size());
    ASSERT(length < values.size() - offset);
    auto start = values.begin() + (long)(offset * 8);
    auto end = values.begin() + (long)((offset + length) * 8);
    return byte_array(context, bits_t(start, end));
}

template <typename ComposerContext> byte_array<ComposerContext> byte_array<ComposerContext>::reverse() const
{
    bits_t bits(values.size());
    size_t offset = bits.size() - 8;
    for (size_t i = 0; i < bits.size(); i += 8, offset -= 8) {
        for (size_t j = 0; j < 8; ++j) {
            bits[offset + j] = values[i + j];
        }
    }
    return byte_array(context, bits);
}

template <typename ComposerContext> std::string byte_array<ComposerContext>::get_value() const
{
    size_t length = values.size();
    size_t num = (length / 8) + (length % 8 != 0);
    std::string bytes(num, 0);
    for (size_t i = 0; i < length; ++i) {
        size_t index = i / 8;
        char shift = static_cast<char>(7 - (i - index * 8));
        char value = static_cast<char>(values[i].get_value() << shift);
        bytes[index] |= value;
    }
    return bytes;
}

INSTANTIATE_STDLIB_TYPE(byte_array);

} // namespace stdlib
} // namespace plonk
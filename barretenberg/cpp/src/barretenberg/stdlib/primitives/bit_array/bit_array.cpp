#include "bit_array.hpp"
#include "../circuit_builders/circuit_builders.hpp"

#include <bitset>

namespace bb::stdlib {

template <typename Builder>
bit_array<Builder>::bit_array(Builder* parent_context, const size_t n)
    : context(parent_context)
    , length(n)
    , values(std::vector<bool_t<Builder>>(n))
{}

template <typename Builder>
bit_array<Builder>::bit_array(Builder* parent_context, const std::string& input)
    : context(parent_context)
{
    length = input.length() * 8;
    values.resize(length);

    for (size_t i = 0; i < input.size(); ++i) {
        char c = input[i];
        std::bitset<8> char_bits = std::bitset<8>(static_cast<unsigned long long>(c));
        // order chars in our buffer, so that 1st char = most significant
        size_t position = length - (8 * (i + 1));
        for (size_t j = 0; j < 8; ++j) {
            // printf("bit [%lu][%lu] = %u\n", i, j, char_bits[j] == true ? 1 : 0);
            witness_t<Builder> value(context, char_bits[j]);
            values[position + j] = value;
        }
    }
}

template <typename Builder>
bit_array<Builder>::bit_array(Builder* parent_context, const std::vector<uint8_t>& input)
    : bit_array(parent_context, std::string(input.begin(), input.end()))
{}

template <typename Builder> bit_array<Builder>::bit_array(uint32<Builder> const& input)
{
    context = input.get_context();
    size_t num_bits = input.get_width();
    values.resize(num_bits);

    for (size_t i = 0; i < num_bits; ++i) {
        values[i] = input.at(i);
    }

    length = num_bits;
}

template <typename Builder> bit_array<Builder>::bit_array(const std::vector<uint32<Builder>>& input)
{
    auto it = std::find_if(input.begin(), input.end(), [](const auto x) { return x.get_context() != nullptr; });
    if (it != std::end(input)) {
        context = it->get_context();
    } else {
        context = nullptr; // hmm
    }

    size_t num_words = input.size();
    size_t num_bits = num_words * 32;

    values.resize(num_bits);
    for (size_t i = 0; i < num_words; ++i) {
        size_t input_index = num_words - 1 - i;
        for (size_t j = 0; j < 32; ++j) {
            values[i * 32 + j] = input[input_index].at(j);
        }
    }
    length = num_bits;
}

template <typename Builder> bit_array<Builder>::bit_array(const bit_array& other)
{
    context = other.context;
    std::copy(other.values.begin(), other.values.end(), std::back_inserter(values));
    length = values.size();
}

template <typename Builder> bit_array<Builder>::bit_array(bit_array&& other)
{
    context = other.context;
    length = other.length;
    values = std::move(other.values); // yoink
}

template <typename Builder> bit_array<Builder>& bit_array<Builder>::operator=(const bit_array& other)
{
    length = other.length;
    context = other.context;
    values = std::vector<bool_t<Builder>>();
    std::copy(other.values.begin(), other.values.end(), std::back_inserter(values));
    return *this;
}

template <typename Builder> bit_array<Builder>& bit_array<Builder>::operator=(bit_array&& other)
{
    length = other.length;
    context = other.context;
    values = std::move(other.values);
    return *this;
}

template <typename Builder> bool_t<Builder>& bit_array<Builder>::operator[](const size_t idx)
{
    return values[idx];
}

template <typename Builder> bool_t<Builder> bit_array<Builder>::operator[](const size_t idx) const
{
    return values[idx];
}

template <typename Builder> std::vector<uint32<Builder>> bit_array<Builder>::to_uint32_vector()
{
    size_t num_uint32s = (length / 32) + (length % 32 != 0);
    std::vector<uint32<Builder>> output;

    for (size_t i = 0; i < num_uint32s; ++i) {
        std::array<bool_t<Builder>, 32> bools;
        size_t end;
        size_t start;
        start = ((num_uint32s - i) * 32) - 32;
        end = start + 32 > length ? length : start + 32;
        for (size_t j = start; j < end; ++j) {
            bools[j - start] = values[j];
        }
        if (start + 32 > length) {
            for (size_t j = end; j < start + 32; ++j) {
                bools[j - start] = bool_t<Builder>(context, false);
            }
        }
        output.push_back(uint32<Builder>(context, bools));
    }
    return output;
}

template <typename Builder> std::string bit_array<Builder>::get_witness_as_string() const
{
    size_t num_chars = length / 8;
    ASSERT(num_chars * 8 == length);

    std::string output;
    output.resize(num_chars);
    for (size_t i = 0; i < num_chars; ++i) {
        std::bitset<8> char_bits;
        size_t position = length - (8 * (i + 1));
        for (size_t j = 0; j < 8; ++j) {
            char_bits[j] = values[position + j].get_value();
        }
        char foo = static_cast<char>(char_bits.to_ulong());
        output[i] = foo;
    }
    return output;
}

template class bit_array<bb::StandardCircuitBuilder>;
template class bit_array<bb::UltraCircuitBuilder>;
template class bit_array<bb::GoblinUltraCircuitBuilder>;

} // namespace bb::stdlib

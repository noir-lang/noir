#pragma once
#include "../circuit_builders/circuit_builders_fwd.hpp"
#include "../uint/uint.hpp"
#include <algorithm>

namespace bb::plonk {
namespace stdlib {

template <typename Builder> class bit_array {
  public:
    bit_array(Builder* parent_context, const size_t n);
    bit_array(Builder* parent_context, const std::string& input);
    bit_array(Builder* parent_context, const std::vector<uint8_t>& input);
    bit_array(const std::vector<uint32<Builder>>& input);
    bit_array(uint32<Builder> const& input);
    bit_array(byte_array<Builder> const& input)
        : context(input.get_context())
        , length(input.size() * 8)
    {
        const auto bytes = input.bytes();
        const size_t num_bits = bytes.size() * 8;
        values.resize(num_bits);
        for (size_t i = 0; i < bytes.size(); ++i) {
            const auto byte = bytes[i];
            field_t<Builder> accumulator(0);
            for (size_t j = 0; j < 8; ++j) {
                const auto bit_witness = uint256_t((uint256_t(byte.get_value()) >> (7 - j)) & uint256_t(1));
                bool_t<Builder> bit = witness_t<Builder>(context, bit_witness);
                values[i * 8 + j] = bit;
                accumulator *= 2;
                accumulator += bit;
            }
            byte.assert_equal(accumulator);
        }
        std::reverse(values.begin(), values.end());
    }

    template <size_t N> bit_array(const std::array<uint32<Builder>, N>& input)
    {
        context = nullptr;
        for (const auto& x : input) {
            if (x.get_context() != nullptr) {
                context = x.get_context();
                break;
            }
        }

        size_t num_words = static_cast<size_t>(N);
        values.resize(num_words * 32);
        for (size_t i = 0; i < num_words; ++i) {
            size_t input_index = num_words - 1 - i;
            for (size_t j = 0; j < 32; ++j) {
                values[i * 32 + j] = input[input_index].at(j);
            }
        }
        length = num_words * 32;
    }

    bit_array(const bit_array& other);
    bit_array(bit_array&& other);

    bit_array& operator=(const bit_array& other);
    bit_array& operator=(bit_array&& other);

    bool_t<Builder>& operator[](const size_t idx);
    bool_t<Builder> operator[](const size_t idx) const;

    explicit operator byte_array<Builder>() const
    {

        std::vector<bool_t<Builder>> rbits(values.rbegin(), values.rend());

        const size_t num_bits = rbits.size();
        const size_t num_bytes = (num_bits / 8) + (num_bits % 8 != 0);

        std::vector<field_t<Builder>> values(num_bytes);

        for (size_t i = 0; i < num_bytes; ++i) {
            size_t end = 8;
            if (i == num_bytes - 1 && (num_bits % 8 != 0)) {
                end = num_bits % 8;
            }
            field_t<Builder> accumulator(0);
            for (size_t j = 0; j < end; ++j) {
                const auto bit = rbits[i * 8 + j];
                const uint256_t scaling_factor = uint256_t(1) << (end - j - 1);
                accumulator += field_t<Builder>(bit) * bb::fr(scaling_factor);
            }
            values[i] = accumulator;
        }
        return byte_array(context, values);
    };

    template <size_t N> operator std::array<uint32<Builder>, N>()
    {
        // ASSERT(N * 32 == length);
        std::array<uint32<Builder>, N> output;
        for (size_t i = 0; i < N; ++i) {
            std::array<bool_t<Builder>, 32> bools;
            size_t end;
            size_t start;
            start = ((N - i) * 32) - 32;
            end = start + 32 > length ? length : start + 32;
            for (size_t j = start; j < end; ++j) {
                bools[j - start] = values[j];
            }
            if (start + 32 > length) {
                for (size_t j = end; j < start + 32; ++j) {
                    bools[j - start] = bool_t<Builder>(context, false);
                }
            }
            output[i] = uint32<Builder>(context, bools);
        }
        return output;
    }

    std::vector<uint32<Builder>> to_uint32_vector();

    template <size_t N> void populate_uint32_array(const size_t starting_index, std::array<uint32<Builder>, N>& output)
    {
        // ASSERT(N * 32 == (length - starting_index));

        size_t num_uint32s = (length / 32) + (length % 32 != 0);
        size_t num_selected_uint32s = N;

        size_t count = 0;
        for (size_t i = (0); i < num_selected_uint32s; ++i) {
            std::array<bool_t<Builder>, 32> bools;
            size_t end;
            size_t start;
            start = ((num_uint32s - i) * 32) - 32;
            end = start + 32 > length ? length : start + 32;
            for (size_t j = start; j < end; ++j) {
                bools[j - start] = values[j - starting_index];
            }
            if (start + 32 > length) {
                for (size_t j = end; j < start + 32; ++j) {
                    bools[j - start] = bool_t<Builder>(context, false);
                }
            }

            output[count] = uint32<Builder>(context, bools);
            ++count;
        }
    }

    std::string get_witness_as_string() const;

    size_t size() const { return length; }

    Builder* get_context() const { return context; }

    void print() const
    {
        size_t num_ulongs = (length / 32) + (length % 32 != 0);
        std::vector<uint32_t> ulong_vector(num_ulongs, 0);
        for (size_t i = 0; i < length; ++i) {
            size_t ulong_index = i / 32;
            uint32_t shift = static_cast<uint32_t>(i - (ulong_index * 32));
            ulong_vector[num_ulongs - 1 - ulong_index] =
                ulong_vector[num_ulongs - 1 - ulong_index] + (static_cast<uint32_t>(values[i].get_value()) << shift);
        }
        printf("[");
        for (size_t i = 0; i < num_ulongs; ++i) {
            printf(" %x", (ulong_vector[i]));
        }
        printf(" ]\n");
    }

    std::vector<bool_t<Builder>> get_bits() const
    {
        const std::vector<bool_t<Builder>> result(values.begin(), values.end());
        return result;
    }

  private:
    Builder* context;
    size_t length;
    std::vector<bool_t<Builder>> values;
};

} // namespace stdlib
} // namespace bb::plonk

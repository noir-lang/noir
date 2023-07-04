#include "../../primitives/plookup/plookup.hpp"
#include "barretenberg/crypto/keccak/keccak.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "keccak.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace proof_system::plonk;

typedef proof_system::UltraCircuitBuilder Composer;
typedef stdlib::byte_array<Composer> byte_array;
typedef stdlib::public_witness_t<Composer> public_witness_t;
typedef stdlib::field_t<Composer> field_ct;
typedef stdlib::witness_t<Composer> witness_ct;
typedef stdlib::uint32<Composer> uint32_ct;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

TEST(stdlib_keccak, keccak_format_input_table)
{
    Composer composer = Composer();

    for (size_t i = 0; i < 25; ++i) {
        uint64_t limb_native = engine.get_random_uint64();
        field_ct limb(witness_ct(&composer, limb_native));
        stdlib::plookup_read<Composer>::read_from_1_to_2_table(plookup::KECCAK_FORMAT_INPUT, limb);
    }

    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_keccak, keccak_format_output_table)
{
    Composer composer = Composer();

    for (size_t i = 0; i < 25; ++i) {
        uint64_t limb_native = engine.get_random_uint64();
        uint256_t extended_native = stdlib::keccak<Composer>::convert_to_sparse(limb_native);
        field_ct limb(witness_ct(&composer, extended_native));
        stdlib::plookup_read<Composer>::read_from_1_to_2_table(plookup::KECCAK_FORMAT_OUTPUT, limb);
    }
    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_keccak, keccak_theta_output_table)
{
    Composer composer = Composer();

    for (size_t i = 0; i < 25; ++i) {
        uint256_t extended_native = 0;
        for (size_t j = 0; j < 8; ++j) {
            extended_native *= 11;
            uint64_t base_value = (engine.get_random_uint64() % 11);
            extended_native += base_value;
        }
        field_ct limb(witness_ct(&composer, extended_native));
        stdlib::plookup_read<Composer>::read_from_1_to_2_table(plookup::KECCAK_THETA_OUTPUT, limb);
    }
    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_keccak, keccak_rho_output_table)
{
    Composer composer = Composer();

    barretenberg::constexpr_for<0, 25, 1>([&]<size_t i> {
        uint256_t extended_native = 0;
        uint256_t binary_native = 0;
        for (size_t j = 0; j < 64; ++j) {
            extended_native *= 11;
            binary_native = binary_native << 1;
            uint64_t base_value = (engine.get_random_uint64() % 3);
            extended_native += base_value;
            binary_native += (base_value & 1);
        }
        const size_t left_bits = stdlib::keccak<Composer>::ROTATIONS[i];
        const size_t right_bits = 64 - left_bits;
        const uint256_t left = binary_native >> right_bits;
        const uint256_t right = binary_native - (left << right_bits);
        const uint256_t binary_rotated = left + (right << left_bits);

        const uint256_t expected_limb = stdlib::keccak<Composer>::convert_to_sparse(binary_rotated);
        // msb only is correct iff rotation == 0 (no need to get msb for rotated lookups)
        const uint256_t expected_msb = (binary_native >> 63);
        field_ct limb(witness_ct(&composer, extended_native));
        field_ct result_msb;
        field_ct result_limb = stdlib::keccak<Composer>::normalize_and_rotate<i>(limb, result_msb);
        EXPECT_EQ(static_cast<uint256_t>(result_limb.get_value()), expected_limb);
        EXPECT_EQ(static_cast<uint256_t>(result_msb.get_value()), expected_msb);
    });

    info("composer gates = ", composer.get_num_gates());
    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_keccak, keccak_chi_output_table)
{
    static constexpr uint64_t chi_normalization_table[5]{
        0, // 1 + 2a - b + c => a xor (~b & c)
        0, 1, 1, 0,
    };
    Composer composer = Composer();

    for (size_t i = 0; i < 25; ++i) {
        uint256_t normalized_native = 0;
        uint256_t extended_native = 0;
        uint256_t binary_native = 0;
        for (size_t j = 0; j < 8; ++j) {
            extended_native *= 11;
            normalized_native *= 11;
            binary_native = binary_native << 1;
            uint64_t base_value = (engine.get_random_uint64() % 5);
            extended_native += base_value;
            normalized_native += chi_normalization_table[base_value];
            binary_native += chi_normalization_table[base_value];
        }
        field_ct limb(witness_ct(&composer, extended_native));
        const auto accumulators =
            stdlib::plookup_read<Composer>::get_lookup_accumulators(plookup::KECCAK_CHI_OUTPUT, limb);

        field_ct normalized = accumulators[plookup::ColumnIdx::C2][0];
        field_ct msb = accumulators[plookup::ColumnIdx::C3][accumulators[plookup::ColumnIdx::C3].size() - 1];

        EXPECT_EQ(static_cast<uint256_t>(normalized.get_value()), normalized_native);
        EXPECT_EQ(static_cast<uint256_t>(msb.get_value()), binary_native >> 63);
    }
    info("composer gates = n", composer.get_num_gates());
    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_keccak, test_format_input_lanes)
{
    Composer composer = Composer();

    for (size_t i = 543; i < 544; ++i) {
        std::cout << "i = " << i << std::endl;
        std::string input;
        for (size_t j = 0; j < i; ++j) {
            input += "a";
        }

        // std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
        std::vector<uint8_t> input_v(input.begin(), input.end());
        const size_t excess_zeroes = i % 543;
        std::vector<uint8_t> input_padded_v(input.begin(), input.end());
        for (size_t k = 0; k < excess_zeroes; ++k) {
            input_padded_v.push_back(0);
        }
        byte_array input_arr(&composer, input_v);
        byte_array input_padded_arr(&composer, input_padded_v);

        auto num_bytes_native = static_cast<uint32_t>(i);
        uint32_ct num_bytes(witness_ct(&composer, num_bytes_native));
        std::vector<field_ct> result = stdlib::keccak<Composer>::format_input_lanes(input_padded_arr, num_bytes);
        std::vector<field_ct> expected = stdlib::keccak<Composer>::format_input_lanes(input_arr, num_bytes_native);

        EXPECT_GT(result.size(), expected.size() - 1);

        for (size_t j = 0; j < expected.size(); ++j) {
            EXPECT_EQ(result[j].get_value(), expected[j].get_value());
        }
        for (size_t j = expected.size(); j < result.size(); ++j) {
            EXPECT_EQ(result[j].get_value(), 0);
        }
    }

    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_keccak, test_single_block)
{
    Composer composer = Composer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array input_arr(&composer, input_v);
    byte_array output = stdlib::keccak<Composer>::hash(input_arr);

    std::vector<uint8_t> expected = stdlib::keccak<Composer>::hash_native(input_v);

    EXPECT_EQ(output.get_value(), expected);

    composer.print_num_gates();

    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_keccak, test_double_block)
{
    Composer composer = Composer();
    std::string input = "";
    for (size_t i = 0; i < 200; ++i) {
        input += "a";
    }
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array input_arr(&composer, input_v);
    byte_array output = stdlib::keccak<Composer>::hash(input_arr);

    std::vector<uint8_t> expected = stdlib::keccak<Composer>::hash_native(input_v);

    EXPECT_EQ(output.get_value(), expected);

    composer.print_num_gates();

    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_keccak, test_double_block_variable_length)
{
    Composer composer = Composer();
    std::string input = "";
    for (size_t i = 0; i < 200; ++i) {
        input += "a";
    }
    std::vector<uint8_t> input_v(input.begin(), input.end());

    // add zero padding
    std::vector<uint8_t> input_v_padded(input_v);
    for (size_t i = 0; i < 51; ++i) {
        input_v_padded.push_back(0);
    }
    byte_array input_arr(&composer, input_v_padded);

    uint32_ct length(witness_ct(&composer, 200));
    byte_array output = stdlib::keccak<Composer>::hash(input_arr, length);

    std::vector<uint8_t> expected = stdlib::keccak<Composer>::hash_native(input_v);

    EXPECT_EQ(output.get_value(), expected);

    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

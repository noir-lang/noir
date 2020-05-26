// #include <crypto/pedersen/pedersen.hpp>
// #include <gtest/gtest.h>
// #include <numeric/bitop/get_msb.hpp>

// #include "./pedersen.hpp"

// using namespace barretenberg;

// namespace {
// auto& engine = numeric::random::get_debug_engine();
// }

// TEST(pedersen, validate_lookup_table_single)
// {
//     const size_t generator_index = 0;
//     fq scalar_multiplier = fq(engine.get_random_uint256() >> 128).from_montgomery_form();
//     const size_t num_entries = (128 / 13) + (128 % 13 != 0);
//     constexpr size_t num_wnaf_bits = 13;
//     uint64_t wnaf_entries[num_entries] = { 0 };
//     bool skew = false;
//     barretenberg::wnaf::fixed_wnaf<128, 1, num_wnaf_bits>(&scalar_multiplier.data[0], &wnaf_entries[0], skew, 0);

//     grumpkin::g1::element generator(crypto::pedersen::get_generator(0));
//     for (size_t i = num_entries - 1; i > 0; --i) {
//         grumpkin::g1::element target(waffle::pedersen_tables::get_generator_value(generator_index, i,
//         wnaf_entries[i])); const uint64_t wnaf_entry = wnaf_entries[i]; const uint64_t negative = (wnaf_entry >> 31);
//         const int value = ((static_cast<int>(wnaf_entry & 0xffffff)) * 2 + 1) * (1 - 2 * static_cast<int>(negative));
//         fq to_mul(value);
//         grumpkin::g1::element expected(generator * to_mul);
//         const grumpkin::g1::affine_element result(
//             waffle::pedersen_tables::get_generator_value(generator_index, i, wnaf_entries[i]));

//         EXPECT_EQ(result, expected);
//         for (size_t j = 0; j < waffle::pedersen_tables::BITS_PER_LOOKUP; ++j) {
//             generator = generator.dbl();
//         }
//     }
// }

// TEST(pedersen, validate_lookup_table_skew)
// {
//     const size_t generator_index = 0;
//     fq scalar_multiplier = fq(engine.get_random_uint256() >> 128).from_montgomery_form();
//     const size_t num_entries = (128 / 13) + (128 % 13 != 0);
//     constexpr size_t num_wnaf_bits = 13;
//     uint64_t wnaf_entries[num_entries] = { 0 };
//     bool skew = false;
//     barretenberg::wnaf::fixed_wnaf<128, 1, num_wnaf_bits>(&scalar_multiplier.data[0], &wnaf_entries[0], skew, 0);
//     const uint64_t wnaf_entry = wnaf_entries[0];

//     const int value = ((static_cast<int>(wnaf_entry & 0xffffff)) * 2 + 1);
//     fq to_mul(value);

//     grumpkin::g1::element generator(crypto::pedersen::get_generator(0));
//     for (size_t i = 0; i < num_entries - 1; ++i) {
//         for (size_t j = 0; j < 13; ++j) {
//             generator = generator.dbl();
//         }
//     }
//     grumpkin::g1::element temp(generator * to_mul);
//     if (skew) {
//         temp -= crypto::pedersen::get_generator(0);
//     }
//     const grumpkin::g1::affine_element expected(temp);
//     const grumpkin::g1::affine_element result(
//         waffle::pedersen_tables::get_skew_generator_value(generator_index, wnaf_entries[0], skew));
//     EXPECT_EQ(result, expected);
// }

// TEST(pedersen, validate_lookup_table)
// {
//     const size_t generator_index = 0;
//     fq scalar_multiplier = fq(engine.get_random_uint256() >> 128).from_montgomery_form();
//     const size_t num_entries = (128 / 13) + (128 % 13 != 0);
//     constexpr size_t num_wnaf_bits = 13;
//     uint64_t wnaf_entries[num_entries] = { 0 };
//     bool skew = false;
//     barretenberg::wnaf::fixed_wnaf<128, 1, num_wnaf_bits>(&scalar_multiplier.data[0], &wnaf_entries[0], skew, 0);

//     grumpkin::g1::element accumulator(
//         waffle::pedersen_tables::get_skew_generator_value(generator_index, wnaf_entries[0], skew));
//     for (size_t i = 1; i < num_entries; ++i) {
//         grumpkin::g1::element to_add(waffle::pedersen_tables::get_generator_value(generator_index, i,
//         wnaf_entries[i])); accumulator = accumulator + to_add;
//     }

//     grumpkin::g1::element expected(crypto::pedersen::get_generator(0) * scalar_multiplier.to_montgomery_form());
//     EXPECT_EQ(grumpkin::g1::affine_element(accumulator), grumpkin::g1::affine_element(expected));
// }

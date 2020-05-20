#include "plookup_composer.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <gtest/gtest.h>
#include <numeric/bitop/get_msb.hpp>
#include "../proof_system/widgets/transition_widgets/create_dummy_transcript.hpp"
#include "../proof_system/widgets/random_widgets/plookup_widget.hpp"

#include "./plookup_tables/sha256.hpp"

using namespace barretenberg;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

std::array<barretenberg::fr, 2> get_values_from_key(const std::array<uint64_t, 2> key)
{
    return { fr(key[0] ^ key[1]), fr(0) };
}
waffle::PLookupBasicTable generate_xor_table()
{
    waffle::PLookupBasicTable table;
    table.id = waffle::PLookupBasicTableId::XOR;
    table.table_index = 1;
    const size_t num_bits = numeric::get_msb(static_cast<uint64_t>(256));
    const size_t num_entries = 1UL << (num_bits / 2);

    table.size = 256;
    table.use_twin_keys = true;

    for (size_t i = 0; i < num_entries; ++i) {
        for (size_t j = 0; j < num_entries; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint64_t output = left ^ right;
            table.column_1.emplace_back(fr(right));
            table.column_2.emplace_back(fr(left));
            table.column_3.emplace_back(fr(output));
        }
    }
    table.get_values_from_key = &get_values_from_key;

    table.column_1_step_size = fr(16);
    table.column_2_step_size = fr(16);
    table.column_3_step_size = fr(16);
    return table;
}

template <uint64_t base, uint64_t num_rotated_bits>
std::array<barretenberg::fr, 2> get_sparse_map_values(const std::array<uint64_t, 2> key)
{
    const auto t0 = numeric::map_into_sparse_form<base>(key[0]);
    const auto t1 = numeric::map_into_sparse_form<base>(numeric::rotate32((uint32_t)key[0], num_rotated_bits));
    return { barretenberg::fr(t0), barretenberg::fr(t1) };
}

waffle::PLookupBasicTable generate_sparse_map()
{
    constexpr uint64_t base = 28;
    constexpr uint64_t num_rotated_bits = 6;
    constexpr uint64_t bits_per_slice = 11;
    waffle::PLookupBasicTable table;
    table.id = waffle::PLookupBasicTableId::SHA256_BASE28_ROTATE6;
    table.table_index = 1;
    table.size = (1U << bits_per_slice);
    table.use_twin_keys = false;

    for (uint64_t i = 0; i < table.size; ++i) {
        const uint64_t source = i;
        const auto target = numeric::map_into_sparse_form<base>(source);
        const auto rotated = numeric::map_into_sparse_form<base>(numeric::rotate32((uint32_t)source, num_rotated_bits));
        table.column_1.emplace_back(barretenberg::fr(source));
        table.column_2.emplace_back(barretenberg::fr(target));
        table.column_3.emplace_back(barretenberg::fr(rotated));
    }

    table.get_values_from_key = &get_sparse_map_values<base, num_rotated_bits>;

    uint256_t sparse_step_size = 1;
    for (size_t i = 0; i < bits_per_slice; ++i) {
        sparse_step_size *= base;
    }
    table.column_1_step_size = barretenberg::fr((1 << 11));
    table.column_2_step_size = barretenberg::fr(sparse_step_size);
    table.column_3_step_size = barretenberg::fr(0);

    return table;
}

TEST(plookup_composer, read_from_table_with_single_key)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    composer.lookup_tables.emplace_back((generate_sparse_map()));

    constexpr uint64_t bit_mask = (1 << 11) - 1;
    for (size_t i = 0; i < 16; ++i) {
        for (size_t j = 0; j < 16; ++j) {
            uint64_t left = static_cast<uint64_t>(engine.get_random_uint32()) & bit_mask;
            uint32_t left_idx = composer.add_variable(fr(left));

            const std::array<uint32_t, 2> result_indices =
                composer.read_from_table(waffle::PLookupBasicTableId::SHA256_BASE28_ROTATE6, left_idx);

            const auto expected_a = numeric::map_into_sparse_form<28>(left);
            const auto expected_b = numeric::map_into_sparse_form<28>(numeric::rotate32((uint32_t)left, 6));

            EXPECT_EQ(composer.get_variable(result_indices[0]), fr(expected_a));
            EXPECT_EQ(composer.get_variable(result_indices[1]), fr(expected_b));
        }
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();
    auto proof = prover.construct_proof();
    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(plookup_composer, read_sequence_with_single_key)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    composer.lookup_tables.emplace_back((generate_sparse_map()));

    constexpr uint64_t base = 28;

    for (size_t i = 0; i < 1; ++i) {
        uint64_t left = static_cast<uint64_t>(engine.get_random_uint32());
        uint32_t left_idx = composer.add_variable(fr(left));

        const uint64_t bit_mask = (1 << 11) - 1;

        uint64_t slices[3]{
            left & bit_mask,
            (left >> 11) & bit_mask,
            (left >> 22) & bit_mask,
        };

        uint64_t expected_accumulators[3]{
            (slices[2] << 22) + (slices[1] << 11) + slices[0],
            (slices[2] << 11) + slices[1],
            slices[2],
        };

        const auto indices = composer.read_sequence_from_table(
            waffle::PLookupBasicTableId::SHA256_BASE28_ROTATE6, left_idx, UINT32_MAX, 3);

        const uint256_t expected_sparse = numeric::map_into_sparse_form<28>(left);

        const fr expected_rotated[3]{
            numeric::map_into_sparse_form<base>(numeric::rotate32((uint32_t)slices[0], 6)),
            numeric::map_into_sparse_form<base>(numeric::rotate32((uint32_t)slices[1], 6)),
            numeric::map_into_sparse_form<base>(numeric::rotate32((uint32_t)slices[2], 6)),
        };

        const fr expected_sparse_accumulators[3]{
            numeric::map_into_sparse_form<base>(expected_accumulators[0]),
            numeric::map_into_sparse_form<base>(expected_accumulators[1]),
            numeric::map_into_sparse_form<base>(expected_accumulators[2]),
        };

        EXPECT_EQ(composer.get_variable(indices[0][0]), left);
        EXPECT_EQ(composer.get_variable(indices[1][0]), expected_sparse);
        EXPECT_EQ(composer.get_variable(indices[0][0]), fr(expected_accumulators[0]));
        EXPECT_EQ(composer.get_variable(indices[0][1]), fr(expected_accumulators[1]));
        EXPECT_EQ(composer.get_variable(indices[0][2]), fr(expected_accumulators[2]));

        EXPECT_EQ(composer.get_variable(indices[1][0]), expected_sparse_accumulators[0]);
        EXPECT_EQ(composer.get_variable(indices[1][1]), expected_sparse_accumulators[1]);
        EXPECT_EQ(composer.get_variable(indices[1][2]), expected_sparse_accumulators[2]);

        EXPECT_EQ(composer.get_variable(indices[2][0]), expected_rotated[0]);
        EXPECT_EQ(composer.get_variable(indices[2][1]), expected_rotated[1]);
        EXPECT_EQ(composer.get_variable(indices[2][2]), expected_rotated[2]);
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(plookup_composer, read_from_table_with_key_pair)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    composer.lookup_tables.emplace_back((generate_xor_table()));

    for (size_t i = 0; i < 16; ++i) {
        for (size_t j = 0; j < 16; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint32_t left_idx = composer.add_variable(fr(left));
            uint32_t right_idx = composer.add_variable(fr(right));

            uint32_t result_idx = composer.read_from_table(waffle::PLookupBasicTableId::XOR, left_idx, right_idx);

            EXPECT_EQ(composer.get_variable(result_idx), fr(left ^ right));
        }
    }
}

TEST(plookup_composer, read_sequence_from_table)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
    composer.lookup_tables.emplace_back((generate_xor_table()));

    for (size_t i = 0; i < 16; i += 2) {
        for (size_t j = 0; j < 16; j += 2) {
            uint64_t left[4]{
                j,
                (j + 1) % 16,
                j,
                (j + 1) % 16,
            };
            uint64_t right[4]{
                i,
                i,
                (i + 1) % 16,
                (i + 1) % 16,
            };
            uint64_t xors[4]{
                left[0] ^ right[0],
                left[1] ^ right[1],
                left[2] ^ right[2],
                left[3] ^ right[3],
            };
            uint64_t left_accumulators[4]{
                left[0] + left[1] * 16 + left[2] * 256 + left[3] * 4096,
                left[1] + left[2] * 16 + left[3] * 256,
                left[2] + left[3] * 16,
                left[3],
            };
            uint64_t right_accumulators[4]{
                right[0] + right[1] * 16 + right[2] * 256 + right[3] * 4096,
                right[1] + right[2] * 16 + right[3] * 256,
                right[2] + right[3] * 16,
                right[3],
            };

            uint64_t xor_accumulators[4]{
                xors[0] + xors[1] * 16 + xors[2] * 256 + xors[3] * 4096,
                xors[1] + xors[2] * 16 + xors[3] * 256,
                xors[2] + xors[3] * 16,
                xors[3],
            };

            uint32_t left_indices[4]{
                composer.add_variable(fr(left_accumulators[0])),
                composer.add_variable(fr(left_accumulators[1])),
                composer.add_variable(fr(left_accumulators[2])),
                composer.add_variable(fr(left_accumulators[3])),
            };
            uint32_t right_indices[4]{
                composer.add_variable(fr(right_accumulators[0])),
                composer.add_variable(fr(right_accumulators[1])),
                composer.add_variable(fr(right_accumulators[2])),
                composer.add_variable(fr(right_accumulators[3])),
            };

            auto xor_indices = composer.read_sequence_from_table(waffle::PLookupBasicTableId::XOR,
                                                                 {
                                                                     { left_indices[0], right_indices[0] },
                                                                     { left_indices[1], right_indices[1] },
                                                                     { left_indices[2], right_indices[2] },
                                                                     { left_indices[3], right_indices[3] },
                                                                 });

            for (size_t i = 0; i < xor_indices.size(); ++i) {
                EXPECT_EQ(composer.get_variable(xor_indices[i]), xor_accumulators[i]);
            }
        }
    }
}

TEST(plookup_composer, read_alternate_sequence_from_table)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
    composer.lookup_tables.emplace_back((generate_xor_table()));

    for (size_t i = 0; i < 16; i += 2) {
        for (size_t j = 0; j < 16; j += 2) {
            uint64_t left[4]{
                j % 16,
                (j + 1) % 16,
                j % 16,
                (j + 1) % 16,
            };
            uint64_t right[4]{
                i % 16,
                i % 16,
                (i + 1) % 16,
                (i + 1) % 16,
            };
            uint64_t xors[4]{
                left[0] ^ right[0],
                left[1] ^ right[1],
                left[2] ^ right[2],
                left[3] ^ right[3],
            };
            uint64_t left_accumulators[4]{
                left[0] + left[1] * 16 + left[2] * 256 + left[3] * 4096,
                left[1] + left[2] * 16 + left[3] * 256,
                left[2] + left[3] * 16,
                left[3],
            };
            uint64_t right_accumulators[4]{
                right[0] + right[1] * 16 + right[2] * 256 + right[3] * 4096,
                right[1] + right[2] * 16 + right[3] * 256,
                right[2] + right[3] * 16,
                right[3],
            };

            uint64_t xor_accumulators[4]{
                xors[0] + xors[1] * 16 + xors[2] * 256 + xors[3] * 4096,
                xors[1] + xors[2] * 16 + xors[3] * 256,
                xors[2] + xors[3] * 16,
                xors[3],
            };

            // uint32_t left_indices[4]{
            //     composer.add_variable(fr(left_accumulators[0])),
            //     composer.add_variable(fr(left_accumulators[1])),
            //     composer.add_variable(fr(left_accumulators[2])),
            //     composer.add_variable(fr(left_accumulators[3])),
            // };
            // uint32_t right_indices[4]{
            //     composer.add_variable(fr(right_accumulators[0])),
            //     composer.add_variable(fr(right_accumulators[1])),
            //     composer.add_variable(fr(right_accumulators[2])),
            //     composer.add_variable(fr(right_accumulators[3])),
            // };

            uint32_t left_index = composer.add_variable(fr(left_accumulators[0]));
            uint32_t right_index = composer.add_variable(fr(right_accumulators[0]));

            auto xor_indices =
                composer.read_sequence_from_table(waffle::PLookupBasicTableId::XOR, left_index, right_index, 4);
            //  {
            //      { left_indices[0], right_indices[0] },
            //      { left_indices[1], right_indices[1] },
            //      { left_indices[2], right_indices[2] },
            //      { left_indices[3], right_indices[3] },
            //  });

            for (size_t i = 0; i < xor_indices[0].size(); ++i) {
                EXPECT_EQ(composer.get_variable(xor_indices[0][i]), left_accumulators[i]);
                EXPECT_EQ(composer.get_variable(xor_indices[1][i]), right_accumulators[i]);
                EXPECT_EQ(composer.get_variable(xor_indices[2][i]), xor_accumulators[i]);
            }
        }
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(plookup_composer, test_quotient_polynomial_absolute_lookup)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
    composer.lookup_tables.emplace_back((generate_xor_table()));

    for (size_t i = 0; i < 16; ++i) {
        for (size_t j = 0; j < 16; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint32_t left_idx = composer.add_variable(fr(left));
            uint32_t right_idx = composer.add_variable(fr(right));

            uint32_t result_idx = composer.read_from_table(waffle::PLookupBasicTableId::XOR, left_idx, right_idx);

            uint32_t add_idx = composer.add_variable(fr(left) + fr(right) + composer.get_variable(result_idx));
            composer.create_big_add_gate(
                { left_idx, right_idx, result_idx, add_idx, fr(1), fr(1), fr(1), fr(-1), fr(0) });
        }
    }

    auto key = composer.compute_proving_key();

    auto witness = composer.compute_witness();

    const auto adjust_ffts = [&key](const std::string& tag, bool origin) {
        auto& selector = origin ? key->permutation_selector_ffts.at(tag) : key->constraint_selector_ffts.at(tag);
        selector.coset_ifft(key->large_domain);
        selector.fft(key->large_domain);
    };

    adjust_ffts("table_value_1_fft", true);
    adjust_ffts("table_value_2_fft", true);
    adjust_ffts("table_value_3_fft", true);
    adjust_ffts("table_value_4_fft", true);
    adjust_ffts("table_type_fft", true);
    adjust_ffts("table_index_fft", true);
    adjust_ffts("q_2_fft", false);
    adjust_ffts("q_m_fft", false);
    adjust_ffts("q_c_fft", false);

    auto transcript = waffle::create_dummy_ultra_transcript();

    waffle::ProverPLookupWidget widget(key.get(), witness.get());
    {
        const size_t n = key->small_domain.size;
        key->wire_ffts.at("w_1_fft") = polynomial(witness->wires.at("w_1"), 4 * n + 4);
        key->wire_ffts.at("w_2_fft") = polynomial(witness->wires.at("w_2"), 4 * n + 4);
        key->wire_ffts.at("w_3_fft") = polynomial(witness->wires.at("w_3"), 4 * n + 4);
    }

    widget.compute_sorted_list_commitment(transcript);

    widget.compute_grand_product_commitment(transcript);

    {
        const size_t n = key->small_domain.size;
        auto& z = witness->wires.at("z_lookup");
        z.fft(key->small_domain);

        fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
        fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());
        const fr gamma_beta_constant = gamma * (fr(1) + beta);
        const fr expected = gamma_beta_constant.pow(n - 1);
        EXPECT_EQ(z[key->small_domain.size - 1], expected);
        z.ifft(key->small_domain);
    }

    {

        const auto adjust_witness_fft = [&key, &witness](const std::string& tag, bool ifft) {
            const size_t n = key->small_domain.size;
            auto& poly = witness->wires.at(tag);
            if (ifft) {
                poly.ifft(key->small_domain);
            }
            auto& poly_fft = key->wire_ffts.at(tag + "_fft");
            for (size_t i = 0; i < n; ++i) {
                poly_fft[i] = poly[i];
            }
            for (size_t i = n; i < 4 * n; ++i) {
                poly_fft[i] = 0;
            }
            poly_fft.fft(key->large_domain);
        };

        adjust_witness_fft("w_1", true);
        adjust_witness_fft("w_2", true);
        adjust_witness_fft("w_3", true);
        adjust_witness_fft("s", false);
        adjust_witness_fft("z_lookup", false);

        auto& quotient_poly = key->quotient_large;
        for (size_t i = 0; i < key->large_domain.size; ++i) {
            quotient_poly[i] = fr(0);
        }

        key->lagrange_1.coset_ifft(key->large_domain);
        key->lagrange_1.fft(key->large_domain);
        for (size_t i = 0; i < 8; ++i) {
            key->lagrange_1[key->large_domain.size + i] = key->lagrange_1[i];
        }
    }

    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    widget.compute_quotient_contribution(alpha, transcript);

    auto& quotient_poly = key->quotient_large;

    for (size_t i = 0; i < key->small_domain.size - 1; ++i) {
        EXPECT_EQ(quotient_poly[i * 4], fr(0));
    }
}

TEST(plookup_composer, test_quotient_polynomial_relative_lookup)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
    composer.lookup_tables.emplace_back((generate_xor_table()));

    for (size_t i = 0; i < 16; i += 2) {
        for (size_t j = 0; j < 16; j += 2) {
            uint64_t left[4]{ j, (j + 1) % 16, j, (j + 1) % 16 };
            uint64_t right[4]{ i, i, (i + 1) % 16, (i + 1) % 16 };

            uint64_t left_accumulators[4]{ left[0] + left[1] * 16 + left[2] * 256 + left[3] * 4096,
                                           left[1] + left[2] * 16 + left[3] * 256,
                                           left[2] + left[3] * 16,
                                           left[3] };
            uint64_t right_accumulators[4]{ right[0] + right[1] * 16 + right[2] * 256 + right[3] * 4096,
                                            right[1] + right[2] * 16 + right[3] * 256,
                                            right[2] + right[3] * 16,
                                            right[3] };

            uint32_t left_indices[4]{ composer.add_variable(fr(left_accumulators[0])),
                                      composer.add_variable(fr(left_accumulators[1])),
                                      composer.add_variable(fr(left_accumulators[2])),
                                      composer.add_variable(fr(left_accumulators[3])) };
            uint32_t right_indices[4]{ composer.add_variable(fr(right_accumulators[0])),
                                       composer.add_variable(fr(right_accumulators[1])),
                                       composer.add_variable(fr(right_accumulators[2])),
                                       composer.add_variable(fr(right_accumulators[3])) };

            auto result_indices = composer.read_sequence_from_table(waffle::PLookupBasicTableId::XOR,
                                                                    { { left_indices[0], right_indices[0] },
                                                                      { left_indices[1], right_indices[1] },
                                                                      { left_indices[2], right_indices[2] },
                                                                      { left_indices[3], right_indices[3] } });

            uint32_t add_idx = composer.add_variable(fr(left_accumulators[0]) + fr(right_accumulators[0]) +
                                                     composer.get_variable(result_indices[0]));

            composer.create_big_add_gate(
                { left_indices[0], right_indices[0], result_indices[0], add_idx, fr(1), fr(1), fr(1), fr(-1), fr(0) });
        }
    }

    auto key = composer.compute_proving_key();

    auto witness = composer.compute_witness();

    const auto adjust_ffts = [&key](const std::string& tag, bool origin) {
        auto& selector = origin ? key->permutation_selector_ffts.at(tag) : key->constraint_selector_ffts.at(tag);
        selector.coset_ifft(key->large_domain);
        selector.fft(key->large_domain);
        selector[key->large_domain.size] = selector[0];
        selector[key->large_domain.size + 1] = selector[1];
        selector[key->large_domain.size + 2] = selector[2];
        selector[key->large_domain.size + 3] = selector[3];
    };

    adjust_ffts("table_value_1_fft", true);
    adjust_ffts("table_value_2_fft", true);
    adjust_ffts("table_value_3_fft", true);
    adjust_ffts("table_value_4_fft", true);
    adjust_ffts("table_type_fft", true);
    adjust_ffts("table_index_fft", true);
    adjust_ffts("q_2_fft", false);
    adjust_ffts("q_m_fft", false);
    adjust_ffts("q_c_fft", false);

    auto transcript = waffle::create_dummy_ultra_transcript();

    waffle::ProverPLookupWidget widget(key.get(), witness.get());
    {
        const size_t n = key->small_domain.size;
        key->wire_ffts.at("w_1_fft") = polynomial(witness->wires.at("w_1"), 4 * n + 4);
        key->wire_ffts.at("w_2_fft") = polynomial(witness->wires.at("w_2"), 4 * n + 4);
        key->wire_ffts.at("w_3_fft") = polynomial(witness->wires.at("w_3"), 4 * n + 4);
    }

    widget.compute_sorted_list_commitment(transcript);

    widget.compute_grand_product_commitment(transcript);

    {
        const size_t n = key->small_domain.size;
        auto& z = witness->wires.at("z_lookup");
        z.fft(key->small_domain);

        fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
        fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());
        const fr gamma_beta_constant = gamma * (fr(1) + beta);
        const fr expected = gamma_beta_constant.pow(n - 1);
        EXPECT_EQ(z[key->small_domain.size - 1], expected);
        z.ifft(key->small_domain);
    }

    {

        const auto adjust_witness_fft = [&key, &witness](const std::string& tag, bool ifft) {
            const size_t n = key->small_domain.size;
            auto& poly = witness->wires.at(tag);
            if (ifft) {
                poly.ifft(key->small_domain);
            }
            auto& poly_fft = key->wire_ffts.at(tag + "_fft");
            for (size_t i = 0; i < n; ++i) {
                poly_fft[i] = poly[i];
            }
            for (size_t i = n; i < 4 * n; ++i) {
                poly_fft[i] = 0;
            }
            poly_fft.fft(key->large_domain);
            poly_fft.add_lagrange_base_coefficient(poly_fft[0]);
            poly_fft.add_lagrange_base_coefficient(poly_fft[0]);
            poly_fft.add_lagrange_base_coefficient(poly_fft[0]);
            poly_fft.add_lagrange_base_coefficient(poly_fft[0]);
        };

        adjust_witness_fft("w_1", true);
        adjust_witness_fft("w_2", true);
        adjust_witness_fft("w_3", true);
        adjust_witness_fft("s", false);
        adjust_witness_fft("z_lookup", false);

        auto& quotient_poly = key->quotient_large;
        for (size_t i = 0; i < key->large_domain.size; ++i) {
            quotient_poly[i] = fr(0);
        }

        key->lagrange_1.coset_ifft(key->large_domain);
        key->lagrange_1.fft(key->large_domain);
        for (size_t i = 0; i < 8; ++i) {
            key->lagrange_1[key->large_domain.size + i] = key->lagrange_1[i];
        }
    }

    fr alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    widget.compute_quotient_contribution(alpha, transcript);

    auto& quotient_poly = key->quotient_large;

    for (size_t i = 0; i < key->small_domain.size - 1; ++i) {
        EXPECT_EQ(quotient_poly[i * 4], fr(0));
    }
}

TEST(plookup_composer, test_relative_lookup_proof)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();
    composer.lookup_tables.emplace_back((generate_xor_table()));

    for (size_t i = 0; i < 16; ++i) {
        for (size_t j = 0; j < 16; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint32_t left_idx = composer.add_variable(fr(left));
            uint32_t right_idx = composer.add_variable(fr(right));

            uint32_t result_idx = composer.read_from_table(waffle::PLookupBasicTableId::XOR, left_idx, right_idx);

            uint32_t add_idx = composer.add_variable(fr(left) + fr(right) + composer.get_variable(result_idx));
            composer.create_big_add_gate(
                { left_idx, right_idx, result_idx, add_idx, fr(1), fr(1), fr(1), fr(-1), fr(0) });
        }
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(plookup_composer, test_no_lookup_proof)
{
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    for (size_t i = 0; i < 16; ++i) {
        for (size_t j = 0; j < 16; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint32_t left_idx = composer.add_variable(fr(left));
            uint32_t right_idx = composer.add_variable(fr(right));
            uint32_t result_idx = composer.add_variable(fr(left ^ right));

            uint32_t add_idx = composer.add_variable(fr(left) + fr(right) + composer.get_variable(result_idx));
            composer.create_big_add_gate(
                { left_idx, right_idx, result_idx, add_idx, fr(1), fr(1), fr(1), fr(-1), fr(0) });
        }
    }

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(plookup_composer, test_elliptic_gate)
{
    typedef grumpkin::g1::affine_element affine_element;
    typedef grumpkin::g1::element element;
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    const affine_element p1 = crypto::pedersen::get_generator(0);
    const affine_element p2 = crypto::pedersen::get_generator(1);
    const affine_element p3(element(p1).dbl());
    const affine_element p4(element(p2) + element(p3));

    const uint32_t x1 = composer.add_variable(p1.x);
    const uint32_t y1 = composer.add_variable(p1.y);
    const uint32_t x2 = composer.add_variable(p2.x);
    const uint32_t y2 = composer.add_variable(p2.y);
    const uint32_t x3 = composer.add_variable(p3.x);
    const uint32_t y3 = composer.add_variable(p3.y);
    const uint32_t x4 = composer.add_variable(p4.x);
    const uint32_t y4 = composer.add_variable(p4.y);


    const uint32_t accumulator_in = composer.zero_idx;
    const uint32_t accumulator_out = composer.zero_idx;

    const waffle::montgomery_ladder_gate gate{ x1, y1, x2, y2, x3, y3, x4, y4, accumulator_in, accumulator_out };

    composer.create_montgomery_ladder_gate(gate);

    composer.create_dummy_gate();
    composer.create_dummy_gate();

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}
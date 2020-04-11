#include "plookup_composer.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <gtest/gtest.h>
#include <numeric/bitop/get_msb.hpp>
#include "../proof_system/widgets/create_dummy_transcript.hpp"
#include "../proof_system/widgets/plookup_widget.hpp"

using namespace barretenberg;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

void generate_xor_table(const size_t size,
                        std::vector<fr>& column_1,
                        std::vector<fr>& column_2,
                        std::vector<fr>& column_3)
{
    const size_t num_bits = numeric::get_msb(static_cast<uint64_t>(size));
    const size_t num_entries = 1UL << (num_bits / 2);
    for (size_t i = 0; i < num_entries; ++i) {
        for (size_t j = 0; j < num_entries; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint64_t output = left ^ right;
            column_1.emplace_back(fr(left));
            column_2.emplace_back(fr(right));
            column_3.emplace_back(fr(output));
        }
    }
}

TEST(plookup_composer, read_from_table_with_key_pair)
{
    const size_t table_size = 256;
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    composer.initialize_precomputed_table(waffle::LookupTableId::XOR, table_size, &generate_xor_table);

    for (size_t i = 0; i < 16; ++i) {
        for (size_t j = 0; j < 16; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint32_t left_idx = composer.add_variable(fr(left));
            uint32_t right_idx = composer.add_variable(fr(right));

            uint32_t result_idx = composer.read_from_table(waffle::LookupTableId::XOR, { left_idx, right_idx });

            EXPECT_EQ(composer.get_variable(result_idx), fr(left ^ right));
        }
    }
}

TEST(plookup_composer, read_from_table_with_single_key)
{
    const size_t table_size = 256;
    waffle::PLookupComposer composer = waffle::PLookupComposer();

    composer.initialize_precomputed_table(waffle::LookupTableId::XOR, table_size, &generate_xor_table);

    for (size_t j = 0; j < 16; ++j) {
        uint64_t left = static_cast<uint64_t>(j);
        uint64_t right = static_cast<uint64_t>(0);
        uint32_t left_idx = composer.add_variable(fr(left));

        auto result_indices = composer.read_from_table(waffle::LookupTableId::XOR, left_idx);

        EXPECT_EQ(composer.get_variable(result_indices.first), fr(right));
        EXPECT_EQ(composer.get_variable(result_indices.second), fr(left ^ right));
    }
}

TEST(plookup_composer, read_sequence_from_table)
{
    const size_t table_size = 256;
    waffle::PLookupComposer composer = waffle::PLookupComposer();
    composer.plookup_step_size = fr(4);
    composer.initialize_precomputed_table(waffle::LookupTableId::XOR, table_size, &generate_xor_table);

    for (size_t i = 0; i < 16; i += 2) {
        for (size_t j = 0; j < 16; j += 2) {
            uint64_t left[4]{
                j,
                j + 1,
                j,
                j + 1,
            };
            uint64_t right[4]{
                i,
                i,
                i + 1,
                i + 1,
            };
            uint64_t xors[4]{
                left[0] ^ right[0],
                left[1] ^ right[1],
                left[2] ^ right[2],
                left[3] ^ right[3],
            };
            uint64_t left_accumulators[4]{
                left[0] + left[1] * 4 + left[2] * 16 + left[3] * 64,
                left[1] + left[2] * 4 + left[3] * 16,
                left[2] + left[3] * 4,
                left[3],
            };
            uint64_t right_accumulators[4]{
                right[0] + right[1] * 4 + right[2] * 16 + right[3] * 64,
                right[1] + right[2] * 4 + right[3] * 16,
                right[2] + right[3] * 4,
                right[3],
            };

            uint64_t xor_accumulators[4]{
                xors[0] + xors[1] * 4 + xors[2] * 16 + xors[3] * 64,
                xors[1] + xors[2] * 4 + xors[3] * 16,
                xors[2] + xors[3] * 4,
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

            auto xor_indices = composer.read_sequence_from_table(waffle::LookupTableId::XOR,
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

TEST(plookup_composer, test_quotient_polynomial_absolute_lookup)
{
    const size_t table_size = 256;
    waffle::PLookupComposer composer = waffle::PLookupComposer();
    composer.plookup_step_size = fr(4);
    composer.initialize_precomputed_table(waffle::LookupTableId::XOR, table_size, &generate_xor_table);

    for (size_t i = 0; i < 16; ++i) {
        for (size_t j = 0; j < 16; ++j) {
            uint64_t left = static_cast<uint64_t>(j);
            uint64_t right = static_cast<uint64_t>(i);
            uint32_t left_idx = composer.add_variable(fr(left));
            uint32_t right_idx = composer.add_variable(fr(right));

            uint32_t result_idx = composer.read_from_table(waffle::LookupTableId::XOR, { left_idx, right_idx });

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

    auto transcript = waffle::create_dummy_ultra_transcript();

    waffle::ProverPLookupWidget widget(key.get(), witness.get());

    widget.compute_sorted_list_commitment(transcript);

    {
        const size_t n = key->small_domain.size;
        key->wire_ffts.at("w_1_fft") = polynomial(witness->wires.at("w_1"), 4 * n + 4);
        key->wire_ffts.at("w_2_fft") = polynomial(witness->wires.at("w_2"), 4 * n + 4);
        key->wire_ffts.at("w_3_fft") = polynomial(witness->wires.at("w_3"), 4 * n + 4);
    }

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

TEST(plookup_composer, test_quotient_polynomial_relative_lookup)
{
    const size_t table_size = 256;
    waffle::PLookupComposer composer = waffle::PLookupComposer();
    composer.plookup_step_size = fr(4);
    composer.initialize_precomputed_table(waffle::LookupTableId::XOR, table_size, &generate_xor_table);

    for (size_t i = 0; i < 16; i += 2) {
        for (size_t j = 0; j < 16; j += 2) {
            uint64_t left[4]{ j, j + 1, j, j + 1 };
            uint64_t right[4]{ i, i, i + 1, i + 1 };

            uint64_t left_accumulators[4]{ left[0] + left[1] * 4 + left[2] * 16 + left[3] * 64,
                                           left[1] + left[2] * 4 + left[3] * 16,
                                           left[2] + left[3] * 4,
                                           left[3] };
            uint64_t right_accumulators[4]{ right[0] + right[1] * 4 + right[2] * 16 + right[3] * 64,
                                            right[1] + right[2] * 4 + right[3] * 16,
                                            right[2] + right[3] * 4,
                                            right[3] };

            uint32_t left_indices[4]{ composer.add_variable(fr(left_accumulators[0])),
                                      composer.add_variable(fr(left_accumulators[1])),
                                      composer.add_variable(fr(left_accumulators[2])),
                                      composer.add_variable(fr(left_accumulators[3])) };
            uint32_t right_indices[4]{ composer.add_variable(fr(right_accumulators[0])),
                                       composer.add_variable(fr(right_accumulators[1])),
                                       composer.add_variable(fr(right_accumulators[2])),
                                       composer.add_variable(fr(right_accumulators[3])) };

            auto result_indices = composer.read_sequence_from_table(waffle::LookupTableId::XOR,
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

    auto transcript = waffle::create_dummy_ultra_transcript();

    waffle::ProverPLookupWidget widget(key.get(), witness.get());

    widget.compute_sorted_list_commitment(transcript);

    {
        const size_t n = key->small_domain.size;
        key->wire_ffts.at("w_1_fft") = polynomial(witness->wires.at("w_1"), 4 * n + 4);
        key->wire_ffts.at("w_2_fft") = polynomial(witness->wires.at("w_2"), 4 * n + 4);
        key->wire_ffts.at("w_3_fft") = polynomial(witness->wires.at("w_3"), 4 * n + 4);
    }

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
#include "../proving_key/proving_key.hpp"
#include "create_dummy_transcript.hpp"
#include "plookup_widget.hpp"
#include <polynomials/polynomial.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;
using namespace waffle;

void create_dummy_circuit(std::shared_ptr<proving_key>& key, std::shared_ptr<program_witness>& witness)
{
    const size_t n = key->n;
    polynomial w_1(n + 1, n + 1);
    polynomial w_2(n + 1, n + 1);
    polynomial w_3(n + 1, n + 1);
    polynomial s(n + 1, n + 1);
    polynomial s_fft(4 * n + 4, 4 * n + 4);
    polynomial z_lookup(n + 1, n + 1);
    polynomial z_lookup_fft(4 * n + 4, 4 * n + 4);
    polynomial table_value_0(n + 1, n + 1);
    polynomial table_value_1(n + 1, n + 1);
    polynomial table_value_2(n + 1, n + 1);
    polynomial table_value_3(n + 1, n + 1);
    polynomial table_type(n, n);
    polynomial table_index(n, n);

    table_value_0[0] = fr(0);
    table_value_0[1] = fr(0);
    table_value_0[2] = fr(1);
    table_value_0[3] = fr(2);

    table_value_1[0] = fr(0);
    table_value_1[1] = fr(0);
    table_value_1[2] = fr(1);
    table_value_1[3] = fr(1);

    table_value_2[0] = fr(0);
    table_value_2[1] = fr(0);
    table_value_2[2] = fr(1);
    table_value_2[3] = fr(1);

    table_value_3[0] = fr(0);
    table_value_3[1] = fr(0);
    table_value_3[2] = fr(1);
    table_value_3[3] = fr(1);

    table_value_0[4] = fr(0);
    table_value_1[4] = fr(0);
    table_value_2[4] = fr(0);
    table_value_3[4] = fr(0);

    table_type[0] = fr(1);
    table_type[1] = fr(0);
    table_type[2] = fr(0);
    table_type[3] = fr(0);

    table_index[0] = fr(1);
    table_index[1] = fr(0);
    table_index[2] = fr(0);
    table_index[3] = fr(0);

    w_1[0] = fr(2);
    w_1[1] = fr(1);
    w_1[2] = fr(2);
    w_1[3] = fr(0);

    w_2[0] = fr(1);
    w_2[1] = fr(1);
    w_2[2] = fr(2);
    w_2[3] = fr(0);

    w_3[0] = fr(1);
    w_3[1] = fr(1);
    w_3[2] = fr(3);
    w_3[3] = fr(0);

    w_1[4] = fr(0);
    w_2[4] = fr(0);
    w_3[4] = fr(0);

    polynomial& w_1_fft = key->wire_ffts.at("w_1_fft");
    polynomial& w_2_fft = key->wire_ffts.at("w_2_fft");
    polynomial& w_3_fft = key->wire_ffts.at("w_3_fft");
    for (size_t i = 0; i < n + 1; ++i) {
        w_1_fft[i] = w_1[i];
        w_2_fft[i] = w_2[i];
        w_3_fft[i] = w_3[i];
    }

    polynomial table_value_0_fft(table_value_0, 4 * n + 4);
    polynomial table_value_1_fft(table_value_1, 4 * n + 4);
    polynomial table_value_2_fft(table_value_2, 4 * n + 4);
    polynomial table_value_3_fft(table_value_3, 4 * n + 4);
    polynomial table_type_fft(table_type, 4 * n + 4);
    polynomial table_index_fft(table_index, 4 * n + 4);

    table_value_0_fft.ifft(key->small_domain);
    table_value_1_fft.ifft(key->small_domain);
    table_value_2_fft.ifft(key->small_domain);
    table_value_3_fft.ifft(key->small_domain);
    table_type_fft.ifft(key->small_domain);
    table_index_fft.ifft(key->small_domain);

    table_value_0_fft.fft(key->large_domain);
    table_value_1_fft.fft(key->large_domain);
    table_value_2_fft.fft(key->large_domain);
    table_value_3_fft.fft(key->large_domain);
    table_type_fft.fft(key->large_domain);
    table_index_fft.fft(key->large_domain);

    table_value_0_fft.add_lagrange_base_coefficient(table_value_0_fft[0]);
    table_value_0_fft.add_lagrange_base_coefficient(table_value_0_fft[1]);
    table_value_0_fft.add_lagrange_base_coefficient(table_value_0_fft[2]);
    table_value_0_fft.add_lagrange_base_coefficient(table_value_0_fft[3]);

    table_value_1_fft.add_lagrange_base_coefficient(table_value_1_fft[0]);
    table_value_1_fft.add_lagrange_base_coefficient(table_value_1_fft[1]);
    table_value_1_fft.add_lagrange_base_coefficient(table_value_1_fft[2]);
    table_value_1_fft.add_lagrange_base_coefficient(table_value_1_fft[3]);

    table_value_2_fft.add_lagrange_base_coefficient(table_value_2_fft[0]);
    table_value_2_fft.add_lagrange_base_coefficient(table_value_2_fft[1]);
    table_value_2_fft.add_lagrange_base_coefficient(table_value_2_fft[2]);
    table_value_2_fft.add_lagrange_base_coefficient(table_value_2_fft[3]);

    table_value_3_fft.add_lagrange_base_coefficient(table_value_3_fft[0]);
    table_value_3_fft.add_lagrange_base_coefficient(table_value_3_fft[1]);
    table_value_3_fft.add_lagrange_base_coefficient(table_value_3_fft[2]);
    table_value_3_fft.add_lagrange_base_coefficient(table_value_3_fft[3]);

    key->lookup_mapping.resize(4);
    key->lookup_mapping[0] = LookupType::ABSOLUTE_LOOKUP;
    key->lookup_mapping[1] = LookupType::NONE;
    key->lookup_mapping[2] = LookupType::NONE;
    key->lookup_mapping[3] = LookupType::NONE;

    key->table_indices.resize(4);
    key->table_indices[0] = 1;
    key->table_indices[1] = 1;
    key->table_indices[2] = 1;
    key->table_indices[3] = 1;
    key->lookup_table_step_size = 0;
    key->num_lookup_tables = 1;

    witness->wires.insert({ "w_1", std::move(w_1) });
    witness->wires.insert({ "w_2", std::move(w_2) });
    witness->wires.insert({ "w_3", std::move(w_3) });
    witness->wires.insert({ "s", std::move(s) });
    witness->wires.insert({ "z_lookup", std::move(z_lookup) });

    key->wire_ffts.insert({ "z_lookup_fft", (z_lookup_fft) });
    key->wire_ffts.insert({ "s_fft", (s_fft) });
    key->permutation_selectors_lagrange_base.insert({ "table_value_1", std::move(table_value_0) });
    key->permutation_selectors_lagrange_base.insert({ "table_value_2", std::move(table_value_1) });
    key->permutation_selectors_lagrange_base.insert({ "table_value_3", std::move(table_value_2) });
    key->permutation_selectors_lagrange_base.insert({ "table_value_4", std::move(table_value_3) });
    key->permutation_selectors_lagrange_base.insert({ "table_type", std::move(table_type) });
    key->permutation_selectors_lagrange_base.insert({ "table_index", std::move(table_index) });

    key->permutation_selector_ffts.insert({ "table_value_1_fft", std::move(table_value_0_fft) });
    key->permutation_selector_ffts.insert({ "table_value_2_fft", std::move(table_value_1_fft) });
    key->permutation_selector_ffts.insert({ "table_value_3_fft", std::move(table_value_2_fft) });
    key->permutation_selector_ffts.insert({ "table_value_4_fft", std::move(table_value_3_fft) });
    key->permutation_selector_ffts.insert({ "table_type_fft", std::move(table_type_fft) });
    key->permutation_selector_ffts.insert({ "table_index_fft", std::move(table_index_fft) });
}

TEST(plookup_widget, compute_sorted_list)
{
    const size_t n = 4;
    const size_t num_gates = n;
    std::shared_ptr<program_witness> witness = std::make_shared<program_witness>();
    auto crs = std::make_unique<FileReferenceStringFactory>("../srs_db");
    std::shared_ptr<proving_key> key = std::make_shared<proving_key>(num_gates, 0, crs->get_prover_crs(num_gates));

    create_dummy_circuit(key, witness);
    auto transcript = create_dummy_ultra_transcript();

    waffle::ProverPLookupWidget widget(key.get(), witness.get());

    widget.compute_sorted_list_commitment(transcript);

    auto s = witness->wires.at("s");
    s.fft(key->small_domain);
}

TEST(plookup_widget, compute_grand_product_commitment)
{
    const size_t n = 4;
    const size_t num_gates = n;
    std::shared_ptr<program_witness> witness = std::make_shared<program_witness>();
    auto crs = std::make_unique<FileReferenceStringFactory>("../srs_db");
    std::shared_ptr<proving_key> key = std::make_shared<proving_key>(num_gates, 0, crs->get_prover_crs(num_gates));

    create_dummy_circuit(key, witness);

    auto transcript = create_dummy_ultra_transcript();

    waffle::ProverPLookupWidget widget(key.get(), witness.get());
    widget.compute_sorted_list_commitment(transcript);
    widget.compute_grand_product_commitment(transcript);

    auto& z = witness->wires.at("z_lookup");
    z.fft(key->small_domain);

    fr beta = fr::serialize_from_buffer(transcript.get_challenge("beta").begin());
    fr gamma = fr::serialize_from_buffer(transcript.get_challenge("beta", 1).begin());
    const fr gamma_beta_constant = gamma * (fr(1) + beta);
    const fr expected = gamma_beta_constant.pow(n - 1);

    EXPECT_EQ(z[3], expected);
    EXPECT_EQ(z[0], fr(1));
}

TEST(plookup_widget, compute_quotient_contribution)
{
    const size_t n = 4;
    const size_t num_gates = n;
    std::shared_ptr<program_witness> witness = std::make_shared<program_witness>();
    auto crs = std::make_unique<FileReferenceStringFactory>("../srs_db");
    std::shared_ptr<proving_key> key = std::make_shared<proving_key>(num_gates, 0, crs->get_prover_crs(num_gates));

    create_dummy_circuit(key, witness);

    auto transcript = create_dummy_ultra_transcript();

    waffle::ProverPLookupWidget widget(key.get(), witness.get());
    widget.compute_sorted_list_commitment(transcript);
    widget.compute_grand_product_commitment(transcript);
    {
        auto& w_1 = witness->wires.at("w_1");
        auto& w_2 = witness->wires.at("w_2");
        auto& w_3 = witness->wires.at("w_3");
        auto& s = witness->wires.at("s");
        auto& z = witness->wires.at("z_lookup");

        auto& w_1_fft = key->wire_ffts.at("w_1_fft");
        auto& w_2_fft = key->wire_ffts.at("w_2_fft");
        auto& w_3_fft = key->wire_ffts.at("w_3_fft");
        auto& s_fft = key->wire_ffts.at("s_fft");
        auto& z_fft = key->wire_ffts.at("z_lookup_fft");

        w_1.ifft(key->small_domain);
        w_2.ifft(key->small_domain);
        w_3.ifft(key->small_domain);

        for (size_t i = 0; i < n; ++i) {
            w_1_fft[i] = w_1[i];
            w_2_fft[i] = w_2[i];
            w_3_fft[i] = w_3[i];
            z_fft[i] = z[i];
            s_fft[i] = s[i];
        }
        for (size_t i = n; i < 4 * n; ++i) {
            w_1_fft[i] = fr(0);
            w_2_fft[i] = fr(0);
            w_3_fft[i] = fr(0);
            s_fft[i] = fr(0);
            z_fft[i] = fr(0);
        }

        w_1_fft.fft(key->large_domain);
        w_2_fft.fft(key->large_domain);
        w_3_fft.fft(key->large_domain);
        s_fft.fft(key->large_domain);
        z_fft.fft(key->large_domain);

        w_1_fft.add_lagrange_base_coefficient(w_1_fft[0]);
        w_1_fft.add_lagrange_base_coefficient(w_1_fft[1]);
        w_1_fft.add_lagrange_base_coefficient(w_1_fft[2]);
        w_1_fft.add_lagrange_base_coefficient(w_1_fft[3]);
        w_2_fft.add_lagrange_base_coefficient(w_2_fft[0]);
        w_2_fft.add_lagrange_base_coefficient(w_2_fft[1]);
        w_2_fft.add_lagrange_base_coefficient(w_2_fft[2]);
        w_2_fft.add_lagrange_base_coefficient(w_2_fft[3]);
        w_3_fft.add_lagrange_base_coefficient(w_3_fft[0]);
        w_3_fft.add_lagrange_base_coefficient(w_3_fft[1]);
        w_3_fft.add_lagrange_base_coefficient(w_3_fft[2]);
        w_3_fft.add_lagrange_base_coefficient(w_3_fft[3]);

        s_fft.add_lagrange_base_coefficient(s_fft[0]);
        s_fft.add_lagrange_base_coefficient(s_fft[1]);
        s_fft.add_lagrange_base_coefficient(s_fft[2]);
        s_fft.add_lagrange_base_coefficient(s_fft[3]);

        z_fft.add_lagrange_base_coefficient(z_fft[0]);
        z_fft.add_lagrange_base_coefficient(z_fft[1]);
        z_fft.add_lagrange_base_coefficient(z_fft[2]);
        z_fft.add_lagrange_base_coefficient(z_fft[3]);

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

    widget.compute_quotient_contribution(fr(1), transcript);

    auto& quotient_poly = key->quotient_large;

    for (size_t i = 0; i < key->small_domain.size - 1; ++i) {
        EXPECT_EQ(quotient_poly[i * 4], fr(0));
    }
}
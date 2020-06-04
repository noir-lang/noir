#include "generalized_permutation_composer.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;

namespace {
auto& engine = numeric::random::get_debug_engine();
}
std::vector<uint32_t> add_variables(waffle::GenPermComposer& composer, std::vector<fr> variables)
{
    std::vector<uint32_t> res;
    for (size_t i = 0; i < variables.size(); i++) {
        res.emplace_back(composer.add_variable(variables[i]));
    }
    return res;
}
TEST(genperm_composer, base_case)
{
    waffle::GenPermComposer composer = waffle::GenPermComposer();
    fr a = fr::one();
    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(a);
    composer.assert_equal(a_idx, b_idx);
    composer.create_add_gate({ a_idx, a_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });

    waffle::TurboProver prover = composer.create_prover();
    waffle::GenPermVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(genperm_composer, test_add_gate_proofs)
{
    waffle::GenPermComposer composer = waffle::GenPermComposer();
    fr a = fr::one();
    fr b = fr::one();
    fr c = a + b;
    fr d = a + c;
    uint32_t a_idx = composer.add_variable(a);
    uint32_t b_idx = composer.add_public_variable(b);
    uint32_t c_idx = composer.add_variable(c);
    uint32_t d_idx = composer.add_variable(d);

    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ d_idx, c_idx, a_idx, fr::one(), fr::neg_one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ b_idx, a_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });

    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });
    composer.create_add_gate({ a_idx, b_idx, c_idx, fr::one(), fr::one(), fr::neg_one(), fr::zero() });

    // TODO: proof fails if one wire contains all zeros. Should we support this?
    uint32_t zero_idx = composer.add_variable(fr::zero());

    composer.create_big_add_gate(
        { zero_idx, zero_idx, zero_idx, a_idx, fr::one(), fr::one(), fr::one(), fr::one(), fr::neg_one() });

    waffle::TurboProver prover = composer.create_prover();

    waffle::GenPermVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(genperm_composer, test_mul_gate_proofs)
{
    waffle::GenPermComposer composer = waffle::GenPermComposer();
    fr q[7]{ fr::random_element(), fr::random_element(), fr::random_element(), fr::random_element(),
             fr::random_element(), fr::random_element(), fr::random_element() };
    fr q_inv[7]{
        q[0].invert(), q[1].invert(), q[2].invert(), q[3].invert(), q[4].invert(), q[5].invert(), q[6].invert(),
    };

    fr a = fr::random_element();
    fr b = fr::random_element();
    fr c = -((((q[0] * a) + (q[1] * b)) + q[3]) * q_inv[2]);
    fr d = -((((q[4] * (a * b)) + q[6]) * q_inv[5]));

    uint32_t a_idx = composer.add_public_variable(a);
    uint32_t b_idx = composer.add_variable(b);
    uint32_t c_idx = composer.add_variable(c);
    uint32_t d_idx = composer.add_variable(d);

    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });

    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });
    composer.create_add_gate({ a_idx, b_idx, c_idx, q[0], q[1], q[2], q[3] });
    composer.create_mul_gate({ a_idx, b_idx, d_idx, q[4], q[5], q[6] });

    uint32_t zero_idx = composer.add_variable(fr::zero());
    uint32_t one_idx = composer.add_variable(fr::one());
    composer.create_big_add_gate(
        { zero_idx, zero_idx, zero_idx, one_idx, fr::one(), fr::one(), fr::one(), fr::one(), fr::neg_one() });

    uint32_t e_idx = composer.add_variable(a - fr::one());
    composer.create_add_gate({ e_idx, b_idx, c_idx, q[0], q[1], q[2], (q[3] + q[0]) });
    waffle::TurboProver prover = composer.create_prover();

    waffle::GenPermVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

TEST(genperm_composer, non_trivial_tag_permutation)
{
    waffle::GenPermComposer composer = waffle::GenPermComposer();
    fr a = fr::random_element();
    fr b = -a;

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(b);
    auto c_idx = composer.add_variable(b);
    auto d_idx = composer.add_variable(a);

    composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::one(), fr::zero(), fr::zero() });
    composer.create_add_gate({ c_idx, d_idx, composer.zero_idx, fr::one(), fr::one(), fr::zero(), fr::zero() });

    composer.create_tag(1, 2);
    composer.create_tag(2, 1);

    composer.assign_tag(a_idx, 1);
    composer.assign_tag(b_idx, 1);
    composer.assign_tag(c_idx, 2);
    composer.assign_tag(d_idx, 2);

    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    waffle::TurboProver prover = composer.create_prover();
    waffle::GenPermVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}
TEST(genperm_composer, non_trivial_tag_permutation_and_cycles)
{
    waffle::GenPermComposer composer = waffle::GenPermComposer();
    fr a = fr::random_element();
    fr c = -a;

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(a);
    composer.assert_equal(a_idx, b_idx);
    auto c_idx = composer.add_variable(c);
    auto d_idx = composer.add_variable(c);
    composer.assert_equal(c_idx, d_idx);
    auto e_idx = composer.add_variable(a);
    auto f_idx = composer.add_variable(a);
    composer.assert_equal(e_idx, f_idx);
    auto g_idx = composer.add_variable(c);
    auto h_idx = composer.add_variable(c);
    composer.assert_equal(g_idx, h_idx);

    composer.create_tag(1, 2);
    composer.create_tag(2, 1);

    composer.assign_tag(a_idx, 1);
    composer.assign_tag(c_idx, 1);
    composer.assign_tag(e_idx, 2);
    composer.assign_tag(g_idx, 2);
    composer.create_dummy_gate();
    composer.create_add_gate({ b_idx, a_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    composer.create_add_gate({ c_idx, g_idx, composer.zero_idx, fr::one(), -fr::one(), fr::zero(), fr::zero() });
    composer.create_add_gate({ e_idx, f_idx, composer.zero_idx, fr::one(), -fr::one(), fr::zero(), fr::zero() });

    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    waffle::TurboProver prover = composer.create_prover();
    waffle::GenPermVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(genperm_composer, bad_tag_permutation)
{
    waffle::GenPermComposer composer = waffle::GenPermComposer();
    fr a = fr::random_element();
    fr b = -a;

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(b);
    auto c_idx = composer.add_variable(b);
    auto d_idx = composer.add_variable(a + 1);

    composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, 1, 1, 0, 0 });
    composer.create_add_gate({ c_idx, d_idx, composer.zero_idx, 1, 1, 0, -1 });

    composer.create_tag(1, 2);
    composer.create_tag(2, 1);

    composer.assign_tag(a_idx, 1);
    composer.assign_tag(b_idx, 1);
    composer.assign_tag(c_idx, 2);
    composer.assign_tag(d_idx, 2);
    waffle::TurboProver prover = composer.create_prover();
    waffle::GenPermVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, false);
}
// Check rejection on mismatch of size of cycle sets between tags
// TEST(genperm_composer, bad_tag_permutation2)
// {
//     waffle::GenPermComposer composer = waffle::GenPermComposer();
//     fr a = fr::random_element();
//     fr b = -a;

//     auto a_idx = composer.add_variable(a);
//     auto b_idx = composer.add_variable(b);
//     auto c_idx = composer.add_variable(b);
//     auto d_idx = composer.add_variable(a);
//     auto e_idx = composer.add_variable(a);
//     auto f_idx = composer.add_variable(b);

//     composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, 1, 1, 0, 0 });
//     composer.create_add_gate({ c_idx, d_idx, composer.zero_idx, 1, 1, 0, -1 });

//     composer.create_tag(1, 2);
//     composer.create_tag(2, 1);

//     composer.assign_tag(a_idx, 1);
//     composer.assign_tag(b_idx, 1);
//     composer.assign_tag(c_idx, 2);
//     composer.assign_tag(d_idx, 2);
//     waffle::TurboProver prover = composer.create_prover();
//     waffle::GenPermVerifier verifier = composer.create_verifier();
//     waffle::plonk_proof proof = prover.construct_proof();

//     bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
//     EXPECT_EQ(result, false);
// }

// same as above but with turbocomposer to check reason of failue is really tag mismatch
TEST(genperm_composer, bad_tag_turbo_permutation)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    fr a = fr::random_element();
    fr b = -a;

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(b);
    auto c_idx = composer.add_variable(b);
    auto d_idx = composer.add_variable(a + 1);

    composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, 1, 1, 0, 0 });
    composer.create_add_gate({ c_idx, d_idx, composer.zero_idx, 1, 1, 0, -1 });

    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    // composer.create_add_gate({ a_idx, b_idx, composer.zero_idx, fr::one(), fr::neg_one(), fr::zero(), fr::zero() });
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(genperm_composer, sort_widget)
{
    waffle::GenPermComposer composer = waffle::GenPermComposer();
    fr a = fr::one();
    fr b = fr(2);
    fr c = fr(3);
    fr d = fr(4);

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(b);
    auto c_idx = composer.add_variable(c);
    auto d_idx = composer.add_variable(d);
    composer.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });
    waffle::TurboProver prover = composer.create_prover();
    waffle::GenPermVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, true);
}

TEST(genperm_composer, sort_with_edges_gate)
{

    fr a = fr::one();
    fr b = fr(2);
    fr c = fr(3);
    fr d = fr(4);
    fr e = fr(5);
    fr f = fr(6);
    fr g = fr(7);
    fr h = fr(8);

    {
        waffle::GenPermComposer composer = waffle::GenPermComposer();
        auto a_idx = composer.add_variable(a);
        auto b_idx = composer.add_variable(b);
        auto c_idx = composer.add_variable(c);
        auto d_idx = composer.add_variable(d);
        auto e_idx = composer.add_variable(e);
        auto f_idx = composer.add_variable(f);
        auto g_idx = composer.add_variable(g);
        auto h_idx = composer.add_variable(h);
        composer.create_sort_constraint_with_edges({ a_idx, b_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, a, h);
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, true);
    }

    {
        waffle::GenPermComposer composer = waffle::GenPermComposer();
        auto a_idx = composer.add_variable(a);
        auto b_idx = composer.add_variable(b);
        auto c_idx = composer.add_variable(c);
        auto d_idx = composer.add_variable(d);
        auto e_idx = composer.add_variable(e);
        auto f_idx = composer.add_variable(f);
        auto g_idx = composer.add_variable(g);
        auto h_idx = composer.add_variable(h);
        composer.create_sort_constraint_with_edges({ a_idx, b_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, a, g);
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
    {
        waffle::GenPermComposer composer = waffle::GenPermComposer();
        auto a_idx = composer.add_variable(a);
        auto b_idx = composer.add_variable(b);
        auto c_idx = composer.add_variable(c);
        auto d_idx = composer.add_variable(d);
        auto e_idx = composer.add_variable(e);
        auto f_idx = composer.add_variable(f);
        auto g_idx = composer.add_variable(g);
        auto h_idx = composer.add_variable(h);
        composer.create_sort_constraint_with_edges({ a_idx, b_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, b, h);
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
    {
        waffle::GenPermComposer composer = waffle::GenPermComposer();
        auto a_idx = composer.add_variable(a);
        // auto b_idx = composer.add_variable(b);
        auto c_idx = composer.add_variable(c);
        auto d_idx = composer.add_variable(d);
        auto e_idx = composer.add_variable(e);
        auto f_idx = composer.add_variable(f);
        auto g_idx = composer.add_variable(g);
        auto h_idx = composer.add_variable(h);
        auto b2_idx = composer.add_variable(fr(15));
        composer.create_sort_constraint_with_edges({ a_idx, b2_idx, c_idx, d_idx, e_idx, f_idx, g_idx, h_idx }, b, h);
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
    {
        waffle::GenPermComposer composer = waffle::GenPermComposer();
        auto idx = add_variables(composer, { 1,  2,  5,  6,  7,  10, 11, 13, 16, 17, 20, 22, 22, 25,
                                             26, 29, 29, 32, 32, 33, 35, 38, 39, 39, 42, 42, 43, 45 });
        composer.create_sort_constraint_with_edges(idx, 1, 45);
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, true);
        // auto new_idx = composer.add_variable(47);
        // idx = add_variables(1,2,5,6,7,10,11,13,16,17,20,22,22,25,26,29,29,32,32,33,35,38,39,39,42,42,43);
        composer.create_sort_constraint_with_edges(idx, 1, 29);
        prover = composer.create_prover();
        verifier = composer.create_verifier();
        proof = prover.construct_proof();

        result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
}
TEST(genperm_composer, range_constraint)
{
    {
        waffle::GenPermComposer composer = waffle::GenPermComposer();
        auto indices = add_variables(composer, { 1, 2, 3, 4, 5, 6, 7, 8 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_range_constraint(indices[i], 8);
        }
        // auto ind = {a_idx,b_idx,c_idx,d_idx,e_idx,f_idx,g_idx,h_idx};
        composer.create_sort_constraint(indices);
        composer.process_range_lists();
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
    {
        waffle::GenPermComposer composer = waffle::GenPermComposer();
        auto indices = add_variables(composer, { 1, 2, 3, 4, 5, 6, 25, 8 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_range_constraint(indices[i], 8);
        }
        composer.create_sort_constraint(indices);
        composer.process_range_lists();
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
    {
        waffle::GenPermComposer composer = waffle::GenPermComposer();
        auto indices =
            add_variables(composer, { 1, 2, 3, 4, 5, 6, 10, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 19, 51 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_range_constraint(indices[i], 128);
        }
        composer.create_dummy_constraint(indices);
        composer.process_range_lists();
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }
    {
        waffle::GenPermComposer composer = waffle::GenPermComposer();
        auto indices =
            add_variables(composer, { 1, 2, 3, 80, 5, 6, 29, 8, 15, 11, 32, 21, 42, 79, 16, 10, 3, 26, 13, 14 });
        for (size_t i = 0; i < indices.size(); i++) {
            composer.create_range_constraint(indices[i], 79);
        }
        composer.create_sort_constraint(indices);
        composer.process_range_lists();
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, false);
    }
}
TEST(genperm_composer, range_with_gates)
{

    waffle::GenPermComposer composer = waffle::GenPermComposer();
    auto idx = add_variables(composer, { 1, 2, 3, 4, 5, 6, 7, 8 });
    for (size_t i = 0; i < idx.size(); i++) {
        composer.create_range_constraint(idx[i], 8);
    }
    // auto ind = {a_idx,b_idx,c_idx,d_idx,e_idx,f_idx,g_idx,h_idx};
    composer.process_range_lists();

    composer.create_add_gate({ idx[0], idx[1], composer.zero_idx, fr::one(), fr::one(), fr::zero(), -3 });
    composer.create_add_gate({ idx[2], idx[3], composer.zero_idx, fr::one(), fr::one(), fr::zero(), -7 });
    composer.create_add_gate({ idx[4], idx[5], composer.zero_idx, fr::one(), fr::one(), fr::zero(), -11 });
    composer.create_add_gate({ idx[6], idx[7], composer.zero_idx, fr::one(), fr::one(), fr::zero(), -15 });
    waffle::TurboProver prover = composer.create_prover();
    waffle::GenPermVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();
    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}
TEST(genperm_composer, sort_widget_complex)
{
    {

        waffle::GenPermComposer composer = waffle::GenPermComposer();
        std::vector<fr> a = { 1, 3, 4, 7, 7, 8, 11, 14, 15, 15, 18, 19, 21, 21, 24, 25, 26, 27, 30, 32 };
        std::vector<uint32_t> ind;
        for (size_t i = 0; i < a.size(); i++)
            ind.emplace_back(composer.add_variable(a[i]));
        composer.create_sort_constraint(ind);
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, true);
    }
    {

        waffle::GenPermComposer composer = waffle::GenPermComposer();
        std::vector<fr> a = { 1, 3, 4, 7, 7, 8, 16, 14, 15, 15, 18, 19, 21, 21, 24, 25, 26, 27, 30, 32 };
        std::vector<uint32_t> ind;
        for (size_t i = 0; i < a.size(); i++)
            ind.emplace_back(composer.add_variable(a[i]));
        composer.create_sort_constraint(ind);
        waffle::TurboProver prover = composer.create_prover();
        waffle::GenPermVerifier verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
        EXPECT_EQ(result, false);
    }
}
TEST(genperm_composer, sort_widget_neg)
{
    waffle::GenPermComposer composer = waffle::GenPermComposer();
    fr a = fr::one();
    fr b = fr(2);
    fr c = fr(3);
    fr d = fr(8);

    auto a_idx = composer.add_variable(a);
    auto b_idx = composer.add_variable(b);
    auto c_idx = composer.add_variable(c);
    auto d_idx = composer.add_variable(d);
    composer.create_sort_constraint({ a_idx, b_idx, c_idx, d_idx });
    waffle::TurboProver prover = composer.create_prover();
    waffle::GenPermVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof); // instance, prover.reference_string.SRS_T2);
    EXPECT_EQ(result, false);
}
TEST(genperm_composer, small_scalar_multipliers)
{
    constexpr size_t num_bits = 63;
    constexpr size_t num_quads_base = (num_bits - 1) >> 1;
    constexpr size_t num_quads = ((num_quads_base << 1) + 1 < num_bits) ? num_quads_base + 1 : num_quads_base;
    constexpr size_t num_wnaf_bits = (num_quads << 1) + 1;
    constexpr size_t initial_exponent = ((num_bits & 1) == 1) ? num_bits - 1 : num_bits;
    constexpr uint64_t bit_mask = (1ULL << num_bits) - 1UL;
    const crypto::pedersen::fixed_base_ladder* ladder = crypto::pedersen::get_ladder(0, num_bits);
    grumpkin::g1::affine_element generator = crypto::pedersen::get_generator(0);

    grumpkin::g1::element origin_points[2];
    origin_points[0] = grumpkin::g1::element(ladder[0].one);
    origin_points[1] = origin_points[0] + generator;
    origin_points[1] = origin_points[1].normalize();

    grumpkin::fr scalar_multiplier_entropy = grumpkin::fr::random_element();
    grumpkin::fr scalar_multiplier_base{ scalar_multiplier_entropy.data[0] & bit_mask, 0, 0, 0 };
    // scalar_multiplier_base.data[0] = scalar_multiplier_base.data[0] | (1ULL);
    scalar_multiplier_base.data[0] = scalar_multiplier_base.data[0] & (~1ULL);
    grumpkin::fr scalar_multiplier = scalar_multiplier_base;

    uint64_t wnaf_entries[num_quads + 1] = { 0 };
    if ((scalar_multiplier_base.data[0] & 1) == 0) {
        scalar_multiplier_base.data[0] -= 2;
    }
    bool skew = false;
    barretenberg::wnaf::fixed_wnaf<num_wnaf_bits, 1, 2>(&scalar_multiplier_base.data[0], &wnaf_entries[0], skew, 0);

    fr accumulator_offset = (fr::one() + fr::one()).pow(static_cast<uint64_t>(initial_exponent)).invert();
    fr origin_accumulators[2]{ fr::one(), accumulator_offset + fr::one() };

    grumpkin::g1::element* multiplication_transcript =
        static_cast<grumpkin::g1::element*>(aligned_alloc(64, sizeof(grumpkin::g1::element) * (num_quads + 1)));
    fr* accumulator_transcript = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * (num_quads + 1)));

    if (skew) {
        multiplication_transcript[0] = origin_points[1];
        accumulator_transcript[0] = origin_accumulators[1];
    } else {
        multiplication_transcript[0] = origin_points[0];
        accumulator_transcript[0] = origin_accumulators[0];
    }

    fr one = fr::one();
    fr three = ((one + one) + one);
    for (size_t i = 0; i < num_quads; ++i) {
        uint64_t entry = wnaf_entries[i + 1] & 0xffffff;
        fr prev_accumulator = accumulator_transcript[i] + accumulator_transcript[i];
        prev_accumulator = prev_accumulator + prev_accumulator;

        grumpkin::g1::affine_element point_to_add = (entry == 1) ? ladder[i + 1].three : ladder[i + 1].one;
        fr scalar_to_add = (entry == 1) ? three : one;
        uint64_t predicate = (wnaf_entries[i + 1] >> 31U) & 1U;
        if (predicate) {
            point_to_add = -point_to_add;
            scalar_to_add.self_neg();
        }
        accumulator_transcript[i + 1] = prev_accumulator + scalar_to_add;
        multiplication_transcript[i + 1] = multiplication_transcript[i] + point_to_add;
    }
    grumpkin::g1::element::batch_normalize(&multiplication_transcript[0], num_quads + 1);

    waffle::fixed_group_init_quad init_quad{ origin_points[0].x,
                                             (origin_points[0].x - origin_points[1].x),
                                             origin_points[0].y,
                                             (origin_points[0].y - origin_points[1].y) };

    waffle::GenPermComposer composer = waffle::GenPermComposer();

    fr x_alpha = accumulator_offset;
    for (size_t i = 0; i < num_quads; ++i) {
        waffle::fixed_group_add_quad round_quad;
        round_quad.d = composer.add_variable(accumulator_transcript[i]);
        round_quad.a = composer.add_variable(multiplication_transcript[i].x);
        round_quad.b = composer.add_variable(multiplication_transcript[i].y);
        round_quad.c = composer.add_variable(x_alpha);
        if ((wnaf_entries[i + 1] & 0xffffffU) == 0) {
            x_alpha = ladder[i + 1].one.x;
        } else {
            x_alpha = ladder[i + 1].three.x;
        }
        round_quad.q_x_1 = ladder[i + 1].q_x_1;
        round_quad.q_x_2 = ladder[i + 1].q_x_2;
        round_quad.q_y_1 = ladder[i + 1].q_y_1;
        round_quad.q_y_2 = ladder[i + 1].q_y_2;

        if (i > 0) {
            composer.create_fixed_group_add_gate(round_quad);
        } else {
            composer.create_fixed_group_add_gate_with_init(round_quad, init_quad);
        }
    }

    waffle::add_quad add_quad{ composer.add_variable(multiplication_transcript[num_quads].x),
                               composer.add_variable(multiplication_transcript[num_quads].y),
                               composer.add_variable(x_alpha),
                               composer.add_variable(accumulator_transcript[num_quads]),
                               fr::zero(),
                               fr::zero(),
                               fr::zero(),
                               fr::zero(),
                               fr::zero() };
    composer.create_big_add_gate(add_quad);

    grumpkin::g1::element expected_point =
        grumpkin::g1::element(generator * scalar_multiplier.to_montgomery_form()).normalize();
    EXPECT_EQ((multiplication_transcript[num_quads].x == expected_point.x), true);
    EXPECT_EQ((multiplication_transcript[num_quads].y == expected_point.y), true);

    uint64_t result_accumulator = accumulator_transcript[num_quads].from_montgomery_form().data[0];
    uint64_t expected_accumulator = scalar_multiplier.data[0];
    EXPECT_EQ(result_accumulator, expected_accumulator);

    waffle::TurboProver prover = composer.create_prover();

    waffle::GenPermVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);

    free(multiplication_transcript);
    free(accumulator_transcript);
}

TEST(genperm_composer, large_scalar_multipliers)
{
    constexpr size_t num_bits = 254;
    constexpr size_t num_quads_base = (num_bits - 1) >> 1;
    constexpr size_t num_quads = ((num_quads_base << 1) + 1 < num_bits) ? num_quads_base + 1 : num_quads_base;
    constexpr size_t num_wnaf_bits = (num_quads << 1) + 1;

    constexpr size_t initial_exponent = num_bits; // ((num_bits & 1) == 1) ? num_bits - 1 : num_bits;
    const crypto::pedersen::fixed_base_ladder* ladder = crypto::pedersen::get_ladder(0, num_bits);
    grumpkin::g1::affine_element generator = crypto::pedersen::get_generator(0);

    grumpkin::g1::element origin_points[2];
    origin_points[0] = grumpkin::g1::element(ladder[0].one);
    origin_points[1] = origin_points[0] + generator;
    origin_points[1] = origin_points[1].normalize();

    grumpkin::fr scalar_multiplier_base = grumpkin::fr::random_element();

    grumpkin::fr scalar_multiplier = scalar_multiplier_base.from_montgomery_form();

    if ((scalar_multiplier.data[0] & 1) == 0) {
        grumpkin::fr two = grumpkin::fr::one() + grumpkin::fr::one();
        scalar_multiplier_base = scalar_multiplier_base - two;
    }
    scalar_multiplier_base = scalar_multiplier_base.from_montgomery_form();
    uint64_t wnaf_entries[num_quads + 1] = { 0 };

    bool skew = false;
    barretenberg::wnaf::fixed_wnaf<num_wnaf_bits, 1, 2>(&scalar_multiplier_base.data[0], &wnaf_entries[0], skew, 0);

    fr accumulator_offset = (fr::one() + fr::one()).pow(static_cast<uint64_t>(initial_exponent)).invert();
    fr origin_accumulators[2]{ fr::one(), accumulator_offset + fr::one() };

    grumpkin::g1::element* multiplication_transcript =
        static_cast<grumpkin::g1::element*>(aligned_alloc(64, sizeof(grumpkin::g1::element) * (num_quads + 1)));
    fr* accumulator_transcript = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * (num_quads + 1)));

    if (skew) {
        multiplication_transcript[0] = origin_points[1];
        accumulator_transcript[0] = origin_accumulators[1];
    } else {
        multiplication_transcript[0] = origin_points[0];
        accumulator_transcript[0] = origin_accumulators[0];
    }

    fr one = fr::one();
    fr three = ((one + one) + one);
    for (size_t i = 0; i < num_quads; ++i) {
        uint64_t entry = wnaf_entries[i + 1] & 0xffffff;
        fr prev_accumulator = accumulator_transcript[i] + accumulator_transcript[i];
        prev_accumulator = prev_accumulator + prev_accumulator;

        grumpkin::g1::affine_element point_to_add = (entry == 1) ? ladder[i + 1].three : ladder[i + 1].one;
        fr scalar_to_add = (entry == 1) ? three : one;
        uint64_t predicate = (wnaf_entries[i + 1] >> 31U) & 1U;
        if (predicate) {
            point_to_add = -point_to_add;
            scalar_to_add.self_neg();
        }
        accumulator_transcript[i + 1] = prev_accumulator + scalar_to_add;
        multiplication_transcript[i + 1] = multiplication_transcript[i] + point_to_add;
    }
    grumpkin::g1::element::batch_normalize(&multiplication_transcript[0], num_quads + 1);

    waffle::fixed_group_init_quad init_quad{ origin_points[0].x,
                                             (origin_points[0].x - origin_points[1].x),
                                             origin_points[0].y,
                                             (origin_points[0].y - origin_points[1].y) };

    waffle::GenPermComposer composer = waffle::GenPermComposer();

    fr x_alpha = accumulator_offset;
    for (size_t i = 0; i < num_quads; ++i) {
        waffle::fixed_group_add_quad round_quad;
        round_quad.d = composer.add_variable(accumulator_transcript[i]);
        round_quad.a = composer.add_variable(multiplication_transcript[i].x);
        round_quad.b = composer.add_variable(multiplication_transcript[i].y);
        round_quad.c = composer.add_variable(x_alpha);
        if ((wnaf_entries[i + 1] & 0xffffffU) == 0) {
            x_alpha = ladder[i + 1].one.x;
        } else {
            x_alpha = ladder[i + 1].three.x;
        }
        round_quad.q_x_1 = ladder[i + 1].q_x_1;
        round_quad.q_x_2 = ladder[i + 1].q_x_2;
        round_quad.q_y_1 = ladder[i + 1].q_y_1;
        round_quad.q_y_2 = ladder[i + 1].q_y_2;

        if (i > 0) {
            composer.create_fixed_group_add_gate(round_quad);
        } else {
            composer.create_fixed_group_add_gate_with_init(round_quad, init_quad);
        }
    }

    waffle::add_quad add_quad{ composer.add_variable(multiplication_transcript[num_quads].x),
                               composer.add_variable(multiplication_transcript[num_quads].y),
                               composer.add_variable(x_alpha),
                               composer.add_variable(accumulator_transcript[num_quads]),
                               fr::zero(),
                               fr::zero(),
                               fr::zero(),
                               fr::zero(),
                               fr::zero() };
    composer.create_big_add_gate(add_quad);

    grumpkin::g1::element expected_point =
        grumpkin::g1::element(generator * scalar_multiplier.to_montgomery_form()).normalize();
    EXPECT_EQ((multiplication_transcript[num_quads].x == expected_point.x), true);
    EXPECT_EQ((multiplication_transcript[num_quads].y == expected_point.y), true);

    fr result_accumulator = (accumulator_transcript[num_quads]);
    fr expected_accumulator =
        fr{ scalar_multiplier.data[0], scalar_multiplier.data[1], scalar_multiplier.data[2], scalar_multiplier.data[3] }
            .to_montgomery_form();
    EXPECT_EQ((result_accumulator == expected_accumulator), true);

    waffle::TurboProver prover = composer.create_prover();

    waffle::GenPermVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);

    free(multiplication_transcript);
    free(accumulator_transcript);
}

// TEST(genperm_composer, range_constraint)
// {
//     waffle::GenPermComposer composer = waffle::GenPermComposer();

//     for (size_t i = 0; i < 10; ++i) {
//         uint32_t value = engine.get_random_uint32();
//         fr witness_value = fr{ value, 0, 0, 0 }.to_montgomery_form();
//         uint32_t witness_index = composer.add_variable(witness_value);

//         // include non-nice numbers of bits, that will bleed over gate boundaries
//         size_t extra_bits = 2 * (i % 4);

//         std::vector<uint32_t> accumulators = composer.create_range_constraint(witness_index, 32 + extra_bits);

//         for (uint32_t j = 0; j < 16; ++j) {
//             uint32_t result = (value >> (30U - (2 * j)));
//             fr source = composer.get_variable(accumulators[j + (extra_bits >> 1)]).from_montgomery_form();
//             uint32_t expected = static_cast<uint32_t>(source.data[0]);
//             EXPECT_EQ(result, expected);
//         }
//         for (uint32_t j = 1; j < 16; ++j) {
//             uint32_t left = (value >> (30U - (2 * j)));
//             uint32_t right = (value >> (30U - (2 * (j - 1))));
//             EXPECT_EQ(left - 4 * right < 4, true);
//         }
//     }

//     uint32_t zero_idx = composer.add_variable(fr::zero());
//     uint32_t one_idx = composer.add_variable(fr::one());
//     composer.create_big_add_gate(
//         { zero_idx, zero_idx, zero_idx, one_idx, fr::one(), fr::one(), fr::one(), fr::one(), fr::neg_one() });

//     waffle::TurboProver prover = composer.create_prover();

//     waffle::GenPermVerifier verifier = composer.create_verifier();

//     waffle::plonk_proof proof = prover.construct_proof();

//     bool result = verifier.verify_proof(proof);

//     EXPECT_EQ(result, true);
// }

TEST(genperm_composer, and_constraint)
{
    waffle::GenPermComposer composer = waffle::GenPermComposer();

    for (size_t i = 0; i < /*10*/ 1; ++i) {
        uint32_t left_value = engine.get_random_uint32();

        fr left_witness_value = fr{ left_value, 0, 0, 0 }.to_montgomery_form();
        uint32_t left_witness_index = composer.add_variable(left_witness_value);

        uint32_t right_value = engine.get_random_uint32();
        fr right_witness_value = fr{ right_value, 0, 0, 0 }.to_montgomery_form();
        uint32_t right_witness_index = composer.add_variable(right_witness_value);

        uint32_t out_value = left_value & right_value;
        // include non-nice numbers of bits, that will bleed over gate boundaries
        size_t extra_bits = 2 * (i % 4);

        waffle::accumulator_triple accumulators =
            composer.create_and_constraint(left_witness_index, right_witness_index, 32 + extra_bits);
        // composer.create_and_constraint(left_witness_index, right_witness_index, 32 + extra_bits);

        for (uint32_t j = 0; j < 16; ++j) {
            uint32_t left_expected = (left_value >> (30U - (2 * j)));
            uint32_t right_expected = (right_value >> (30U - (2 * j)));
            uint32_t out_expected = left_expected & right_expected;

            fr left_source = composer.get_variable(accumulators.left[j + (extra_bits >> 1)]).from_montgomery_form();
            uint32_t left_result = static_cast<uint32_t>(left_source.data[0]);

            fr right_source = composer.get_variable(accumulators.right[j + (extra_bits >> 1)]).from_montgomery_form();
            uint32_t right_result = static_cast<uint32_t>(right_source.data[0]);

            fr out_source = composer.get_variable(accumulators.out[j + (extra_bits >> 1)]).from_montgomery_form();
            uint32_t out_result = static_cast<uint32_t>(out_source.data[0]);

            EXPECT_EQ(left_result, left_expected);
            EXPECT_EQ(right_result, right_expected);
            EXPECT_EQ(out_result, out_expected);
        }
        for (uint32_t j = 1; j < 16; ++j) {
            uint32_t left = (left_value >> (30U - (2 * j)));
            uint32_t right = (left_value >> (30U - (2 * (j - 1))));
            EXPECT_EQ(left - 4 * right < 4, true);

            left = (right_value >> (30U - (2 * j)));
            right = (right_value >> (30U - (2 * (j - 1))));
            EXPECT_EQ(left - 4 * right < 4, true);

            left = (out_value >> (30U - (2 * j)));
            right = (out_value >> (30U - (2 * (j - 1))));
            EXPECT_EQ(left - 4 * right < 4, true);
        }
    }

    uint32_t zero_idx = composer.add_variable(fr::zero());
    uint32_t one_idx = composer.add_variable(fr::one());
    composer.create_big_add_gate(
        { zero_idx, zero_idx, zero_idx, one_idx, fr::one(), fr::one(), fr::one(), fr::one(), fr::neg_one() });

    waffle::TurboProver prover = composer.create_prover();

    waffle::GenPermVerifier verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);

    EXPECT_EQ(result, true);
}

// // TEST(genperm_composer, xor_constraint)
// // {
// //     waffle::TurboComposer composer = waffle::TurboComposer();

// //     for (size_t i = 0; i < /*10*/ 1; ++i) {
// //         uint32_t left_value = engine.get_random_uint32();

// //         fr left_witness_value = fr{ left_value, 0, 0, 0 }.to_montgomery_form();
// //         uint32_t left_witness_index = composer.add_variable(left_witness_value);

// //         uint32_t right_value = engine.get_random_uint32();
// //         fr right_witness_value = fr{ right_value, 0, 0, 0 }.to_montgomery_form();
// //         uint32_t right_witness_index = composer.add_variable(right_witness_value);

// //         uint32_t out_value = left_value ^ right_value;
// //         // include non-nice numbers of bits, that will bleed over gate boundaries
// //         size_t extra_bits = 2 * (i % 4);

// //         waffle::accumulator_triple accumulators =
// //             composer.create_xor_constraint(left_witness_index, right_witness_index, 32 + extra_bits);

// //         for (uint32_t j = 0; j < 16; ++j) {
// //             uint32_t left_expected = (left_value >> (30U - (2 * j)));
// //             uint32_t right_expected = (right_value >> (30U - (2 * j)));
// //             uint32_t out_expected = left_expected ^ right_expected;

// //             fr left_source = composer.get_variable(accumulators.left[j + (extra_bits >>
// 1)]).from_montgomery_form();
// //             uint32_t left_result = static_cast<uint32_t>(left_source.data[0]);

// //             fr right_source = composer.get_variable(accumulators.right[j + (extra_bits >>
// 1)]).from_montgomery_form();
// //             uint32_t right_result = static_cast<uint32_t>(right_source.data[0]);

// //             fr out_source = composer.get_variable(accumulators.out[j + (extra_bits >>
// 1)]).from_montgomery_form();
// //             uint32_t out_result = static_cast<uint32_t>(out_source.data[0]);

// //             EXPECT_EQ(left_result, left_expected);
// //             EXPECT_EQ(right_result, right_expected);
// //             EXPECT_EQ(out_result, out_expected);
// //         }
// //         for (uint32_t j = 1; j < 16; ++j) {
// //             uint32_t left = (left_value >> (30U - (2 * j)));
// //             uint32_t right = (left_value >> (30U - (2 * (j - 1))));
// //             EXPECT_EQ(left - 4 * right < 4, true);

// //             left = (right_value >> (30U - (2 * j)));
// //             right = (right_value >> (30U - (2 * (j - 1))));
// //             EXPECT_EQ(left - 4 * right < 4, true);

// //             left = (out_value >> (30U - (2 * j)));
// //             right = (out_value >> (30U - (2 * (j - 1))));
// //             EXPECT_EQ(left - 4 * right < 4, true);
// //         }
// //     }

// //     uint32_t zero_idx = composer.add_variable(fr::zero());
// //     uint32_t one_idx = composer.add_variable(fr::one());
// //     composer.create_big_add_gate(
// //         { zero_idx, zero_idx, zero_idx, one_idx, fr::one(), fr::one(), fr::one(), fr::one(), fr::neg_one() });

// //     waffle::TurboProver prover = composer.create_prover();

// //     waffle::TurboVerifier verifier = composer.create_verifier();

// //     waffle::plonk_proof proof = prover.construct_proof();

// //     bool result = verifier.verify_proof(proof);

// //     EXPECT_EQ(result, true);
// // }

// // TEST(genperm_composer, big_add_gate_with_bit_extract)
// // {
// //     waffle::TurboComposer composer = waffle::TurboComposer();

// //     const auto generate_constraints = [&composer](uint32_t quad_value) {
// //         uint32_t quad_accumulator_left =
// //             (engine.get_random_uint32() & 0x3fffffff) - quad_value; // make sure this won't overflow
// //         uint32_t quad_accumulator_right = (4 * quad_accumulator_left) + quad_value;

// //         uint32_t left_idx = composer.add_variable(uint256_t(quad_accumulator_left));
// //         uint32_t right_idx = composer.add_variable(uint256_t(quad_accumulator_right));

// //         uint32_t input = engine.get_random_uint32();
// //         uint32_t output = input + (quad_value > 1 ? 1 : 0);

// //         waffle::add_quad gate{ composer.add_variable(uint256_t(input)),
// //                                composer.add_variable(uint256_t(output)),
// //                                right_idx,
// //                                left_idx,
// //                                fr(6),
// //                                -fr(6),
// //                                fr::zero(),
// //                                fr::zero(),
// //                                fr::zero() };

// //         composer.create_big_add_gate_with_bit_extraction(gate);
// //     };

// //     generate_constraints(0);
// //     generate_constraints(1);
// //     generate_constraints(2);
// //     generate_constraints(3);

// //     waffle::TurboProver prover = composer.create_prover();

// //     waffle::TurboVerifier verifier = composer.create_verifier();

// //     waffle::plonk_proof proof = prover.construct_proof();

// //     bool result = verifier.verify_proof(proof);

// //     EXPECT_EQ(result, true);
// //}

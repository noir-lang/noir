#include <gtest/gtest.h>

#include <numeric/random/engine.hpp>

#include <ecc/curves/bn254/fq.hpp>
#include <ecc/curves/bn254/fr.hpp>

#include "./bigfield.hpp"
#include "../bool/bool.hpp"
#include "../field/field.hpp"
#include <plonk/composer/turbo_composer.hpp>
#include <plonk/proof_system/prover/prover.hpp>
#include <plonk/proof_system/verifier/verifier.hpp>
#include <plonk/proof_system/widgets/arithmetic_widget.hpp>

#include <polynomials/polynomial_arithmetic.hpp>
#include <memory>

namespace test_stdlib_bigfield {
using namespace barretenberg;
using namespace plonk;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

typedef stdlib::bool_t<waffle::TurboComposer> bool_t;
typedef stdlib::field_t<waffle::TurboComposer> field_t;
typedef stdlib::bigfield<waffle::TurboComposer, barretenberg::Bn254FqParams> bigfield;
typedef stdlib::witness_t<waffle::TurboComposer> witness_t;
typedef stdlib::public_witness_t<waffle::TurboComposer> public_witness_t;

TEST(stdlib_bigfield, test_mul)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        fq inputs[3]{ fq::random_element(), fq::random_element(), fq::random_element() };
        bigfield a(witness_t(&composer, barretenberg::fr(uint256_t(inputs[0]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[0]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield b(witness_t(&composer, barretenberg::fr(uint256_t(inputs[1]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[1]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        uint64_t before = composer.get_num_gates();
        bigfield c = a * b;
        uint64_t after = composer.get_num_gates();
        if (i == num_repetitions - 1) {
            std::cout << "num gates per mul = " << after - before << std::endl;
        }
        // uint256_t modulus{ barretenberg::Bn254FqParams::modulus_0,
        //                    barretenberg::Bn254FqParams::modulus_1,
        //                    barretenberg::Bn254FqParams::modulus_2,
        //                    barretenberg::Bn254FqParams::modulus_3 };

        fq expected = (inputs[0] * inputs[1]);
        expected = expected.from_montgomery_form();
        uint512_t result = c.get_value();

        EXPECT_EQ(result.lo.data[0], expected.data[0]);
        EXPECT_EQ(result.lo.data[1], expected.data[1]);
        EXPECT_EQ(result.lo.data[2], expected.data[2]);
        EXPECT_EQ(result.lo.data[3], expected.data[3]);
        EXPECT_EQ(result.hi.data[0], 0ULL);
        EXPECT_EQ(result.hi.data[1], 0ULL);
        EXPECT_EQ(result.hi.data[2], 0ULL);
        EXPECT_EQ(result.hi.data[3], 0ULL);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_bigfield, test_sqr)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 10;
    for (size_t i = 0; i < num_repetitions; ++i) {
        fq inputs[3]{ fq::random_element(), fq::random_element(), fq::random_element() };
        bigfield a(witness_t(&composer, barretenberg::fr(uint256_t(inputs[0]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[0]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        uint64_t before = composer.get_num_gates();
        bigfield c = a.sqr();
        uint64_t after = composer.get_num_gates();
        if (i == num_repetitions - 1) {
            std::cout << "num gates per mul = " << after - before << std::endl;
        }
        // uint256_t modulus{ barretenberg::Bn254FqParams::modulus_0,
        //                    barretenberg::Bn254FqParams::modulus_1,
        //                    barretenberg::Bn254FqParams::modulus_2,
        //                    barretenberg::Bn254FqParams::modulus_3 };

        fq expected = (inputs[0].sqr());
        expected = expected.from_montgomery_form();
        uint512_t result = c.get_value();

        EXPECT_EQ(result.lo.data[0], expected.data[0]);
        EXPECT_EQ(result.lo.data[1], expected.data[1]);
        EXPECT_EQ(result.lo.data[2], expected.data[2]);
        EXPECT_EQ(result.lo.data[3], expected.data[3]);
        EXPECT_EQ(result.hi.data[0], 0ULL);
        EXPECT_EQ(result.hi.data[1], 0ULL);
        EXPECT_EQ(result.hi.data[2], 0ULL);
        EXPECT_EQ(result.hi.data[3], 0ULL);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_bigfield, test_div)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 10;
    for (size_t i = 0; i < num_repetitions; ++i) {
        fq inputs[3]{ fq::random_element(), fq::random_element(), fq::random_element() };
        inputs[0] = inputs[0].reduce_once().reduce_once();
        inputs[1] = inputs[1].reduce_once().reduce_once();
        bigfield a(witness_t(&composer, barretenberg::fr(uint256_t(inputs[0]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[0]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield b(witness_t(&composer, barretenberg::fr(uint256_t(inputs[1]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[1]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield c = a / b;
        // uint256_t modulus{ barretenberg::Bn254FqParams::modulus_0,
        //                    barretenberg::Bn254FqParams::modulus_1,
        //                    barretenberg::Bn254FqParams::modulus_2,
        //                    barretenberg::Bn254FqParams::modulus_3 };

        fq expected = (inputs[0] / inputs[1]);
        expected = expected.reduce_once().reduce_once();
        expected = expected.from_montgomery_form();
        uint512_t result = c.get_value();

        EXPECT_EQ(result.lo.data[0], expected.data[0]);
        EXPECT_EQ(result.lo.data[1], expected.data[1]);
        EXPECT_EQ(result.lo.data[2], expected.data[2]);
        EXPECT_EQ(result.lo.data[3], expected.data[3]);
        EXPECT_EQ(result.hi.data[0], 0ULL);
        EXPECT_EQ(result.hi.data[1], 0ULL);
        EXPECT_EQ(result.hi.data[2], 0ULL);
        EXPECT_EQ(result.hi.data[3], 0ULL);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_bigfield, test_add_and_div)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        fq inputs[4]{ fq::random_element(), fq::random_element(), fq::random_element() };
        bigfield a(witness_t(&composer, barretenberg::fr(uint256_t(inputs[0]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[0]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield b(witness_t(&composer, barretenberg::fr(uint256_t(inputs[1]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[1]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield c(witness_t(&composer, barretenberg::fr(uint256_t(inputs[2]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[2]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield d(witness_t(&composer, barretenberg::fr(uint256_t(inputs[3]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[3]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield e = (a + b) / (c + d);
        // uint256_t modulus{ barretenberg::Bn254FqParams::modulus_0,
        //                    barretenberg::Bn254FqParams::modulus_1,
        //                    barretenberg::Bn254FqParams::modulus_2,
        //                    barretenberg::Bn254FqParams::modulus_3 };

        fq expected = (inputs[0] + inputs[1]) / (inputs[2] + inputs[3]);
        expected = expected.reduce_once().reduce_once();
        expected = expected.from_montgomery_form();
        uint512_t result = e.get_value();

        EXPECT_EQ(result.lo.data[0], expected.data[0]);
        EXPECT_EQ(result.lo.data[1], expected.data[1]);
        EXPECT_EQ(result.lo.data[2], expected.data[2]);
        EXPECT_EQ(result.lo.data[3], expected.data[3]);
        EXPECT_EQ(result.hi.data[0], 0ULL);
        EXPECT_EQ(result.hi.data[1], 0ULL);
        EXPECT_EQ(result.hi.data[2], 0ULL);
        EXPECT_EQ(result.hi.data[3], 0ULL);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_bigfield, test_add_and_mul)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 10;
    for (size_t i = 0; i < num_repetitions; ++i) {
        fq inputs[4]{ fq::random_element(), fq::random_element(), fq::random_element(), fq::random_element() };

        bigfield a(witness_t(&composer, barretenberg::fr(uint256_t(inputs[0]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[0]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield b(witness_t(&composer, barretenberg::fr(uint256_t(inputs[1]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[1]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield c(witness_t(&composer, barretenberg::fr(uint256_t(inputs[2]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[2]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield d(witness_t(&composer, barretenberg::fr(uint256_t(inputs[3]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[3]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield e = (a + b) * (c + d);
        fq expected = (inputs[0] + inputs[1]) * (inputs[2] + inputs[3]);
        expected = expected.from_montgomery_form();
        uint512_t result = e.get_value();

        EXPECT_EQ(result.lo.data[0], expected.data[0]);
        EXPECT_EQ(result.lo.data[1], expected.data[1]);
        EXPECT_EQ(result.lo.data[2], expected.data[2]);
        EXPECT_EQ(result.lo.data[3], expected.data[3]);
        EXPECT_EQ(result.hi.data[0], 0ULL);
        EXPECT_EQ(result.hi.data[1], 0ULL);
        EXPECT_EQ(result.hi.data[2], 0ULL);
        EXPECT_EQ(result.hi.data[3], 0ULL);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_bigfield, test_add_and_mul_with_constants)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 10;
    for (size_t i = 0; i < num_repetitions; ++i) {
        fq inputs[2]{ fq::random_element(), fq::random_element() };
        uint256_t constants[2]{ engine.get_random_uint256(), engine.get_random_uint256() };
        bigfield a(witness_t(&composer, barretenberg::fr(uint256_t(inputs[0]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[0]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield b(&composer, constants[0]);
        bigfield c(witness_t(&composer, barretenberg::fr(uint256_t(inputs[1]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[1]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield d(&composer, constants[1]);
        bigfield e = (a + b) * (c + d);
        fq expected = (inputs[0] + constants[0]) * (inputs[1] + constants[1]);
        expected = expected.from_montgomery_form();
        uint512_t result = e.get_value();

        EXPECT_EQ(result.lo.data[0], expected.data[0]);
        EXPECT_EQ(result.lo.data[1], expected.data[1]);
        EXPECT_EQ(result.lo.data[2], expected.data[2]);
        EXPECT_EQ(result.lo.data[3], expected.data[3]);
        EXPECT_EQ(result.hi.data[0], 0ULL);
        EXPECT_EQ(result.hi.data[1], 0ULL);
        EXPECT_EQ(result.hi.data[2], 0ULL);
        EXPECT_EQ(result.hi.data[3], 0ULL);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_bigfield, test_sub_and_mul)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 10;
    for (size_t i = 0; i < num_repetitions; ++i) {
        fq inputs[4]{ fq::random_element(), fq::random_element(), fq::random_element(), fq::random_element() };

        bigfield a(witness_t(&composer, barretenberg::fr(uint256_t(inputs[0]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[0]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield b(witness_t(&composer, barretenberg::fr(uint256_t(inputs[1]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[1]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield c(witness_t(&composer, barretenberg::fr(uint256_t(inputs[2]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[2]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield d(witness_t(&composer, barretenberg::fr(uint256_t(inputs[3]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[3]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield e = (a - b) * (c - d);
        fq expected = (inputs[0] - inputs[1]) * (inputs[2] - inputs[3]);
        expected = expected.from_montgomery_form();
        uint512_t result = e.get_value();

        EXPECT_EQ(result.lo.data[0], expected.data[0]);
        EXPECT_EQ(result.lo.data[1], expected.data[1]);
        EXPECT_EQ(result.lo.data[2], expected.data[2]);
        EXPECT_EQ(result.lo.data[3], expected.data[3]);
        EXPECT_EQ(result.hi.data[0], 0ULL);
        EXPECT_EQ(result.hi.data[1], 0ULL);
        EXPECT_EQ(result.hi.data[2], 0ULL);
        EXPECT_EQ(result.hi.data[3], 0ULL);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_bigfield, test_conditional_negate)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {

        fq inputs[4]{ fq::random_element(), fq::random_element(), fq::random_element(), fq::random_element() };

        bigfield a(witness_t(&composer, barretenberg::fr(uint256_t(inputs[0]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[0]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        // bigfield b(witness_t(&composer, barretenberg::fr(uint256_t(inputs[1]).slice(0, bigfield::NUM_LIMB_BITS *
        // 2))),
        //            witness_t(&composer, barretenberg::fr(uint256_t(inputs[1]).slice(bigfield::NUM_LIMB_BITS * 2,
        //            bigfield::NUM_LIMB_BITS * 4))));

        bool_t predicate_a(witness_t(&composer, true));
        // bool_t predicate_b(witness_t(&composer, false));

        bigfield c = a.conditional_negate(predicate_a);
        bigfield d = a.conditional_negate(!predicate_a);
        bigfield e = c + d;
        uint512_t c_out = c.get_value();
        uint512_t d_out = d.get_value();
        uint512_t e_out = e.get_value();

        barretenberg::fq result_c(c_out.lo);
        barretenberg::fq result_d(d_out.lo);
        barretenberg::fq result_e(e_out.lo);

        barretenberg::fq expected_c = (-inputs[0]);
        barretenberg::fq expected_d = inputs[0];

        EXPECT_EQ(result_c, expected_c);
        EXPECT_EQ(result_d, expected_d);
        EXPECT_EQ(result_e, barretenberg::fq(0));
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_bigfield, test_group_operation)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        g1::affine_element P1(g1::element::random_element());
        g1::affine_element P2(g1::element::random_element());

        bigfield x1(witness_t(&composer, barretenberg::fr(uint256_t(P1.x).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                    witness_t(&composer,
                              barretenberg::fr(
                                  uint256_t(P1.x).slice(bigfield::NUM_LIMB_BITS * 2, bigfield::NUM_LIMB_BITS * 4))));
        bigfield y1(witness_t(&composer, barretenberg::fr(uint256_t(P1.y).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                    witness_t(&composer,
                              barretenberg::fr(
                                  uint256_t(P1.y).slice(bigfield::NUM_LIMB_BITS * 2, bigfield::NUM_LIMB_BITS * 4))));
        bigfield x2(witness_t(&composer, barretenberg::fr(uint256_t(P2.x).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                    witness_t(&composer,
                              barretenberg::fr(
                                  uint256_t(P2.x).slice(bigfield::NUM_LIMB_BITS * 2, bigfield::NUM_LIMB_BITS * 4))));
        bigfield y2(witness_t(&composer, barretenberg::fr(uint256_t(P2.y).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                    witness_t(&composer,
                              barretenberg::fr(
                                  uint256_t(P2.y).slice(bigfield::NUM_LIMB_BITS * 2, bigfield::NUM_LIMB_BITS * 4))));
        uint64_t before = composer.get_num_gates();

        bigfield lambda = (y2 - y1) / (x2 - x1);
        bigfield x3 = lambda.sqr() - (x2 + x1);
        bigfield y3 = (x1 - x3) * lambda - y1;
        uint64_t after = composer.get_num_gates();
        std::cout << "added gates = " << after - before << std::endl;
        g1::affine_element P3(g1::element(P1) + g1::element(P2));
        fq expected_x = P3.x;
        fq expected_y = P3.y;
        expected_x = expected_x.from_montgomery_form();
        expected_y = expected_y.from_montgomery_form();
        uint512_t result_x = x3.get_value() % bigfield::modulus_u512;
        uint512_t result_y = y3.get_value() % bigfield::modulus_u512;
        EXPECT_EQ(result_x.lo.data[0], expected_x.data[0]);
        EXPECT_EQ(result_x.lo.data[1], expected_x.data[1]);
        EXPECT_EQ(result_x.lo.data[2], expected_x.data[2]);
        EXPECT_EQ(result_x.lo.data[3], expected_x.data[3]);
        EXPECT_EQ(result_y.lo.data[0], expected_y.data[0]);
        EXPECT_EQ(result_y.lo.data[1], expected_y.data[1]);
        EXPECT_EQ(result_y.lo.data[2], expected_y.data[2]);
        EXPECT_EQ(result_y.lo.data[3], expected_y.data[3]);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_bigfield, test_reduce)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 10;
    for (size_t i = 0; i < num_repetitions; ++i) {

        fq inputs[4]{ fq::random_element(), fq::random_element(), fq::random_element(), fq::random_element() };

        bigfield a(witness_t(&composer, barretenberg::fr(uint256_t(inputs[0]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[0]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));
        bigfield b(witness_t(&composer, barretenberg::fr(uint256_t(inputs[1]).slice(0, bigfield::NUM_LIMB_BITS * 2))),
                   witness_t(&composer,
                             barretenberg::fr(uint256_t(inputs[1]).slice(bigfield::NUM_LIMB_BITS * 2,
                                                                         bigfield::NUM_LIMB_BITS * 4))));

        bigfield c = a;
        fq expected = inputs[0];
        for (size_t i = 0; i < 16; ++i) {
            c = b * b + c;
            expected = inputs[1] * inputs[1] + expected;
        }
        // bigfield c = a + a + a + a - b - b - b - b;
        c.self_reduce();
        fq result = fq(c.get_value().lo);
        EXPECT_EQ(result, expected);
        EXPECT_EQ(c.get_value().get_msb() < 254, true);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
} // namespace test_stdlib_bigfield
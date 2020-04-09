#include <gtest/gtest.h>

#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../bool/bool.hpp"
#include "../field/field.hpp"

#include <plonk/composer/turbo_composer.hpp>
#include <plonk/proof_system/prover/prover.hpp>
#include <plonk/proof_system/verifier/verifier.hpp>
#include <plonk/proof_system/widgets/arithmetic_widget.hpp>

#include <polynomials/polynomial_arithmetic.hpp>

#include <ecc/curves/bn254/fq.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <ecc/curves/bn254/g1.hpp>
#include <ecc/curves/secp256r1/secp256r1.hpp>

#include <memory>
#include <numeric/random/engine.hpp>

using namespace barretenberg;
using namespace plonk;

namespace plonk {
namespace stdlib {
namespace bn254 {
typedef typename plonk::stdlib::bigfield<typename waffle::TurboComposer, typename barretenberg::Bn254FqParams> fq;
typedef typename plonk::stdlib::bigfield<waffle::TurboComposer, barretenberg::Bn254FrParams> fr;
typedef typename plonk::stdlib::element<waffle::TurboComposer, fq, fr, barretenberg::g1>
    g1;

} // namespace bn254
namespace alt_bn254 {
typedef typename plonk::stdlib::bigfield<typename waffle::TurboComposer, typename barretenberg::Bn254FqParams> fq;
typedef typename plonk::stdlib::field_t<typename waffle::TurboComposer> fr;
typedef typename plonk::stdlib::element<waffle::TurboComposer, fq, fr, barretenberg::g1>
    g1;
} // namespace alt_bn254
namespace secp256r {
typedef typename plonk::stdlib::bigfield<waffle::TurboComposer, secp256r1::Secp256r1FqParams> fq;
typedef typename plonk::stdlib::bigfield<waffle::TurboComposer, secp256r1::Secp256r1FrParams> fr;
typedef typename plonk::stdlib::element<waffle::TurboComposer, fq, fr, secp256r1::g1> g1;

} // namespace secp256r

} // namespace stdlib
} // namespace plonk
typedef stdlib::bool_t<waffle::TurboComposer> bool_t;
typedef stdlib::field_t<waffle::TurboComposer> field_t;
typedef stdlib::witness_t<waffle::TurboComposer> witness_t;
typedef stdlib::public_witness_t<waffle::TurboComposer> public_witness_t;

stdlib::bn254::g1 convert_inputs(waffle::TurboComposer* ctx, const barretenberg::g1::affine_element& input)
{
    uint256_t x_u256(input.x);
    uint256_t y_u256(input.y);

    stdlib::bn254::fq x(witness_t(ctx, barretenberg::fr(x_u256.slice(0, stdlib::bn254::fq::NUM_LIMB_BITS * 2))),
                        witness_t(ctx,
                                  barretenberg::fr(x_u256.slice(stdlib::bn254::fq::NUM_LIMB_BITS * 2,
                                                                stdlib::bn254::fq::NUM_LIMB_BITS * 4))));
    stdlib::bn254::fq y(witness_t(ctx, barretenberg::fr(y_u256.slice(0, stdlib::bn254::fq::NUM_LIMB_BITS * 2))),
                        witness_t(ctx,
                                  barretenberg::fr(y_u256.slice(stdlib::bn254::fq::NUM_LIMB_BITS * 2,
                                                                stdlib::bn254::fq::NUM_LIMB_BITS * 4))));

    return stdlib::bn254::g1(x, y);
}

stdlib::alt_bn254::g1 convert_inputs_alt_bn254(waffle::TurboComposer* ctx,
                                               const barretenberg::g1::affine_element& input)
{
    uint256_t x_u256(input.x);
    uint256_t y_u256(input.y);

    stdlib::alt_bn254::fq x(witness_t(ctx, barretenberg::fr(x_u256.slice(0, stdlib::alt_bn254::fq::NUM_LIMB_BITS * 2))),
                            witness_t(ctx,
                                      barretenberg::fr(x_u256.slice(stdlib::alt_bn254::fq::NUM_LIMB_BITS * 2,
                                                                    stdlib::alt_bn254::fq::NUM_LIMB_BITS * 4))));
    stdlib::alt_bn254::fq y(witness_t(ctx, barretenberg::fr(y_u256.slice(0, stdlib::alt_bn254::fq::NUM_LIMB_BITS * 2))),
                            witness_t(ctx,
                                      barretenberg::fr(y_u256.slice(stdlib::alt_bn254::fq::NUM_LIMB_BITS * 2,
                                                                    stdlib::alt_bn254::fq::NUM_LIMB_BITS * 4))));

    return stdlib::alt_bn254::g1(x, y);
}

stdlib::bn254::fr convert_inputs(waffle::TurboComposer* ctx, const barretenberg::fr& scalar)
{
    uint256_t scalar_u256(scalar);

    stdlib::bn254::fr x(witness_t(ctx, barretenberg::fr(scalar_u256.slice(0, stdlib::bn254::fq::NUM_LIMB_BITS * 2))),
                        witness_t(ctx,
                                  barretenberg::fr(scalar_u256.slice(stdlib::bn254::fq::NUM_LIMB_BITS * 2,
                                                                     stdlib::bn254::fq::NUM_LIMB_BITS * 4))));

    return x;
}

stdlib::secp256r::g1 convert_inputs_secp256r1(waffle::TurboComposer* ctx, const secp256r1::g1::affine_element& input)
{
    uint256_t x_u256(input.x);
    uint256_t y_u256(input.y);

    stdlib::secp256r::fq x(witness_t(ctx, barretenberg::fr(x_u256.slice(0, stdlib::secp256r::fq::NUM_LIMB_BITS * 2))),
                           witness_t(ctx,
                                     barretenberg::fr(x_u256.slice(stdlib::secp256r::fq::NUM_LIMB_BITS * 2,
                                                                   stdlib::secp256r::fq::NUM_LIMB_BITS * 4))));
    stdlib::secp256r::fq y(witness_t(ctx, barretenberg::fr(y_u256.slice(0, stdlib::secp256r::fq::NUM_LIMB_BITS * 2))),
                           witness_t(ctx,
                                     barretenberg::fr(y_u256.slice(stdlib::secp256r::fq::NUM_LIMB_BITS * 2,
                                                                   stdlib::secp256r::fq::NUM_LIMB_BITS * 4))));

    return stdlib::secp256r::g1(x, y);
}

stdlib::secp256r::fr convert_inputs_secp256r1(waffle::TurboComposer* ctx, const secp256r1::fr& scalar)
{
    uint256_t scalar_u256(scalar);

    stdlib::secp256r::fr x(
        witness_t(ctx, barretenberg::fr(scalar_u256.slice(0, stdlib::secp256r::fq::NUM_LIMB_BITS * 2))),
        witness_t(ctx,
                  barretenberg::fr(scalar_u256.slice(stdlib::secp256r::fq::NUM_LIMB_BITS * 2,
                                                     stdlib::secp256r::fq::NUM_LIMB_BITS * 4))));

    return x;
}

TEST(stdlib_biggroup, test_add)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 10;
    for (size_t i = 0; i < num_repetitions; ++i) {
        barretenberg::g1::affine_element input_a(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_b(barretenberg::g1::element::random_element());

        stdlib::bn254::g1 a = convert_inputs(&composer, input_a);
        stdlib::bn254::g1 b = convert_inputs(&composer, input_b);

        stdlib::bn254::g1 c = a + b;

        barretenberg::g1::affine_element c_expected(barretenberg::g1::element(input_a) +
                                                    barretenberg::g1::element(input_b));

        uint256_t c_x_u256 = c.x.get_value().lo;
        uint256_t c_y_u256 = c.y.get_value().lo;

        barretenberg::fq c_x_result(c_x_u256);
        barretenberg::fq c_y_result(c_y_u256);

        EXPECT_EQ(c_x_result, c_expected.x);
        EXPECT_EQ(c_y_result, c_expected.y);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_sub)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 10;
    for (size_t i = 0; i < num_repetitions; ++i) {
        barretenberg::g1::affine_element input_a(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_b(barretenberg::g1::element::random_element());

        stdlib::bn254::g1 a = convert_inputs(&composer, input_a);
        stdlib::bn254::g1 b = convert_inputs(&composer, input_b);

        stdlib::bn254::g1 c = a - b;

        barretenberg::g1::affine_element c_expected(barretenberg::g1::element(input_a) -
                                                    barretenberg::g1::element(input_b));

        uint256_t c_x_u256 = c.x.get_value().lo;
        uint256_t c_y_u256 = c.y.get_value().lo;

        barretenberg::fq c_x_result(c_x_u256);
        barretenberg::fq c_y_result(c_y_u256);

        EXPECT_EQ(c_x_result, c_expected.x);
        EXPECT_EQ(c_y_result, c_expected.y);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_dbl)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 10;
    for (size_t i = 0; i < num_repetitions; ++i) {
        barretenberg::g1::affine_element input_a(barretenberg::g1::element::random_element());

        stdlib::bn254::g1 a = convert_inputs(&composer, input_a);

        stdlib::bn254::g1 c = a.dbl();

        barretenberg::g1::affine_element c_expected(barretenberg::g1::element(input_a).dbl());

        uint256_t c_x_u256 = c.x.get_value().lo;
        uint256_t c_y_u256 = c.y.get_value().lo;

        barretenberg::fq c_x_result(c_x_u256);
        barretenberg::fq c_y_result(c_y_u256);

        EXPECT_EQ(c_x_result, c_expected.x);
        EXPECT_EQ(c_y_result, c_expected.y);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_montgomery_ladder)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        barretenberg::g1::affine_element input_a(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_b(barretenberg::g1::element::random_element());

        stdlib::bn254::g1 a = convert_inputs(&composer, input_a);
        stdlib::bn254::g1 b = convert_inputs(&composer, input_b);

        stdlib::bn254::g1 c = a.montgomery_ladder(b);

        barretenberg::g1::affine_element c_expected(barretenberg::g1::element(input_a).dbl() +
                                                    barretenberg::g1::element(input_b));

        uint256_t c_x_u256 = c.x.get_value().lo;
        uint256_t c_y_u256 = c.y.get_value().lo;

        barretenberg::fq c_x_result(c_x_u256);
        barretenberg::fq c_y_result(c_y_u256);

        EXPECT_EQ(c_x_result, c_expected.x);
        EXPECT_EQ(c_y_result, c_expected.y);
    }
    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_mul)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        barretenberg::g1::affine_element input(barretenberg::g1::element::random_element());
        barretenberg::fr scalar(barretenberg::fr::random_element());
        if ((scalar.from_montgomery_form().get_bit(0) & 1) == 1) {
            scalar -= barretenberg::fr(1); // make sure to add skew
        }
        stdlib::bn254::g1 P = convert_inputs(&composer, input);
        stdlib::bn254::fr x = convert_inputs(&composer, scalar);

        stdlib::bn254::g1 c = P * x;
        barretenberg::g1::affine_element c_expected(barretenberg::g1::element(input) * scalar);

        barretenberg::fq c_x_result(c.x.get_value().lo);
        barretenberg::fq c_y_result(c.y.get_value().lo);

        EXPECT_EQ(c_x_result, c_expected.x);
        EXPECT_EQ(c_y_result, c_expected.y);
    }
    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    std::cout << "creating verifier " << std::endl;
    waffle::TurboVerifier verifier = composer.create_verifier();
    std::cout << "creating proof " << std::endl;
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_twin_mul)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        barretenberg::g1::affine_element input_a(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_b(barretenberg::g1::element::random_element());
        barretenberg::fr scalar_a(barretenberg::fr::random_element());
        barretenberg::fr scalar_b(barretenberg::fr::random_element());
        if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
            scalar_a -= barretenberg::fr(1); // make a have skew
        }
        if ((scalar_b.from_montgomery_form().get_bit(0) & 1) == 0) {
            scalar_b += barretenberg::fr(1); // make b not have skew
        }
        stdlib::bn254::g1 P_a = convert_inputs(&composer, input_a);
        stdlib::bn254::fr x_a = convert_inputs(&composer, scalar_a);
        stdlib::bn254::g1 P_b = convert_inputs(&composer, input_b);
        stdlib::bn254::fr x_b = convert_inputs(&composer, scalar_b);

        stdlib::bn254::g1 c = stdlib::bn254::g1::batch_mul({ P_a, P_b }, { x_a, x_b });
        barretenberg::g1::element input_c = (barretenberg::g1::element(input_a) * scalar_a);
        barretenberg::g1::element input_d = (barretenberg::g1::element(input_b) * scalar_b);
        barretenberg::g1::affine_element expected(input_c + input_d);
        barretenberg::fq c_x_result(c.x.get_value().lo);
        barretenberg::fq c_y_result(c.y.get_value().lo);

        EXPECT_EQ(c_x_result, expected.x);
        EXPECT_EQ(c_y_result, expected.y);
    }
    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    std::cout << "creating verifier " << std::endl;
    waffle::TurboVerifier verifier = composer.create_verifier();
    std::cout << "creating proof " << std::endl;
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_triple_mul)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        barretenberg::g1::affine_element input_a(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_b(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_c(barretenberg::g1::element::random_element());
        barretenberg::fr scalar_a(barretenberg::fr::random_element());
        barretenberg::fr scalar_b(barretenberg::fr::random_element());
        barretenberg::fr scalar_c(barretenberg::fr::random_element());
        if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
            scalar_a -= barretenberg::fr(1); // make a have skew
        }
        if ((scalar_b.from_montgomery_form().get_bit(0) & 1) == 0) {
            scalar_b += barretenberg::fr(1); // make b not have skew
        }
        stdlib::bn254::g1 P_a = convert_inputs(&composer, input_a);
        stdlib::bn254::fr x_a = convert_inputs(&composer, scalar_a);
        stdlib::bn254::g1 P_b = convert_inputs(&composer, input_b);
        stdlib::bn254::fr x_b = convert_inputs(&composer, scalar_b);
        stdlib::bn254::g1 P_c = convert_inputs(&composer, input_c);
        stdlib::bn254::fr x_c = convert_inputs(&composer, scalar_c);

        stdlib::bn254::g1 c = stdlib::bn254::g1::batch_mul({ P_a, P_b, P_c }, { x_a, x_b, x_c });
        barretenberg::g1::element input_e = (barretenberg::g1::element(input_a) * scalar_a);
        barretenberg::g1::element input_f = (barretenberg::g1::element(input_b) * scalar_b);
        barretenberg::g1::element input_g = (barretenberg::g1::element(input_c) * scalar_c);

        barretenberg::g1::affine_element expected(input_e + input_f + input_g);
        barretenberg::fq c_x_result(c.x.get_value().lo);
        barretenberg::fq c_y_result(c.y.get_value().lo);

        EXPECT_EQ(c_x_result, expected.x);
        EXPECT_EQ(c_y_result, expected.y);
    }
    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    std::cout << "creating verifier " << std::endl;
    waffle::TurboVerifier verifier = composer.create_verifier();
    std::cout << "creating proof " << std::endl;
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_quad_mul)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        barretenberg::g1::affine_element input_a(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_b(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_c(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_d(barretenberg::g1::element::random_element());
        barretenberg::fr scalar_a(barretenberg::fr::random_element());
        barretenberg::fr scalar_b(barretenberg::fr::random_element());
        barretenberg::fr scalar_c(barretenberg::fr::random_element());
        barretenberg::fr scalar_d(barretenberg::fr::random_element());
        if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
            scalar_a -= barretenberg::fr(1); // make a have skew
        }
        if ((scalar_b.from_montgomery_form().get_bit(0) & 1) == 0) {
            scalar_b += barretenberg::fr(1); // make b not have skew
        }
        stdlib::bn254::g1 P_a = convert_inputs(&composer, input_a);
        stdlib::bn254::fr x_a = convert_inputs(&composer, scalar_a);
        stdlib::bn254::g1 P_b = convert_inputs(&composer, input_b);
        stdlib::bn254::fr x_b = convert_inputs(&composer, scalar_b);
        stdlib::bn254::g1 P_c = convert_inputs(&composer, input_c);
        stdlib::bn254::fr x_c = convert_inputs(&composer, scalar_c);
        stdlib::bn254::g1 P_d = convert_inputs(&composer, input_d);
        stdlib::bn254::fr x_d = convert_inputs(&composer, scalar_d);

        stdlib::bn254::g1 c = stdlib::bn254::g1::batch_mul({ P_a, P_b, P_c, P_d }, { x_a, x_b, x_c, x_d });
        barretenberg::g1::element input_e = (barretenberg::g1::element(input_a) * scalar_a);
        barretenberg::g1::element input_f = (barretenberg::g1::element(input_b) * scalar_b);
        barretenberg::g1::element input_g = (barretenberg::g1::element(input_c) * scalar_c);
        barretenberg::g1::element input_h = (barretenberg::g1::element(input_d) * scalar_d);

        barretenberg::g1::affine_element expected(input_e + input_f + input_g + input_h);
        barretenberg::fq c_x_result(c.x.get_value().lo);
        barretenberg::fq c_y_result(c.y.get_value().lo);

        EXPECT_EQ(c_x_result, expected.x);
        EXPECT_EQ(c_y_result, expected.y);
    }
    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    std::cout << "creating verifier " << std::endl;
    waffle::TurboVerifier verifier = composer.create_verifier();
    std::cout << "creating proof " << std::endl;
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_quad_mul_alt_bn254)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        barretenberg::g1::affine_element input_a(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_b(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_c(barretenberg::g1::element::random_element());
        barretenberg::g1::affine_element input_d(barretenberg::g1::element::random_element());
        barretenberg::fr scalar_a(barretenberg::fr::random_element());
        barretenberg::fr scalar_b(barretenberg::fr::random_element());
        barretenberg::fr scalar_c(barretenberg::fr::random_element());
        barretenberg::fr scalar_d(barretenberg::fr::random_element());
        if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
            scalar_a -= barretenberg::fr(1); // make a have skew
        }
        if ((scalar_b.from_montgomery_form().get_bit(0) & 1) == 0) {
            scalar_b += barretenberg::fr(1); // make b not have skew
        }
        stdlib::alt_bn254::g1 P_a = convert_inputs_alt_bn254(&composer, input_a);
        stdlib::alt_bn254::fr x_a = witness_t(&composer, scalar_a);
        stdlib::alt_bn254::g1 P_b = convert_inputs_alt_bn254(&composer, input_b);
        stdlib::alt_bn254::fr x_b = witness_t(&composer, scalar_b);
        stdlib::alt_bn254::g1 P_c = convert_inputs_alt_bn254(&composer, input_c);
        stdlib::alt_bn254::fr x_c = witness_t(&composer, scalar_c);
        stdlib::alt_bn254::g1 P_d = convert_inputs_alt_bn254(&composer, input_d);
        stdlib::alt_bn254::fr x_d = witness_t(&composer, scalar_d);

        stdlib::alt_bn254::g1 c = stdlib::alt_bn254::g1::batch_mul({ P_a, P_b, P_c, P_d }, { x_a, x_b, x_c, x_d });
        barretenberg::g1::element input_e = (barretenberg::g1::element(input_a) * scalar_a);
        barretenberg::g1::element input_f = (barretenberg::g1::element(input_b) * scalar_b);
        barretenberg::g1::element input_g = (barretenberg::g1::element(input_c) * scalar_c);
        barretenberg::g1::element input_h = (barretenberg::g1::element(input_d) * scalar_d);

        barretenberg::g1::affine_element expected(input_e + input_f + input_g + input_h);
        barretenberg::fq c_x_result(c.x.get_value().lo);
        barretenberg::fq c_y_result(c.y.get_value().lo);

        EXPECT_EQ(c_x_result, expected.x);
        EXPECT_EQ(c_y_result, expected.y);
    }
    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    std::cout << "creating verifier " << std::endl;
    waffle::TurboVerifier verifier = composer.create_verifier();
    std::cout << "creating proof " << std::endl;
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_one)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        barretenberg::fr scalar_a(barretenberg::fr::random_element());
        if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
            scalar_a -= barretenberg::fr(1); // make a have skew
        }
        stdlib::bn254::g1 P_a = stdlib::bn254::g1::one(&composer);
        // std::cout << "a" << std::endl;
        // P_a = P_a.dbl();
        // std::cout << "b" << std::endl;
        stdlib::bn254::fr x_a = convert_inputs(&composer, scalar_a);
        stdlib::bn254::g1 c = P_a * x_a;
        barretenberg::g1::affine_element expected(barretenberg::g1::one * scalar_a);
        barretenberg::fq c_x_result(c.x.get_value().lo);
        barretenberg::fq c_y_result(c.y.get_value().lo);

        EXPECT_EQ(c_x_result, expected.x);
        EXPECT_EQ(c_y_result, expected.y);
    }
    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    std::cout << "creating verifier " << std::endl;
    waffle::TurboVerifier verifier = composer.create_verifier();
    std::cout << "creating proof " << std::endl;
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_one_secp256r1)
{
    waffle::TurboComposer composer = waffle::TurboComposer();
    size_t num_repetitions = 1;
    for (size_t i = 0; i < num_repetitions; ++i) {
        secp256r1::fr scalar_a(secp256r1::fr::random_element());
        if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
            scalar_a -= secp256r1::fr(1); // make a have skew
        }
        stdlib::secp256r::g1 P_a = stdlib::secp256r::g1::one(&composer);
        // std::cout << "a" << std::endl;
        // P_a = P_a.dbl();
        // std::cout << "b" << std::endl;
        stdlib::secp256r::fr x_a = convert_inputs_secp256r1(&composer, scalar_a);
        stdlib::secp256r::g1 c = P_a * x_a;
        secp256r1::g1::affine_element expected(secp256r1::g1::one * scalar_a);
        secp256r1::fq c_x_result(c.x.get_value().lo);
        secp256r1::fq c_y_result(c.y.get_value().lo);

        EXPECT_EQ(c_x_result, expected.x);
        EXPECT_EQ(c_y_result, expected.y);
    }
    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    std::cout << "creating verifier " << std::endl;
    waffle::TurboVerifier verifier = composer.create_verifier();
    std::cout << "creating proof " << std::endl;
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_batch_mul)
{
    const size_t num_points = 5;
    waffle::TurboComposer composer = waffle::TurboComposer();
    std::vector<barretenberg::g1::affine_element> points;
    std::vector<barretenberg::fr> scalars;
    for (size_t i = 0; i < num_points; ++i) {
        points.push_back(barretenberg::g1::affine_element(barretenberg::g1::element::random_element()));
        scalars.push_back(barretenberg::fr::random_element());
    }

    std::vector<stdlib::alt_bn254::g1> circuit_points;
    std::vector<stdlib::alt_bn254::fr> circuit_scalars;
    for (size_t i = 0; i < num_points; ++i) {
        circuit_points.push_back(convert_inputs_alt_bn254(&composer, points[i]));
        circuit_scalars.push_back(witness_t(&composer, scalars[i]));
    }

    stdlib::alt_bn254::g1 result_point = stdlib::alt_bn254::g1::batch_mul(circuit_points, circuit_scalars);

    barretenberg::g1::element expected_point = barretenberg::g1::one;
    expected_point.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        expected_point += (barretenberg::g1::element(points[i]) * scalars[i]);
    }
    expected_point = expected_point.normalize();
    barretenberg::fq result_x(result_point.x.get_value().lo);
    barretenberg::fq result_y(result_point.y.get_value().lo);

    EXPECT_EQ(result_x, expected_point.x);
    EXPECT_EQ(result_y, expected_point.y);

    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    std::cout << "creating verifier " << std::endl;
    waffle::TurboVerifier verifier = composer.create_verifier();
    std::cout << "creating proof " << std::endl;
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_batch_mul_short_scalars)
{
    const size_t num_points = 11;
    waffle::TurboComposer composer = waffle::TurboComposer();
    std::vector<barretenberg::g1::affine_element> points;
    std::vector<barretenberg::fr> scalars;
    for (size_t i = 0; i < num_points; ++i) {
        points.push_back(barretenberg::g1::affine_element(barretenberg::g1::element::random_element()));
        uint256_t scalar_raw = barretenberg::fr::random_element();
        scalar_raw.data[2] = 0ULL;
        scalar_raw.data[3] = 0ULL;
        scalars.push_back(barretenberg::fr(scalar_raw));
    }

    std::vector<stdlib::alt_bn254::g1> circuit_points;
    std::vector<stdlib::alt_bn254::fr> circuit_scalars;
    for (size_t i = 0; i < num_points; ++i) {
        circuit_points.push_back(convert_inputs_alt_bn254(&composer, points[i]));
        circuit_scalars.push_back(witness_t(&composer, scalars[i]));
    }

    stdlib::alt_bn254::g1 result_point = stdlib::alt_bn254::g1::batch_mul(circuit_points, circuit_scalars, 128);

    barretenberg::g1::element expected_point = barretenberg::g1::one;
    expected_point.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        expected_point += (barretenberg::g1::element(points[i]) * scalars[i]);
    }
    expected_point = expected_point.normalize();
    barretenberg::fq result_x(result_point.x.get_value().lo);
    barretenberg::fq result_y(result_point.y.get_value().lo);

    EXPECT_EQ(result_x, expected_point.x);
    EXPECT_EQ(result_y, expected_point.y);

    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    std::cout << "creating verifier " << std::endl;
    waffle::TurboVerifier verifier = composer.create_verifier();
    std::cout << "creating proof " << std::endl;
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_biggroup, test_mixed_batch_mul)
{
    const size_t num_big_points = 10;
    const size_t num_small_points = 11;
    waffle::TurboComposer composer = waffle::TurboComposer();
    std::vector<barretenberg::g1::affine_element> big_points;
    std::vector<barretenberg::fr> big_scalars;
    std::vector<barretenberg::g1::affine_element> small_points;
    std::vector<barretenberg::fr> small_scalars;

    for (size_t i = 0; i < num_big_points; ++i) {
        big_points.push_back(barretenberg::g1::affine_element(barretenberg::g1::element::random_element()));
        big_scalars.push_back(barretenberg::fr::random_element());
    }
    for (size_t i = 0; i < num_small_points; ++i) {
        small_points.push_back(barretenberg::g1::affine_element(barretenberg::g1::element::random_element()));
        uint256_t scalar_raw = barretenberg::fr::random_element();
        scalar_raw.data[2] = 0ULL;
        scalar_raw.data[3] = 0ULL;
        small_scalars.push_back(barretenberg::fr(scalar_raw));
    }

    std::vector<stdlib::alt_bn254::g1> big_circuit_points;
    std::vector<stdlib::alt_bn254::fr> big_circuit_scalars;
    std::vector<stdlib::alt_bn254::g1> small_circuit_points;
    std::vector<stdlib::alt_bn254::fr> small_circuit_scalars;
    for (size_t i = 0; i < num_big_points; ++i) {
        big_circuit_points.push_back(convert_inputs_alt_bn254(&composer, big_points[i]));
        big_circuit_scalars.push_back(witness_t(&composer, big_scalars[i]));
    }
    for (size_t i = 0; i < num_small_points; ++i) {
        small_circuit_points.push_back(convert_inputs_alt_bn254(&composer, small_points[i]));
        small_circuit_scalars.push_back(witness_t(&composer, small_scalars[i]));
    }
    stdlib::alt_bn254::g1 result_point = stdlib::alt_bn254::g1::mixed_batch_mul(
        big_circuit_points, big_circuit_scalars, small_circuit_points, small_circuit_scalars, 128);

    barretenberg::g1::element expected_point = barretenberg::g1::one;
    expected_point.self_set_infinity();
    for (size_t i = 0; i < num_big_points; ++i) {
        expected_point += (barretenberg::g1::element(big_points[i]) * big_scalars[i]);
    }
    for (size_t i = 0; i < num_small_points; ++i) {
        expected_point += (barretenberg::g1::element(small_points[i]) * small_scalars[i]);
    }
    expected_point = expected_point.normalize();
    barretenberg::fq result_x(result_point.x.get_value().lo);
    barretenberg::fq result_y(result_point.y.get_value().lo);

    EXPECT_EQ(result_x, expected_point.x);
    EXPECT_EQ(result_y, expected_point.y);

    std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
    waffle::TurboProver prover = composer.create_prover();
    std::cout << "creating verifier " << std::endl;
    waffle::TurboVerifier verifier = composer.create_verifier();
    std::cout << "creating proof " << std::endl;
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
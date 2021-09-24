#include <common/test.hpp>

#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../bool/bool.hpp"
#include "../field/field.hpp"

#include <plonk/proof_system/prover/prover.hpp>
#include <plonk/proof_system/verifier/verifier.hpp>

#include <polynomials/polynomial_arithmetic.hpp>

#include <stdlib/primitives/curves/bn254.hpp>
#include <stdlib/primitives/curves/secp256r1.hpp>
#include <ecc/curves/secp256r1/secp256r1.hpp>

#include <memory>
#include <numeric/random/engine.hpp>

using namespace barretenberg;
using namespace plonk;

// SEE BOTTOM FOR REMNANTS OF TESTS FOR PLOOKUP AND NOTE ON UPDATING THOSE

template <typename Composer> class StdlibBiggroup : public testing::Test {
    typedef stdlib::bn254<Composer> bn254;
    typedef stdlib::secp256r1_ct<Composer> secp256r1_ct;
    typedef typename bn254::fr_ct fr_ct;
    typedef typename bn254::bigfr_ct bigfr_ct;
    typedef typename bn254::g1_ct g1_ct;
    typedef typename bn254::g1_bigfr_ct g1_bigfr_ct;
    typedef typename bn254::fq_ct fq_ct;
    typedef typename bn254::public_witness_ct public_witness_ct;
    typedef typename bn254::witness_ct witness_ct;

    static g1_bigfr_ct convert_inputs_bigfr(Composer* ctx, const g1::affine_element& input)
    {
        uint256_t x_u256(input.x);
        uint256_t y_u256(input.y);

        fq_ct x(witness_ct(ctx, fr(x_u256.slice(0, fq_ct::NUM_LIMB_BITS * 2))),
                witness_ct(ctx, fr(x_u256.slice(fq_ct::NUM_LIMB_BITS * 2, fq_ct::NUM_LIMB_BITS * 4))));
        fq_ct y(witness_ct(ctx, fr(y_u256.slice(0, fq_ct::NUM_LIMB_BITS * 2))),
                witness_ct(ctx, fr(y_u256.slice(fq_ct::NUM_LIMB_BITS * 2, fq_ct::NUM_LIMB_BITS * 4))));

        return g1_bigfr_ct(x, y);
    }

    static bigfr_ct convert_inputs_bigfr(Composer* ctx, const fr& scalar)
    {
        uint256_t scalar_u256(scalar);

        bigfr_ct x(witness_ct(ctx, fr(scalar_u256.slice(0, fq_ct::NUM_LIMB_BITS * 2))),
                   witness_ct(ctx, fr(scalar_u256.slice(fq_ct::NUM_LIMB_BITS * 2, fq_ct::NUM_LIMB_BITS * 4))));

        return x;
    }

    static g1_ct convert_inputs(Composer* ctx, const g1::affine_element& input)
    {
        uint256_t x_u256(input.x);
        uint256_t y_u256(input.y);

        fq_ct x(witness_ct(ctx, fr(x_u256.slice(0, fq_ct::NUM_LIMB_BITS * 2))),
                witness_ct(ctx, fr(x_u256.slice(fq_ct::NUM_LIMB_BITS * 2, fq_ct::NUM_LIMB_BITS * 4))));
        fq_ct y(witness_ct(ctx, fr(y_u256.slice(0, fq_ct::NUM_LIMB_BITS * 2))),
                witness_ct(ctx, fr(y_u256.slice(fq_ct::NUM_LIMB_BITS * 2, fq_ct::NUM_LIMB_BITS * 4))));

        return g1_ct(x, y);
    }

    static typename secp256r1_ct::bigfr_ct convert_inputs_bigfr_secp256r1(Composer* ctx, const secp256r1::fr& scalar)
    {
        uint256_t scalar_u256(scalar);

        typename secp256r1_ct::bigfr_ct x(
            witness_ct(ctx, fr(scalar_u256.slice(0, secp256r1_ct::fq_ct::NUM_LIMB_BITS * 2))),
            witness_ct(
                ctx,
                fr(scalar_u256.slice(secp256r1_ct::fq_ct::NUM_LIMB_BITS * 2, secp256r1_ct::fq_ct::NUM_LIMB_BITS * 4))));

        return x;
    }

  public:
    static void test_add()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 10;
        for (size_t i = 0; i < num_repetitions; ++i) {
            g1::affine_element input_a(g1::element::random_element());
            g1::affine_element input_b(g1::element::random_element());

            g1_bigfr_ct a = convert_inputs_bigfr(&composer, input_a);
            g1_bigfr_ct b = convert_inputs_bigfr(&composer, input_b);

            g1_bigfr_ct c = a + b;

            g1::affine_element c_expected(g1::element(input_a) + g1::element(input_b));

            uint256_t c_x_u256 = c.x.get_value().lo;
            uint256_t c_y_u256 = c.y.get_value().lo;

            fq c_x_result(c_x_u256);
            fq c_y_result(c_y_u256);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_sub()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 10;
        for (size_t i = 0; i < num_repetitions; ++i) {
            g1::affine_element input_a(g1::element::random_element());
            g1::affine_element input_b(g1::element::random_element());

            g1_bigfr_ct a = convert_inputs_bigfr(&composer, input_a);
            g1_bigfr_ct b = convert_inputs_bigfr(&composer, input_b);

            g1_bigfr_ct c = a - b;

            g1::affine_element c_expected(g1::element(input_a) - g1::element(input_b));

            uint256_t c_x_u256 = c.x.get_value().lo;
            uint256_t c_y_u256 = c.y.get_value().lo;

            fq c_x_result(c_x_u256);
            fq c_y_result(c_y_u256);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_dbl()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 10;
        for (size_t i = 0; i < num_repetitions; ++i) {
            g1::affine_element input_a(g1::element::random_element());

            g1_bigfr_ct a = convert_inputs_bigfr(&composer, input_a);

            g1_bigfr_ct c = a.dbl();

            g1::affine_element c_expected(g1::element(input_a).dbl());

            uint256_t c_x_u256 = c.x.get_value().lo;
            uint256_t c_y_u256 = c.y.get_value().lo;

            fq c_x_result(c_x_u256);
            fq c_y_result(c_y_u256);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_montgomery_ladder()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            g1::affine_element input_a(g1::element::random_element());
            g1::affine_element input_b(g1::element::random_element());

            g1_bigfr_ct a = convert_inputs_bigfr(&composer, input_a);
            g1_bigfr_ct b = convert_inputs_bigfr(&composer, input_b);

            g1_bigfr_ct c = a.montgomery_ladder(b);

            g1::affine_element c_expected(g1::element(input_a).dbl() + g1::element(input_b));

            uint256_t c_x_u256 = c.x.get_value().lo;
            uint256_t c_y_u256 = c.y.get_value().lo;

            fq c_x_result(c_x_u256);
            fq c_y_result(c_y_u256);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }
        auto prover = composer.create_prover();
        auto verifier = composer.create_verifier();
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_mul()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            g1::affine_element input(g1::element::random_element());
            fr scalar(fr::random_element());
            if ((scalar.from_montgomery_form().get_bit(0) & 1) == 1) {
                scalar -= fr(1); // make sure to add skew
            }
            g1_ct P = convert_inputs(&composer, input);
            fr_ct x = witness_ct(&composer, scalar);

            std::cout << "gates before mul " << composer.get_num_gates() << std::endl;
            g1_ct c = P * x;
            std::cout << "composer aftr mul " << composer.get_num_gates() << std::endl;
            g1::affine_element c_expected(g1::element(input) * scalar);

            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }
        std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
        auto prover = composer.create_prover();
        std::cout << "creating verifier " << std::endl;
        auto verifier = composer.create_verifier();
        std::cout << "creating proof " << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_twin_mul()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            g1::affine_element input_a(g1::element::random_element());
            g1::affine_element input_b(g1::element::random_element());
            fr scalar_a(fr::random_element());
            fr scalar_b(fr::random_element());
            if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // make a have skew
            }
            if ((scalar_b.from_montgomery_form().get_bit(0) & 1) == 0) {
                scalar_b += fr(1); // make b not have skew
            }
            g1_bigfr_ct P_a = convert_inputs_bigfr(&composer, input_a);
            bigfr_ct x_a = convert_inputs_bigfr(&composer, scalar_a);
            g1_bigfr_ct P_b = convert_inputs_bigfr(&composer, input_b);
            bigfr_ct x_b = convert_inputs_bigfr(&composer, scalar_b);

            g1_bigfr_ct c = g1_bigfr_ct::batch_mul({ P_a, P_b }, { x_a, x_b });
            g1::element input_c = (g1::element(input_a) * scalar_a);
            g1::element input_d = (g1::element(input_b) * scalar_b);
            g1::affine_element expected(input_c + input_d);
            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, expected.x);
            EXPECT_EQ(c_y_result, expected.y);
        }
        std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
        auto prover = composer.create_prover();
        std::cout << "creating verifier " << std::endl;
        auto verifier = composer.create_verifier();
        std::cout << "creating proof " << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_triple_mul()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            g1::affine_element input_a(g1::element::random_element());
            g1::affine_element input_b(g1::element::random_element());
            g1::affine_element input_c(g1::element::random_element());
            fr scalar_a(fr::random_element());
            fr scalar_b(fr::random_element());
            fr scalar_c(fr::random_element());
            if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // make a have skew
            }
            if ((scalar_b.from_montgomery_form().get_bit(0) & 1) == 0) {
                scalar_b += fr(1); // make b not have skew
            }
            g1_bigfr_ct P_a = convert_inputs_bigfr(&composer, input_a);
            bigfr_ct x_a = convert_inputs_bigfr(&composer, scalar_a);
            g1_bigfr_ct P_b = convert_inputs_bigfr(&composer, input_b);
            bigfr_ct x_b = convert_inputs_bigfr(&composer, scalar_b);
            g1_bigfr_ct P_c = convert_inputs_bigfr(&composer, input_c);
            bigfr_ct x_c = convert_inputs_bigfr(&composer, scalar_c);

            g1_bigfr_ct c = g1_bigfr_ct::batch_mul({ P_a, P_b, P_c }, { x_a, x_b, x_c });
            g1::element input_e = (g1::element(input_a) * scalar_a);
            g1::element input_f = (g1::element(input_b) * scalar_b);
            g1::element input_g = (g1::element(input_c) * scalar_c);

            g1::affine_element expected(input_e + input_f + input_g);
            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, expected.x);
            EXPECT_EQ(c_y_result, expected.y);
        }
        std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
        auto prover = composer.create_prover();
        std::cout << "creating verifier " << std::endl;
        auto verifier = composer.create_verifier();
        std::cout << "creating proof " << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_quad_mul_bigfr()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            g1::affine_element input_a(g1::element::random_element());
            g1::affine_element input_b(g1::element::random_element());
            g1::affine_element input_c(g1::element::random_element());
            g1::affine_element input_d(g1::element::random_element());
            fr scalar_a(fr::random_element());
            fr scalar_b(fr::random_element());
            fr scalar_c(fr::random_element());
            fr scalar_d(fr::random_element());
            if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // make a have skew
            }
            if ((scalar_b.from_montgomery_form().get_bit(0) & 1) == 0) {
                scalar_b += fr(1); // make b not have skew
            }
            g1_bigfr_ct P_a = convert_inputs_bigfr(&composer, input_a);
            bigfr_ct x_a = convert_inputs_bigfr(&composer, scalar_a);
            g1_bigfr_ct P_b = convert_inputs_bigfr(&composer, input_b);
            bigfr_ct x_b = convert_inputs_bigfr(&composer, scalar_b);
            g1_bigfr_ct P_c = convert_inputs_bigfr(&composer, input_c);
            bigfr_ct x_c = convert_inputs_bigfr(&composer, scalar_c);
            g1_bigfr_ct P_d = convert_inputs_bigfr(&composer, input_d);
            bigfr_ct x_d = convert_inputs_bigfr(&composer, scalar_d);

            g1_bigfr_ct c = g1_bigfr_ct::batch_mul({ P_a, P_b, P_c, P_d }, { x_a, x_b, x_c, x_d });
            g1::element input_e = (g1::element(input_a) * scalar_a);
            g1::element input_f = (g1::element(input_b) * scalar_b);
            g1::element input_g = (g1::element(input_c) * scalar_c);
            g1::element input_h = (g1::element(input_d) * scalar_d);

            g1::affine_element expected(input_e + input_f + input_g + input_h);
            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, expected.x);
            EXPECT_EQ(c_y_result, expected.y);
        }
        std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
        auto prover = composer.create_prover();
        std::cout << "creating verifier " << std::endl;
        auto verifier = composer.create_verifier();
        std::cout << "creating proof " << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_quad_mul()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            g1::affine_element input_a(g1::element::random_element());
            g1::affine_element input_b(g1::element::random_element());
            g1::affine_element input_c(g1::element::random_element());
            g1::affine_element input_d(g1::element::random_element());
            fr scalar_a(fr::random_element());
            fr scalar_b(fr::random_element());
            fr scalar_c(fr::random_element());
            fr scalar_d(fr::random_element());
            if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // make a have skew
            }
            if ((scalar_b.from_montgomery_form().get_bit(0) & 1) == 0) {
                scalar_b += fr(1); // make b not have skew
            }
            g1_ct P_a = convert_inputs(&composer, input_a);
            fr_ct x_a = witness_ct(&composer, scalar_a);
            g1_ct P_b = convert_inputs(&composer, input_b);
            fr_ct x_b = witness_ct(&composer, scalar_b);
            g1_ct P_c = convert_inputs(&composer, input_c);
            fr_ct x_c = witness_ct(&composer, scalar_c);
            g1_ct P_d = convert_inputs(&composer, input_d);
            fr_ct x_d = witness_ct(&composer, scalar_d);

            g1_ct c = g1_ct::batch_mul({ P_a, P_b, P_c, P_d }, { x_a, x_b, x_c, x_d });
            g1::element input_e = (g1::element(input_a) * scalar_a);
            g1::element input_f = (g1::element(input_b) * scalar_b);
            g1::element input_g = (g1::element(input_c) * scalar_c);
            g1::element input_h = (g1::element(input_d) * scalar_d);

            g1::affine_element expected(input_e + input_f + input_g + input_h);
            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, expected.x);
            EXPECT_EQ(c_y_result, expected.y);
        }
        std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
        auto prover = composer.create_prover();
        std::cout << "creating verifier " << std::endl;
        auto verifier = composer.create_verifier();
        std::cout << "creating proof " << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_one()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            fr scalar_a(fr::random_element());
            if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // make a have skew
            }
            g1_bigfr_ct P_a = g1_bigfr_ct::one(&composer);
            bigfr_ct x_a = convert_inputs_bigfr(&composer, scalar_a);
            g1_bigfr_ct c = P_a * x_a;
            g1::affine_element expected(g1::one * scalar_a);
            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, expected.x);
            EXPECT_EQ(c_y_result, expected.y);
        }
        std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
        auto prover = composer.create_prover();
        std::cout << "creating verifier " << std::endl;
        auto verifier = composer.create_verifier();
        std::cout << "creating proof " << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_one_secp256r1()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            typename secp256r1::fr scalar_a(secp256r1::fr::random_element());
            if ((scalar_a.from_montgomery_form().get_bit(0) & 1) == 1) {
                scalar_a -= secp256r1::fr(1); // make a have skew
            }
            typename secp256r1_ct::g1_bigfr_ct P_a = secp256r1_ct::g1_bigfr_ct::one(&composer);
            typename secp256r1_ct::bigfr_ct x_a = convert_inputs_bigfr_secp256r1(&composer, scalar_a);
            typename secp256r1_ct::g1_bigfr_ct c = P_a * x_a;
            secp256r1::g1::affine_element expected(secp256r1::g1::one * scalar_a);
            secp256r1::fq c_x_result(c.x.get_value().lo);
            secp256r1::fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, expected.x);
            EXPECT_EQ(c_y_result, expected.y);
        }
        std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
        auto prover = composer.create_prover();
        std::cout << "creating verifier " << std::endl;
        auto verifier = composer.create_verifier();
        std::cout << "creating proof " << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_batch_mul()
    {
        const size_t num_points = 5;
        auto composer = Composer("../srs_db/ignition/");
        std::vector<g1::affine_element> points;
        std::vector<fr> scalars;
        for (size_t i = 0; i < num_points; ++i) {
            points.push_back(g1::affine_element(g1::element::random_element()));
            scalars.push_back(fr::random_element());
        }

        std::vector<g1_ct> circuit_points;
        std::vector<fr_ct> circuit_scalars;
        for (size_t i = 0; i < num_points; ++i) {
            circuit_points.push_back(convert_inputs(&composer, points[i]));
            circuit_scalars.push_back(witness_ct(&composer, scalars[i]));
        }

        g1_ct result_point = g1_ct::batch_mul(circuit_points, circuit_scalars);

        g1::element expected_point = g1::one;
        expected_point.self_set_infinity();
        for (size_t i = 0; i < num_points; ++i) {
            expected_point += (g1::element(points[i]) * scalars[i]);
        }
        expected_point = expected_point.normalize();
        fq result_x(result_point.x.get_value().lo);
        fq result_y(result_point.y.get_value().lo);

        EXPECT_EQ(result_x, expected_point.x);
        EXPECT_EQ(result_y, expected_point.y);

        std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
        auto prover = composer.create_prover();
        std::cout << "creating verifier " << std::endl;
        auto verifier = composer.create_verifier();
        std::cout << "creating proof " << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_batch_mul_short_scalars()
    {
        const size_t num_points = 11;
        auto composer = Composer("../srs_db/ignition/");
        std::vector<g1::affine_element> points;
        std::vector<fr> scalars;
        for (size_t i = 0; i < num_points; ++i) {
            points.push_back(g1::affine_element(g1::element::random_element()));
            uint256_t scalar_raw = fr::random_element();
            scalar_raw.data[2] = 0ULL;
            scalar_raw.data[3] = 0ULL;
            scalars.push_back(fr(scalar_raw));
        }
        std::vector<g1_ct> circuit_points;
        std::vector<fr_ct> circuit_scalars;
        for (size_t i = 0; i < num_points; ++i) {
            circuit_points.push_back(convert_inputs(&composer, points[i]));
            circuit_scalars.push_back(witness_ct(&composer, scalars[i]));
        }

        g1_ct result_point = g1_ct::batch_mul(circuit_points, circuit_scalars, 128);

        g1::element expected_point = g1::one;
        expected_point.self_set_infinity();
        for (size_t i = 0; i < num_points; ++i) {
            expected_point += (g1::element(points[i]) * scalars[i]);
        }
        expected_point = expected_point.normalize();
        fq result_x(result_point.x.get_value().lo);
        fq result_y(result_point.y.get_value().lo);

        EXPECT_EQ(result_x, expected_point.x);
        EXPECT_EQ(result_y, expected_point.y);

        std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
        auto prover = composer.create_prover();
        std::cout << "creating verifier " << std::endl;
        auto verifier = composer.create_verifier();
        std::cout << "creating proof " << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_mixed_batch_mul()
    {
        const size_t num_big_points = 10;
        const size_t num_small_points = 11;
        auto composer = Composer("../srs_db/ignition/");
        std::vector<g1::affine_element> big_points;
        std::vector<fr> big_scalars;
        std::vector<g1::affine_element> small_points;
        std::vector<fr> small_scalars;

        for (size_t i = 0; i < num_big_points; ++i) {
            big_points.push_back(g1::affine_element(g1::element::random_element()));
            big_scalars.push_back(fr::random_element());
        }
        for (size_t i = 0; i < num_small_points; ++i) {
            small_points.push_back(g1::affine_element(g1::element::random_element()));
            uint256_t scalar_raw = fr::random_element();
            scalar_raw.data[2] = 0ULL;
            scalar_raw.data[3] = 0ULL;
            small_scalars.push_back(fr(scalar_raw));
        }

        std::vector<g1_ct> big_circuit_points;
        std::vector<fr_ct> big_circuit_scalars;
        std::vector<g1_ct> small_circuit_points;
        std::vector<fr_ct> small_circuit_scalars;
        for (size_t i = 0; i < num_big_points; ++i) {
            big_circuit_points.push_back(convert_inputs(&composer, big_points[i]));
            big_circuit_scalars.push_back(witness_ct(&composer, big_scalars[i]));
        }
        for (size_t i = 0; i < num_small_points; ++i) {
            small_circuit_points.push_back(convert_inputs(&composer, small_points[i]));
            small_circuit_scalars.push_back(witness_ct(&composer, small_scalars[i]));
        }
        g1_ct result_point = g1_ct::mixed_batch_mul(
            big_circuit_points, big_circuit_scalars, small_circuit_points, small_circuit_scalars, 128);

        g1::element expected_point = g1::one;
        expected_point.self_set_infinity();
        for (size_t i = 0; i < num_big_points; ++i) {
            expected_point += (g1::element(big_points[i]) * big_scalars[i]);
        }
        for (size_t i = 0; i < num_small_points; ++i) {
            expected_point += (g1::element(small_points[i]) * small_scalars[i]);
        }
        expected_point = expected_point.normalize();
        fq result_x(result_point.x.get_value().lo);
        fq result_y(result_point.y.get_value().lo);

        EXPECT_EQ(result_x, expected_point.x);
        EXPECT_EQ(result_y, expected_point.y);

        std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
        auto prover = composer.create_prover();
        std::cout << "creating verifier " << std::endl;
        auto verifier = composer.create_verifier();
        std::cout << "creating proof " << std::endl;
        waffle::plonk_proof proof = prover.construct_proof();
        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }
};

// Set types for which our typed tests will be built.
typedef testing::Types<waffle::StandardComposer,
                       waffle::TurboComposer //,
                       // waffle::PlookupComposer
                       >
    ComposerTypes;
// Define test suite
TYPED_TEST_SUITE(StdlibBiggroup, ComposerTypes);

TYPED_TEST(StdlibBiggroup, Add)
{
    TestFixture::test_add();
}

TYPED_TEST(StdlibBiggroup, Sub)
{
    TestFixture::test_sub();
}

TYPED_TEST(StdlibBiggroup, Dbl)
{
    TestFixture::test_dbl();
}

TYPED_TEST(StdlibBiggroup, MontgomeryLadder)
{
    TestFixture::test_montgomery_ladder();
}

HEAVY_TYPED_TEST(StdlibBiggroup, Mul)
{
    TestFixture::test_mul();
}

HEAVY_TYPED_TEST(StdlibBiggroup, TwinMul)
{
    TestFixture::test_twin_mul();
}

HEAVY_TYPED_TEST(StdlibBiggroup, TripleMul)
{
    TestFixture::test_triple_mul();
}

HEAVY_TYPED_TEST(StdlibBiggroup, QuadMulBigFr)
{
    TestFixture::test_quad_mul_bigfr();
}

HEAVY_TYPED_TEST(StdlibBiggroup, QuadMul)
{
    TestFixture::test_quad_mul();
}

HEAVY_TYPED_TEST(StdlibBiggroup, One)
{
    TestFixture::test_one();
}

HEAVY_TYPED_TEST(StdlibBiggroup, OneSECP256r1)
{
    TestFixture::test_one_secp256r1();
}

HEAVY_TYPED_TEST(StdlibBiggroup, BatchMul)
{
    TestFixture::test_batch_mul();
}

HEAVY_TYPED_TEST(StdlibBiggroup, BatchMulShortScalars)
{
    TestFixture::test_batch_mul_short_scalars();
}

HEAVY_TYPED_TEST(StdlibBiggroup, MixedBatchMul)
{
    TestFixture::test_mixed_batch_mul();
}

// // REMNANTS OF TESTS OF PLOOKUP

// Use of the following namespaces is deprecated, having been replaced
// in the above by appropriate use of the methods defined in
// stdlib/primitives/curves/bn254.hpp and stdlib/primitives/curves/sepc256r1.hpp.
// Notes on the replacement:
// - As defined below, plonk::stdlib::bn254 has Fr given by a bigfield,
//   while  plonk::stdlib::alt_bn254 has Fr given by a field_t.
//   Therefore, to update what's below to use
//     typedef stdlib::bn254<Composer> bn254
//  as defined in our class StdlibBiggroup, one should make the change
//   - stdlib::bn254::fr ~> bigfr_ct
//   - stdlib::alt_bn254::fr ~> fr_ct
//   - stdlib::bn254::g1 ~> g1_bigfr_ct
//   - stdlib::alt_bn254::g1 ~> g1_ct
//  along with other, more obvious changes.
//
// - Among the tests using Plookup, only test_mul was uncommented at the time
//   of the refactor of this document to use TYPED_TESTS's.

// namespace plonk {
// namespace stdlib {
// namespace bn254 {
// typedef typename plonk::stdlib::bigfield<typename waffle::TurboComposer, typename Bn254FqParams> fq;
// // Q:why not use regular fr?
// typedef typename plonk::stdlib::bigfield<waffle::TurboComposer, Bn254FrParams> fr;
// typedef typename plonk::stdlib::element<waffle::TurboComposer, fq, fr, g1> g1;

// typedef typename plonk::stdlib::bigfield<typename waffle::PlookupComposer, typename Bn254FqParams>
// plfq; typedef typename plonk::stdlib::bigfield<waffle::PlookupComposer, Bn254FrParams> plfr; typedef
// typename plonk::stdlib::element<waffle::PlookupComposer, plfq, plfr, g1> plg1;

// } // namespace bn254
// namespace alt_bn254 {
// typedef typename plonk::stdlib::bigfield<typename waffle::TurboComposer, typename Bn254FqParams> fq;
// typedef typename plonk::stdlib::field_t<typename waffle::TurboComposer> fr;
// typedef typename plonk::stdlib::element<waffle::TurboComposer, fq, fr, g1> g1;

// typedef typename plonk::stdlib::bigfield<typename waffle::PlookupComposer, typename Bn254FqParams>
// plfq; typedef typename plonk::stdlib::field_t<typename waffle::PlookupComposer> plfr; typedef typename
// plonk::stdlib::element<waffle::PlookupComposer, plfq, plfr, g1> plg1; } // namespace alt_bn254
// namespace secp256r {
// typedef typename plonk::stdlib::bigfield<waffle::TurboComposer, secp256r1::Secp256r1FqParams> fq;
// typedef typename plonk::stdlib::bigfield<waffle::TurboComposer, secp256r1::Secp256r1FrParams> fr;
// typedef typename plonk::stdlib::element<waffle::TurboComposer, fq, fr, secp256r1::g1> g1;

// } // namespace secp256r

// } // namespace stdlib
// } // namespace plonk
// typedef stdlib::bool_t<waffle::TurboComposer> bool_t;
// typedef stdlib::field_t<waffle::TurboComposer> field_t;
// typedef stdlib::witness_t<waffle::TurboComposer> witness_t;
// typedef stdlib::public_witness_t<waffle::TurboComposer> public_witness_t;

// typedef stdlib::bool_t<waffle::PlookupComposer> boolpl_t;
// typedef stdlib::field_t<waffle::PlookupComposer> fieldpl_t;
// typedef stdlib::witness_t<waffle::PlookupComposer> witnesspl_t;
// typedef stdlib::public_witness_t<waffle::PlookupComposer> public_witnesspl_t;

// stdlib::bn254::plg1 convert_inputs(waffle::PlookupComposer* ctx, const g1::affine_element& input)
// {
//     uint256_t x_u256(input.x);
//     uint256_t y_u256(input.y);

//     stdlib::bn254::plfq x(witnesspl_t(ctx, fr(x_u256.slice(0, stdlib::bn254::plfq::NUM_LIMB_BITS *
//     2))),
//                           witnesspl_t(ctx,
//                                       fr(x_u256.slice(stdlib::bn254::plfq::NUM_LIMB_BITS * 2,
//                                                                     stdlib::bn254::plfq::NUM_LIMB_BITS * 4))));
//     stdlib::bn254::plfq y(witnesspl_t(ctx, fr(y_u256.slice(0, stdlib::bn254::plfq::NUM_LIMB_BITS *
//     2))),
//                           witnesspl_t(ctx,
//                                       fr(y_u256.slice(stdlib::bn254::plfq::NUM_LIMB_BITS * 2,
//                                                                     stdlib::bn254::plfq::NUM_LIMB_BITS * 4))));

//     return stdlib::bn254::plg1(x, y);
// }

// stdlib::alt_bn254::plg1 convert_inputs_alt_bn254(waffle::PlookupComposer* ctx,
//                                                  const g1::affine_element& input)
// {
//     uint256_t x_u256(input.x);
//     uint256_t y_u256(input.y);

//     stdlib::alt_bn254::plfq x(
//         witnesspl_t(ctx, fr(x_u256.slice(0, stdlib::alt_bn254::plfq::NUM_LIMB_BITS * 2))),
//         witnesspl_t(ctx,
//                     fr(x_u256.slice(stdlib::alt_bn254::plfq::NUM_LIMB_BITS * 2,
//                                                   stdlib::alt_bn254::plfq::NUM_LIMB_BITS * 4))));
//     stdlib::alt_bn254::plfq y(
//         witnesspl_t(ctx, fr(y_u256.slice(0, stdlib::alt_bn254::plfq::NUM_LIMB_BITS * 2))),
//         witnesspl_t(ctx,
//                     fr(y_u256.slice(stdlib::alt_bn254::plfq::NUM_LIMB_BITS * 2,
//                                                   stdlib::alt_bn254::plfq::NUM_LIMB_BITS * 4))));

//     return stdlib::alt_bn254::plg1(x, y);
// }

// stdlib::bn254::plfr convert_inputs(waffle::PlookupComposer* ctx, const fr& scalar)
// {
//     uint256_t scalar_u256(scalar);

//     stdlib::bn254::plfr x(
//         witnesspl_t(ctx, fr(scalar_u256.slice(0, stdlib::bn254::plfq::NUM_LIMB_BITS * 2))),
//         witnesspl_t(ctx,
//                     fr(scalar_u256.slice(stdlib::bn254::plfq::NUM_LIMB_BITS * 2,
//                                                        stdlib::bn254::plfq::NUM_LIMB_BITS * 4))));

//     return x;
// }

// HEAVY_TEST(stdlib_biggroup_plookup, test_mul)
// {
//     waffle::PlookupComposer composer = waffle::PlookupComposer();
//     size_t num_repetitions = 1;
//     for (size_t i = 0; i < num_repetitions; ++i) {
//         g1::affine_element input(g1::element::random_element());
//         fr scalar(fr::random_element());
//         if ((scalar.from_montgomery_form().get_bit(0) & 1) == 1) {
//             scalar -= fr(1); // make sure to add skew
//         }
//         stdlib::alt_bn254::plg1 P = convert_inputs_alt_bn254(&composer, input);
//         stdlib::alt_bn254::plfr x = witnesspl_t(&composer, scalar);

//         std::cout << "gates before mul " << composer.get_num_gates() << std::endl;
//         stdlib::alt_bn254::plg1 c = P * x;
//         std::cout << "composer aftr mul " << composer.get_num_gates() << std::endl;
//         g1::affine_element c_expected(g1::element(input) * scalar);

//         fq c_x_result(c.x.get_value().lo);
//         fq c_y_result(c.y.get_value().lo);

//         EXPECT_EQ(c_x_result, c_expected.x);
//         EXPECT_EQ(c_y_result, c_expected.y);
//     }
//     std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
//     composer.process_range_lists();
//     waffle::PlookupProver prover = composer.create_prover();
//     std::cout << "creating verifier " << std::endl;
//     waffle::PlookupVerifier verifier = composer.create_verifier();
//     std::cout << "creating proof " << std::endl;
//     waffle::plonk_proof proof = prover.construct_proof();
//     bool proof_result = verifier.verify_proof(proof);
//     EXPECT_EQ(proof_result, true);
// }

// // // The remaining tests were commented out.

// // TEST(stdlib_biggroup_plookup, test_sub)
// // {
// //     waffle::PlookupComposer composer = waffle::PlookupComposer();
// //     size_t num_repetitions = 10;
// //     for (size_t i = 0; i < num_repetitions; ++i) {
// //         g1::affine_element input_a(g1::element::random_element());
// //         g1::affine_element input_b(g1::element::random_element());

// //         stdlib::bn254::g1 a = convert_inputs(&composer, input_a);
// //         stdlib::bn254::g1 b = convert_inputs(&composer, input_b);

// //         stdlib::bn254::g1 c = a - b;

// //         g1::affine_element c_expected(g1::element(input_a) -
// //                                                     g1::element(input_b));

// //         uint256_t c_x_u256 = c.x.get_value().lo;
// //         uint256_t c_y_u256 = c.y.get_value().lo;

// //         fq c_x_result(c_x_u256);
// //         fq c_y_result(c_y_u256);

// //         EXPECT_EQ(c_x_result, c_expected.x);
// //         EXPECT_EQ(c_y_result, c_expected.y);
// //     }
// //     waffle::PlookupProver prover = composer.create_prover();
// //     waffle::PlookupVerifier verifier = composer.create_verifier();
// //     waffle::plonk_proof proof = prover.construct_proof();
// //     bool proof_result = verifier.verify_proof(proof);
// //     EXPECT_EQ(proof_result, true);
// // }

// // TEST(stdlib_biggroup_plookup, test_dbl)
// // {
// //     waffle::PlookupComposer composer = waffle::PlookupComposer();
// //     size_t num_repetitions = 10;
// //     for (size_t i = 0; i < num_repetitions; ++i) {
// //         g1::affine_element input_a(g1::element::random_element());

// //         stdlib::bn254::g1 a = convert_inputs(&composer, input_a);

// //         stdlib::bn254::g1 c = a.dbl();

// //         g1::affine_element c_expected(g1::element(input_a).dbl());

// //         uint256_t c_x_u256 = c.x.get_value().lo;
// //         uint256_t c_y_u256 = c.y.get_value().lo;

// //         fq c_x_result(c_x_u256);
// //         fq c_y_result(c_y_u256);

// //         EXPECT_EQ(c_x_result, c_expected.x);
// //         EXPECT_EQ(c_y_result, c_expected.y);
// //     }
// //     waffle::PlookupProver prover = composer.create_prover();
// //     waffle::PlookupVerifier verifier = composer.create_verifier();
// //     waffle::plonk_proof proof = prover.construct_proof();
// //     bool proof_result = verifier.verify_proof(proof);
// //     EXPECT_EQ(proof_result, true);
// // }

// // TEST(stdlib_biggroup_plookup, test_montgomery_ladder)
// // {
// //     waffle::PlookupComposer composer = waffle::PlookupComposer();
// //     size_t num_repetitions = 1;
// //     for (size_t i = 0; i < num_repetitions; ++i) {
// //         g1::affine_element input_a(g1::element::random_element());
// //         g1::affine_element input_b(g1::element::random_element());

// //         stdlib::bn254::g1 a = convert_inputs(&composer, input_a);
// //         stdlib::bn254::g1 b = convert_inputs(&composer, input_b);

// //         stdlib::bn254::g1 c = a.montgomery_ladder(b);

// //         g1::affine_element c_expected(g1::element(input_a).dbl() +
// //                                                     g1::element(input_b));

// //         uint256_t c_x_u256 = c.x.get_value().lo;
// //         uint256_t c_y_u256 = c.y.get_value().lo;

// //         fq c_x_result(c_x_u256);
// //         fq c_y_result(c_y_u256);

// //         EXPECT_EQ(c_x_result, c_expected.x);
// //         EXPECT_EQ(c_y_result, c_expected.y);
// //     }
// //     waffle::PlookupProver prover = composer.create_prover();
// //     waffle::PlookupVerifier verifier = composer.create_verifier();
// //     waffle::plonk_proof proof = prover.construct_proof();
// //     bool proof_result = verifier.verify_proof(proof);
// //     EXPECT_EQ(proof_result, true);
// // }

// // HEAVY_TEST(stdlib_biggroup_plookup, test_mul)
// // {
// //     waffle::PlookupComposer composer = waffle::PlookupComposer();
// //     size_t num_repetitions = 1;
// //     for (size_t i = 0; i < num_repetitions; ++i) {
// //         g1::affine_element input(g1::element::random_element());
// //         fr scalar(fr::random_element());
// //         if ((scalar.from_montgomery_form().get_bit(0) & 1) == 1) {
// //             scalar -= fr(1); // make sure to add skew
// //         }
// //         stdlib::bn254::g1 P = convert_inputs(&composer, input);
// //         stdlib::bn254::fr x = convert_inputs(&composer, scalar);

// //         stdlib::bn254::g1 c = P * x;
// //         g1::affine_element c_expected(g1::element(input) * scalar);

// //         fq c_x_result(c.x.get_value().lo);
// //         barretenberg::fq c_y_result(c.y.get_value().lo);

// //         EXPECT_EQ(c_x_result, c_expected.x);
// //         EXPECT_EQ(c_y_result, c_expected.y);
// //     }
// //     std::cout << "composer gates = " << composer.get_num_gates() << std::endl;
// //     waffle::PlookupProver prover = composer.create_prover();
// //     std::cout << "creating verifier " << std::endl;
// //     waffle::PlookupVerifier verifier = composer.create_verifier();
// //     std::cout << "creating proof " << std::endl;
// //     waffle::plonk_proof proof = prover.construct_proof();
// //     bool proof_result = verifier.verify_proof(proof);
// //     EXPECT_EQ(proof_result, true);
// // }
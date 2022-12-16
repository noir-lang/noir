#include <common/test.hpp>
#include <type_traits>

#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../bool/bool.hpp"
#include "../field/field.hpp"

#include <stdlib/primitives/curves/bn254.hpp>
#include <stdlib/primitives/curves/secp256r1.hpp>
#include <stdlib/primitives/curves/secp256k1.hpp>

#include <memory>
#include <numeric/random/engine.hpp>

namespace test_stdlib_biggroup {
namespace {
auto& engine = numeric::random::get_debug_engine();
}

#define GET_COMPOSER_NAME_STRING(composer)                                                                             \
    (typeid(composer) == typeid(waffle::StandardComposer)                                                              \
         ? "StandardPlonk"                                                                                             \
         : typeid(composer) == typeid(waffle::TurboComposer) ? "TurboPlonk" : "NULLPlonk")

// using namespace barretenberg;
using namespace plonk;

// One can only define a TYPED_TEST with a single template paramter.
// Our workaround is to pass parameters of the following type.
template <typename _Curve, bool _use_bigfield = false> struct TestType {
  public:
    using Curve = _Curve;
    static const bool use_bigfield = _use_bigfield;
    using element_ct =
        typename std::conditional<_use_bigfield, typename Curve::g1_bigfr_ct, typename Curve::g1_ct>::type;
    // the field of scalars acting on element_ct
    using scalar_ct = typename std::conditional<_use_bigfield, typename Curve::bigfr_ct, typename Curve::fr_ct>::type;
};

template <typename TestType> class stdlib_biggroup : public testing::Test {
    using Curve = typename TestType::Curve;
    using element_ct = typename TestType::element_ct;
    using scalar_ct = typename TestType::scalar_ct;

    using fq = typename Curve::fq;
    using fr = typename Curve::fr;
    using g1 = typename Curve::g1;
    using affine_element = typename g1::affine_element;
    using element = typename g1::element;

    using Composer = typename Curve::Composer;

    static constexpr auto EXPECT_VERIFICATION = [](Composer& composer, bool expected_result = true) {
        info("composer gates = ", composer.get_num_gates());
        auto prover = composer.create_prover();
        info("creating verifier");
        auto verifier = composer.create_verifier();
        info("creating proof");
        waffle::plonk_proof proof = prover.construct_proof();
        bool verified = verifier.verify_proof(proof);
        EXPECT_EQ(verified, expected_result);
    };

  public:
    static void test_add()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 10;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input_a(element::random_element());
            affine_element input_b(element::random_element());

            element_ct a = element_ct::from_witness(&composer, input_a);
            element_ct b = element_ct::from_witness(&composer, input_b);

            uint64_t before = composer.get_num_gates();
            element_ct c = a + b;
            uint64_t after = composer.get_num_gates();
            if (i == num_repetitions - 1) {
                std::cout << "num gates per add = " << after - before << std::endl;
                benchmark_info(GET_COMPOSER_NAME_STRING(Composer), "Biggroup", "ADD", "Gate Count", after - before);
            }

            affine_element c_expected(element(input_a) + element(input_b));

            uint256_t c_x_u256 = c.x.get_value().lo;
            uint256_t c_y_u256 = c.y.get_value().lo;

            fq c_x_result(c_x_u256);
            fq c_y_result(c_y_u256);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_sub()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 10;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input_a(element::random_element());
            affine_element input_b(element::random_element());

            element_ct a = element_ct::from_witness(&composer, input_a);
            element_ct b = element_ct::from_witness(&composer, input_b);

            element_ct c = a - b;

            affine_element c_expected(element(input_a) - element(input_b));

            uint256_t c_x_u256 = c.x.get_value().lo;
            uint256_t c_y_u256 = c.y.get_value().lo;

            fq c_x_result(c_x_u256);
            fq c_y_result(c_y_u256);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_dbl()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 10;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input_a(element::random_element());

            element_ct a = element_ct::from_witness(&composer, input_a);

            element_ct c = a.dbl();

            affine_element c_expected(element(input_a).dbl());

            uint256_t c_x_u256 = c.x.get_value().lo;
            uint256_t c_y_u256 = c.y.get_value().lo;

            fq c_x_result(c_x_u256);
            fq c_y_result(c_y_u256);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_montgomery_ladder()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input_a(element::random_element());
            affine_element input_b(element::random_element());

            element_ct a = element_ct::from_witness(&composer, input_a);
            element_ct b = element_ct::from_witness(&composer, input_b);

            element_ct c = a.montgomery_ladder(b);

            affine_element c_expected(element(input_a).dbl() + element(input_b));

            uint256_t c_x_u256 = c.x.get_value().lo;
            uint256_t c_y_u256 = c.y.get_value().lo;

            fq c_x_result(c_x_u256);
            fq c_y_result(c_y_u256);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_mul()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input(element::random_element());
            fr scalar(fr::random_element());
            if (uint256_t(scalar).get_bit(0)) {
                scalar -= fr(1); // make sure to add skew
            }
            element_ct P = element_ct::from_witness(&composer, input);
            scalar_ct x = scalar_ct::from_witness(&composer, scalar);

            std::cerr << "gates before mul " << composer.get_num_gates() << std::endl;
            element_ct c = P * x;
            std::cerr << "composer aftr mul " << composer.get_num_gates() << std::endl;
            affine_element c_expected(element(input) * scalar);

            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_twin_mul()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input_a(element::random_element());
            affine_element input_b(element::random_element());
            fr scalar_a(fr::random_element());
            fr scalar_b(fr::random_element());
            if ((uint256_t(scalar_a).get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // skew bit is 1
            }
            if ((uint256_t(scalar_b).get_bit(0) & 1) == 0) {
                scalar_b += fr(1); // skew bit is 0
            }
            element_ct P_a = element_ct::from_witness(&composer, input_a);
            scalar_ct x_a = scalar_ct::from_witness(&composer, scalar_a);
            element_ct P_b = element_ct::from_witness(&composer, input_b);
            scalar_ct x_b = scalar_ct::from_witness(&composer, scalar_b);

            element_ct c = element_ct::batch_mul({ P_a, P_b }, { x_a, x_b });
            element input_c = (element(input_a) * scalar_a);
            element input_d = (element(input_b) * scalar_b);
            affine_element expected(input_c + input_d);
            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, expected.x);
            EXPECT_EQ(c_y_result, expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_triple_mul()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input_a(element::random_element());
            affine_element input_b(element::random_element());
            affine_element input_c(element::random_element());
            fr scalar_a(fr::random_element());
            fr scalar_b(fr::random_element());
            fr scalar_c(fr::random_element());
            if ((uint256_t(scalar_a).get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // skew bit is 1
            }
            if ((uint256_t(scalar_b).get_bit(0) & 1) == 0) {
                scalar_b += fr(1); // skew bit is 0
            }
            element_ct P_a = element_ct::from_witness(&composer, input_a);
            scalar_ct x_a = scalar_ct::from_witness(&composer, scalar_a);
            element_ct P_b = element_ct::from_witness(&composer, input_b);
            scalar_ct x_b = scalar_ct::from_witness(&composer, scalar_b);
            element_ct P_c = element_ct::from_witness(&composer, input_c);
            scalar_ct x_c = scalar_ct::from_witness(&composer, scalar_c);

            element_ct c = element_ct::batch_mul({ P_a, P_b, P_c }, { x_a, x_b, x_c });
            element input_e = (element(input_a) * scalar_a);
            element input_f = (element(input_b) * scalar_b);
            element input_g = (element(input_c) * scalar_c);

            affine_element expected(input_e + input_f + input_g);
            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, expected.x);
            EXPECT_EQ(c_y_result, expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_quad_mul()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input_a(element::random_element());
            affine_element input_b(element::random_element());
            affine_element input_c(element::random_element());
            affine_element input_d(element::random_element());
            fr scalar_a(fr::random_element());
            fr scalar_b(fr::random_element());
            fr scalar_c(fr::random_element());
            fr scalar_d(fr::random_element());
            if ((uint256_t(scalar_a).get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // skew bit is 1
            }
            if ((uint256_t(scalar_b).get_bit(0) & 1) == 0) {
                scalar_b += fr(1); // skew bit is 0
            }
            element_ct P_a = element_ct::from_witness(&composer, input_a);
            scalar_ct x_a = scalar_ct::from_witness(&composer, scalar_a);
            element_ct P_b = element_ct::from_witness(&composer, input_b);
            scalar_ct x_b = scalar_ct::from_witness(&composer, scalar_b);
            element_ct P_c = element_ct::from_witness(&composer, input_c);
            scalar_ct x_c = scalar_ct::from_witness(&composer, scalar_c);
            element_ct P_d = element_ct::from_witness(&composer, input_d);
            scalar_ct x_d = scalar_ct::from_witness(&composer, scalar_d);

            element_ct c = element_ct::batch_mul({ P_a, P_b, P_c, P_d }, { x_a, x_b, x_c, x_d });
            element input_e = (element(input_a) * scalar_a);
            element input_f = (element(input_b) * scalar_b);
            element input_g = (element(input_c) * scalar_c);
            element input_h = (element(input_d) * scalar_d);

            affine_element expected(input_e + input_f + input_g + input_h);
            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, expected.x);
            EXPECT_EQ(c_y_result, expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_one()
    {
        auto composer = Composer("../srs_db/ignition/");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            fr scalar_a(fr::random_element());
            if ((uint256_t(scalar_a).get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // skew bit is 1
            }
            element_ct P_a = element_ct::one(&composer);
            scalar_ct x_a = scalar_ct::from_witness(&composer, scalar_a);
            element_ct c = P_a * x_a;
            affine_element expected(g1::one * scalar_a);
            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, expected.x);
            EXPECT_EQ(c_y_result, expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_batch_mul()
    {
        const size_t num_points = 5;
        auto composer = Composer("../srs_db/ignition/");
        std::vector<affine_element> points;
        std::vector<fr> scalars;
        for (size_t i = 0; i < num_points; ++i) {
            points.push_back(affine_element(element::random_element()));
            scalars.push_back(fr::random_element());
        }

        std::vector<element_ct> circuit_points;
        std::vector<scalar_ct> circuit_scalars;
        for (size_t i = 0; i < num_points; ++i) {
            circuit_points.push_back(element_ct::from_witness(&composer, points[i]));
            circuit_scalars.push_back(scalar_ct::from_witness(&composer, scalars[i]));
        }

        element_ct result_point = element_ct::batch_mul(circuit_points, circuit_scalars);

        element expected_point = g1::one;
        expected_point.self_set_infinity();
        for (size_t i = 0; i < num_points; ++i) {
            expected_point += (element(points[i]) * scalars[i]);
        }

        expected_point = expected_point.normalize();
        fq result_x(result_point.x.get_value().lo);
        fq result_y(result_point.y.get_value().lo);

        EXPECT_EQ(result_x, expected_point.x);
        EXPECT_EQ(result_y, expected_point.y);

        EXPECT_VERIFICATION(composer);
    }

    static void test_chain_add()
    {
        Composer composer = Composer();
        size_t num_repetitions = 10;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input_a(element::random_element());
            affine_element input_b(element::random_element());
            affine_element input_c(element::random_element());

            element_ct a = element_ct::from_witness(&composer, input_a);
            element_ct b = element_ct::from_witness(&composer, input_b);
            element_ct c = element_ct::from_witness(&composer, input_c);

            auto acc = element_ct::chain_add_start(a, b);
            auto acc_out = element_ct::chain_add(c, acc);

            auto lambda_prev = (input_b.y - input_a.y) / (input_b.x - input_a.x);
            auto x3_prev = lambda_prev * lambda_prev - input_b.x - input_a.x;
            auto y3_prev = lambda_prev * (input_a.x - x3_prev) - input_a.y;
            auto lambda = (y3_prev - input_c.y) / (x3_prev - input_c.x);
            auto x3 = lambda * lambda - x3_prev - input_c.x;

            uint256_t x3_u256 = acc_out.x3_prev.get_value().lo;
            uint256_t lambda_u256 = acc_out.lambda_prev.get_value().lo;

            fq x3_result(x3_u256);
            fq lambda_result(lambda_u256);

            EXPECT_EQ(x3_result, x3);
            EXPECT_EQ(lambda_result, lambda);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_double_montgomery_ladder()
    {
        Composer composer = Composer();
        size_t num_repetitions = 10;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element acc_small(element::random_element());
            element_ct acc_big = element_ct::from_witness(&composer, acc_small);

            affine_element add_1_small_0(element::random_element());
            element_ct add_1_big_0 = element_ct::from_witness(&composer, add_1_small_0);
            affine_element add_2_small_0(element::random_element());
            element_ct add_2_big_0 = element_ct::from_witness(&composer, add_2_small_0);

            affine_element add_1_small_1(element::random_element());
            element_ct add_1_big_1 = element_ct::from_witness(&composer, add_1_small_1);
            affine_element add_2_small_1(element::random_element());
            element_ct add_2_big_1 = element_ct::from_witness(&composer, add_2_small_1);

            typename element_ct::chain_add_accumulator add_1 = element_ct::chain_add_start(add_1_big_0, add_1_big_1);
            typename element_ct::chain_add_accumulator add_2 = element_ct::chain_add_start(add_2_big_0, add_2_big_1);
            acc_big.double_montgomery_ladder(add_1, add_2);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_compute_naf()
    {
        Composer composer = Composer();
        size_t num_repetitions(32);
        for (size_t i = 0; i < num_repetitions; i++) {
            fr scalar_val = fr::random_element();
            scalar_ct scalar = scalar_ct::from_witness(&composer, scalar_val);
            auto naf = element_ct::compute_naf(scalar);
            // scalar = -naf[254] + \sum_{i=0}^{253}(1-2*naf[i]) 2^{253-i}
            fr reconstructed_val(0);
            for (size_t i = 0; i < 254; i++) {
                reconstructed_val += (fr(1) - fr(2) * fr(naf[i].witness_bool)) * fr(uint256_t(1) << (253 - i));
            };
            reconstructed_val -= fr(naf[254].witness_bool);
            EXPECT_EQ(scalar_val, reconstructed_val);
        }
        EXPECT_VERIFICATION(composer);
    }

    static void test_compute_wnaf()
    {
        Composer composer = Composer();

        fr scalar_val = fr::random_element();
        scalar_ct scalar = scalar_ct::from_witness(&composer, scalar_val);
        element_ct::compute_wnaf(scalar);

        EXPECT_VERIFICATION(composer);
    }

    static void test_wnaf_batch_mul()
    {
        Composer composer = Composer("../srs_db/ignition");
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input(element::random_element());
            fr scalar(fr::random_element());
            if ((uint256_t(scalar).get_bit(0) & 1) == 1) {
                scalar -= fr(1); // make sure to add skew
            }
            element_ct P = element_ct::from_witness(&composer, input);
            scalar_ct x = scalar_ct::from_witness(&composer, scalar);

            std::cerr << "gates before mul " << composer.get_num_gates() << std::endl;
            element_ct c = element_ct::wnaf_batch_mul({ P }, { x });
            std::cerr << "composer aftr mul " << composer.get_num_gates() << std::endl;
            affine_element c_expected(element(input) * scalar);

            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_batch_mul_short_scalars()
    {
        const size_t num_points = 11;
        auto composer = Composer("../srs_db/ignition/");
        std::vector<affine_element> points;
        std::vector<fr> scalars;
        for (size_t i = 0; i < num_points; ++i) {
            points.push_back(affine_element(element::random_element()));
            uint256_t scalar_raw = fr::random_element();
            scalar_raw.data[2] = 0ULL;
            scalar_raw.data[3] = 0ULL;
            scalars.push_back(fr(scalar_raw));
        }
        std::vector<element_ct> circuit_points;
        std::vector<scalar_ct> circuit_scalars;
        for (size_t i = 0; i < num_points; ++i) {
            circuit_points.push_back(element_ct::from_witness(&composer, points[i]));
            circuit_scalars.push_back(scalar_ct::from_witness(&composer, scalars[i]));
        }

        element_ct result_point = element_ct::batch_mul(circuit_points, circuit_scalars, 128);

        element expected_point = g1::one;
        expected_point.self_set_infinity();
        for (size_t i = 0; i < num_points; ++i) {
            expected_point += (element(points[i]) * scalars[i]);
        }

        expected_point = expected_point.normalize();
        fq result_x(result_point.x.get_value().lo);
        fq result_y(result_point.y.get_value().lo);

        EXPECT_EQ(result_x, expected_point.x);
        EXPECT_EQ(result_y, expected_point.y);

        EXPECT_VERIFICATION(composer);
    }

    static void test_wnaf_batch_mul_128_bit()
    {
        Composer composer = Composer();
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            affine_element input(element::random_element());
            uint256_t scalar_u256(0, 0, 0, 0);
            scalar_u256.data[0] = engine.get_random_uint64();
            scalar_u256.data[1] = engine.get_random_uint64();
            fr scalar(scalar_u256);
            if ((uint256_t(scalar).get_bit(0) & 1) == 1) {
                scalar -= fr(1); // make sure to add skew
            }
            element_ct P = element_ct::from_witness(&composer, input);
            scalar_ct x = scalar_ct::from_witness(&composer, scalar);

            std::cerr << "gates before mul " << composer.get_num_gates() << std::endl;
            // Note: need >136 bits to complete this when working over bigfield
            element_ct c = element_ct::template wnaf_batch_mul<128>({ P }, { x });
            std::cerr << "composer aftr mul " << composer.get_num_gates() << std::endl;
            affine_element c_expected(element(input) * scalar);

            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_wnaf_batch_4()
    {
        Composer composer = Composer();
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            const auto get_128_bit_scalar = []() {
                uint256_t scalar_u256(0, 0, 0, 0);
                scalar_u256.data[0] = engine.get_random_uint64();
                scalar_u256.data[1] = engine.get_random_uint64();
                fr scalar(scalar_u256);
                return scalar;
            };
            affine_element input1(element::random_element());
            affine_element input2(element::random_element());
            affine_element input3(element::random_element());
            affine_element input4(element::random_element());

            element_ct P1 = element_ct::from_witness(&composer, input1);
            element_ct P2 = element_ct::from_witness(&composer, input2);
            element_ct P3 = element_ct::from_witness(&composer, input3);
            element_ct P4 = element_ct::from_witness(&composer, input4);

            fr scalar1 = get_128_bit_scalar();
            fr scalar2 = get_128_bit_scalar();
            fr scalar3 = get_128_bit_scalar();
            fr scalar4 = get_128_bit_scalar();
            scalar_ct x1 = scalar_ct::from_witness(&composer, scalar1);
            scalar_ct x2 = scalar_ct::from_witness(&composer, scalar2);
            scalar_ct x3 = scalar_ct::from_witness(&composer, scalar3);
            scalar_ct x4 = scalar_ct::from_witness(&composer, scalar4);

            std::cerr << "gates before mul " << composer.get_num_gates() << std::endl;
            element_ct c = element_ct::batch_mul({ P1, P2, P3, P4 }, { x1, x2, x3, x4 }, 128);
            std::cerr << "composer aftr mul " << composer.get_num_gates() << std::endl;

            element out = input1 * scalar1;
            out += (input2 * scalar2);
            out += (input3 * scalar3);
            out += (input4 * scalar4);
            affine_element c_expected(out);

            fq c_x_result(c.x.get_value().lo);
            fq c_y_result(c.y.get_value().lo);

            EXPECT_EQ(c_x_result, c_expected.x);
            EXPECT_EQ(c_y_result, c_expected.y);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_bn254_endo_batch_mul()
    {
        const size_t num_big_points = 2;
        const size_t num_small_points = 1;
        auto composer = Composer("../srs_db/ignition/");
        std::vector<affine_element> big_points;
        std::vector<fr> big_scalars;
        std::vector<affine_element> small_points;
        std::vector<fr> small_scalars;

        for (size_t i = 0; i < num_big_points; ++i) {
            big_points.push_back(affine_element(element::random_element()));
            big_scalars.push_back(fr::random_element());
        }
        for (size_t i = 0; i < num_small_points; ++i) {
            small_points.push_back(affine_element(element::random_element()));
            uint256_t scalar_raw = fr::random_element();
            scalar_raw.data[2] = 0ULL;
            scalar_raw.data[3] = 0ULL;
            small_scalars.push_back(fr(scalar_raw));
        }

        std::vector<element_ct> big_circuit_points;
        std::vector<scalar_ct> big_circuit_scalars;
        std::vector<element_ct> small_circuit_points;
        std::vector<scalar_ct> small_circuit_scalars;
        for (size_t i = 0; i < num_big_points; ++i) {
            big_circuit_points.push_back(element_ct::from_witness(&composer, big_points[i]));
            big_circuit_scalars.push_back(scalar_ct::from_witness(&composer, big_scalars[i]));
        }
        for (size_t i = 0; i < num_small_points; ++i) {
            small_circuit_points.push_back(element_ct::from_witness(&composer, small_points[i]));
            small_circuit_scalars.push_back(scalar_ct::from_witness(&composer, small_scalars[i]));
        }

        element_ct result_point = element_ct::bn254_endo_batch_mul(
            big_circuit_points, big_circuit_scalars, small_circuit_points, small_circuit_scalars, 128);

        element expected_point = g1::one;
        expected_point.self_set_infinity();
        for (size_t i = 0; i < num_big_points; ++i) {
            expected_point += (element(big_points[i]) * big_scalars[i]);
        }
        for (size_t i = 0; i < num_small_points; ++i) {
            expected_point += (element(small_points[i]) * small_scalars[i]);
        }

        expected_point = expected_point.normalize();
        fq result_x(result_point.x.get_value().lo);
        fq result_y(result_point.y.get_value().lo);

        EXPECT_EQ(result_x, expected_point.x);
        EXPECT_EQ(result_y, expected_point.y);

        EXPECT_VERIFICATION(composer);
    }

    static void test_mixed_mul_bn254_endo()
    {
        Composer composer = Composer("../srs_db/ignition");
        size_t num_repetitions = 1;

        const auto get_small_scalar = []() {
            fr t1 = fr::random_element();
            t1 = t1.from_montgomery_form();
            t1.data[2] = 0;
            t1.data[3] = 0;
            return t1.to_montgomery_form();
        };
        for (size_t i = 0; i < num_repetitions; ++i) {
            std::vector<element_ct> small_points(25);
            std::vector<element_ct> big_points(5);
            std::vector<element_ct> double_points(11);
            std::vector<scalar_ct> small_scalars(25);
            std::vector<scalar_ct> big_scalars(5);
            std::vector<scalar_ct> double_scalars(11);

            std::vector<affine_element> small_points_w(25);
            std::vector<affine_element> big_points_w(5);
            std::vector<affine_element> double_points_w(11);
            std::vector<fr> small_scalars_w(25);
            std::vector<fr> big_scalars_w(5);
            std::vector<fr> double_scalars_w(11);

            for (size_t i = 0; i < 25; ++i) {
                small_points_w[i] = affine_element(element::random_element());
                small_scalars_w[i] = get_small_scalar();
                small_points[i] = element_ct::from_witness(&composer, small_points_w[i]);
                small_scalars[i] = scalar_ct::from_witness(&composer, small_scalars_w[i]);
            }
            for (size_t i = 0; i < 5; ++i) {
                big_points_w[i] = affine_element(element::random_element());
                big_scalars_w[i] = fr::random_element();
                big_points[i] = element_ct::from_witness(&composer, big_points_w[i]);
                big_scalars[i] = scalar_ct::from_witness(&composer, big_scalars_w[i]);
            }
            for (size_t i = 0; i < 11; ++i) {
                double_points_w[i] = affine_element(element::random_element());
                double_scalars_w[i] = get_small_scalar();
                double_points[i] = element_ct::from_witness(&composer, double_points_w[i]);
                double_scalars[i] = scalar_ct::from_witness(&composer, double_scalars_w[i]);
            }

            fr omega = get_small_scalar();

            const auto double_opening_result = element_ct::batch_mul(double_points, double_scalars, 128);
            small_points.push_back(double_opening_result);
            small_scalars.push_back(scalar_ct::from_witness(&composer, omega));

            auto opening_result =
                element_ct::bn254_endo_batch_mul(big_points, big_scalars, small_points, small_scalars, 128);

            opening_result = opening_result + double_opening_result;
            opening_result = opening_result.normalize();

            element expected = g1::one;
            expected.self_set_infinity();
            for (size_t i = 0; i < 11; ++i) {
                expected += (double_points_w[i] * double_scalars_w[i]);
            }
            expected *= (omega + 1);
            for (size_t i = 0; i < 25; ++i) {
                expected += (small_points_w[i] * small_scalars_w[i]);
            }
            for (size_t i = 0; i < 5; ++i) {
                expected += (big_points_w[i] * big_scalars_w[i]);
            }
            expected = expected.normalize();

            fq result_x(opening_result.x.get_value().lo);
            fq result_y(opening_result.y.get_value().lo);

            EXPECT_EQ(result_x, expected.x);
            EXPECT_EQ(result_y, expected.y);
        }

        EXPECT_VERIFICATION(composer);
    };

    static void test_wnaf_secp256k1()
    {
        Composer composer = Composer();
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            fr scalar_a(fr::random_element());
            if ((uint256_t(scalar_a).get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // skew bit is 1
            }
            scalar_ct x_a = scalar_ct::from_witness(&composer, scalar_a);
            element_ct::template compute_secp256k1_endo_wnaf<4, 0, 3>(x_a);
            element_ct::template compute_secp256k1_endo_wnaf<4, 1, 2>(x_a);
            element_ct::template compute_secp256k1_endo_wnaf<4, 2, 1>(x_a);
            element_ct::template compute_secp256k1_endo_wnaf<4, 3, 0>(x_a);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_wnaf_8bit_secp256k1()
    {
        Composer composer = Composer();
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            fr scalar_a(fr::random_element());
            if ((uint256_t(scalar_a).get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // skew bit is 1
            }
            scalar_ct x_a = scalar_ct::from_witness(&composer, scalar_a);
            element_ct::template compute_secp256k1_endo_wnaf<8, 0, 3>(x_a);
            element_ct::template compute_secp256k1_endo_wnaf<8, 1, 2>(x_a);
            element_ct::template compute_secp256k1_endo_wnaf<8, 2, 1>(x_a);
            element_ct::template compute_secp256k1_endo_wnaf<8, 3, 0>(x_a);
        }

        EXPECT_VERIFICATION(composer);
    }

    static void test_ecdsa_mul_secp256k1()
    {
        Composer composer = Composer();
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; ++i) {
            fr scalar_a(fr::random_element());
            fr scalar_b(fr::random_element());
            fr scalar_c(fr::random_element());
            if ((uint256_t(scalar_a).get_bit(0) & 1) == 1) {
                scalar_a -= fr(1); // skew bit is 1
            }
            element_ct P_a = element_ct::from_witness(&composer, g1::one * scalar_c);
            scalar_ct u1 = scalar_ct::from_witness(&composer, scalar_a);
            scalar_ct u2 = scalar_ct::from_witness(&composer, scalar_b);

            fr alo;
            fr ahi;
            fr blo;
            fr bhi;

            fr::split_into_endomorphism_scalars(scalar_a.from_montgomery_form(), alo, ahi);
            fr::split_into_endomorphism_scalars(scalar_b.from_montgomery_form(), blo, bhi);

            auto output = element_ct::secp256k1_ecdsa_mul(P_a, u1, u2);

            auto expected = affine_element(g1::one * (scalar_c * scalar_b) + g1::one * scalar_a);
            EXPECT_EQ(output.x.get_value().lo, uint256_t(expected.x));
            EXPECT_EQ(output.y.get_value().lo, uint256_t(expected.y));
        }

        EXPECT_VERIFICATION(composer);
    }
};

enum UseBigfield { No, Yes };
typedef testing::Types<TestType<stdlib::bn254<waffle::UltraComposer>, UseBigfield::No>,
                       TestType<stdlib::bn254<waffle::TurboComposer>, UseBigfield::No>,
                       TestType<stdlib::bn254<waffle::StandardComposer>, UseBigfield::No>,
                       TestType<stdlib::bn254<waffle::UltraComposer>, UseBigfield::Yes>,
                       TestType<stdlib::bn254<waffle::TurboComposer>, UseBigfield::Yes>,
                       TestType<stdlib::bn254<waffle::StandardComposer>, UseBigfield::Yes>,
                       TestType<stdlib::secp256r1<waffle::UltraComposer>, UseBigfield::Yes>,
                       TestType<stdlib::secp256k1<waffle::UltraComposer>, UseBigfield::Yes>>
    TestTypes;

TYPED_TEST_SUITE(stdlib_biggroup, TestTypes);

TYPED_TEST(stdlib_biggroup, add)
{
    TestFixture::test_add();
}
TYPED_TEST(stdlib_biggroup, sub)
{
    TestFixture::test_sub();
}
TYPED_TEST(stdlib_biggroup, dbl)
{
    TestFixture::test_dbl();
}
TYPED_TEST(stdlib_biggroup, montgomery_ladder)
{
    TestFixture::test_montgomery_ladder();
}
HEAVY_TYPED_TEST(stdlib_biggroup, mul)
{
    TestFixture::test_mul();
}
HEAVY_TYPED_TEST(stdlib_biggroup, twin_mul)
{
    TestFixture::test_twin_mul();
}
HEAVY_TYPED_TEST(stdlib_biggroup, triple_mul)
{
    TestFixture::test_triple_mul();
}
HEAVY_TYPED_TEST(stdlib_biggroup, quad_mul)
{
    TestFixture::test_quad_mul();
}
HEAVY_TYPED_TEST(stdlib_biggroup, one)
{
    TestFixture::test_one();
}
HEAVY_TYPED_TEST(stdlib_biggroup, batch_mul)
{
    TestFixture::test_batch_mul();
}
HEAVY_TYPED_TEST(stdlib_biggroup, chain_add)
{

    TestFixture::test_chain_add();
}
HEAVY_TYPED_TEST(stdlib_biggroup, double_montgomery_ladder)
{

    TestFixture::test_double_montgomery_ladder();
}

HEAVY_TYPED_TEST(stdlib_biggroup, compute_naf)
{
    // ULTRATODO: make this work for secp curves
    if constexpr (TypeParam::Curve::type == waffle::CurveType::BN254) {
        size_t num_repetitions = 1;
        for (size_t i = 0; i < num_repetitions; i++) {
            TestFixture::test_compute_naf();
        }
    } else {
        GTEST_SKIP();
    }
}

/* These tests only work for UltraComposer */
HEAVY_TYPED_TEST(stdlib_biggroup, wnaf_batch_mul)
{
    if constexpr (TypeParam::Curve::Composer::type == waffle::ComposerType::PLOOKUP) {
        TestFixture::test_compute_wnaf();
    } else {
        GTEST_SKIP();
    }
}

/* the following test was only developed as a test of UltraComposer. It fails for Turbo and Standard in the case where
   Fr is a bigfield. */
HEAVY_TYPED_TEST(stdlib_biggroup, compute_wnaf)
{
    if constexpr (TypeParam::Curve::Composer::type != waffle::UltraComposer::type && TypeParam::use_bigfield) {
        GTEST_SKIP();
    } else {
        TestFixture::test_compute_wnaf();
    }
}

/* batch_mul with specified value of max_num_bits does not work for a biggroup formed over a big scalar field.
   We skip such cases in the next group of tests. */
HEAVY_TYPED_TEST(stdlib_biggroup, batch_mul_short_scalars)
{
    if constexpr (TypeParam::use_bigfield) {
        GTEST_SKIP();
    } else {
        TestFixture::test_batch_mul_short_scalars();
    }
}
HEAVY_TYPED_TEST(stdlib_biggroup, wnaf_batch_mul_128_bit)
{
    if constexpr (TypeParam::use_bigfield) {
        GTEST_SKIP();
    } else {
        TestFixture::test_wnaf_batch_mul_128_bit();
    }
}
HEAVY_TYPED_TEST(stdlib_biggroup, wnaf_batch_4)
{
    if constexpr (TypeParam::use_bigfield) {
        GTEST_SKIP();
    } else {
        TestFixture::test_wnaf_batch_4();
    }
}

/* The following tests are specific to BN254 and don't work when Fr is a bigfield */
HEAVY_TYPED_TEST(stdlib_biggroup, bn254_endo_batch_mul)
{
    if constexpr (TypeParam::Curve::type == waffle::CurveType::BN254 && !TypeParam::use_bigfield) {
        TestFixture::test_bn254_endo_batch_mul();
    } else {
        GTEST_SKIP();
    }
}
HEAVY_TYPED_TEST(stdlib_biggroup, mixed_mul_bn254_endo)
{
    if constexpr (TypeParam::Curve::type == waffle::CurveType::BN254 && !TypeParam::use_bigfield) {
        TestFixture::test_mixed_mul_bn254_endo();
    } else {
        GTEST_SKIP();
    }
}

/* The following tests are specific to SECP256k1 */
HEAVY_TYPED_TEST(stdlib_biggroup, wnaf_secp256k1)
{
    if constexpr (TypeParam::Curve::type == waffle::CurveType::SECP256K1) {
        TestFixture::test_wnaf_secp256k1();
    } else {
        GTEST_SKIP();
    }
}
HEAVY_TYPED_TEST(stdlib_biggroup, wnaf_8bit_secp256k1)
{
    if constexpr (TypeParam::Curve::type == waffle::CurveType::SECP256K1) {
        TestFixture::test_wnaf_8bit_secp256k1();
    } else {
        GTEST_SKIP();
    }
}
HEAVY_TYPED_TEST(stdlib_biggroup, ecdsa_mul_secp256k1)
{
    if constexpr (TypeParam::Curve::type == waffle::CurveType::SECP256K1) {
        TestFixture::test_ecdsa_mul_secp256k1();
    } else {
        GTEST_SKIP();
    }
}
} // namespace test_stdlib_biggroup
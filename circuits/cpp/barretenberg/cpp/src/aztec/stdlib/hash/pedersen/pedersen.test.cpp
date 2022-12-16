#include "pedersen.hpp"
#include "pedersen_plookup.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <crypto/pedersen/pedersen_lookup.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <numeric/random/engine.hpp>
#include <common/test.hpp>
#include <stdlib/primitives/curves/bn254.hpp>

namespace test_stdlib_pedersen {
using namespace barretenberg;
using namespace plonk;
namespace {
auto& engine = numeric::random::get_debug_engine();
}

template <typename Composer> class stdlib_pedersen : public testing::Test {
    typedef stdlib::bn254<Composer> curve;

    typedef typename curve::byte_array_ct byte_array_ct;
    typedef typename curve::fr_ct fr_ct;
    typedef typename curve::witness_ct witness_ct;
    typedef typename curve::public_witness_ct public_witness_ct;
    typedef typename stdlib::pedersen<Composer> pedersen;

  public:
    static grumpkin::g1::element pedersen_recover(const fr& left_in, const fr& right_in)
    {
        bool left_skew = false;
        bool right_skew = false;

        uint64_t left_wnafs[256] = { 0 };
        uint64_t right_wnafs[256] = { 0 };
        fr converted_left = left_in.from_montgomery_form();
        fr converted_right = right_in.from_montgomery_form();

        uint64_t* left_scalar = &(converted_left.data[0]);
        uint64_t* right_scalar = &(converted_right.data[0]);

        barretenberg::wnaf::fixed_wnaf<255, 1, 2>(left_scalar, &left_wnafs[0], left_skew, 0);
        barretenberg::wnaf::fixed_wnaf<255, 1, 2>(right_scalar, &right_wnafs[0], right_skew, 0);

        const auto compute_split_scalar = [](uint64_t* wnafs, const size_t range) {
            grumpkin::fr result = grumpkin::fr::zero();
            grumpkin::fr three = grumpkin::fr{ 3, 0, 0, 0 }.to_montgomery_form();
            for (size_t i = 0; i < range; ++i) {
                uint64_t entry = wnafs[i];
                grumpkin::fr prev = result + result;
                prev = prev + prev;
                if ((entry & 0xffffff) == 0) {
                    if (((entry >> 31UL) & 1UL) == 1UL) {
                        result = prev - grumpkin::fr::one();
                    } else {
                        result = prev + grumpkin::fr::one();
                    }
                } else {
                    if (((entry >> 31UL) & 1UL) == 1UL) {
                        result = prev - three;
                    } else {
                        result = prev + three;
                    }
                }
            }
            return result;
        };

        grumpkin::fr grumpkin_scalars[4]{ compute_split_scalar(&left_wnafs[0], 126),
                                          compute_split_scalar(&left_wnafs[126], 2),
                                          compute_split_scalar(&right_wnafs[0], 126),
                                          compute_split_scalar(&right_wnafs[126], 2) };

        grumpkin::g1::affine_element grumpkin_points[4]{
            crypto::pedersen::get_generator_data(crypto::pedersen::DEFAULT_GEN_1).generator,
            crypto::pedersen::get_generator_data(crypto::pedersen::DEFAULT_GEN_1).aux_generator,
            crypto::pedersen::get_generator_data(crypto::pedersen::DEFAULT_GEN_2).generator,
            crypto::pedersen::get_generator_data(crypto::pedersen::DEFAULT_GEN_2).aux_generator,
        };

        grumpkin::g1::element result_points[4]{
            grumpkin_points[0] * grumpkin_scalars[0],
            grumpkin_points[1] * grumpkin_scalars[1],
            grumpkin_points[2] * grumpkin_scalars[2],
            grumpkin_points[3] * grumpkin_scalars[3],
        };

        grumpkin::g1::element hash_output_left;
        grumpkin::g1::element hash_output_right;

        hash_output_left = result_points[0] + result_points[1];
        hash_output_right = result_points[2] + result_points[3];

        if (left_skew) {
            grumpkin::g1::affine_element left_skew_gen =
                crypto::pedersen::get_generator_data(crypto::pedersen::DEFAULT_GEN_1).skew_generator;
            hash_output_left -= left_skew_gen;
        }
        if (right_skew) {
            grumpkin::g1::affine_element right_skew_gen =
                crypto::pedersen::get_generator_data(crypto::pedersen::DEFAULT_GEN_2).skew_generator;
            hash_output_right -= right_skew_gen;
        }

        grumpkin::g1::element hash_output;
        hash_output = hash_output_left + hash_output_right;
        hash_output = hash_output.normalize();

        return hash_output;
    }

    static fr wnaf_recover(const fr& scalar)
    {
        bool skew = false;

        uint64_t wnafs[256] = { 0 };
        fr converted_scalar = scalar.from_montgomery_form();
        barretenberg::wnaf::fixed_wnaf<255, 1, 2>(&(converted_scalar.data[0]), &wnafs[0], skew, 0);

        uint256_t four_power = (uint256_t(1) << 254);
        uint256_t result = 0;
        for (size_t i = 0; i < 128; i++) {
            uint64_t quad = 2 * (wnafs[i] & stdlib::WNAF_MASK) + 1;
            bool sign = (wnafs[i] >> 31) & 1;
            if (sign) {
                result -= uint256_t(quad) * four_power;
            } else {
                result += uint256_t(quad) * four_power;
            }
            four_power >>= 2;
        }
        result -= skew;

        return fr(result);
    }

    static void test_pedersen()
    {

        Composer composer = Composer("../srs_db/ignition/");

        fr left_in = fr::random_element();
        fr right_in = fr::random_element();

        // ensure left has skew 1, right has skew 0
        if ((left_in.from_montgomery_form().data[0] & 1) == 1) {
            left_in += fr::one();
        }
        if ((right_in.from_montgomery_form().data[0] & 1) == 0) {
            right_in += fr::one();
        }

        fr_ct left = public_witness_ct(&composer, left_in);
        fr_ct right = witness_ct(&composer, right_in);

        composer.fix_witness(left.witness_index, left.get_value());
        composer.fix_witness(right.witness_index, right.get_value());

        fr_ct out = pedersen::compress(left, right);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);

        auto hash_output = pedersen_recover(left_in, right_in);

        fr recovered_left = wnaf_recover(left_in);
        fr recovered_right = wnaf_recover(right_in);
        EXPECT_EQ(left_in, recovered_left);
        EXPECT_EQ(right_in, recovered_right);

        EXPECT_EQ(out.get_value(), hash_output.x);

        fr compress_native = crypto::pedersen::compress_native({ left.get_value(), right.get_value() });
        EXPECT_EQ(out.get_value(), compress_native);
    }

    static void test_pedersen_edge_cases()
    {
        Composer composer = Composer("../srs_db/ignition/");

        fr zero_fr = fr::zero();
        fr one_fr = fr::one();
        fr r_minus_one_fr = fr::modulus - 1;
        fr r_minus_two_fr = fr::modulus - 2;
        fr r_fr = fr::modulus;

        fr_ct zero = witness_ct(&composer, zero_fr);
        fr_ct one = witness_ct(&composer, one_fr);
        fr_ct r_minus_one = witness_ct(&composer, r_minus_one_fr);
        fr_ct r_minus_two = witness_ct(&composer, r_minus_two_fr);
        fr_ct r = witness_ct(&composer, r_fr);

        fr_ct out_1_with_zero = pedersen::compress(zero, one);
        fr_ct out_1_with_r = pedersen::compress(r, one);
        fr_ct out_2 = pedersen::compress(r_minus_one, r_minus_two);
        fr_ct out_with_zero = pedersen::compress(out_1_with_zero, out_2);
        fr_ct out_with_r = pedersen::compress(out_1_with_r, out_2);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);

        auto hash_output_1_with_zero = pedersen_recover(zero_fr, one_fr);
        auto hash_output_1_with_r = pedersen_recover(r_fr, one_fr);
        auto hash_output_2 = pedersen_recover(r_minus_one_fr, r_minus_two_fr);

        EXPECT_EQ(out_1_with_zero.get_value(), hash_output_1_with_zero.x);
        EXPECT_EQ(out_1_with_r.get_value(), hash_output_1_with_r.x);
        EXPECT_EQ(out_2.get_value(), hash_output_2.x);
        EXPECT_EQ(bool(out_1_with_zero.get_value() == out_1_with_r.get_value()), true);

        fr recovered_zero = wnaf_recover(zero_fr);
        fr recovered_r = wnaf_recover(r_fr);
        fr recovered_one = wnaf_recover(one_fr);
        fr recovered_r_minus_one = wnaf_recover(r_minus_one_fr);
        fr recovered_r_minus_two = wnaf_recover(r_minus_two_fr);
        EXPECT_EQ(zero_fr, recovered_zero);
        EXPECT_EQ(r_fr, recovered_r);
        EXPECT_EQ(bool(recovered_zero == recovered_r), true);
        EXPECT_EQ(one_fr, recovered_one);
        EXPECT_EQ(r_minus_one_fr, recovered_r_minus_one);
        EXPECT_EQ(r_minus_two_fr, recovered_r_minus_two);

        fr compress_native_1_with_zero = crypto::pedersen::compress_native({ zero.get_value(), one.get_value() });
        fr compress_native_1_with_r = crypto::pedersen::compress_native({ r.get_value(), one.get_value() });
        fr compress_native_2 = crypto::pedersen::compress_native({ r_minus_one.get_value(), r_minus_two.get_value() });
        fr compress_native_with_zero =
            crypto::pedersen::compress_native({ out_1_with_zero.get_value(), out_2.get_value() });
        fr compress_native_with_r = crypto::pedersen::compress_native({ out_1_with_r.get_value(), out_2.get_value() });

        EXPECT_EQ(out_1_with_zero.get_value(), compress_native_1_with_zero);
        EXPECT_EQ(out_1_with_r.get_value(), compress_native_1_with_r);
        EXPECT_EQ(out_2.get_value(), compress_native_2);
        EXPECT_EQ(out_with_zero.get_value(), compress_native_with_zero);
        EXPECT_EQ(out_with_r.get_value(), compress_native_with_r);
        EXPECT_EQ(compress_native_with_zero, compress_native_with_r);
    }

    static void test_pedersen_large()
    {
        Composer composer = Composer("../srs_db/ignition/");

        fr left_in = fr::random_element();
        fr right_in = fr::random_element();
        // ensure left has skew 1, right has skew 0
        if ((left_in.from_montgomery_form().data[0] & 1) == 1) {
            left_in += fr::one();
        }
        if ((right_in.from_montgomery_form().data[0] & 1) == 0) {
            right_in += fr::one();
        }
        fr_ct left = witness_ct(&composer, left_in);
        fr_ct right = witness_ct(&composer, right_in);

        for (size_t i = 0; i < 256; ++i) {
            left = pedersen::compress(left, right);
        }

        composer.set_public_input(left.witness_index);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_pedersen_large_unrolled()
    {
        Composer composer = Composer("../srs_db/ignition/");

        fr left_in = fr::random_element();
        fr right_in = fr::random_element();
        // ensure left has skew 1, right has skew 0
        if ((left_in.from_montgomery_form().data[0] & 1) == 1) {
            left_in += fr::one();
        }
        if ((right_in.from_montgomery_form().data[0] & 1) == 0) {
            right_in += fr::one();
        }
        fr_ct left = witness_ct(&composer, left_in);
        fr_ct right = witness_ct(&composer, right_in);

        for (size_t i = 0; i < 256; ++i) {
            left = pedersen::compress(left, right);
        }

        composer.set_public_input(left.witness_index);

        auto prover = composer.create_unrolled_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_unrolled_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool result = verifier.verify_proof(proof);
        EXPECT_EQ(result, true);
    }

    static void test_compress_byte_array()
    {
        const size_t num_input_bytes = 351;

        Composer composer = Composer("../srs_db/ignition/");

        std::vector<uint8_t> input;
        input.reserve(num_input_bytes);
        for (size_t i = 0; i < num_input_bytes; ++i) {
            input.push_back(engine.get_random_uint8());
        }

        fr expected = crypto::pedersen::compress_native(input);

        byte_array_ct circuit_input(&composer, input);
        auto result = pedersen::compress(circuit_input);

        EXPECT_EQ(result.get_value(), expected);

        auto prover = composer.create_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_multi_compress()
    {
        Composer composer = Composer("../srs_db/ignition/");

        for (size_t i = 0; i < 7; ++i) {
            std::vector<barretenberg::fr> inputs;
            inputs.push_back(barretenberg::fr::random_element());
            inputs.push_back(barretenberg::fr::random_element());
            inputs.push_back(barretenberg::fr::random_element());
            inputs.push_back(barretenberg::fr::random_element());

            if (i == 1) {
                inputs[0] = barretenberg::fr(0);
            }
            if (i == 2) {
                inputs[1] = barretenberg::fr(0);
                inputs[2] = barretenberg::fr(0);
            }
            if (i == 3) {
                inputs[3] = barretenberg::fr(0);
            }
            if (i == 4) {
                inputs[0] = barretenberg::fr(0);
                inputs[3] = barretenberg::fr(0);
            }
            if (i == 5) {
                inputs[0] = barretenberg::fr(0);
                inputs[1] = barretenberg::fr(0);
                inputs[2] = barretenberg::fr(0);
                inputs[3] = barretenberg::fr(0);
            }
            if (i == 6) {
                inputs[1] = barretenberg::fr(1);
            }
            std::vector<fr_ct> witnesses;
            for (auto input : inputs) {
                witnesses.push_back(witness_ct(&composer, input));
            }

            barretenberg::fr expected = crypto::pedersen::compress_native(inputs);

            fr_ct result = pedersen::compress(witnesses);
            EXPECT_EQ(result.get_value(), expected);
        }

        auto prover = composer.create_unrolled_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_unrolled_verifier();

        waffle::plonk_proof proof = prover.construct_proof();

        bool proof_result = verifier.verify_proof(proof);
        EXPECT_EQ(proof_result, true);
    }

    static void test_compress_eight()
    {
        Composer composer = Composer("../srs_db/ignition/");

        std::vector<grumpkin::fq> inputs;
        inputs.reserve(8);
        std::vector<plonk::stdlib::field_t<Composer>> witness_inputs;

        for (size_t i = 0; i < 8; ++i) {
            inputs.emplace_back(barretenberg::fr::random_element());
            witness_inputs.emplace_back(witness_ct(&composer, inputs[i]));
        }

        constexpr size_t hash_idx = 10;
        grumpkin::fq expected = crypto::pedersen::compress_native(inputs, hash_idx);
        auto result = pedersen::compress(witness_inputs, hash_idx);

        EXPECT_EQ(result.get_value(), expected);
    }

    static void test_compress_constants()
    {
        Composer composer = Composer("../srs_db/ignition/");

        std::vector<barretenberg::fr> inputs;
        std::vector<plonk::stdlib::field_t<Composer>> witness_inputs;

        for (size_t i = 0; i < 8; ++i) {
            inputs.push_back(barretenberg::fr::random_element());
            if (i % 2 == 1) {
                witness_inputs.push_back(witness_ct(&composer, inputs[i]));
            } else {
                witness_inputs.push_back(fr_ct(&composer, inputs[i]));
            }
        }

        barretenberg::fr expected = crypto::pedersen::compress_native(inputs);
        auto result = pedersen::compress(witness_inputs);

        EXPECT_EQ(result.get_value(), expected);
    }
};

typedef testing::Types<waffle::UltraComposer, waffle::TurboComposer, waffle::StandardComposer> ComposerTypes;

TYPED_TEST_SUITE(stdlib_pedersen, ComposerTypes);

TYPED_TEST(stdlib_pedersen, small)
{
    TestFixture::test_pedersen();
};

TYPED_TEST(stdlib_pedersen, edge_cases)
{
    TestFixture::test_pedersen_edge_cases();
};

HEAVY_TYPED_TEST(stdlib_pedersen, large)
{
    TestFixture::test_pedersen_large();
};

HEAVY_TYPED_TEST(stdlib_pedersen, large_unrolled)
{
    TestFixture::test_pedersen_large_unrolled();
};

TYPED_TEST(stdlib_pedersen, compress_byte_array)
{
    TestFixture::test_compress_byte_array();
};

TYPED_TEST(stdlib_pedersen, multi_compress)
{
    TestFixture::test_multi_compress();
};

TYPED_TEST(stdlib_pedersen, compress_eight)
{
    TestFixture::test_compress_eight();
};

TYPED_TEST(stdlib_pedersen, compress_constants)
{
    TestFixture::test_compress_constants();
};

// Tests of Plookup-based Pedersen hash
namespace plookup_pedersen_tests {
typedef stdlib::field_t<waffle::UltraComposer> field_ct;
typedef stdlib::witness_t<waffle::UltraComposer> witness_ct;
TEST(stdlib_pedersen, test_pedersen_plookup)
{
    waffle::UltraComposer composer = waffle::UltraComposer();

    fr left_in = fr::random_element();
    fr right_in = fr::random_element();

    field_ct left = witness_ct(&composer, left_in);
    field_ct right = witness_ct(&composer, right_in);

    field_ct result = stdlib::pedersen_plookup<waffle::UltraComposer>::compress(left, right);

    fr expected = crypto::pedersen::lookup::hash_pair(left_in, right_in);

    EXPECT_EQ(result.get_value(), expected);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_pedersen, test_compress_many_plookup)
{
    waffle::UltraComposer composer = waffle::UltraComposer();

    std::vector<fr> input_values{
        fr::random_element(), fr::random_element(), fr::random_element(),
        fr::random_element(), fr::random_element(), fr::random_element(),
    };
    std::vector<field_ct> inputs;
    for (const auto& input : input_values) {
        inputs.emplace_back(witness_ct(&composer, input));
    }

    const size_t hash_idx = 20;

    field_ct result = stdlib::pedersen_plookup<waffle::UltraComposer>::compress(inputs, hash_idx);

    auto expected = crypto::pedersen::lookup::compress_native(input_values, hash_idx);

    EXPECT_EQ(result.get_value(), expected);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_pedersen, test_merkle_damgard_compress_plookup)
{
    waffle::UltraComposer composer = waffle::UltraComposer();

    std::vector<fr> input_values{
        fr::random_element(), fr::random_element(), fr::random_element(),
        fr::random_element(), fr::random_element(), fr::random_element(),
    };
    std::vector<field_ct> inputs;
    for (const auto& input : input_values) {
        inputs.emplace_back(witness_ct(&composer, input));
    }
    field_ct iv = witness_ct(&composer, fr(10));

    field_ct result = stdlib::pedersen_plookup<waffle::UltraComposer>::merkle_damgard_compress(inputs, iv).x;

    auto expected = crypto::pedersen::lookup::merkle_damgard_compress(input_values, 10);

    EXPECT_EQ(result.get_value(), expected.normalize().x);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    waffle::plonk_proof proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
} // namespace plookup_pedersen_tests
} // namespace test_stdlib_pedersen

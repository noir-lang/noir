#include "pedersen.hpp"
#include "pedersen_plookup.hpp"
#include <crypto/pedersen/pedersen.hpp>
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

        bool left_skew = false;
        bool right_skew = false;

        uint64_t left_wnafs[255] = { 0 };
        uint64_t right_wnafs[255] = { 0 };

        if ((left_in.from_montgomery_form().data[0] & 1) == 0) {
            fr two = fr::one() + fr::one();
            left_in = left_in - two;
        }
        if ((right_in.from_montgomery_form().data[0] & 1) == 0) {
            fr two = fr::one() + fr::one();
            right_in = right_in - two;
        }
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
                if ((entry & stdlib::WNAF_MASK) == 0) {
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
        if (left_skew) {
            grumpkin_scalars[1] += grumpkin::fr::one();
        }
        if (right_skew) {
            grumpkin_scalars[3] += grumpkin::fr::one();
        }

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

        grumpkin::g1::element hash_output;
        hash_output = hash_output_left + hash_output_right;
        hash_output = hash_output.normalize();

        EXPECT_EQ(out.get_value(), hash_output.x);

        fr compress_native = crypto::pedersen::compress_native({ left.get_value(), right.get_value() });
        EXPECT_EQ(out.get_value(), compress_native);
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

        auto expected = crypto::pedersen::compress_native(input);

        byte_array_ct circuit_input(&composer, input);
        auto result = pedersen::compress(circuit_input);

        EXPECT_EQ(result.get_value(), expected);

        auto prover = composer.create_unrolled_prover();

        printf("composer gates = %zu\n", composer.get_num_gates());
        auto verifier = composer.create_unrolled_verifier();

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

            fr_ct result = pedersen::compress(witnesses, true);
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

        std::array<barretenberg::fr, 8> inputs;
        std::array<plonk::stdlib::field_t<Composer>, 8> witness_inputs;

        for (size_t i = 0; i < 8; ++i) {
            inputs[i] = barretenberg::fr::random_element();
            witness_inputs[i] = witness_ct(&composer, inputs[i]);
        }

        barretenberg::fr expected = crypto::pedersen::compress_native(inputs);
        auto result = pedersen::compress(witness_inputs);

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

typedef testing::Types<waffle::StandardComposer,
                       waffle::TurboComposer //,
                       // waffle::PlookupComposer
                       >
    ComposerTypes;

TYPED_TEST_SUITE(stdlib_pedersen, ComposerTypes);

TYPED_TEST(stdlib_pedersen, small)
{
    TestFixture::test_pedersen();
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

} // namespace test_stdlib_pedersen

// PLOOKUP REMNANTS BELOW HERE

// TEST(stdlib_pedersen, test_pedersen_plookup)
// {
//     typedef stdlib::field_t<waffle::PlookupComposer> field_pt;
//     typedef stdlib::witness_t<waffle::PlookupComposer> witness_pt;

//     waffle::PlookupComposer composer = waffle::PlookupComposer();

//     fr left_in = fr::random_element();
//     fr right_in = fr::random_element();

//     field_pt left = witness_pt(&composer, left_in);
//     field_pt right = witness_pt(&composer, right_in);

//     field_pt result = stdlib::pedersen<waffle::PlookupComposer>::compress(left, right);

//     fr expected = crypto::pedersen::sidon::compress_native(left_in, right_in);

//     EXPECT_EQ(result.get_value(), expected);

//     auto prover = composer.create_prover();

//     printf("composer gates = %zu\n", composer.get_num_gates());
//     auto verifier = composer.create_verifier();

//     waffle::plonk_proof proof = prover.construct_proof();

//     bool proof_result = verifier.verify_proof(proof);
//     EXPECT_EQ(proof_result, true);
// }

// TEST(stdlib_pedersen, test_compress_many_plookup)
// {
//     typedef stdlib::field_t<waffle::PlookupComposer> field_pt;
//     typedef stdlib::witness_t<waffle::PlookupComposer> witness_pt;

//     waffle::PlookupComposer composer = waffle::PlookupComposer();

//     std::vector<fr> input_values{
//         fr::random_element(), fr::random_element(), fr::random_element(),
//         fr::random_element(), fr::random_element(), fr::random_element(),
//     };
//     std::vector<field_pt> inputs;
//     for (const auto& input : input_values) {
//         inputs.emplace_back(witness_pt(&composer, input));
//     }

//     field_pt result = stdlib::pedersen<waffle::PlookupComposer>::compress(inputs);

//     auto t0 = crypto::pedersen::sidon::compress_native(input_values[0], input_values[1]);
//     auto t1 = crypto::pedersen::sidon::compress_native(input_values[2], input_values[3]);
//     auto t2 = crypto::pedersen::sidon::compress_native(input_values[4], input_values[5]);
//     auto t3 = crypto::pedersen::sidon::compress_native(0, 0);

//     auto t4 = crypto::pedersen::sidon::compress_native(t0, t1);
//     auto t5 = crypto::pedersen::sidon::compress_native(t2, t3);

//     auto expected = crypto::pedersen::sidon::compress_native(t4, t5);

//     EXPECT_EQ(result.get_value(), expected);

//     auto prover = composer.create_prover();

//     printf("composer gates = %zu\n", composer.get_num_gates());
//     auto verifier = composer.create_verifier();

//     waffle::plonk_proof proof = prover.construct_proof();

//     bool proof_result = verifier.verify_proof(proof);
//     EXPECT_EQ(proof_result, true);
// }

// TEST(stdlib_pedersen, test_sidon_compress_constants)
// {
//     typedef stdlib::field_t<waffle::PlookupComposer> field_pt;
//     typedef stdlib::witness_t<waffle::PlookupComposer> witness_pt;

//     waffle::PlookupComposer composer = waffle::PlookupComposer();

//     std::vector<barretenberg::fr> inputs;
//     std::vector<plonk::stdlib::field_t<waffle::PlookupComposer>> witness_inputs;

//     for (size_t i = 0; i < 8; ++i) {
//         inputs.push_back(barretenberg::fr::random_element());
//         if (i % 2 == 1) {
//             witness_inputs.push_back(witness_pt(&composer, inputs[i]));
//         } else {
//             witness_inputs.push_back(field_pt(&composer, inputs[i]));
//         }
//     }

//     barretenberg::fr expected = crypto::pedersen::sidon::compress_native(inputs);
//     auto result = stdlib::pedersen<waffle::PlookupComposer>::compress(witness_inputs);

//     EXPECT_EQ(result.get_value(), expected);
// }
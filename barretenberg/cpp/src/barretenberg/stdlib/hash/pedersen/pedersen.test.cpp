#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"
#include "pedersen.hpp"

using namespace bb;
namespace {
auto& engine = numeric::get_debug_randomness();
}

template <typename Builder> class StdlibPedersen : public testing::Test {
    using _curve = stdlib::bn254<Builder>;

    using byte_array_ct = typename _curve::byte_array_ct;
    using fr_ct = typename _curve::ScalarField;
    using witness_ct = typename _curve::witness_ct;
    using public_witness_ct = typename _curve::public_witness_ct;
    using pedersen_hash = typename stdlib::pedersen_hash<Builder>;

  public:
    static void test_pedersen()
    {

        Builder builder;

        fr left_in = fr::random_element();
        fr right_in = fr::random_element();

        // ensure left has skew 1, right has skew 0
        if ((left_in.from_montgomery_form().data[0] & 1) == 1) {
            left_in += fr::one();
        }
        if ((right_in.from_montgomery_form().data[0] & 1) == 0) {
            right_in += fr::one();
        }

        fr_ct left = public_witness_ct(&builder, left_in);
        fr_ct right = witness_ct(&builder, right_in);

        builder.fix_witness(left.witness_index, left.get_value());
        builder.fix_witness(right.witness_index, right.get_value());

        fr_ct out = pedersen_hash::hash({ left, right });

        info("num gates = ", builder.get_num_gates());

        bool result = builder.check_circuit();
        EXPECT_EQ(result, true);

        fr hash_native = crypto::pedersen_hash::hash({ left.get_value(), right.get_value() });
        EXPECT_EQ(out.get_value(), hash_native);
    }

    static void test_pedersen_edge_cases()
    {
        Builder builder;

        fr zero_fr = fr::zero();
        fr one_fr = fr::one();
        fr r_minus_one_fr = fr::modulus - 1;
        fr r_minus_two_fr = fr::modulus - 2;
        fr r_fr = fr::modulus;

        fr_ct zero = witness_ct(&builder, zero_fr);
        fr_ct one = witness_ct(&builder, one_fr);
        fr_ct r_minus_one = witness_ct(&builder, r_minus_one_fr);
        fr_ct r_minus_two = witness_ct(&builder, r_minus_two_fr);
        fr_ct r = witness_ct(&builder, r_fr);

        fr_ct out_1_with_zero = pedersen_hash::hash({ zero, one });
        fr_ct out_1_with_r = pedersen_hash::hash({ r, one });
        fr_ct out_2 = pedersen_hash::hash({ r_minus_one, r_minus_two });
        fr_ct out_with_zero = pedersen_hash::hash({ out_1_with_zero, out_2 });
        fr_ct out_with_r = pedersen_hash::hash({ out_1_with_r, out_2 });

        info("num gates = ", builder.get_num_gates());

        bool result = builder.check_circuit();
        EXPECT_EQ(result, true);

        EXPECT_EQ(bool(out_1_with_zero.get_value() == out_1_with_r.get_value()), true);

        fr hash_native_1_with_zero = crypto::pedersen_hash::hash({ zero.get_value(), one.get_value() });
        fr hash_native_1_with_r = crypto::pedersen_hash::hash({ r.get_value(), one.get_value() });
        fr hash_native_2 = crypto::pedersen_hash::hash({ r_minus_one.get_value(), r_minus_two.get_value() });
        fr hash_native_with_zero = crypto::pedersen_hash::hash({ out_1_with_zero.get_value(), out_2.get_value() });
        fr hash_native_with_r = crypto::pedersen_hash::hash({ out_1_with_r.get_value(), out_2.get_value() });

        EXPECT_EQ(out_1_with_zero.get_value(), hash_native_1_with_zero);
        EXPECT_EQ(out_1_with_r.get_value(), hash_native_1_with_r);
        EXPECT_EQ(out_2.get_value(), hash_native_2);
        EXPECT_EQ(out_with_zero.get_value(), hash_native_with_zero);
        EXPECT_EQ(out_with_r.get_value(), hash_native_with_r);
        EXPECT_EQ(hash_native_with_zero, hash_native_with_r);
    }

    static void test_pedersen_large()
    {
        Builder builder;

        fr left_in = fr::random_element();
        fr right_in = fr::random_element();
        // ensure left has skew 1, right has skew 0
        if ((left_in.from_montgomery_form().data[0] & 1) == 1) {
            left_in += fr::one();
        }
        if ((right_in.from_montgomery_form().data[0] & 1) == 0) {
            right_in += fr::one();
        }
        fr_ct left = witness_ct(&builder, left_in);
        fr_ct right = witness_ct(&builder, right_in);

        for (size_t i = 0; i < 256; ++i) {
            left = pedersen_hash::hash({ left, right });
        }

        builder.set_public_input(left.witness_index);

        info("num gates = ", builder.get_num_gates());

        bool result = builder.check_circuit();
        EXPECT_EQ(result, true);
    }

    static void test_hash_byte_array()
    {
        const size_t num_input_bytes = 351;

        Builder builder;

        std::vector<uint8_t> input;
        input.reserve(num_input_bytes);
        for (size_t i = 0; i < num_input_bytes; ++i) {
            input.push_back(engine.get_random_uint8());
        }

        fr expected = crypto::pedersen_hash::hash_buffer(input);

        byte_array_ct circuit_input(&builder, input);
        auto result = pedersen_hash::hash_buffer(circuit_input);

        EXPECT_EQ(result.get_value(), expected);

        info("num gates = ", builder.get_num_gates());

        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    }

    static void test_multi_hash()
    {
        Builder builder;

        for (size_t i = 0; i < 7; ++i) {
            std::vector<fr> inputs;
            inputs.push_back(bb::fr::random_element());
            inputs.push_back(bb::fr::random_element());
            inputs.push_back(bb::fr::random_element());
            inputs.push_back(bb::fr::random_element());

            if (i == 1) {
                inputs[0] = fr(0);
            }
            if (i == 2) {
                inputs[1] = fr(0);
                inputs[2] = fr(0);
            }
            if (i == 3) {
                inputs[3] = fr(0);
            }
            if (i == 4) {
                inputs[0] = fr(0);
                inputs[3] = fr(0);
            }
            if (i == 5) {
                inputs[0] = fr(0);
                inputs[1] = fr(0);
                inputs[2] = fr(0);
                inputs[3] = fr(0);
            }
            if (i == 6) {
                inputs[1] = fr(1);
            }
            std::vector<fr_ct> witnesses;
            for (auto input : inputs) {
                witnesses.push_back(witness_ct(&builder, input));
            }

            fr expected = crypto::pedersen_hash::hash(inputs);

            fr_ct result = pedersen_hash::hash(witnesses);
            EXPECT_EQ(result.get_value(), expected);
        }

        info("num gates = ", builder.get_num_gates());

        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    }

    static void test_hash_eight()
    {
        Builder builder;

        std::vector<grumpkin::fq> inputs;
        inputs.reserve(8);
        std::vector<stdlib::field_t<Builder>> witness_inputs;

        for (size_t i = 0; i < 8; ++i) {
            inputs.emplace_back(bb::fr::random_element());
            witness_inputs.emplace_back(witness_ct(&builder, inputs[i]));
        }

        constexpr size_t hash_idx = 10;
        grumpkin::fq expected = crypto::pedersen_hash::hash(inputs, hash_idx);
        auto result = pedersen_hash::hash(witness_inputs, hash_idx);

        EXPECT_EQ(result.get_value(), expected);
    }

    static void test_hash_constants()
    {
        Builder builder;

        std::vector<fr> inputs;
        std::vector<stdlib::field_t<Builder>> witness_inputs;

        for (size_t i = 0; i < 8; ++i) {
            inputs.push_back(bb::fr::random_element());
            if (i % 2 == 1) {
                witness_inputs.push_back(witness_ct(&builder, inputs[i]));
            } else {
                witness_inputs.push_back(fr_ct(&builder, inputs[i]));
            }
        }

        fr expected = crypto::pedersen_hash::hash(inputs);
        auto result = pedersen_hash::hash(witness_inputs);

        EXPECT_EQ(result.get_value(), expected);
    }
};

using CircuitTypes = testing::Types<bb::StandardCircuitBuilder, bb::UltraCircuitBuilder>;

TYPED_TEST_SUITE(StdlibPedersen, CircuitTypes);

TYPED_TEST(StdlibPedersen, TestHash)
{
    using Builder = TypeParam;
    using field_ct = stdlib::field_t<Builder>;
    using witness_ct = stdlib::witness_t<Builder>;
    auto composer = Builder();

    const size_t num_inputs = 10;

    std::vector<field_ct> inputs;
    std::vector<fr> inputs_native;

    for (size_t i = 0; i < num_inputs; ++i) {
        const auto element = fr::random_element(&engine);
        inputs_native.emplace_back(element);
        inputs.emplace_back(field_ct(witness_ct(&composer, element)));
    }

    auto result = stdlib::pedersen_hash<Builder>::hash(inputs);
    auto expected = crypto::pedersen_hash::hash(inputs_native);

    EXPECT_EQ(result.get_value(), expected);

    bool proof_result = composer.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TYPED_TEST(StdlibPedersen, Small)
{
    TestFixture::test_pedersen();
};

TYPED_TEST(StdlibPedersen, EdgeCases)
{
    TestFixture::test_pedersen_edge_cases();
};

HEAVY_TYPED_TEST(StdlibPedersen, Large)
{
    TestFixture::test_pedersen_large();
};

TYPED_TEST(StdlibPedersen, HashByteArray)
{
    TestFixture::test_hash_byte_array();
};

TYPED_TEST(StdlibPedersen, MultiHash)
{
    TestFixture::test_multi_hash();
};

TYPED_TEST(StdlibPedersen, HashEight)
{
    TestFixture::test_hash_eight();
};

TYPED_TEST(StdlibPedersen, HashConstants)
{
    TestFixture::test_hash_constants();
};

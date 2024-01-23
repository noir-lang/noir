#include "poseidon2.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/crypto/poseidon2/poseidon2.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"

using namespace bb;
namespace {
auto& engine = numeric::get_debug_randomness();
}

template <typename Builder> class StdlibPoseidon2 : public testing::Test {
    using _curve = stdlib::bn254<Builder>;

    using byte_array_ct = typename _curve::byte_array_ct;
    using fr_ct = typename _curve::ScalarField;
    using witness_ct = typename _curve::witness_ct;
    using public_witness_ct = typename _curve::public_witness_ct;
    using poseidon2 = typename stdlib::poseidon2<Builder>;
    using native_poseidon2 = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>;

  public:
    /**
     * @brief Call poseidon2 on a vector of inputs
     *
     * @param num_inputs
     */
    static void test_hash(size_t num_inputs)
    {
        using field_ct = stdlib::field_t<Builder>;
        using witness_ct = stdlib::witness_t<Builder>;
        auto builder = Builder();

        std::vector<field_ct> inputs;
        std::vector<fr> inputs_native;

        for (size_t i = 0; i < num_inputs; ++i) {
            const auto element = fr::random_element(&engine);
            inputs_native.emplace_back(element);
            inputs.emplace_back(field_ct(witness_ct(&builder, element)));
        }

        auto result = stdlib::poseidon2<Builder>::hash(builder, inputs);
        auto expected = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(inputs_native);

        EXPECT_EQ(result.get_value(), expected);

        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    }

    /**
     * @brief Call poseidon2 on two inputs repeatedly.
     *
     * @param num_inputs
     */
    static void test_hash_repeated_pairs(size_t num_inputs)
    {
        Builder builder;

        fr left_in = fr::random_element();
        fr right_in = fr::random_element();

        fr_ct left = witness_ct(&builder, left_in);
        fr_ct right = witness_ct(&builder, right_in);

        // num_inputs - 1 iterations since the first hash hashes two elements
        for (size_t i = 0; i < num_inputs - 1; ++i) {
            left = poseidon2::hash(builder, { left, right });
        }

        builder.set_public_input(left.witness_index);

        info("num gates = ", builder.get_num_gates());

        bool result = builder.check_circuit();
        EXPECT_EQ(result, true);
    }
    /**
     * @brief Call poseidon2 hash_buffer on a vector of bytes
     *
     * @param num_input_bytes
     */
    static void test_hash_byte_array(size_t num_input_bytes)
    {
        Builder builder;

        std::vector<uint8_t> input;
        input.reserve(num_input_bytes);
        for (size_t i = 0; i < num_input_bytes; ++i) {
            input.push_back(engine.get_random_uint8());
        }

        fr expected = native_poseidon2::hash_buffer(input);

        byte_array_ct circuit_input(&builder, input);
        auto result = poseidon2::hash_buffer(builder, circuit_input);

        EXPECT_EQ(result.get_value(), expected);

        info("num gates = ", builder.get_num_gates());

        bool proof_result = builder.check_circuit();
        EXPECT_EQ(proof_result, true);
    }

    static void test_hash_zeros(size_t num_inputs)
    {
        Builder builder;

        std::vector<fr> inputs;
        inputs.reserve(num_inputs);
        std::vector<stdlib::field_t<Builder>> witness_inputs;

        for (size_t i = 0; i < num_inputs; ++i) {
            inputs.emplace_back(0);
            witness_inputs.emplace_back(witness_ct(&builder, inputs[i]));
        }

        fr expected = native_poseidon2::hash(inputs);
        auto result = poseidon2::hash(builder, witness_inputs);

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

        fr expected = native_poseidon2::hash(inputs);
        auto result = poseidon2::hash(builder, witness_inputs);

        EXPECT_EQ(result.get_value(), expected);
    }
};

using CircuitTypes = testing::Types<bb::GoblinUltraCircuitBuilder>;

TYPED_TEST_SUITE(StdlibPoseidon2, CircuitTypes);

TYPED_TEST(StdlibPoseidon2, TestHashZeros)
{
    TestFixture::test_hash_zeros(8);
};

TYPED_TEST(StdlibPoseidon2, TestHashSmall)
{
    TestFixture::test_hash(10);
}

TYPED_TEST(StdlibPoseidon2, TestHashLarge)
{
    TestFixture::test_hash(1000);
}

TYPED_TEST(StdlibPoseidon2, TestHashRepeatedPairs)
{
    TestFixture::test_hash_repeated_pairs(256);
}

TYPED_TEST(StdlibPoseidon2, TestHashByteArraySmall)
{
    TestFixture::test_hash_byte_array(351);
};

TYPED_TEST(StdlibPoseidon2, TestHashByteArrayLarge)
{
    TestFixture::test_hash_byte_array(31000);
};

TYPED_TEST(StdlibPoseidon2, TestHashConstants)
{
    TestFixture::test_hash_constants();
};

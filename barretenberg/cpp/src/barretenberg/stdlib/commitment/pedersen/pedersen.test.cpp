#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/crypto/pedersen_commitment/c_bind.hpp"
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
    using pedersen_commitment = typename stdlib::pedersen_commitment<Builder>;

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

        auto out = pedersen_commitment::commit({ left, right });

        info("num gates = ", builder.get_num_gates());

        bool result = builder.check_circuit();
        EXPECT_EQ(result, true);

        auto commit_native = crypto::pedersen_commitment::commit_native({ left.get_value(), right.get_value() });

        EXPECT_EQ(out.x.get_value(), commit_native.x);
        EXPECT_EQ(out.y.get_value(), commit_native.y);
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

        auto expected = crypto::pedersen_commitment::commit_native(inputs);
        auto result = pedersen_commitment::commit(witness_inputs);

        EXPECT_EQ(result.x.get_value(), expected.x);
        EXPECT_EQ(result.y.get_value(), expected.y);
    }
};

using CircuitTypes = testing::Types<bb::StandardCircuitBuilder, bb::UltraCircuitBuilder>;

TYPED_TEST_SUITE(StdlibPedersen, CircuitTypes);

TYPED_TEST(StdlibPedersen, Small)
{
    TestFixture::test_pedersen();
};

TYPED_TEST(StdlibPedersen, HashConstants)
{
    TestFixture::test_hash_constants();
};

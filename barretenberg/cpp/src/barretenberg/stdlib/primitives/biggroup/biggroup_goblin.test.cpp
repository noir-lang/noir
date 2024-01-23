#include "barretenberg/common/test.hpp"
#include <type_traits>

#include "../biggroup/biggroup.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders.hpp"

#include "barretenberg/stdlib/primitives/curves/bn254.hpp"

#include "barretenberg/numeric/random/engine.hpp"
#include <memory>

namespace {
auto& engine = numeric::get_debug_randomness();
}

using namespace bb;

template <typename Curve> class stdlib_biggroup_goblin : public testing::Test {
    using element_ct = typename Curve::Element;
    using scalar_ct = typename Curve::ScalarField;

    using fq = typename Curve::BaseFieldNative;
    using fr = typename Curve::ScalarFieldNative;
    using g1 = typename Curve::GroupNative;
    using affine_element = typename g1::affine_element;
    using element = typename g1::element;

    using Builder = typename Curve::Builder;

    static constexpr auto EXPECT_CIRCUIT_CORRECTNESS = [](Builder& builder, bool expected_result = true) {
        info("builder gates = ", builder.get_num_gates());
        EXPECT_EQ(builder.check_circuit(), expected_result);
    };

  public:
    /**
     * @brief Test goblin-style batch mul
     * @details Check that 1) Goblin-style batch mul returns correct value, and 2) resulting circuit is correct
     *
     */
    static void test_goblin_style_batch_mul()
    {
        const size_t num_points = 5;
        Builder builder;

        std::vector<affine_element> points;
        std::vector<fr> scalars;
        for (size_t i = 0; i < num_points; ++i) {
            points.push_back(affine_element(element::random_element()));
            scalars.push_back(fr::random_element());
        }

        std::vector<element_ct> circuit_points;
        std::vector<scalar_ct> circuit_scalars;
        for (size_t i = 0; i < num_points; ++i) {
            circuit_points.push_back(element_ct::from_witness(&builder, points[i]));
            circuit_scalars.push_back(scalar_ct::from_witness(&builder, scalars[i]));
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

        EXPECT_CIRCUIT_CORRECTNESS(builder);
    }
};

using TestTypes = testing::Types<stdlib::bn254<bb::GoblinUltraCircuitBuilder>>;

TYPED_TEST_SUITE(stdlib_biggroup_goblin, TestTypes);

HEAVY_TYPED_TEST(stdlib_biggroup_goblin, batch_mul)
{
    TestFixture::test_goblin_style_batch_mul();
}

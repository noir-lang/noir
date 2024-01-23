#include "barretenberg/stdlib/primitives/group/cycle_group.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include <gtest/gtest.h>

#define STDLIB_TYPE_ALIASES                                                                                            \
    using Builder = TypeParam;                                                                                         \
    using cycle_group_ct = stdlib::cycle_group<Builder>;                                                               \
    using Curve = typename stdlib::cycle_group<Builder>::Curve;                                                        \
    using Element = typename Curve::Element;                                                                           \
    using AffineElement = typename Curve::AffineElement;                                                               \
    using Group = typename Curve::Group;                                                                               \
    using bool_ct = stdlib::bool_t<Builder>;                                                                           \
    using witness_ct = stdlib::witness_t<Builder>;

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();
}
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-local-typedefs"

template <class Builder> class CycleGroupTest : public ::testing::Test {
  public:
    using Curve = typename stdlib::cycle_group<Builder>::Curve;
    using Group = typename Curve::Group;

    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;

    static constexpr size_t num_generators = 110;
    static inline std::array<AffineElement, num_generators> generators{};

    static void SetUpTestSuite()
    {

        for (size_t i = 0; i < num_generators; ++i) {
            generators[i] = Group::one * Curve::ScalarField::random_element(&engine);
        }
    };
};

using CircuitTypes = ::testing::Types<bb::StandardCircuitBuilder, bb::UltraCircuitBuilder>;
TYPED_TEST_SUITE(CycleGroupTest, CircuitTypes);

TYPED_TEST(CycleGroupTest, TestDbl)
{
    STDLIB_TYPE_ALIASES;
    auto builder = Builder();

    auto lhs = TestFixture::generators[0];
    cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
    cycle_group_ct c;
    std::cout << "pre = " << builder.get_num_gates() << std::endl;
    for (size_t i = 0; i < 3; ++i) {
        c = a.dbl();
    }
    std::cout << "post = " << builder.get_num_gates() << std::endl;
    AffineElement expected(Element(lhs).dbl());
    AffineElement result = c.get_value();
    EXPECT_EQ(result, expected);

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TYPED_TEST(CycleGroupTest, TestUnconditionalAdd)
{
    STDLIB_TYPE_ALIASES;
    auto builder = Builder();

    auto add =
        [&](const AffineElement& lhs, const AffineElement& rhs, const bool lhs_constant, const bool rhs_constant) {
            cycle_group_ct a = lhs_constant ? cycle_group_ct(lhs) : cycle_group_ct::from_witness(&builder, lhs);
            cycle_group_ct b = rhs_constant ? cycle_group_ct(rhs) : cycle_group_ct::from_witness(&builder, rhs);
            cycle_group_ct c = a.unconditional_add(b);
            AffineElement expected(Element(lhs) + Element(rhs));
            AffineElement result = c.get_value();
            EXPECT_EQ(result, expected);
        };

    add(TestFixture::generators[0], TestFixture::generators[1], false, false);
    add(TestFixture::generators[0], TestFixture::generators[1], false, true);
    add(TestFixture::generators[0], TestFixture::generators[1], true, false);
    add(TestFixture::generators[0], TestFixture::generators[1], true, true);

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TYPED_TEST(CycleGroupTest, TestConstrainedUnconditionalAddSucceed)
{
    STDLIB_TYPE_ALIASES;
    auto builder = Builder();

    auto lhs = TestFixture::generators[0];
    auto rhs = TestFixture::generators[1];

    // case 1. valid unconditional add
    cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
    cycle_group_ct b = cycle_group_ct::from_witness(&builder, rhs);
    cycle_group_ct c = a.checked_unconditional_add(b);
    AffineElement expected(Element(lhs) + Element(rhs));
    AffineElement result = c.get_value();
    EXPECT_EQ(result, expected);

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TYPED_TEST(CycleGroupTest, TestConstrainedUnconditionalAddFail)
{
    STDLIB_TYPE_ALIASES;
    auto builder = Builder();

    auto lhs = TestFixture::generators[0];
    auto rhs = -TestFixture::generators[0]; // ruh roh

    // case 2. invalid unconditional add
    cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
    cycle_group_ct b = cycle_group_ct::from_witness(&builder, rhs);
    a.checked_unconditional_add(b);

    EXPECT_TRUE(builder.failed());

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, false);
}

TYPED_TEST(CycleGroupTest, TestAdd)
{
    STDLIB_TYPE_ALIASES;
    auto builder = Builder();

    auto lhs = TestFixture::generators[0];
    auto rhs = -TestFixture::generators[1];

    cycle_group_ct point_at_infinity = cycle_group_ct::from_witness(&builder, rhs);
    point_at_infinity.set_point_at_infinity(bool_ct(witness_ct(&builder, true)));

    // case 1. no edge-cases triggered
    {
        cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
        cycle_group_ct b = cycle_group_ct::from_witness(&builder, rhs);
        cycle_group_ct c = a + b;
        AffineElement expected(Element(lhs) + Element(rhs));
        AffineElement result = c.get_value();
        EXPECT_EQ(result, expected);
    }

    // case 2. lhs is point at infinity
    {
        cycle_group_ct a = point_at_infinity;
        cycle_group_ct b = cycle_group_ct::from_witness(&builder, rhs);
        cycle_group_ct c = a + b;
        AffineElement result = c.get_value();
        EXPECT_EQ(result, rhs);
    }

    // case 3. rhs is point at infinity
    {
        cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
        cycle_group_ct b = point_at_infinity;
        cycle_group_ct c = a + b;
        AffineElement result = c.get_value();
        EXPECT_EQ(result, lhs);
    }

    // case 4. both points are at infinity
    {
        cycle_group_ct a = point_at_infinity;
        cycle_group_ct b = point_at_infinity;
        cycle_group_ct c = a + b;
        EXPECT_TRUE(c.is_point_at_infinity().get_value());
        EXPECT_TRUE(c.get_value().is_point_at_infinity());
    }

    // case 5. lhs = -rhs
    {
        cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
        cycle_group_ct b = cycle_group_ct::from_witness(&builder, -lhs);
        cycle_group_ct c = a + b;
        EXPECT_TRUE(c.is_point_at_infinity().get_value());
        EXPECT_TRUE(c.get_value().is_point_at_infinity());
    }

    // case 6. lhs = rhs
    {
        cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
        cycle_group_ct b = cycle_group_ct::from_witness(&builder, lhs);
        cycle_group_ct c = a + b;
        AffineElement expected((Element(lhs)).dbl());
        AffineElement result = c.get_value();
        EXPECT_EQ(result, expected);
    }

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TYPED_TEST(CycleGroupTest, TestUnconditionalSubtract)
{
    STDLIB_TYPE_ALIASES;
    auto builder = Builder();

    auto add =
        [&](const AffineElement& lhs, const AffineElement& rhs, const bool lhs_constant, const bool rhs_constant) {
            cycle_group_ct a = lhs_constant ? cycle_group_ct(lhs) : cycle_group_ct::from_witness(&builder, lhs);
            cycle_group_ct b = rhs_constant ? cycle_group_ct(rhs) : cycle_group_ct::from_witness(&builder, rhs);
            cycle_group_ct c = a.unconditional_subtract(b);
            AffineElement expected(Element(lhs) - Element(rhs));
            AffineElement result = c.get_value();
            EXPECT_EQ(result, expected);
        };

    add(TestFixture::generators[0], TestFixture::generators[1], false, false);
    add(TestFixture::generators[0], TestFixture::generators[1], false, true);
    add(TestFixture::generators[0], TestFixture::generators[1], true, false);
    add(TestFixture::generators[0], TestFixture::generators[1], true, true);

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TYPED_TEST(CycleGroupTest, TestConstrainedUnconditionalSubtractSucceed)
{
    STDLIB_TYPE_ALIASES;
    auto builder = Builder();

    auto lhs = TestFixture::generators[0];
    auto rhs = TestFixture::generators[1];

    // case 1. valid unconditional add
    cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
    cycle_group_ct b = cycle_group_ct::from_witness(&builder, rhs);
    cycle_group_ct c = a.checked_unconditional_subtract(b);
    AffineElement expected(Element(lhs) - Element(rhs));
    AffineElement result = c.get_value();
    EXPECT_EQ(result, expected);

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TYPED_TEST(CycleGroupTest, TestConstrainedUnconditionalSubtractFail)
{
    STDLIB_TYPE_ALIASES;
    auto builder = Builder();

    auto lhs = TestFixture::generators[0];
    auto rhs = -TestFixture::generators[0]; // ruh roh

    // case 2. invalid unconditional add
    cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
    cycle_group_ct b = cycle_group_ct::from_witness(&builder, rhs);
    a.checked_unconditional_subtract(b);

    EXPECT_TRUE(builder.failed());

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, false);
}

TYPED_TEST(CycleGroupTest, TestSubtract)
{
    STDLIB_TYPE_ALIASES;
    using bool_ct = stdlib::bool_t<Builder>;
    using witness_ct = stdlib::witness_t<Builder>;
    auto builder = Builder();

    auto lhs = TestFixture::generators[0];
    auto rhs = -TestFixture::generators[1];

    cycle_group_ct point_at_infinity = cycle_group_ct::from_witness(&builder, rhs);
    point_at_infinity.set_point_at_infinity(bool_ct(witness_ct(&builder, true)));

    // case 1. no edge-cases triggered
    {
        cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
        cycle_group_ct b = cycle_group_ct::from_witness(&builder, rhs);
        cycle_group_ct c = a - b;
        AffineElement expected(Element(lhs) - Element(rhs));
        AffineElement result = c.get_value();
        EXPECT_EQ(result, expected);
    }

    // case 2. lhs is point at infinity
    {
        cycle_group_ct a = point_at_infinity;
        cycle_group_ct b = cycle_group_ct::from_witness(&builder, rhs);
        cycle_group_ct c = a - b;
        AffineElement result = c.get_value();
        EXPECT_EQ(result, -rhs);
    }

    // case 3. rhs is point at infinity
    {
        cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
        cycle_group_ct b = point_at_infinity;
        cycle_group_ct c = a - b;
        AffineElement result = c.get_value();
        EXPECT_EQ(result, lhs);
    }

    // case 4. both points are at infinity
    {
        cycle_group_ct a = point_at_infinity;
        cycle_group_ct b = point_at_infinity;
        cycle_group_ct c = a - b;
        EXPECT_TRUE(c.is_point_at_infinity().get_value());
        EXPECT_TRUE(c.get_value().is_point_at_infinity());
    }

    // case 5. lhs = -rhs
    {
        cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
        cycle_group_ct b = cycle_group_ct::from_witness(&builder, -lhs);
        cycle_group_ct c = a - b;
        AffineElement expected((Element(lhs)).dbl());
        AffineElement result = c.get_value();
        EXPECT_EQ(result, expected);
    }

    // case 6. lhs = rhs
    {
        cycle_group_ct a = cycle_group_ct::from_witness(&builder, lhs);
        cycle_group_ct b = cycle_group_ct::from_witness(&builder, lhs);
        cycle_group_ct c = a - b;
        EXPECT_TRUE(c.is_point_at_infinity().get_value());
        EXPECT_TRUE(c.get_value().is_point_at_infinity());
    }

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}

TYPED_TEST(CycleGroupTest, TestBatchMul)
{
    STDLIB_TYPE_ALIASES;
    auto builder = Builder();

    const size_t num_muls = 1;

    // case 1, general MSM with inputs that are combinations of constant and witnesses
    {
        std::vector<cycle_group_ct> points;
        std::vector<typename cycle_group_ct::cycle_scalar> scalars;
        Element expected = Group::point_at_infinity;

        for (size_t i = 0; i < num_muls; ++i) {
            auto element = TestFixture::generators[i];
            typename Group::subgroup_field scalar = Group::subgroup_field::random_element(&engine);

            // 1: add entry where point, scalar are witnesses
            expected += (element * scalar);
            points.emplace_back(cycle_group_ct::from_witness(&builder, element));
            scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, scalar));

            // 2: add entry where point is constant, scalar is witness
            expected += (element * scalar);
            points.emplace_back(cycle_group_ct(element));
            scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, scalar));

            // 3: add entry where point is witness, scalar is constant
            expected += (element * scalar);
            points.emplace_back(cycle_group_ct::from_witness(&builder, element));
            scalars.emplace_back(typename cycle_group_ct::cycle_scalar(scalar));

            // 4: add entry where point is constant, scalar is constant
            expected += (element * scalar);
            points.emplace_back(cycle_group_ct(element));
            scalars.emplace_back(typename cycle_group_ct::cycle_scalar(scalar));
        }
        auto result = cycle_group_ct::batch_mul(scalars, points);
        EXPECT_EQ(result.get_value(), AffineElement(expected));
    }

    // case 2, MSM that produces point at infinity
    {
        std::vector<cycle_group_ct> points;
        std::vector<typename cycle_group_ct::cycle_scalar> scalars;

        auto element = TestFixture::generators[0];
        typename Group::subgroup_field scalar = Group::subgroup_field::random_element(&engine);
        points.emplace_back(cycle_group_ct::from_witness(&builder, element));
        scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, scalar));

        points.emplace_back(cycle_group_ct::from_witness(&builder, element));
        scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, -scalar));

        auto result = cycle_group_ct::batch_mul(scalars, points);
        EXPECT_TRUE(result.is_point_at_infinity().get_value());
    }

    // case 3. Multiply by zero
    {
        std::vector<cycle_group_ct> points;
        std::vector<typename cycle_group_ct::cycle_scalar> scalars;

        auto element = TestFixture::generators[0];
        typename Group::subgroup_field scalar = 0;
        points.emplace_back(cycle_group_ct::from_witness(&builder, element));
        scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, scalar));
        auto result = cycle_group_ct::batch_mul(scalars, points);
        EXPECT_TRUE(result.is_point_at_infinity().get_value());
    }

    // case 4. Inputs are points at infinity
    {
        std::vector<cycle_group_ct> points;
        std::vector<typename cycle_group_ct::cycle_scalar> scalars;

        auto element = TestFixture::generators[0];
        typename Group::subgroup_field scalar = Group::subgroup_field::random_element(&engine);

        // is_infinity = witness
        {
            cycle_group_ct point = cycle_group_ct::from_witness(&builder, element);
            point.set_point_at_infinity(witness_ct(&builder, true));
            points.emplace_back(point);
            scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, scalar));
        }
        // is_infinity = constant
        {
            cycle_group_ct point = cycle_group_ct::from_witness(&builder, element);
            point.set_point_at_infinity(true);
            points.emplace_back(point);
            scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, scalar));
        }
        auto result = cycle_group_ct::batch_mul(scalars, points);
        EXPECT_TRUE(result.is_point_at_infinity().get_value());
    }

    // case 5, fixed-base MSM with inputs that are combinations of constant and witnesses (group elements are in lookup
    // table)
    {
        std::vector<cycle_group_ct> points;
        std::vector<typename cycle_group_ct::cycle_scalar> scalars;
        std::vector<typename Group::coordinate_field> scalars_native;
        Element expected = Group::point_at_infinity;
        for (size_t i = 0; i < num_muls; ++i) {
            auto element = plookup::fixed_base::table::LHS_GENERATOR_POINT;
            typename Group::subgroup_field scalar = Group::subgroup_field::random_element(&engine);

            // 1: add entry where point is constant, scalar is witness
            expected += (element * scalar);
            points.emplace_back(element);
            scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, scalar));
            scalars_native.emplace_back(uint256_t(scalar));

            // 2: add entry where point is constant, scalar is constant
            element = plookup::fixed_base::table::RHS_GENERATOR_POINT;
            expected += (element * scalar);
            points.emplace_back(element);
            scalars.emplace_back(typename cycle_group_ct::cycle_scalar(scalar));
            scalars_native.emplace_back(uint256_t(scalar));
        }
        auto result = cycle_group_ct::batch_mul(scalars, points);
        EXPECT_EQ(result.get_value(), AffineElement(expected));
        EXPECT_EQ(result.get_value(), crypto::pedersen_commitment::commit_native(scalars_native));
    }

    // case 6, fixed-base MSM with inputs that are combinations of constant and witnesses (some group elements are in
    // lookup table)
    {
        std::vector<cycle_group_ct> points;
        std::vector<typename cycle_group_ct::cycle_scalar> scalars;
        std::vector<typename Group::subgroup_field> scalars_native;
        Element expected = Group::point_at_infinity;
        for (size_t i = 0; i < num_muls; ++i) {
            auto element = plookup::fixed_base::table::LHS_GENERATOR_POINT;
            typename Group::subgroup_field scalar = Group::subgroup_field::random_element(&engine);

            // 1: add entry where point is constant, scalar is witness
            expected += (element * scalar);
            points.emplace_back(element);
            scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, scalar));
            scalars_native.emplace_back(scalar);

            // 2: add entry where point is constant, scalar is constant
            element = plookup::fixed_base::table::RHS_GENERATOR_POINT;
            expected += (element * scalar);
            points.emplace_back(element);
            scalars.emplace_back(typename cycle_group_ct::cycle_scalar(scalar));
            scalars_native.emplace_back(scalar);

            // // 3: add entry where point is constant, scalar is witness
            scalar = Group::subgroup_field::random_element(&engine);
            element = Group::one * Group::subgroup_field::random_element(&engine);
            expected += (element * scalar);
            points.emplace_back(element);
            scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, scalar));
            scalars_native.emplace_back(scalar);
        }
        auto result = cycle_group_ct::batch_mul(scalars, points);
        EXPECT_EQ(result.get_value(), AffineElement(expected));
    }

    // case 7, Fixed-base MSM where input scalars are 0
    {
        std::vector<cycle_group_ct> points;
        std::vector<typename cycle_group_ct::cycle_scalar> scalars;

        for (size_t i = 0; i < num_muls; ++i) {
            auto element = plookup::fixed_base::table::LHS_GENERATOR_POINT;
            typename Group::subgroup_field scalar = 0;

            // 1: add entry where point is constant, scalar is witness
            points.emplace_back((element));
            scalars.emplace_back(cycle_group_ct::cycle_scalar::from_witness(&builder, scalar));

            // // 2: add entry where point is constant, scalar is constant
            points.emplace_back((element));
            scalars.emplace_back(typename cycle_group_ct::cycle_scalar(scalar));
        }
        auto result = cycle_group_ct::batch_mul(scalars, points);
        EXPECT_EQ(result.is_point_at_infinity().get_value(), true);
    }

    bool check_result = builder.check_circuit();
    EXPECT_EQ(check_result, true);
}

TYPED_TEST(CycleGroupTest, TestMul)
{
    STDLIB_TYPE_ALIASES
    auto builder = Builder();

    const size_t num_muls = 5;

    // case 1, general MSM with inputs that are combinations of constant and witnesses
    {
        cycle_group_ct point;
        typename cycle_group_ct::cycle_scalar scalar;
        for (size_t i = 0; i < num_muls; ++i) {
            auto element = TestFixture::generators[i];
            typename Group::subgroup_field native_scalar = Group::subgroup_field::random_element(&engine);

            // 1: add entry where point, scalar are witnesses
            point = (cycle_group_ct::from_witness(&builder, element));
            scalar = (cycle_group_ct::cycle_scalar::from_witness(&builder, native_scalar));
            EXPECT_EQ((point * scalar).get_value(), (element * native_scalar));

            // 2: add entry where point is constant, scalar is witness
            point = (cycle_group_ct(element));
            scalar = (cycle_group_ct::cycle_scalar::from_witness(&builder, native_scalar));

            EXPECT_EQ((point * scalar).get_value(), (element * native_scalar));

            // 3: add entry where point is witness, scalar is constant
            point = (cycle_group_ct::from_witness(&builder, element));
            EXPECT_EQ((point * scalar).get_value(), (element * native_scalar));

            // 4: add entry where point is constant, scalar is constant
            point = (cycle_group_ct(element));
            EXPECT_EQ((point * scalar).get_value(), (element * native_scalar));
        }
    }

    bool proof_result = builder.check_circuit();
    EXPECT_EQ(proof_result, true);
}
#pragma GCC diagnostic pop

#include "./constraint_manager.hpp"
#include "./arithmetic_constraint.hpp"
#include "./grand_product_computation_constraint.hpp"
#include "./grand_product_initialization_constraint.hpp"
#include "./multivariates.hpp"
#include "./univariate.hpp"
#include "./challenge_container.hpp"
#include <ecc/curves/bn254/fr.hpp>
#include <numeric/random/engine.hpp>
#include "../transcript.hpp"

#include <common/mem.hpp>
#include <gtest/gtest.h>

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

using namespace honk::sumcheck;

namespace honk {
namespace sumcheck {

template <typename T> class ConstraintManagerTests : public testing::Test {};

typedef testing::Types<barretenberg::fr> FieldTypes;
TYPED_TEST_SUITE(ConstraintManagerTests, FieldTypes);

TYPED_TEST(ConstraintManagerTests, Constructor)
{
    using Field = TypeParam;
    using ArithmeticConstraint = ArithmeticConstraint<Field>;
    using GrandProductComputationConstraint = GrandProductComputationConstraint<Field>;
    using GrandProductInitializationConstraint = GrandProductInitializationConstraint<Field>;
    using ConstraintManager = ConstraintManager<ArithmeticConstraint,
                                                GrandProductComputationConstraint,
                                                GrandProductInitializationConstraint>;
    auto constraint_manager = ConstraintManager();
    EXPECT_EQ(std::get<0>(constraint_manager.constraints).CONSTRAINT_LENGTH, 4);
    EXPECT_EQ(std::get<1>(constraint_manager.constraints).CONSTRAINT_LENGTH, 5);
    EXPECT_EQ(std::get<2>(constraint_manager.constraints).CONSTRAINT_LENGTH, 3);
}
} // namespace sumcheck
} // namespace honk
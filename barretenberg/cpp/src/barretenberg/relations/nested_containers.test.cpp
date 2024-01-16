#include "barretenberg/relations/nested_containers.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include <gtest/gtest.h>

using namespace proof_system;

namespace proof_system::nested_contianers_tests {

using FF = bb::fr;

class NestedContainers : public testing::Test {};

TEST_F(NestedContainers, Univariate)
{
    static constexpr std::array<size_t, 3> LENGTHS = { 0, 1, 2 };
    static constexpr TupleOfUnivariates<FF, LENGTHS> tuple;
    static constexpr auto result0 = bb::Univariate<FF, 0>();
    static constexpr auto result1 = bb::Univariate<FF, 1>();
    static constexpr auto result2 = bb::Univariate<FF, 2>();
    EXPECT_EQ(std::get<0>(tuple), result0);
    EXPECT_EQ(std::get<1>(tuple), result1);
    EXPECT_EQ(std::get<2>(tuple), result2);
}

} // namespace proof_system::nested_contianers_tests

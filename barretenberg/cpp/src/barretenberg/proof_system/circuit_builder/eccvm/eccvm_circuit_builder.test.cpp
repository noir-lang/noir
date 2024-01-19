#include "eccvm_circuit_builder.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include <gtest/gtest.h>

using namespace bb;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

namespace eccvm_circuit_builder_tests {

template <typename Flavor> class ECCVMCircuitBuilderTests : public ::testing::Test {};

using FlavorTypes = ::testing::Types<bb::honk::flavor::ECCVM>;
TYPED_TEST_SUITE(ECCVMCircuitBuilderTests, FlavorTypes);

TYPED_TEST(ECCVMCircuitBuilderTests, BaseCase)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    bb::ECCVMCircuitBuilder<Flavor> circuit;
    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    typename G1::element b = generators[1];
    typename G1::element c = generators[2];
    Fr x = Fr::random_element(&engine);
    Fr y = Fr::random_element(&engine);

    typename G1::element expected_1 = (a * x) + a + a + (b * y) + (b * x) + (b * x);
    typename G1::element expected_2 = (a * x) + c + (b * x);

    circuit.add_accumulate(a);
    circuit.mul_accumulate(a, x);
    circuit.mul_accumulate(b, x);
    circuit.mul_accumulate(b, y);
    circuit.add_accumulate(a);
    circuit.mul_accumulate(b, x);
    circuit.eq_and_reset(expected_1);
    circuit.add_accumulate(c);
    circuit.mul_accumulate(a, x);
    circuit.mul_accumulate(b, x);
    circuit.eq_and_reset(expected_2);
    circuit.mul_accumulate(a, x);
    circuit.mul_accumulate(b, x);
    circuit.mul_accumulate(c, x);

    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, Add)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    bb::ECCVMCircuitBuilder<Flavor> circuit;

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];

    circuit.add_accumulate(a);

    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, Mul)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    bb::ECCVMCircuitBuilder<Flavor> circuit;
    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    circuit.mul_accumulate(a, x);

    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, ShortMul)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    bb::ECCVMCircuitBuilder<Flavor> circuit;
    auto generators = G1::derive_generators("test generators", 3);

    typename G1::element a = generators[0];
    uint256_t small_x = 0;
    // make sure scalar is less than 127 bits to fit in z1
    small_x.data[0] = engine.get_random_uint64();
    small_x.data[1] = engine.get_random_uint64() & 0xFFFFFFFFFFFFULL;
    Fr x = small_x;

    circuit.mul_accumulate(a, x);
    circuit.eq_and_reset(a * small_x);

    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EqFails)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    bb::ECCVMCircuitBuilder<Flavor> circuit;

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    circuit.mul_accumulate(a, x);
    circuit.eq_and_reset(a);
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, false);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EmptyRow)
{
    using Flavor = TypeParam;
    bb::ECCVMCircuitBuilder<Flavor> circuit;

    circuit.empty_row();

    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EmptyRowBetweenOps)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    bb::ECCVMCircuitBuilder<Flavor> circuit;

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    typename G1::element expected_1 = (a * x);

    circuit.mul_accumulate(a, x);
    circuit.empty_row();
    circuit.eq_and_reset(expected_1);

    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EndWithEq)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    bb::ECCVMCircuitBuilder<Flavor> circuit;

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    typename G1::element expected_1 = (a * x);

    circuit.mul_accumulate(a, x);
    circuit.eq_and_reset(expected_1);

    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EndWithAdd)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    bb::ECCVMCircuitBuilder<Flavor> circuit;

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    typename G1::element expected_1 = (a * x);

    circuit.mul_accumulate(a, x);
    circuit.eq_and_reset(expected_1);
    circuit.add_accumulate(a);

    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EndWithMul)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    bb::ECCVMCircuitBuilder<Flavor> circuit;

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    circuit.add_accumulate(a);
    circuit.eq_and_reset(a);
    circuit.mul_accumulate(a, x);

    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EndWithNoop)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    bb::ECCVMCircuitBuilder<Flavor> circuit;

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    circuit.add_accumulate(a);
    circuit.eq_and_reset(a);
    circuit.mul_accumulate(a, x);
    circuit.empty_row();
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, MSM)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;

    static constexpr size_t max_num_msms = 9;
    auto generators = G1::derive_generators("test generators", max_num_msms);

    const auto try_msms = [&](const size_t num_msms, auto& circuit) {
        std::vector<typename G1::element> points;
        std::vector<Fr> scalars;
        typename G1::element expected = G1::point_at_infinity;
        for (size_t i = 0; i < num_msms; ++i) {
            points.emplace_back(generators[i]);
            scalars.emplace_back(Fr::random_element(&engine));
            expected += (points[i] * scalars[i]);
            circuit.mul_accumulate(points[i], scalars[i]);
        }
        circuit.eq_and_reset(expected);
    };

    // single msms
    for (size_t j = 1; j < max_num_msms; ++j) {
        using Flavor = TypeParam;
        bb::ECCVMCircuitBuilder<Flavor> circuit;
        try_msms(j, circuit);
        bool result = circuit.check_circuit();
        EXPECT_EQ(result, true);
    }
    // chain msms
    bb::ECCVMCircuitBuilder<Flavor> circuit;
    for (size_t j = 1; j < 9; ++j) {
        try_msms(j, circuit);
    }
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}
} // namespace eccvm_circuit_builder_tests
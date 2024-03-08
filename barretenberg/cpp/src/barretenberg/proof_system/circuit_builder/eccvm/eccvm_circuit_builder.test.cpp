#include "eccvm_circuit_builder.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include <gtest/gtest.h>

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();

template <typename Flavor> class ECCVMCircuitBuilderTests : public ::testing::Test {};

using FlavorTypes = ::testing::Types<ECCVMFlavor>;
} // namespace

TYPED_TEST_SUITE(ECCVMCircuitBuilderTests, FlavorTypes);

TYPED_TEST(ECCVMCircuitBuilderTests, BaseCase)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    typename G1::element b = generators[1];
    typename G1::element c = generators[2];
    Fr x = Fr::random_element(&engine);
    Fr y = Fr::random_element(&engine);

    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    op_queue->add_accumulate(a);
    op_queue->mul_accumulate(a, x);
    op_queue->mul_accumulate(b, x);
    op_queue->mul_accumulate(b, y);
    op_queue->add_accumulate(a);
    op_queue->mul_accumulate(b, x);
    op_queue->eq();
    op_queue->add_accumulate(c);
    op_queue->mul_accumulate(a, x);
    op_queue->mul_accumulate(b, x);
    op_queue->eq();
    op_queue->mul_accumulate(a, x);
    op_queue->mul_accumulate(b, x);
    op_queue->mul_accumulate(c, x);

    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, Add)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];

    op_queue->add_accumulate(a);

    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, Mul)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->mul_accumulate(a, x);

    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, ShortMul)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);

    typename G1::element a = generators[0];
    uint256_t small_x = 0;
    // make sure scalar is less than 127 bits to fit in z1
    small_x.data[0] = engine.get_random_uint64();
    small_x.data[1] = engine.get_random_uint64() & 0xFFFFFFFFFFFFULL;
    Fr x = small_x;

    op_queue->mul_accumulate(a, x);
    op_queue->eq();

    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EqFails)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    using ECCVMOperation = eccvm::VMOperation<G1>;
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->mul_accumulate(a, x);
    // Tamper with the eq op such that the expected value is incorect
    op_queue->raw_ops.emplace_back(ECCVMOperation{ .add = false,
                                                   .mul = false,
                                                   .eq = true,
                                                   .reset = true,
                                                   .base_point = a,
                                                   .z1 = 0,
                                                   .z2 = 0,
                                                   .mul_scalar_full = 0 });
    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, false);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EmptyRow)
{
    using Flavor = TypeParam;
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    op_queue->empty_row();

    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EmptyRowBetweenOps)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->mul_accumulate(a, x);
    op_queue->empty_row();
    op_queue->eq();

    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EndWithEq)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->mul_accumulate(a, x);
    op_queue->eq();

    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EndWithAdd)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->mul_accumulate(a, x);
    op_queue->eq();
    op_queue->add_accumulate(a);

    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EndWithMul)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->add_accumulate(a);
    op_queue->eq();
    op_queue->mul_accumulate(a, x);

    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(ECCVMCircuitBuilderTests, EndWithNoop)
{
    using Flavor = TypeParam;
    using G1 = typename Flavor::CycleGroup;
    using Fr = typename G1::Fr;
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->add_accumulate(a);
    op_queue->eq();
    op_queue->mul_accumulate(a, x);

    op_queue->empty_row();
    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
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

    const auto compute_msms = [&](const size_t num_msms, auto& op_queue) {
        std::vector<typename G1::element> points;
        std::vector<Fr> scalars;
        typename G1::element expected = G1::point_at_infinity;
        for (size_t i = 0; i < num_msms; ++i) {
            points.emplace_back(generators[i]);
            scalars.emplace_back(Fr::random_element(&engine));
            expected += (points[i] * scalars[i]);
            op_queue->mul_accumulate(points[i], scalars[i]);
        }
        op_queue->eq();
    };

    // single msms
    for (size_t j = 1; j < max_num_msms; ++j) {
        using Flavor = TypeParam;
        std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

        compute_msms(j, op_queue);
        ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
        bool result = circuit.check_circuit();
        EXPECT_EQ(result, true);
    }
    // chain msms
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    for (size_t j = 1; j < 9; ++j) {
        compute_msms(j, op_queue);
    }
    ECCVMCircuitBuilder<Flavor> circuit{ op_queue };
    bool result = circuit.check_circuit();
    EXPECT_EQ(result, true);
}

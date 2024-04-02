#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/eccvm/eccvm_flavor.hpp"
#include "barretenberg/eccvm/eccvm_trace_checker.hpp"
#include <gtest/gtest.h>

using namespace bb;
using G1 = bb::g1;
using Fr = typename G1::Fr;

namespace {
auto& engine = numeric::get_debug_randomness();
} // namespace

TEST(ECCVMCircuitBuilderTests, BaseCase)
{
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

    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

TEST(ECCVMCircuitBuilderTests, Add)
{
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];

    op_queue->add_accumulate(a);

    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

TEST(ECCVMCircuitBuilderTests, Mul)
{
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->mul_accumulate(a, x);

    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

TEST(ECCVMCircuitBuilderTests, ShortMul)
{
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

    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

TEST(ECCVMCircuitBuilderTests, EqFails)
{
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
    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, false);
}

TEST(ECCVMCircuitBuilderTests, EmptyRow)
{
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    op_queue->empty_row();

    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

TEST(ECCVMCircuitBuilderTests, EmptyRowBetweenOps)
{
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->mul_accumulate(a, x);
    op_queue->empty_row();
    op_queue->eq();

    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

TEST(ECCVMCircuitBuilderTests, EndWithEq)
{
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->mul_accumulate(a, x);
    op_queue->eq();

    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

TEST(ECCVMCircuitBuilderTests, EndWithAdd)
{
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->mul_accumulate(a, x);
    op_queue->eq();
    op_queue->add_accumulate(a);

    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

TEST(ECCVMCircuitBuilderTests, EndWithMul)
{
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->add_accumulate(a);
    op_queue->eq();
    op_queue->mul_accumulate(a, x);

    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

TEST(ECCVMCircuitBuilderTests, EndWithNoop)
{
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    auto generators = G1::derive_generators("test generators", 3);
    typename G1::element a = generators[0];
    Fr x = Fr::random_element(&engine);

    op_queue->add_accumulate(a);
    op_queue->eq();
    op_queue->mul_accumulate(a, x);

    op_queue->empty_row();
    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

TEST(ECCVMCircuitBuilderTests, MSM)
{
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
        std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

        compute_msms(j, op_queue);
        ECCVMCircuitBuilder circuit{ op_queue };
        bool result = ECCVMTraceChecker::check(circuit);
        EXPECT_EQ(result, true);
    }
    // chain msms
    std::shared_ptr<ECCOpQueue> op_queue = std::make_shared<ECCOpQueue>();

    for (size_t j = 1; j < 9; ++j) {
        compute_msms(j, op_queue);
    }
    ECCVMCircuitBuilder circuit{ op_queue };
    bool result = ECCVMTraceChecker::check(circuit);
    EXPECT_EQ(result, true);
}

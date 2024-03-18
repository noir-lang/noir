#include "barretenberg/proof_system/op_queue/ecc_op_queue.hpp"
#include <gtest/gtest.h>

using namespace bb;

TEST(ECCOpQueueTest, Basic)
{
    ECCOpQueue op_queue;
    op_queue.add_accumulate(bb::g1::affine_one);
    EXPECT_EQ(op_queue.raw_ops[0].base_point, bb::g1::affine_one);
    op_queue.empty_row();
    EXPECT_EQ(op_queue.raw_ops[1].add, false);
}

TEST(ECCOpQueueTest, InternalAccumulatorCorrectness)
{
    using point = g1::affine_element;
    using scalar = fr;

    // Compute a simple point accumulation natively
    auto P1 = point::random_element();
    auto P2 = point::random_element();
    auto z = scalar::random_element();
    auto P_expected = P1 + P2 * z;

    // Add the same operations to the ECC op queue; the native computation is performed under the hood.
    ECCOpQueue op_queue;
    op_queue.add_accumulate(P1);
    op_queue.mul_accumulate(P2, z);

    // The correct result should now be stored in the accumulator within the op queue
    EXPECT_EQ(op_queue.get_accumulator(), P_expected);

    // Equivalently, we can check that the equality op returns the correct point
    EXPECT_EQ(op_queue.eq(), P_expected);

    // Adding an equality op should reset the accumulator to zero (the point at infinity)
    EXPECT_TRUE(op_queue.get_accumulator().is_point_at_infinity());
}

TEST(ECCOpQueueTest, PrependAndSwapTests)
{
    using point = g1::affine_element;
    using scalar = fr;

    // Compute a simple point accumulation natively
    auto P1 = point::random_element();
    auto P2 = point::random_element();
    auto z = scalar::random_element();

    // Add operations to a
    ECCOpQueue op_queue_a;
    op_queue_a.add_accumulate(P1 + P1);
    op_queue_a.mul_accumulate(P2, z + z);
    op_queue_a.reset();
    // Add different operations to b
    ECCOpQueue op_queue_b;
    op_queue_b.mul_accumulate(P2, z);
    op_queue_b.add_accumulate(P1);
    op_queue_b.reset();

    // Add same operations as to a
    ECCOpQueue op_queue_c;
    op_queue_c.add_accumulate(P1 + P1);
    op_queue_c.mul_accumulate(P2, z + z);
    op_queue_c.reset();

    // Swap b with a
    std::swap(op_queue_b, op_queue_a);

    // Check b==c
    for (size_t i = 0; i < op_queue_c.raw_ops.size(); i++) {
        EXPECT_EQ(op_queue_b.raw_ops[i], op_queue_c.raw_ops[i]);
    }

    // Prepend b to a
    op_queue_a.prepend_previous_queue(op_queue_b);

    // Append same operations as now in a to c
    op_queue_c.mul_accumulate(P2, z);
    op_queue_c.add_accumulate(P1);
    op_queue_c.reset();

    // Check a==c
    for (size_t i = 0; i < op_queue_c.raw_ops.size(); i++) {
        EXPECT_EQ(op_queue_a.raw_ops[i], op_queue_c.raw_ops[i]);
    }
}
#include <gtest/gtest.h>
#include "barretenberg/proof_system/op_queue/ecc_op_queue.hpp"

namespace proof_system::test_flavor {
TEST(ECCOpQueueTest, Basic)
{
    ECCOpQueue op_queue;
    op_queue.add_accumulate(barretenberg::g1::affine_one);
    EXPECT_EQ(op_queue.raw_ops[0].base_point, barretenberg::g1::affine_one);
    op_queue.empty_row();
    EXPECT_EQ(op_queue.raw_ops[1].add, false);
}

} // namespace proof_system::test_flavor

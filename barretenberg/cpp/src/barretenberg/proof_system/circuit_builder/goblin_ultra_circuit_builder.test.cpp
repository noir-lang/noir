#include "barretenberg/crypto/generators/generator_data.hpp"
#include "ultra_circuit_builder.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;

namespace {
auto& engine = numeric::random::get_debug_engine();
}
namespace proof_system {

/**
 * @brief Test the queueing of simple ecc ops via the Goblin builder
 * @details There are two things to check here: 1) When ecc ops are queued by the builder, the corresponding native
 * operations are performed correctly by the internal ecc op queue, and 2) The ecc op gate operands are correctly
 * encoded in the op_wires, i.e. the operands can be reconstructed as expected.
 *
 */
TEST(UltraCircuitBuilder, GoblinSimple)
{
    const size_t CHUNK_SIZE = plonk::NUM_LIMB_BITS_IN_FIELD_SIMULATION * 2;

    auto builder = UltraCircuitBuilder();

    // Compute a simple point accumulation natively
    auto P1 = g1::affine_element::random_element();
    auto P2 = g1::affine_element::random_element();
    auto z = fr::random_element();
    auto P_expected = P1 + P2 * z;

    // Add gates corresponding to the above operations
    builder.queue_ecc_add_accum(P1);
    builder.queue_ecc_mul_accum(P2, z);

    // Add equality op gates based on the internal accumulator
    auto eq_op_tuple = builder.queue_ecc_eq();

    // Check that we can reconstruct the coordinates of P_expected from the data in variables
    auto P_result_x_lo = uint256_t(builder.variables[eq_op_tuple.x_lo]);
    auto P_result_x_hi = uint256_t(builder.variables[eq_op_tuple.x_hi]);
    auto P_result_x = P_result_x_lo + (P_result_x_hi << CHUNK_SIZE);
    auto P_result_y_lo = uint256_t(builder.variables[eq_op_tuple.y_lo]);
    auto P_result_y_hi = uint256_t(builder.variables[eq_op_tuple.y_hi]);
    auto P_result_y = P_result_y_lo + (P_result_y_hi << CHUNK_SIZE);
    EXPECT_EQ(P_result_x, uint256_t(P_expected.x));
    EXPECT_EQ(P_result_y, uint256_t(P_expected.y));

    // Check that the accumulator in the op queue has been reset to 0
    auto accumulator = builder.op_queue.get_accumulator();
    EXPECT_EQ(accumulator, g1::affine_point_at_infinity);

    // Check number of ecc op "gates"/rows = 3 ops * 2 rows per op = 6
    EXPECT_EQ(builder.num_ecc_op_gates, 6);

    // Check that the expected op codes have been correctly recorded in the 1st op wire
    EXPECT_EQ(builder.ecc_op_wire_1[0], EccOpCode::ADD_ACCUM);
    EXPECT_EQ(builder.ecc_op_wire_1[2], EccOpCode::MUL_ACCUM);
    EXPECT_EQ(builder.ecc_op_wire_1[4], EccOpCode::EQUALITY);

    // Check that we can reconstruct the coordinates of P1 from the op_wires
    auto P1_x_lo = uint256_t(builder.variables[builder.ecc_op_wire_2[0]]);
    auto P1_x_hi = uint256_t(builder.variables[builder.ecc_op_wire_3[0]]);
    auto P1_x = P1_x_lo + (P1_x_hi << CHUNK_SIZE);
    EXPECT_EQ(P1_x, uint256_t(P1.x));
    auto P1_y_lo = uint256_t(builder.variables[builder.ecc_op_wire_4[0]]);
    auto P1_y_hi = uint256_t(builder.variables[builder.ecc_op_wire_2[1]]);
    auto P1_y = P1_y_lo + (P1_y_hi << CHUNK_SIZE);
    EXPECT_EQ(P1_y, uint256_t(P1.y));

    // Check that we can reconstruct the coordinates of P2 from the op_wires
    auto P2_x_lo = uint256_t(builder.variables[builder.ecc_op_wire_2[2]]);
    auto P2_x_hi = uint256_t(builder.variables[builder.ecc_op_wire_3[2]]);
    auto P2_x = P2_x_lo + (P2_x_hi << CHUNK_SIZE);
    EXPECT_EQ(P2_x, uint256_t(P2.x));
    auto P2_y_lo = uint256_t(builder.variables[builder.ecc_op_wire_4[2]]);
    auto P2_y_hi = uint256_t(builder.variables[builder.ecc_op_wire_2[3]]);
    auto P2_y = P2_y_lo + (P2_y_hi << CHUNK_SIZE);
    EXPECT_EQ(P2_y, uint256_t(P2.y));
}
} // namespace proof_system
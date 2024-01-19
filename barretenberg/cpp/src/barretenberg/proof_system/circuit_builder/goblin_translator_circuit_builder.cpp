/**
 * @file goblin_translator_circuit_builder.cpp
 * @author @Rumata888
 * @brief Circuit Logic generation for Goblin Plonk translator (checks equivalence of Queues/Transcripts for ECCVM and
 * Recursive Circuits)
 *
 * @copyright Copyright (c) 2023
 *
 */
#include "goblin_translator_circuit_builder.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/plonk/proof_system/constants.hpp"
#include "barretenberg/proof_system/op_queue/ecc_op_queue.hpp"
#include <cstddef>
namespace bb {
using ECCVMOperation = ECCOpQueue::ECCVMOperation;

/**
 * @brief Given the transcript values from the EccOpQueue, the values of the previous accumulator, batching challenge
 * and input x, compute witness for one step of accumulation
 *
 * @tparam Fq
 * @tparam Fr
 * @param op_code Opcode value
 * @param p_x_lo Low 136 bits of P.x
 * @param p_x_hi High 118 bits of P.x
 * @param p_y_lo Low 136 bits of P.y
 * @param p_y_hi High 118 bits of P.y
 * @param z1 z1 scalar
 * @param z2 z2 scalar
 * @param previous_accumulator The value of the previous accumulator (we assume standard decomposition into limbs)
 * @param batching_challenge_v The value of the challenge for batching polynomial evaluations
 * @param evaluation_input_x The value at which we evaluate the polynomials
 * @return GoblinTranslatorCircuitBuilder::AccumulationInput
 */
template <typename Fq, typename Fr>
GoblinTranslatorCircuitBuilder::AccumulationInput generate_witness_values(Fr op_code,
                                                                          Fr p_x_lo,
                                                                          Fr p_x_hi,
                                                                          Fr p_y_lo,
                                                                          Fr p_y_hi,
                                                                          Fr z1,
                                                                          Fr z2,
                                                                          Fq previous_accumulator,
                                                                          Fq batching_challenge_v,
                                                                          Fq evaluation_input_x)
{
    // All parameters are well-described in the header, this is just for convenience
    constexpr size_t NUM_LIMB_BITS = GoblinTranslatorCircuitBuilder::NUM_LIMB_BITS;
    constexpr size_t NUM_BINARY_LIMBS = GoblinTranslatorCircuitBuilder::NUM_BINARY_LIMBS;
    constexpr size_t NUM_MICRO_LIMBS = GoblinTranslatorCircuitBuilder::NUM_MICRO_LIMBS;
    constexpr size_t NUM_LAST_LIMB_BITS = GoblinTranslatorCircuitBuilder::NUM_LAST_LIMB_BITS;
    constexpr size_t MICRO_LIMB_BITS = GoblinTranslatorCircuitBuilder::MICRO_LIMB_BITS;
    constexpr size_t TOP_STANDARD_MICROLIMB_BITS = NUM_LAST_LIMB_BITS % MICRO_LIMB_BITS;
    constexpr size_t NUM_Z_BITS = GoblinTranslatorCircuitBuilder::NUM_Z_BITS;
    constexpr size_t TOP_Z_MICROLIMB_BITS = (NUM_Z_BITS % NUM_LIMB_BITS) % MICRO_LIMB_BITS;
    constexpr size_t TOP_QUOTIENT_MICROLIMB_BITS =
        (GoblinTranslatorCircuitBuilder::NUM_QUOTIENT_BITS % NUM_LIMB_BITS) % MICRO_LIMB_BITS;
    constexpr auto shift_1 = GoblinTranslatorCircuitBuilder::SHIFT_1;
    constexpr auto neg_modulus_limbs = GoblinTranslatorCircuitBuilder::NEGATIVE_MODULUS_LIMBS;
    constexpr auto shift_2_inverse = GoblinTranslatorCircuitBuilder::SHIFT_2_INVERSE;

    /**
     * @brief A small function to transform a native element Fq into its bigfield representation in Fr scalars
     *
     * @details We transform Fq into an integer and then split it into 68-bit limbs, then convert them to Fr.
     *
     */
    auto base_element_to_limbs = [](Fq& original) {
        uint256_t original_uint = original;
        return std::array<Fr, NUM_BINARY_LIMBS>({
            Fr(original_uint.slice(0, NUM_LIMB_BITS)),
            Fr(original_uint.slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS)),
            Fr(original_uint.slice(2 * NUM_LIMB_BITS, 3 * NUM_LIMB_BITS)),
            Fr(original_uint.slice(3 * NUM_LIMB_BITS, 4 * NUM_LIMB_BITS)),
        });
    };
    /**
     * @brief A small function to transform a uint512_t element into its 4 68-bit limbs in Fr scalars
     *
     * @details Split and integer stored in uint512_T into 4 68-bit chunks (we assume that it is lower than 2²⁷²),
     * convert to Fr
     *
     */
    auto uint512_t_to_limbs = [](uint512_t& original) {
        return std::array<Fr, NUM_BINARY_LIMBS>{ Fr(original.slice(0, NUM_LIMB_BITS).lo),
                                                 Fr(original.slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS).lo),
                                                 Fr(original.slice(2 * NUM_LIMB_BITS, 3 * NUM_LIMB_BITS).lo),
                                                 Fr(original.slice(3 * NUM_LIMB_BITS, 4 * NUM_LIMB_BITS).lo) };
    };

    /**
     * @brief A method for splitting wide limbs (P_x_lo, P_y_hi, etc) into two limbs
     *
     */
    auto split_wide_limb_into_2_limbs = [](Fr& wide_limb) {
        return std::array<Fr, GoblinTranslatorCircuitBuilder::NUM_Z_LIMBS>{
            Fr(uint256_t(wide_limb).slice(0, NUM_LIMB_BITS)),
            Fr(uint256_t(wide_limb).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS))
        };
    };
    /**
     * @brief A method to split a full 68-bit limb into 5 14-bit limb and 1 shifted limb for a more secure constraint
     *
     */
    auto split_standard_limb_into_micro_limbs = [](Fr& limb) {
        static_assert(MICRO_LIMB_BITS == 14);
        return std::array<Fr, NUM_MICRO_LIMBS>{
            uint256_t(limb).slice(0, MICRO_LIMB_BITS),
            uint256_t(limb).slice(MICRO_LIMB_BITS, 2 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(2 * MICRO_LIMB_BITS, 3 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(3 * MICRO_LIMB_BITS, 4 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(4 * MICRO_LIMB_BITS, 5 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(4 * MICRO_LIMB_BITS, 5 * MICRO_LIMB_BITS)
                << (MICRO_LIMB_BITS - (NUM_LIMB_BITS % MICRO_LIMB_BITS)),
        };
    };

    /**
     * @brief A method to split the top 50-bit limb into 4 14-bit limbs and 1 shifted limb for a more secure constraint
     * (plus there is 1 extra space for other constraints)
     *
     */
    auto split_top_limb_into_micro_limbs = [](Fr& limb, size_t last_limb_bits) {
        static_assert(MICRO_LIMB_BITS == 14);
        return std::array<Fr, NUM_MICRO_LIMBS>{ uint256_t(limb).slice(0, MICRO_LIMB_BITS),
                                                uint256_t(limb).slice(MICRO_LIMB_BITS, 2 * MICRO_LIMB_BITS),
                                                uint256_t(limb).slice(2 * MICRO_LIMB_BITS, 3 * MICRO_LIMB_BITS),
                                                uint256_t(limb).slice(3 * MICRO_LIMB_BITS, 4 * MICRO_LIMB_BITS),
                                                uint256_t(limb).slice(3 * MICRO_LIMB_BITS, 4 * MICRO_LIMB_BITS)
                                                    << (MICRO_LIMB_BITS - (last_limb_bits % MICRO_LIMB_BITS)),
                                                0 };
    };

    /**
     * @brief A method for splitting the top 60-bit z limb into microlimbs (differs from the 68-bit limb by the shift in
     * the last limb)
     *
     */
    auto split_top_z_limb_into_micro_limbs = [](Fr& limb, size_t last_limb_bits) {
        static_assert(MICRO_LIMB_BITS == 14);
        return std::array<Fr, NUM_MICRO_LIMBS>{ uint256_t(limb).slice(0, MICRO_LIMB_BITS),
                                                uint256_t(limb).slice(MICRO_LIMB_BITS, 2 * MICRO_LIMB_BITS),
                                                uint256_t(limb).slice(2 * MICRO_LIMB_BITS, 3 * MICRO_LIMB_BITS),
                                                uint256_t(limb).slice(3 * MICRO_LIMB_BITS, 4 * MICRO_LIMB_BITS),
                                                uint256_t(limb).slice(4 * MICRO_LIMB_BITS, 5 * MICRO_LIMB_BITS),
                                                uint256_t(limb).slice(4 * MICRO_LIMB_BITS, 5 * MICRO_LIMB_BITS)
                                                    << (MICRO_LIMB_BITS - (last_limb_bits % MICRO_LIMB_BITS)) };
    };

    /**
     * @brief Split a 72-bit relation limb into 6 14-bit limbs (we can allow the slack here, since we only need to
     * ensure non-overflow of the modulus)
     *
     */
    auto split_relation_limb_into_micro_limbs = [](Fr& limb) {
        static_assert(MICRO_LIMB_BITS == 14);
        return std::array<Fr, 6>{
            uint256_t(limb).slice(0, MICRO_LIMB_BITS),
            uint256_t(limb).slice(MICRO_LIMB_BITS, 2 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(2 * MICRO_LIMB_BITS, 3 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(3 * MICRO_LIMB_BITS, 4 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(4 * MICRO_LIMB_BITS, 5 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(5 * MICRO_LIMB_BITS, 6 * MICRO_LIMB_BITS),
        };
    };
    //  x and powers of v are given to us in challenge form, so the verifier has to deal with this :)
    Fq v_squared;
    Fq v_cubed;
    Fq v_quarted;
    v_squared = batching_challenge_v * batching_challenge_v;
    v_cubed = v_squared * batching_challenge_v;
    v_quarted = v_cubed * batching_challenge_v;

    // Convert the accumulator, powers of v and x into "bigfield" form
    auto previous_accumulator_limbs = base_element_to_limbs(previous_accumulator);
    auto v_witnesses = base_element_to_limbs(batching_challenge_v);
    auto v_squared_witnesses = base_element_to_limbs(v_squared);
    auto v_cubed_witnesses = base_element_to_limbs(v_cubed);
    auto v_quarted_witnesses = base_element_to_limbs(v_quarted);
    auto x_witnesses = base_element_to_limbs(evaluation_input_x);

    // To calculate the quotient, we need to evaluate the expression in integers. So we need uint512_t versions of all
    // elements involved
    auto uint_previous_accumulator = uint512_t(previous_accumulator);
    auto uint_x = uint512_t(evaluation_input_x);
    auto uint_op = uint512_t(op_code);
    auto uint_p_x = uint512_t(uint256_t(p_x_lo) + (uint256_t(p_x_hi) << (NUM_LIMB_BITS << 1)));
    auto uint_p_y = uint512_t(uint256_t(p_y_lo) + (uint256_t(p_y_hi) << (NUM_LIMB_BITS << 1)));
    auto uint_z1 = uint512_t(z1);
    auto uint_z2 = uint512_t(z2);
    auto uint_v = uint512_t(batching_challenge_v);
    auto uint_v_squared = uint512_t(v_squared);
    auto uint_v_cubed = uint512_t(v_cubed);
    auto uint_v_quarted = uint512_t(v_quarted);

    // Construct Fq for op, P.x, P.y, z_1, z_2 for use in witness computation
    Fq base_op = Fq(uint256_t(op_code));
    Fq base_p_x = Fq(uint256_t(p_x_lo) + (uint256_t(p_x_hi) << (NUM_LIMB_BITS << 1)));
    Fq base_p_y = Fq(uint256_t(p_y_lo) + (uint256_t(p_y_hi) << (NUM_LIMB_BITS << 1)));
    Fq base_z_1 = Fq(uint256_t(z1));
    Fq base_z_2 = Fq(uint256_t(z2));

    // Construct bigfield representations of P.x and P.y
    auto [p_x_0, p_x_1] = split_wide_limb_into_2_limbs(p_x_lo);
    auto [p_x_2, p_x_3] = split_wide_limb_into_2_limbs(p_x_hi);
    std::array<Fr, NUM_BINARY_LIMBS> p_x_limbs = { p_x_0, p_x_1, p_x_2, p_x_3 };
    auto [p_y_0, p_y_1] = split_wide_limb_into_2_limbs(p_y_lo);
    auto [p_y_2, p_y_3] = split_wide_limb_into_2_limbs(p_y_hi);
    std::array<Fr, NUM_BINARY_LIMBS> p_y_limbs = { p_y_0, p_y_1, p_y_2, p_y_3 };

    // Construct bigfield representations of z1 and z2 only using 2 limbs each
    auto z_1_limbs = split_wide_limb_into_2_limbs(z1);
    auto z_2_limbs = split_wide_limb_into_2_limbs(z2);

    // The formula is `accumulator = accumulator⋅x + (op + v⋅p.x + v²⋅p.y + v³⋅z₁ + v⁴z₂)`. We need to compute the
    // remainder (new accumulator value)

    Fq remainder = previous_accumulator * evaluation_input_x + base_z_2 * v_quarted + base_z_1 * v_cubed +
                   base_p_y * v_squared + base_p_x * batching_challenge_v + base_op;

    // We also need to compute the quotient
    uint512_t quotient_by_modulus = uint_previous_accumulator * uint_x + uint_z2 * uint_v_quarted +
                                    uint_z1 * uint_v_cubed + uint_p_y * uint_v_squared + uint_p_x * uint_v + uint_op -
                                    uint512_t(remainder);

    uint512_t quotient = quotient_by_modulus / uint512_t(Fq::modulus);

    ASSERT(quotient_by_modulus == (quotient * uint512_t(Fq::modulus)));

    // Compute quotient and remainder bigfield representation
    auto remainder_limbs = base_element_to_limbs(remainder);
    std::array<Fr, NUM_BINARY_LIMBS> quotient_limbs = uint512_t_to_limbs(quotient);

    // We will divide by shift_2 instantly in the relation itself, but first we need to compute the low part (0*0) and
    // the high part (0*1, 1*0) multiplied by a single limb shift
    Fr low_wide_relation_limb_part_1 = previous_accumulator_limbs[0] * x_witnesses[0] + op_code +
                                       v_witnesses[0] * p_x_limbs[0] + v_squared_witnesses[0] * p_y_limbs[0] +
                                       v_cubed_witnesses[0] * z_1_limbs[0] + v_quarted_witnesses[0] * z_2_limbs[0] +
                                       quotient_limbs[0] * neg_modulus_limbs[0] -
                                       remainder_limbs[0]; // This covers the lowest limb

    Fr low_wide_relation_limb =
        low_wide_relation_limb_part_1 +
        (previous_accumulator_limbs[1] * x_witnesses[0] + previous_accumulator_limbs[0] * x_witnesses[1] +
         v_witnesses[1] * p_x_limbs[0] + p_x_limbs[1] * v_witnesses[0] + v_squared_witnesses[1] * p_y_limbs[0] +
         v_squared_witnesses[0] * p_y_limbs[1] + v_cubed_witnesses[1] * z_1_limbs[0] +
         z_1_limbs[1] * v_cubed_witnesses[0] + v_quarted_witnesses[1] * z_2_limbs[0] +
         v_quarted_witnesses[0] * z_2_limbs[1] + quotient_limbs[0] * neg_modulus_limbs[1] +
         quotient_limbs[1] * neg_modulus_limbs[0] - remainder_limbs[1]) *
            shift_1;

    // Low bits have to be zero
    ASSERT(uint256_t(low_wide_relation_limb).slice(0, 2 * NUM_LIMB_BITS) == 0);

    Fr low_wide_relation_limb_divided = low_wide_relation_limb * shift_2_inverse;

    // The high relation limb is the accumulation of the low limb divided by 2¹³⁶ and the combination of limbs with
    // indices (0*2,1*1,2*0) with limbs with indices (0*3,1*2,2*1,3*0) multiplied by 2⁶⁸

    Fr high_wide_relation_limb =
        low_wide_relation_limb_divided + previous_accumulator_limbs[2] * x_witnesses[0] +
        previous_accumulator_limbs[1] * x_witnesses[1] + previous_accumulator_limbs[0] * x_witnesses[2] +
        v_witnesses[2] * p_x_limbs[0] + v_witnesses[1] * p_x_limbs[1] + v_witnesses[0] * p_x_limbs[2] +
        v_squared_witnesses[2] * p_y_limbs[0] + v_squared_witnesses[1] * p_y_limbs[1] +
        v_squared_witnesses[0] * p_y_limbs[2] + v_cubed_witnesses[2] * z_1_limbs[0] +
        v_cubed_witnesses[1] * z_1_limbs[1] + v_quarted_witnesses[2] * z_2_limbs[0] +
        v_quarted_witnesses[1] * z_2_limbs[1] + quotient_limbs[2] * neg_modulus_limbs[0] +
        quotient_limbs[1] * neg_modulus_limbs[1] + quotient_limbs[0] * neg_modulus_limbs[2] - remainder_limbs[2] +
        (previous_accumulator_limbs[3] * x_witnesses[0] + previous_accumulator_limbs[2] * x_witnesses[1] +
         previous_accumulator_limbs[1] * x_witnesses[2] + previous_accumulator_limbs[0] * x_witnesses[3] +
         v_witnesses[3] * p_x_limbs[0] + v_witnesses[2] * p_x_limbs[1] + v_witnesses[1] * p_x_limbs[2] +
         v_witnesses[0] * p_x_limbs[3] + v_squared_witnesses[3] * p_y_limbs[0] + v_squared_witnesses[2] * p_y_limbs[1] +
         v_squared_witnesses[1] * p_y_limbs[2] + v_squared_witnesses[0] * p_y_limbs[3] +
         v_cubed_witnesses[3] * z_1_limbs[0] + v_cubed_witnesses[2] * z_1_limbs[1] +
         v_quarted_witnesses[3] * z_2_limbs[0] + v_quarted_witnesses[2] * z_2_limbs[1] +
         quotient_limbs[3] * neg_modulus_limbs[0] + quotient_limbs[2] * neg_modulus_limbs[1] +
         quotient_limbs[1] * neg_modulus_limbs[2] + quotient_limbs[0] * neg_modulus_limbs[3] - remainder_limbs[3]) *
            shift_1;

    // Check that the results lower 136 bits are zero
    ASSERT(uint256_t(high_wide_relation_limb).slice(0, 2 * NUM_LIMB_BITS) == 0);

    // Get divided version
    auto high_wide_relation_limb_divided = high_wide_relation_limb * shift_2_inverse;

    const auto last_limb_index = GoblinTranslatorCircuitBuilder::NUM_BINARY_LIMBS - 1;

    const auto NUM_Z_LIMBS = GoblinTranslatorCircuitBuilder::NUM_Z_LIMBS;
    std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> P_x_microlimbs;
    std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> P_y_microlimbs;
    std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_Z_LIMBS> z_1_microlimbs;
    std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_Z_LIMBS> z_2_microlimbs;
    std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> current_accumulator_microlimbs;
    std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> quotient_microlimbs;
    // Split P_x into microlimbs for range constraining
    for (size_t i = 0; i < last_limb_index; i++) {
        P_x_microlimbs[i] = split_standard_limb_into_micro_limbs(p_x_limbs[i]);
    }
    P_x_microlimbs[last_limb_index] =
        split_top_limb_into_micro_limbs(p_x_limbs[last_limb_index], TOP_STANDARD_MICROLIMB_BITS);

    // Split P_y into microlimbs for range constraining
    for (size_t i = 0; i < last_limb_index; i++) {
        P_y_microlimbs[i] = split_standard_limb_into_micro_limbs(p_y_limbs[i]);
    }
    P_y_microlimbs[last_limb_index] =
        split_top_limb_into_micro_limbs(p_y_limbs[last_limb_index], TOP_STANDARD_MICROLIMB_BITS);

    // Split z scalars into microlimbs for range constraining
    for (size_t i = 0; i < NUM_Z_LIMBS - 1; i++) {
        z_1_microlimbs[i] = split_standard_limb_into_micro_limbs(z_1_limbs[i]);
        z_2_microlimbs[i] = split_standard_limb_into_micro_limbs(z_2_limbs[i]);
    }
    z_1_microlimbs[GoblinTranslatorCircuitBuilder::NUM_Z_LIMBS - 1] = split_top_z_limb_into_micro_limbs(
        z_1_limbs[GoblinTranslatorCircuitBuilder::NUM_Z_LIMBS - 1], TOP_Z_MICROLIMB_BITS);
    z_2_microlimbs[GoblinTranslatorCircuitBuilder::NUM_Z_LIMBS - 1] = split_top_z_limb_into_micro_limbs(
        z_2_limbs[GoblinTranslatorCircuitBuilder::NUM_Z_LIMBS - 1], TOP_Z_MICROLIMB_BITS);

    // Split current accumulator into microlimbs for range constraining
    for (size_t i = 0; i < last_limb_index; i++) {
        current_accumulator_microlimbs[i] = split_standard_limb_into_micro_limbs(remainder_limbs[i]);
    }
    current_accumulator_microlimbs[last_limb_index] =
        split_top_limb_into_micro_limbs(remainder_limbs[last_limb_index], TOP_STANDARD_MICROLIMB_BITS);

    // Split quotient into microlimbs for range constraining
    for (size_t i = 0; i < last_limb_index; i++) {
        quotient_microlimbs[i] = split_standard_limb_into_micro_limbs(quotient_limbs[i]);
    }
    quotient_microlimbs[last_limb_index] =
        split_top_limb_into_micro_limbs(quotient_limbs[last_limb_index], TOP_QUOTIENT_MICROLIMB_BITS);

    // Start filling the witness container
    GoblinTranslatorCircuitBuilder::AccumulationInput input{
        .op_code = op_code,
        .P_x_lo = p_x_lo,
        .P_x_hi = p_x_hi,
        .P_x_limbs = p_x_limbs,
        .P_x_microlimbs = P_x_microlimbs,
        .P_y_lo = p_y_lo,
        .P_y_hi = p_y_hi,
        .P_y_limbs = p_y_limbs,
        .P_y_microlimbs = P_y_microlimbs,
        .z_1 = z1,
        .z_1_limbs = z_1_limbs,
        .z_1_microlimbs = z_1_microlimbs,
        .z_2 = z2,
        .z_2_limbs = z_2_limbs,
        .z_2_microlimbs = z_2_microlimbs,
        .previous_accumulator = previous_accumulator_limbs,
        .current_accumulator = remainder_limbs,
        .current_accumulator_microlimbs = current_accumulator_microlimbs,
        .quotient_binary_limbs = quotient_limbs,
        .quotient_microlimbs = quotient_microlimbs,
        .relation_wide_limbs = { low_wide_relation_limb_divided, high_wide_relation_limb_divided },
        .relation_wide_microlimbs = { split_relation_limb_into_micro_limbs(low_wide_relation_limb_divided),
                                      split_relation_limb_into_micro_limbs(high_wide_relation_limb_divided) },
        .x_limbs = x_witnesses,
        .v_limbs = v_witnesses,
        .v_squared_limbs = v_squared_witnesses,
        .v_cubed_limbs = v_cubed_witnesses,
        .v_quarted_limbs = v_quarted_witnesses,

    };

    return input;
}
/**
 * @brief Create a single accumulation gate
 *
 * @param acc_step
 */
void GoblinTranslatorCircuitBuilder::create_accumulation_gate(const AccumulationInput acc_step)
{
    // The first wires OpQueue/Transcript wires
    // Opcode should be {0,1,2,3,4,8}
    ASSERT(acc_step.op_code == 0 || acc_step.op_code == 1 || acc_step.op_code == 2 || acc_step.op_code == 3 ||
           acc_step.op_code == 4 || acc_step.op_code == 8);

    auto& op_wire = std::get<WireIds::OP>(wires);
    op_wire.push_back(add_variable(acc_step.op_code));
    // Every second op value in the transcript (indices 3, 5, etc) are not defined so let's just put zero there
    op_wire.push_back(zero_idx);

    /**
     * @brief Insert two values into the same wire sequentially
     *
     */
    auto insert_pair_into_wire = [this](WireIds wire_index, Fr first, Fr second) {
        auto& current_wire = wires[wire_index];
        current_wire.push_back(add_variable(first));
        current_wire.push_back(add_variable(second));
    };

    // Check and insert P_x_lo and P_y_hi into wire 1
    ASSERT(uint256_t(acc_step.P_x_lo) <= MAX_LOW_WIDE_LIMB_SIZE);
    ASSERT(uint256_t(acc_step.P_y_hi) <= MAX_HIGH_WIDE_LIMB_SIZE);
    insert_pair_into_wire(WireIds::X_LOW_Y_HI, acc_step.P_x_lo, acc_step.P_y_hi);

    // Check and insert P_x_hi and z_1 into wire 2
    ASSERT(uint256_t(acc_step.P_x_hi) <= MAX_HIGH_WIDE_LIMB_SIZE);
    ASSERT(uint256_t(acc_step.z_1) <= MAX_LOW_WIDE_LIMB_SIZE);
    insert_pair_into_wire(WireIds::X_HIGH_Z_1, acc_step.P_x_hi, acc_step.z_1);

    // Check and insert P_y_lo and z_2 into wire 3
    ASSERT(uint256_t(acc_step.P_y_lo) <= MAX_LOW_WIDE_LIMB_SIZE);
    ASSERT(uint256_t(acc_step.z_2) <= MAX_LOW_WIDE_LIMB_SIZE);
    insert_pair_into_wire(WireIds::Y_LOW_Z_2, acc_step.P_y_lo, acc_step.z_2);

    // Check decomposition of values from the Queue into limbs used in bigfield evaluations
    ASSERT(acc_step.P_x_lo == (acc_step.P_x_limbs[0] + acc_step.P_x_limbs[1] * SHIFT_1));
    ASSERT(acc_step.P_x_hi == (acc_step.P_x_limbs[2] + acc_step.P_x_limbs[3] * SHIFT_1));
    ASSERT(acc_step.P_y_lo == (acc_step.P_y_limbs[0] + acc_step.P_y_limbs[1] * SHIFT_1));
    ASSERT(acc_step.P_y_hi == (acc_step.P_y_limbs[2] + acc_step.P_y_limbs[3] * SHIFT_1));
    ASSERT(acc_step.z_1 == (acc_step.z_1_limbs[0] + acc_step.z_1_limbs[1] * SHIFT_1));
    ASSERT(acc_step.z_2 == (acc_step.z_2_limbs[0] + acc_step.z_2_limbs[1] * SHIFT_1));

    /**
     * @brief Check correctness of limbs values
     *
     */
    auto check_binary_limbs_maximum_values = []<size_t total_limbs>(const std::array<Fr, total_limbs>& limbs,
                                                                    const uint256_t& MAX_LAST_LIMB =
                                                                        (uint256_t(1) << NUM_LAST_LIMB_BITS)) {
        for (size_t i = 0; i < total_limbs - 1; i++) {
            ASSERT(uint256_t(limbs[i]) < SHIFT_1);
        }
        ASSERT(uint256_t(limbs[total_limbs - 1]) < MAX_LAST_LIMB);
    };
    /**
     * @brief Check correctness of values for range constraint limbs
     *
     */
    auto check_micro_limbs_maximum_values =
        []<size_t binary_limb_count, size_t micro_limb_count>(
            const std::array<std::array<Fr, micro_limb_count>, binary_limb_count>& limbs) {
            for (size_t i = 0; i < binary_limb_count; i++) {
                for (size_t j = 0; j < micro_limb_count; j++) {
                    ASSERT(uint256_t(limbs[i][j]) < MICRO_SHIFT);
                }
            }
        };

    const auto MAX_Z_LAST_LIMB = uint256_t(1) << (NUM_Z_BITS - NUM_LIMB_BITS);
    const auto MAX_QUOTIENT_LAST_LIMB = uint256_t(1) << (NUM_LAST_QUOTIENT_LIMB_BITS);
    // Check limb values are in 68-bit range
    check_binary_limbs_maximum_values(acc_step.P_x_limbs);
    check_binary_limbs_maximum_values(acc_step.P_y_limbs);
    check_binary_limbs_maximum_values(acc_step.z_1_limbs, /*MAX_LAST_LIMB=*/MAX_Z_LAST_LIMB);
    check_binary_limbs_maximum_values(acc_step.z_2_limbs, /*MAX_LAST_LIMB=*/MAX_Z_LAST_LIMB);
    check_binary_limbs_maximum_values(acc_step.previous_accumulator);
    check_binary_limbs_maximum_values(acc_step.current_accumulator);
    check_binary_limbs_maximum_values(acc_step.quotient_binary_limbs, /*MAX_LAST_LIMB=*/MAX_QUOTIENT_LAST_LIMB);

    // Insert limbs used in bigfield evaluations
    insert_pair_into_wire(P_X_LOW_LIMBS, acc_step.P_x_limbs[0], acc_step.P_x_limbs[1]);
    insert_pair_into_wire(P_X_HIGH_LIMBS, acc_step.P_x_limbs[2], acc_step.P_x_limbs[3]);
    insert_pair_into_wire(P_Y_LOW_LIMBS, acc_step.P_y_limbs[0], acc_step.P_y_limbs[1]);
    insert_pair_into_wire(P_Y_HIGH_LIMBS, acc_step.P_y_limbs[2], acc_step.P_y_limbs[3]);
    insert_pair_into_wire(Z_LOW_LIMBS, acc_step.z_1_limbs[0], acc_step.z_2_limbs[0]);
    insert_pair_into_wire(Z_HIGH_LIMBS, acc_step.z_1_limbs[1], acc_step.z_2_limbs[1]);
    insert_pair_into_wire(
        QUOTIENT_LOW_BINARY_LIMBS, acc_step.quotient_binary_limbs[0], acc_step.quotient_binary_limbs[1]);
    insert_pair_into_wire(
        QUOTIENT_HIGH_BINARY_LIMBS, acc_step.quotient_binary_limbs[2], acc_step.quotient_binary_limbs[3]);
    insert_pair_into_wire(RELATION_WIDE_LIMBS, acc_step.relation_wide_limbs[0], acc_step.relation_wide_limbs[1]);

    // Check limbs used in range constraints are in range
    check_micro_limbs_maximum_values(acc_step.P_x_microlimbs);
    check_micro_limbs_maximum_values(acc_step.P_y_microlimbs);
    check_micro_limbs_maximum_values(acc_step.z_1_microlimbs);
    check_micro_limbs_maximum_values(acc_step.z_2_microlimbs);
    check_micro_limbs_maximum_values(acc_step.current_accumulator_microlimbs);

    // Check that relation limbs are in range
    ASSERT(uint256_t(acc_step.relation_wide_limbs[0]) < MAX_RELATION_WIDE_LIMB_SIZE);
    ASSERT(uint256_t(acc_step.relation_wide_limbs[1]) < MAX_RELATION_WIDE_LIMB_SIZE);

    /**
     * @brief Put several values in sequential wires
     *
     */
    auto lay_limbs_in_row =
        [this]<size_t array_size>(std::array<Fr, array_size> input, WireIds starting_wire, size_t number_of_elements) {
            ASSERT(number_of_elements <= array_size);
            for (size_t i = 0; i < number_of_elements; i++) {
                wires[starting_wire + i].push_back(add_variable(input[i]));
            }
        };

    // We are using some leftover crevices for relation_wide_microlimbs
    auto low_relation_microlimbs = acc_step.relation_wide_microlimbs[0];
    auto high_relation_microlimbs = acc_step.relation_wide_microlimbs[1];

    // We have 4 wires specifically for the relation microlimbs
    insert_pair_into_wire(
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_0, low_relation_microlimbs[0], high_relation_microlimbs[0]);
    insert_pair_into_wire(
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_1, low_relation_microlimbs[1], high_relation_microlimbs[1]);
    insert_pair_into_wire(
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_2, low_relation_microlimbs[2], high_relation_microlimbs[2]);
    insert_pair_into_wire(
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_3, low_relation_microlimbs[3], high_relation_microlimbs[3]);

    // Next ones go into top P_x and P_y, current accumulator and quotient unused microlimbs

    // Insert the second highest low relation microlimb into the space left in P_x range constraints highest wire
    auto top_p_x_microlimbs = acc_step.P_x_microlimbs[NUM_BINARY_LIMBS - 1];
    top_p_x_microlimbs[NUM_MICRO_LIMBS - 1] = low_relation_microlimbs[NUM_MICRO_LIMBS - 2];

    // Insert the second highest high relation microlimb into the space left in P_y range constraints highest wire
    auto top_p_y_microlimbs = acc_step.P_y_microlimbs[NUM_BINARY_LIMBS - 1];
    top_p_y_microlimbs[NUM_MICRO_LIMBS - 1] = high_relation_microlimbs[NUM_MICRO_LIMBS - 2];

    // The highest low relation microlimb goes into the crevice left in current accumulator microlimbs
    auto top_current_accumulator_microlimbs = acc_step.current_accumulator_microlimbs[NUM_BINARY_LIMBS - 1];
    top_current_accumulator_microlimbs[NUM_MICRO_LIMBS - 1] = low_relation_microlimbs[NUM_MICRO_LIMBS - 1];

    // The highest high relation microlimb goes into the quotient crevice
    auto top_quotient_microlimbs = acc_step.quotient_microlimbs[NUM_BINARY_LIMBS - 1];
    top_quotient_microlimbs[NUM_MICRO_LIMBS - 1] = high_relation_microlimbs[NUM_MICRO_LIMBS - 1];

    // Now put all microlimbs into appropriate wires
    lay_limbs_in_row(acc_step.P_x_microlimbs[0], P_X_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.P_x_microlimbs[1], P_X_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.P_x_microlimbs[2], P_X_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(top_p_x_microlimbs, P_X_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.P_y_microlimbs[0], P_Y_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.P_y_microlimbs[1], P_Y_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.P_y_microlimbs[2], P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(top_p_y_microlimbs, P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.z_1_microlimbs[0], Z_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.z_2_microlimbs[0], Z_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.z_1_microlimbs[1], Z_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.z_2_microlimbs[1], Z_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.current_accumulator, ACCUMULATORS_BINARY_LIMBS_0, NUM_BINARY_LIMBS);
    lay_limbs_in_row(acc_step.previous_accumulator, ACCUMULATORS_BINARY_LIMBS_0, NUM_BINARY_LIMBS);
    lay_limbs_in_row(
        acc_step.current_accumulator_microlimbs[0], ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(
        acc_step.current_accumulator_microlimbs[1], ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(
        acc_step.current_accumulator_microlimbs[2], ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(top_current_accumulator_microlimbs, ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.quotient_microlimbs[0], QUOTIENT_LOW_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.quotient_microlimbs[1], QUOTIENT_LOW_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(acc_step.quotient_microlimbs[2], QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS);
    lay_limbs_in_row(top_quotient_microlimbs, QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS);

    num_gates += 2;

    // Check that all the wires are filled equally
    bb::constexpr_for<0, TOTAL_COUNT, 1>([&]<size_t i>() { ASSERT(std::get<i>(wires).size() == num_gates); });
}

/**
 * @brief Given an ECCVM operation, previous accumulator and necessary challenges, compute witnesses for one
 * accumulation
 *
 * @tparam Fq
 * @return GoblinTranslatorCircuitBuilder::AccumulationInput
 */
template <typename Fq>
GoblinTranslatorCircuitBuilder::AccumulationInput compute_witness_values_for_one_ecc_op(const ECCVMOperation& ecc_op,
                                                                                        Fq previous_accumulator,
                                                                                        Fq batching_challenge_v,
                                                                                        Fq evaluation_input_x)
{
    using Fr = bb::fr;

    // Get the Opcode value
    Fr op(ecc_op.get_opcode_value());
    Fr p_x_lo(0);
    Fr p_x_hi(0);
    Fr p_y_lo(0);
    Fr p_y_hi(0);

    // Split P.x and P.y into their representations in bn254 transcript
    p_x_lo = Fr(uint256_t(ecc_op.base_point.x).slice(0, 2 * GoblinTranslatorCircuitBuilder::NUM_LIMB_BITS));
    p_x_hi = Fr(uint256_t(ecc_op.base_point.x)
                    .slice(2 * GoblinTranslatorCircuitBuilder::NUM_LIMB_BITS,
                           4 * GoblinTranslatorCircuitBuilder::NUM_LIMB_BITS));
    p_y_lo = Fr(uint256_t(ecc_op.base_point.y).slice(0, 2 * GoblinTranslatorCircuitBuilder::NUM_LIMB_BITS));
    p_y_hi = Fr(uint256_t(ecc_op.base_point.y)
                    .slice(2 * GoblinTranslatorCircuitBuilder::NUM_LIMB_BITS,
                           4 * GoblinTranslatorCircuitBuilder::NUM_LIMB_BITS));

    // Generate the full witness values
    return generate_witness_values(op,
                                   p_x_lo,
                                   p_x_hi,
                                   p_y_lo,
                                   p_y_hi,
                                   Fr(ecc_op.z1),
                                   Fr(ecc_op.z2),
                                   previous_accumulator,
                                   batching_challenge_v,
                                   evaluation_input_x);
}
void GoblinTranslatorCircuitBuilder::feed_ecc_op_queue_into_circuit(std::shared_ptr<ECCOpQueue> ecc_op_queue)
{
    using Fq = bb::fq;
    std::vector<Fq> accumulator_trace;
    Fq current_accumulator(0);
    if (ecc_op_queue->raw_ops.empty()) {
        return;
    }
    // Rename for ease of use
    auto x = evaluation_input_x;
    auto v = batching_challenge_v;

    // We need to precompute the accumulators at each step, because in the actual circuit we compute the values starting
    // from the later indices. We need to know the previous accumulator to create the gate
    for (size_t i = 0; i < ecc_op_queue->raw_ops.size(); i++) {
        auto& ecc_op = ecc_op_queue->raw_ops[ecc_op_queue->raw_ops.size() - 1 - i];
        current_accumulator *= x;
        current_accumulator +=
            (Fq(ecc_op.get_opcode_value()) +
             v * (ecc_op.base_point.x + v * (ecc_op.base_point.y + v * (ecc_op.z1 + v * ecc_op.z2))));
        accumulator_trace.push_back(current_accumulator);
    }

    // We don't care about the last value since we'll recompute it during witness generation anyway
    accumulator_trace.pop_back();

    for (auto& raw_op : ecc_op_queue->raw_ops) {
        Fq previous_accumulator = 0;
        // Pop the last value from accumulator trace and use it as previous accumulator
        if (!accumulator_trace.empty()) {
            previous_accumulator = accumulator_trace.back();
            accumulator_trace.pop_back();
        }
        // Compute witness values
        auto one_accumulation_step = compute_witness_values_for_one_ecc_op(raw_op, previous_accumulator, v, x);

        // And put them into the wires
        create_accumulation_gate(one_accumulation_step);
    }
}
bool GoblinTranslatorCircuitBuilder::check_circuit()
{
    // Compute the limbs of evaluation_input_x and powers of batching_challenge_v (these go into the relation)
    RelationInputs relation_inputs = compute_relation_inputs_limbs(batching_challenge_v, evaluation_input_x);
    // Get the main wires (we will operate with range constraint wires mainly through indices, since this is easier)
    auto& op_wire = std::get<OP>(wires);
    auto& x_lo_y_hi_wire = std::get<X_LOW_Y_HI>(wires);
    auto& x_hi_z_1_wire = std::get<X_HIGH_Z_1>(wires);
    auto& y_lo_z_2_wire = std::get<Y_LOW_Z_2>(wires);
    auto& p_x_0_p_x_1_wire = std::get<P_X_LOW_LIMBS>(wires);
    auto& p_x_2_p_x_3_wire = std::get<P_X_HIGH_LIMBS>(wires);
    auto& p_y_0_p_y_1_wire = std::get<P_Y_LOW_LIMBS>(wires);
    auto& p_y_2_p_y_3_wire = std::get<P_Y_HIGH_LIMBS>(wires);
    auto& z_lo_wire = std::get<Z_LOW_LIMBS>(wires);
    auto& z_hi_wire = std::get<Z_HIGH_LIMBS>(wires);
    auto& accumulators_binary_limbs_0_wire = std::get<ACCUMULATORS_BINARY_LIMBS_0>(wires);
    auto& accumulators_binary_limbs_1_wire = std::get<ACCUMULATORS_BINARY_LIMBS_1>(wires);
    auto& accumulators_binary_limbs_2_wire = std::get<ACCUMULATORS_BINARY_LIMBS_2>(wires);
    auto& accumulators_binary_limbs_3_wire = std::get<ACCUMULATORS_BINARY_LIMBS_3>(wires);
    auto& quotient_low_binary_limbs = std::get<QUOTIENT_LOW_BINARY_LIMBS>(wires);
    auto& quotient_high_binary_limbs = std::get<QUOTIENT_HIGH_BINARY_LIMBS>(wires);
    auto& relation_wide_limbs_wire = std::get<RELATION_WIDE_LIMBS>(wires);
    auto reconstructed_evaluation_input_x = Fr(uint256_t(evaluation_input_x));
    auto reconstructed_batching_evaluation_v = Fr(uint256_t(batching_challenge_v));
    auto reconstructed_batching_evaluation_v2 = Fr(uint256_t(batching_challenge_v.pow(2)));
    auto reconstructed_batching_evaluation_v3 = Fr(uint256_t(batching_challenge_v.pow(3)));
    auto reconstructed_batching_evaluation_v4 = Fr(uint256_t(batching_challenge_v.pow(4)));
    /**
     * @brief Get elements at the same index from several sequential wires and put them into a vector
     *
     */
    auto get_sequential_micro_chunks = [this](size_t gate_index, WireIds starting_wire_index, size_t chunk_count) {
        std::vector<Fr> chunks;
        for (size_t i = starting_wire_index; i < starting_wire_index + chunk_count; i++) {
            chunks.push_back(get_variable(wires[i][gate_index]));
        }
        return chunks;
    };

    /**
     * @brief Reconstruct the value of one regular limb used in relation computation from micro chunks used to
     * create range constraints
     *
     * @details We might ant to skip several items at the end, since those will be shifted or used
     * for another decomposition
     *
     */
    auto accumulate_limb_from_micro_chunks = [](const std::vector<Fr>& chunks, const int skipped_at_end = 1) {
        Fr mini_accumulator(0);
        auto end = chunks.end();
        std::advance(end, -skipped_at_end);
        for (auto it = end; it != chunks.begin();) {
            --it;
            mini_accumulator = mini_accumulator * MICRO_SHIFT + *it;
        }
        return mini_accumulator;
    };
    /**
     * @brief Go through each gate
     *
     */
    for (size_t i = 1; i < num_gates - 1; i++) {
        bool gate_is_odd = i & 1;
        // The main relation is computed between odd and the next even indices. For example, 1 and 2
        if (gate_is_odd) {
            // Get the values of P.x
            Fr op_code = get_variable(op_wire[i]);
            Fr p_x_lo = get_variable(x_lo_y_hi_wire[i]);
            Fr p_x_hi = get_variable(x_hi_z_1_wire[i]);
            Fr p_x_0 = get_variable(p_x_0_p_x_1_wire[i]);
            Fr p_x_1 = get_variable(p_x_0_p_x_1_wire[i + 1]);
            Fr p_x_2 = get_variable(p_x_2_p_x_3_wire[i]);
            Fr p_x_3 = get_variable(p_x_2_p_x_3_wire[i + 1]);
            const std::vector p_x_binary_limbs = { p_x_0, p_x_1, p_x_2, p_x_3 };

            // P.y
            Fr p_y_lo = get_variable(y_lo_z_2_wire[i]);
            Fr p_y_hi = get_variable(x_lo_y_hi_wire[i + 1]);
            Fr p_y_0 = get_variable(p_y_0_p_y_1_wire[i]);
            Fr p_y_1 = get_variable(p_y_0_p_y_1_wire[i + 1]);
            Fr p_y_2 = get_variable(p_y_2_p_y_3_wire[i]);
            Fr p_y_3 = get_variable(p_y_2_p_y_3_wire[i + 1]);
            const std::vector p_y_binary_limbs = { p_y_0, p_y_1, p_y_2, p_y_3 };
            // z1, z2
            Fr z_1 = get_variable(x_hi_z_1_wire[i + 1]);
            Fr z_2 = get_variable(y_lo_z_2_wire[i + 1]);

            Fr z_1_lo = get_variable(z_lo_wire[i]);
            Fr z_2_lo = get_variable(z_lo_wire[i + 1]);
            Fr z_1_hi = get_variable(z_hi_wire[i]);
            Fr z_2_hi = get_variable(z_hi_wire[i + 1]);

            const std::vector z_1_binary_limbs = { z_1_lo, z_1_hi };
            const std::vector z_2_binary_limbs = { z_2_lo, z_2_hi };
            // Relation limbs
            Fr low_wide_relation_limb = get_variable(relation_wide_limbs_wire[i]);
            Fr high_wide_relation_limb = get_variable(relation_wide_limbs_wire[i + 1]);

            // Current accumulator (updated value)
            const std::vector current_accumulator_binary_limbs = {
                get_variable(accumulators_binary_limbs_0_wire[i]),
                get_variable(accumulators_binary_limbs_1_wire[i]),
                get_variable(accumulators_binary_limbs_2_wire[i]),
                get_variable(accumulators_binary_limbs_3_wire[i]),
            };

            // Previous accumulator
            const std::vector previous_accumulator_binary_limbs = {
                get_variable(accumulators_binary_limbs_0_wire[i + 1]),
                get_variable(accumulators_binary_limbs_1_wire[i + 1]),
                get_variable(accumulators_binary_limbs_2_wire[i + 1]),
                get_variable(accumulators_binary_limbs_3_wire[i + 1]),
            };

            // Quotient
            const std::vector quotient_binary_limbs = {
                get_variable(quotient_low_binary_limbs[i]),
                get_variable(quotient_low_binary_limbs[i + 1]),
                get_variable(quotient_high_binary_limbs[i]),
                get_variable(quotient_high_binary_limbs[i + 1]),
            };

            // Get micro chunks for checking decomposition and range
            auto p_x_micro_chunks = {
                get_sequential_micro_chunks(i, P_X_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i + 1, P_X_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i, P_X_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i + 1, P_X_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS)
            };
            auto p_y_micro_chunks = {
                get_sequential_micro_chunks(i, P_Y_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i + 1, P_Y_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i, P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i + 1, P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS)
            };
            auto z_1_micro_chunks = {
                get_sequential_micro_chunks(i, Z_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i, Z_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
            };

            auto z_2_micro_chunks = {

                get_sequential_micro_chunks(i + 1, Z_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i + 1, Z_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS)
            };

            auto current_accumulator_micro_chunks = {
                get_sequential_micro_chunks(i, ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i + 1, ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i, ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i + 1, ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
            };
            auto quotient_micro_chunks = {
                get_sequential_micro_chunks(i, QUOTIENT_LOW_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i + 1, QUOTIENT_LOW_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i, QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS),
                get_sequential_micro_chunks(i + 1, QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS),
            };

            // Lambda for checking the correctness of decomposition of values in the Queue into limbs for checking
            // the relation
            auto check_wide_limb_into_binary_limb_relation = [](const std::vector<Fr>& wide_limbs,
                                                                const std::vector<Fr>& binary_limbs) {
                ASSERT(wide_limbs.size() * 2 == binary_limbs.size());
                for (size_t i = 0; i < wide_limbs.size(); i++) {
                    if ((binary_limbs[i * 2] + Fr(SHIFT_1) * binary_limbs[i * 2 + 1]) != wide_limbs[i]) {
                        return false;
                    }
                }
                return true;
            };
            // Check that everything has been decomposed correctly
            // P.xₗₒ = P.xₗₒ_0 + SHIFT_1 * P.xₗₒ_1
            // P.xₕᵢ  = P.xₕᵢ_0 + SHIFT_1 * P.xₕᵢ_1
            // z_1 = z_1ₗₒ + SHIFT_1 * z_1ₕᵢ
            // z_2 = z_2ₗₒ + SHIFT_2 * z_1ₕᵢ
            if (!(check_wide_limb_into_binary_limb_relation({ p_x_lo, p_x_hi }, p_x_binary_limbs) &&
                  check_wide_limb_into_binary_limb_relation({ p_y_lo, p_y_hi }, p_y_binary_limbs) &&
                  check_wide_limb_into_binary_limb_relation({ z_1 }, z_1_binary_limbs) &&
                  check_wide_limb_into_binary_limb_relation({ z_2 }, z_2_binary_limbs))) {
                return false;
            }

            enum LimbSeriesType { STANDARD_COORDINATE, Z_SCALAR, QUOTIENT };

            // Check that limbs have been decomposed into microlimbs correctly
            // value = ∑ (2ˡ)ⁱ⋅ chunkᵢ, where 2ˡ is the shift
            auto check_micro_limb_decomposition_correctness = [&accumulate_limb_from_micro_chunks](
                                                                  const std::vector<Fr>& binary_limbs,
                                                                  const std::vector<std::vector<Fr>>& micro_limbs,
                                                                  const LimbSeriesType limb_series_type) {
                // Shifts for decompositions
                constexpr auto SHIFT_12_TO_14 = Fr(4);
                constexpr auto SHIFT_10_TO_14 = Fr(16);
                constexpr auto SHIFT_8_TO_14 = Fr(64);
                constexpr auto SHIFT_4_TO_14 = Fr(1024);

                ASSERT(binary_limbs.size() == micro_limbs.size());
                // First check that all the microlimbs are properly range constrained
                for (auto& micro_limb_series : micro_limbs) {
                    for (auto& micro_limb : micro_limb_series) {
                        if (uint256_t(micro_limb) > MAX_MICRO_LIMB_SIZE) {
                            return false;
                        }
                    }
                }
                // For low limbs the last microlimb is used with the shift, so we skip it when reconstructing
                // the limb
                const size_t SKIPPED_FOR_LOW_LIMBS = 1;
                for (size_t i = 0; i < binary_limbs.size() - 1; i++) {
                    if (binary_limbs[i] != accumulate_limb_from_micro_chunks(micro_limbs[i], SKIPPED_FOR_LOW_LIMBS)) {
                        return false;
                    }
                    // Check last additional constraint (68->70)
                    if (micro_limbs[i][NUM_MICRO_LIMBS - 1] != (SHIFT_12_TO_14 * micro_limbs[i][NUM_MICRO_LIMBS - 2])) {
                        return false;
                    }
                }

                const size_t SKIPPED_FOR_STANDARD = 2;
                const size_t SKIPPED_FOR_Z_SCALARS = 1;
                const size_t SKIPPED_FOR_QUOTIENT = 2;
                switch (limb_series_type) {
                case STANDARD_COORDINATE:
                    // For standard Fq value the highest limb is 50 bits, so we skip the top 2 microlimbs
                    if (binary_limbs[binary_limbs.size() - 1] !=
                        accumulate_limb_from_micro_chunks(micro_limbs[binary_limbs.size() - 1], SKIPPED_FOR_STANDARD)) {
                        return false;
                    }
                    // Check last additional constraint (50->56)
                    if (micro_limbs[binary_limbs.size() - 1][NUM_MICRO_LIMBS - SKIPPED_FOR_STANDARD] !=
                        (SHIFT_8_TO_14 *
                         micro_limbs[binary_limbs.size() - 1][NUM_MICRO_LIMBS - SKIPPED_FOR_STANDARD - 1])) {

                        return false;
                    }
                    break;
                // For z top limbs we need as many microlimbs as for the low limbs
                case Z_SCALAR:
                    if (binary_limbs[binary_limbs.size() - 1] !=
                        accumulate_limb_from_micro_chunks(micro_limbs[binary_limbs.size() - 1],
                                                          SKIPPED_FOR_Z_SCALARS)) {
                        return false;
                    }
                    // Check last additional constraint (60->70)
                    if (micro_limbs[binary_limbs.size() - 1][NUM_MICRO_LIMBS - SKIPPED_FOR_Z_SCALARS] !=
                        (SHIFT_4_TO_14 *
                         micro_limbs[binary_limbs.size() - 1][NUM_MICRO_LIMBS - SKIPPED_FOR_Z_SCALARS - 1])) {
                        return false;
                    }
                    break;
                // Quotient also doesn't need the top 2
                case QUOTIENT:
                    if (binary_limbs[binary_limbs.size() - 1] !=
                        accumulate_limb_from_micro_chunks(micro_limbs[binary_limbs.size() - 1], SKIPPED_FOR_QUOTIENT)) {
                        return false;
                    }
                    // Check last additional constraint (52->56)
                    if (micro_limbs[binary_limbs.size() - 1][NUM_MICRO_LIMBS - SKIPPED_FOR_QUOTIENT] !=
                        (SHIFT_10_TO_14 *
                         micro_limbs[binary_limbs.size() - 1][NUM_MICRO_LIMBS - SKIPPED_FOR_QUOTIENT - 1])) {
                        return false;
                    }
                    break;
                default:
                    abort();
                }

                return true;
            };
            // Check all micro limb decompositions
            if (!check_micro_limb_decomposition_correctness(p_x_binary_limbs, p_x_micro_chunks, STANDARD_COORDINATE)) {
                return false;
            }
            if (!check_micro_limb_decomposition_correctness(p_y_binary_limbs, p_y_micro_chunks, STANDARD_COORDINATE)) {
                return false;
            }
            if (!check_micro_limb_decomposition_correctness(z_1_binary_limbs, z_1_micro_chunks, Z_SCALAR)) {
                return false;
            }
            if (!check_micro_limb_decomposition_correctness(z_2_binary_limbs, z_2_micro_chunks, Z_SCALAR)) {
                return false;
            }
            if (!check_micro_limb_decomposition_correctness(
                    current_accumulator_binary_limbs, current_accumulator_micro_chunks, STANDARD_COORDINATE)) {
                return false;
            }
            if (!check_micro_limb_decomposition_correctness(quotient_binary_limbs, quotient_micro_chunks, QUOTIENT)) {
                return false;
            }

            // The logic we are trying to enforce is:
            // current_accumulator = previous_accumulator ⋅ x + op_code + P.x ⋅ v + P.y ⋅ v² + z_1 ⋅ v³ + z_2 ⋅ v⁴
            // mod Fq To ensure this we transform the relation into the form: previous_accumulator ⋅ x + op + P.x ⋅
            // v + P.y ⋅ v² + z_1 ⋅ v³ + z_2 ⋅ v⁴ - quotient ⋅ p - current_accumulator = 0 However, we don't have
            // integers. Despite that, we can approximate integers for a certain range, if we know that there will
            // not be any overflows. For now we set the range to 2²⁷² ⋅ r. We can evaluate the logic modulo 2²⁷²
            // with range constraints and r is native.
            //
            // previous_accumulator ⋅ x + op + P.x ⋅ v + P.y ⋅ v² + z_1 ⋅ v³ + z_2 ⋅ v⁴ - quotient ⋅ p -
            // current_accumulator = 0 =>
            // 1. previous_accumulator ⋅ x + op + P.x ⋅ v + P.y ⋅ v² + z_1 ⋅ v³ + z_2 ⋅ v⁴ + quotient ⋅ (-p mod
            // 2²⁷²) - current_accumulator = 0 mod 2²⁷²
            // 2. previous_accumulator ⋅ x + op + P.x ⋅ v + P.y ⋅ v² + z_1 ⋅ v³ + z_2 ⋅ v⁴ - quotient ⋅ p -
            // current_accumulator = 0 mod r
            //
            // The second relation is straightforward and easy to check. The first, not so much. We have to evaluate
            // certain bit chunks of the equation and ensure that they are zero. For example, for the lowest limb it
            // would be (inclusive ranges):
            //
            // previous_accumulator[0:67] ⋅ x[0:67] + op + P.x[0:67] ⋅ v[0:67] + P.y[0:67] ⋅ v²[0:67] + z_1[0:67] ⋅
            // v³[0:67] + z_2[0:67] ⋅ v⁴[0:67] + quotient[0:67] ⋅ (-p mod 2²⁷²)[0:67] - current_accumulator[0:67] =
            // intermediate_value; (we don't take parts of op, because it's supposed to be between 0 and 3)
            //
            // We could check that this intermediate_value is equal to  0 mod 2⁶⁸ by dividing it by 2⁶⁸ and
            // constraining it. For efficiency, we actually compute wider evaluations for 136 bits, which require us
            // to also obtain and shift products of [68:135] by [0:67] and [0:67] by [68:135] bits.
            // The result of division goes into the next evaluation (the same as a carry flag would)
            // So the lowest wide limb is : (∑everything[0:67]⋅everything[0:67] +
            // 2⁶⁸⋅(∑everything[0:67]⋅everything[68:135]))/ 2¹³⁶
            //
            // The high is:
            // (low_limb + ∑everything[0:67]⋅everything[136:203] + ∑everything[68:135]⋅everything[68:135] +
            // 2⁶⁸(∑everything[0:67]⋅everything[204:271] + ∑everything[68:135]⋅everything[136:203])) / 2¹³⁶
            //
            // We also limit computation on limbs of op, z_1 and z_2, since we know that op has only the lowest limb
            // and z_1 and z_2 have only the two lowest limbs
            Fr low_wide_limb_relation_check =

                (previous_accumulator_binary_limbs[0] * relation_inputs.x_limbs[0] + op_code +
                 relation_inputs.v_limbs[0] * p_x_0 + relation_inputs.v_squared_limbs[0] * p_y_0 +
                 relation_inputs.v_cubed_limbs[0] * z_1_lo + relation_inputs.v_quarted_limbs[0] * z_2_lo +
                 quotient_binary_limbs[0] * NEGATIVE_MODULUS_LIMBS[0] - current_accumulator_binary_limbs[0]) +
                (previous_accumulator_binary_limbs[1] * relation_inputs.x_limbs[0] +
                 relation_inputs.v_limbs[1] * p_x_0 + relation_inputs.v_squared_limbs[1] * p_y_0 +
                 relation_inputs.v_cubed_limbs[1] * z_1_lo + relation_inputs.v_quarted_limbs[1] * z_2_lo +
                 quotient_binary_limbs[1] * NEGATIVE_MODULUS_LIMBS[0] +
                 previous_accumulator_binary_limbs[0] * relation_inputs.x_limbs[1] +
                 relation_inputs.v_limbs[0] * p_x_1 + relation_inputs.v_squared_limbs[0] * p_y_1 +
                 relation_inputs.v_cubed_limbs[0] * z_1_hi + relation_inputs.v_quarted_limbs[0] * z_2_hi +
                 quotient_binary_limbs[0] * NEGATIVE_MODULUS_LIMBS[1] - current_accumulator_binary_limbs[1]) *
                    Fr(SHIFT_1);
            if (low_wide_limb_relation_check != (low_wide_relation_limb * SHIFT_2)) {
                return false;
            }
            Fr high_wide_relation_limb_check =
                low_wide_relation_limb + previous_accumulator_binary_limbs[2] * relation_inputs.x_limbs[0] +
                previous_accumulator_binary_limbs[1] * relation_inputs.x_limbs[1] +
                previous_accumulator_binary_limbs[0] * relation_inputs.x_limbs[2] + relation_inputs.v_limbs[2] * p_x_0 +
                relation_inputs.v_limbs[1] * p_x_1 + relation_inputs.v_limbs[0] * p_x_2 +
                relation_inputs.v_squared_limbs[2] * p_y_0 + relation_inputs.v_squared_limbs[1] * p_y_1 +
                relation_inputs.v_squared_limbs[0] * p_y_2 + relation_inputs.v_cubed_limbs[2] * z_1_lo +
                relation_inputs.v_cubed_limbs[1] * z_1_hi + relation_inputs.v_quarted_limbs[2] * z_2_lo +
                relation_inputs.v_quarted_limbs[1] * z_2_hi + quotient_binary_limbs[2] * NEGATIVE_MODULUS_LIMBS[0] +
                quotient_binary_limbs[1] * NEGATIVE_MODULUS_LIMBS[1] +
                quotient_binary_limbs[0] * NEGATIVE_MODULUS_LIMBS[2] - current_accumulator_binary_limbs[2] +
                (previous_accumulator_binary_limbs[3] * relation_inputs.x_limbs[0] +
                 previous_accumulator_binary_limbs[2] * relation_inputs.x_limbs[1] +
                 previous_accumulator_binary_limbs[1] * relation_inputs.x_limbs[2] +
                 previous_accumulator_binary_limbs[0] * relation_inputs.x_limbs[3] +
                 relation_inputs.v_limbs[3] * p_x_0 + relation_inputs.v_limbs[2] * p_x_1 +
                 relation_inputs.v_limbs[1] * p_x_2 + relation_inputs.v_limbs[0] * p_x_3 +
                 relation_inputs.v_squared_limbs[3] * p_y_0 + relation_inputs.v_squared_limbs[2] * p_y_1 +
                 relation_inputs.v_squared_limbs[1] * p_y_2 + relation_inputs.v_squared_limbs[0] * p_y_3 +
                 relation_inputs.v_cubed_limbs[3] * z_1_lo + relation_inputs.v_cubed_limbs[2] * z_1_hi +
                 relation_inputs.v_quarted_limbs[3] * z_2_lo + relation_inputs.v_quarted_limbs[2] * z_2_hi +
                 quotient_binary_limbs[3] * NEGATIVE_MODULUS_LIMBS[0] +
                 quotient_binary_limbs[2] * NEGATIVE_MODULUS_LIMBS[1] +
                 quotient_binary_limbs[1] * NEGATIVE_MODULUS_LIMBS[2] +
                 quotient_binary_limbs[0] * NEGATIVE_MODULUS_LIMBS[3] - current_accumulator_binary_limbs[3]) *
                    SHIFT_1;
            if (high_wide_relation_limb_check != (high_wide_relation_limb * SHIFT_2)) {
                return false;
            }
            // Apart from checking the correctness of the evaluation modulo 2²⁷² we also need to ensure that the
            // logic works in our scalar field. For this we reconstruct the scalar field values from individual
            // limbs
            auto reconstructed_p_x = (p_x_0 + p_x_1 * SHIFT_1 + p_x_2 * SHIFT_2 + p_x_3 * SHIFT_3);
            auto reconstructed_p_y = (p_y_0 + p_y_1 * SHIFT_1 + p_y_2 * SHIFT_2 + p_y_3 * SHIFT_3);
            auto reconstructed_current_accumulator =
                (current_accumulator_binary_limbs[0] + current_accumulator_binary_limbs[1] * SHIFT_1 +
                 current_accumulator_binary_limbs[2] * SHIFT_2 + current_accumulator_binary_limbs[3] * SHIFT_3);
            auto reconstructed_previous_accumulator =
                (previous_accumulator_binary_limbs[0] + previous_accumulator_binary_limbs[1] * SHIFT_1 +
                 previous_accumulator_binary_limbs[2] * SHIFT_2 + previous_accumulator_binary_limbs[3] * SHIFT_3);

            auto reconstructed_z1 = (z_1_lo + z_1_hi * SHIFT_1);
            auto reconstructed_z2 = (z_2_lo + z_2_hi * SHIFT_1);
            auto reconstructed_quotient = (quotient_binary_limbs[0] + quotient_binary_limbs[1] * SHIFT_1 +
                                           quotient_binary_limbs[2] * SHIFT_2 + quotient_binary_limbs[3] * SHIFT_3);

            // Check the relation
            if (!(reconstructed_previous_accumulator * reconstructed_evaluation_input_x + op_code +
                  reconstructed_p_x * reconstructed_batching_evaluation_v +
                  reconstructed_p_y * reconstructed_batching_evaluation_v2 +
                  reconstructed_z1 * reconstructed_batching_evaluation_v3 +
                  reconstructed_z2 * reconstructed_batching_evaluation_v4 +
                  reconstructed_quotient * NEGATIVE_MODULUS_LIMBS[4] - reconstructed_current_accumulator)
                     .is_zero()) {
                return false;
            };

        } else {
            // Check the accumulator is copied correctly
            const std::vector current_accumulator_binary_limbs_copy = {
                get_variable(accumulators_binary_limbs_0_wire[i]),
                get_variable(accumulators_binary_limbs_1_wire[i]),
                get_variable(accumulators_binary_limbs_2_wire[i]),
                get_variable(accumulators_binary_limbs_3_wire[i]),
            };
            const std::vector current_accumulator_binary_limbs = {
                get_variable(accumulators_binary_limbs_0_wire[i + 1]),
                get_variable(accumulators_binary_limbs_1_wire[i + 1]),
                get_variable(accumulators_binary_limbs_2_wire[i + 1]),
                get_variable(accumulators_binary_limbs_3_wire[i + 1]),
            };

            for (size_t j = 0; j < current_accumulator_binary_limbs.size(); j++) {
                if (current_accumulator_binary_limbs_copy[j] != current_accumulator_binary_limbs[j]) {
                    return false;
                }
            }
        }
    }
    return true;
};
template GoblinTranslatorCircuitBuilder::AccumulationInput generate_witness_values(
    bb::fr, bb::fr, bb::fr, bb::fr, bb::fr, bb::fr, bb::fr, bb::fq, bb::fq, bb::fq);
} // namespace bb
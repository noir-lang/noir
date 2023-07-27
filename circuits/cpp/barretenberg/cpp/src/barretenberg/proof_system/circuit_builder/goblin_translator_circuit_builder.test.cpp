#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "goblin_translator_circuit_builder.hpp"
#include <array>
#include <cstddef>
#include <gtest/gtest.h>

using namespace barretenberg;
namespace {
auto& engine = numeric::random::get_debug_engine();
}
namespace proof_system {
template <typename Fq, typename Fr>
GoblinTranslatorCircuitBuilder::AccumulationInput generate_witness_values(
    Fr op_code, Fr p_x_lo, Fr p_x_hi, Fr p_y_lo, Fr p_y_hi, Fr z_1, Fr z_2, Fq previous_accumulator, Fq v, Fq x)
{
    constexpr size_t NUM_LIMB_BITS = GoblinTranslatorCircuitBuilder::NUM_LIMB_BITS;
    constexpr size_t MICRO_LIMB_BITS = GoblinTranslatorCircuitBuilder::MICRO_LIMB_BITS;
    constexpr auto shift_1 = GoblinTranslatorCircuitBuilder::SHIFT_1;
    constexpr auto shift_2 = GoblinTranslatorCircuitBuilder::SHIFT_2;
    constexpr auto modulus_u512 = GoblinTranslatorCircuitBuilder::MODULUS_U512;
    constexpr auto neg_modulus_limbs = GoblinTranslatorCircuitBuilder::NEGATIVE_MODULUS_LIMBS;
    constexpr auto shift_2_inverse = GoblinTranslatorCircuitBuilder::SHIFT_2_INVERSE;

    /**
     * @brief A small function to transform a native element Fq into its bigfield representation  in Fr scalars
     *
     */
    auto base_element_to_bigfield = [](Fq& original) {
        uint256_t original_uint = original;
        return std::array<Fr, 5>({ Fr(original_uint.slice(0, NUM_LIMB_BITS)),
                                   Fr(original_uint.slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS)),
                                   Fr(original_uint.slice(2 * NUM_LIMB_BITS, 3 * NUM_LIMB_BITS)),
                                   Fr(original_uint.slice(3 * NUM_LIMB_BITS, 4 * NUM_LIMB_BITS)),
                                   Fr(original_uint) });
    };
    /**
     * @brief A small function to transform a uint512_t element into its bigfield representation  in Fr scalars
     *
     */
    auto uint512_t_to_bigfield = [&shift_2](uint512_t& original) {
        return std::make_tuple(Fr(original.slice(0, NUM_LIMB_BITS).lo),
                               Fr(original.slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS).lo),
                               Fr(original.slice(2 * NUM_LIMB_BITS, 3 * NUM_LIMB_BITS).lo),
                               Fr(original.slice(3 * NUM_LIMB_BITS, 4 * NUM_LIMB_BITS).lo),
                               Fr(original.slice(0, NUM_LIMB_BITS * 2).lo) +
                                   Fr(original.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 4).lo) * shift_2);
    };

    /**
     * @brief A method for splitting wide limbs (P_x_lo, P_y_hi, etc) into two limbs
     *
     */
    auto split_wide_limb_into_2_limbs = [](Fr& wide_limb) {
        return std::make_tuple(Fr(uint256_t(wide_limb).slice(0, NUM_LIMB_BITS)),
                               Fr(uint256_t(wide_limb).slice(NUM_LIMB_BITS, 2 * NUM_LIMB_BITS)));
    };
    auto split_standard_limb_into_micro_limbs = [](Fr& limb) {
        return std::array<Fr, 6>{
            uint256_t(limb).slice(0, MICRO_LIMB_BITS),
            uint256_t(limb).slice(MICRO_LIMB_BITS, 2 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(2 * MICRO_LIMB_BITS, 3 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(3 * MICRO_LIMB_BITS, 4 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(4 * MICRO_LIMB_BITS, 5 * MICRO_LIMB_BITS),
            uint256_t(limb).slice(5 * MICRO_LIMB_BITS, 6 * MICRO_LIMB_BITS),
        };
    };
    //  x and powers of v are given to use in challenge form, so the verifier has to deal with this :)
    Fq v_squared;
    Fq v_cubed;
    Fq v_quarted;
    v_squared = v * v;
    v_cubed = v_squared * v;
    v_quarted = v_cubed * v;

    // Convert the accumulator, powers of v and x into "bigfield" form
    auto previous_accumulator_witnesses = base_element_to_bigfield(previous_accumulator);
    auto v_witnesses = base_element_to_bigfield(v);
    auto v_squared_witnesses = base_element_to_bigfield(v_squared);
    auto v_cubed_witnesses = base_element_to_bigfield(v_cubed);
    auto v_quarted_witnesses = base_element_to_bigfield(v_quarted);
    auto x_witnesses = base_element_to_bigfield(x);

    // To calculate the quotient, we need to evaluate the expression in integers. So we need uint512_t versions of all
    // elements involved
    auto uint_previous_accumulator = uint512_t(previous_accumulator);
    auto uint_x = uint512_t(x);
    auto uint_op = uint512_t(op_code);
    auto uint_p_x = uint512_t(uint256_t(p_x_lo) + (uint256_t(p_x_hi) << (NUM_LIMB_BITS << 1)));
    auto uint_p_y = uint512_t(uint256_t(p_y_lo) + (uint256_t(p_y_hi) << (NUM_LIMB_BITS << 1)));
    auto uint_z_1 = uint512_t(z_1);
    auto uint_z_2 = uint512_t(z_2);
    auto uint_v = uint512_t(v);
    auto uint_v_squared = uint512_t(v_squared);
    auto uint_v_cubed = uint512_t(v_cubed);
    auto uint_v_quarted = uint512_t(v_quarted);

    // Construct Fq for op, P.x, P.y, z_1, z_2 for use in witness computation
    Fq base_op = Fq(uint256_t(op_code));
    Fq base_p_x = Fq(uint256_t(p_x_lo) + (uint256_t(p_x_hi) << (NUM_LIMB_BITS << 1)));
    Fq base_p_y = Fq(uint256_t(p_y_lo) + (uint256_t(p_y_hi) << (NUM_LIMB_BITS << 1)));
    Fq base_z_1 = Fq(uint256_t(z_1));
    Fq base_z_2 = Fq(uint256_t(z_2));

    // Construct bigfield representations of P.x and P.y
    auto [p_x_0, p_x_1] = split_wide_limb_into_2_limbs(p_x_lo);
    auto [p_x_2, p_x_3] = split_wide_limb_into_2_limbs(p_x_hi);
    Fr p_x_prime = p_x_lo + p_x_hi * Fr(shift_2);
    std::array<Fr, 5> p_x_witnesses = { p_x_0, p_x_1, p_x_2, p_x_3, p_x_prime };
    auto [p_y_0, p_y_1] = split_wide_limb_into_2_limbs(p_y_lo);
    auto [p_y_2, p_y_3] = split_wide_limb_into_2_limbs(p_y_hi);
    Fr p_y_prime = p_y_lo + p_y_hi * Fr(shift_2);
    std::array<Fr, 5> p_y_witnesses = { p_y_0, p_y_1, p_y_2, p_y_3, p_y_prime };

    // Construct bigfield representations of z1 and z2 only using 2 limbs each
    // z_1 and z_2 are low enough to act as their own prime limbs
    auto [z_1_lo, z_1_hi] = split_wide_limb_into_2_limbs(z_1);
    auto [z_2_lo, z_2_hi] = split_wide_limb_into_2_limbs(z_2);

    // The formula is `accumulator = accumulator⋅x + (op + v⋅p.x + v²⋅p.y + v³⋅z₁ + v⁴z₂)`. We need to compute the
    // remainder (new accumulator value)

    Fq remainder = previous_accumulator * x + base_z_2 * v_quarted + base_z_1 * v_cubed + base_p_y * v_squared +
                   base_p_x * v + base_op;
    uint512_t quotient_by_modulus = uint_previous_accumulator * uint_x + uint_z_2 * uint_v_quarted +
                                    uint_z_1 * uint_v_cubed + uint_p_y * uint_v_squared + uint_p_x * uint_v + uint_op -
                                    uint512_t(remainder);

    uint512_t quotient = quotient_by_modulus / uint512_t(Fq::modulus);
    constexpr uint512_t MAX_CONSTRAINED_SIZE = uint512_t(1) << 254;
    constexpr uint512_t MAX_Z_SIZE = uint512_t(1) << (NUM_LIMB_BITS * 2);
    numeric::uint1024_t max_quotient =
        (uint1024_t(MAX_CONSTRAINED_SIZE) * MAX_CONSTRAINED_SIZE * 3 + MAX_Z_SIZE * MAX_CONSTRAINED_SIZE * 2 + 4) /
        modulus_u512;
    info("Max quotient: ", max_quotient);
    info("Max quotient range constraint: ", max_quotient.get_msb() + 1);

    auto [remainder_0, remainder_1, remainder_2, remainder_3, remainder_prime] = base_element_to_bigfield(remainder);
    std::array<Fr, 5> remainder_witnesses = { remainder_0, remainder_1, remainder_2, remainder_3, remainder_prime };
    auto [quotient_0, quotient_1, quotient_2, quotient_3, quotient_prime] = uint512_t_to_bigfield(quotient);
    std::array<Fr, 5> quotient_witnesses = { quotient_0, quotient_1, quotient_2, quotient_3, quotient_prime };

    // We will divide by shift_2 instantly in the relation itself, but first we need to compute the low part (0*0) and
    // the high part (0*1, 1*0) multiplied by a signle limb shift
    Fr low_wide_relation_limb_part_1 =
        previous_accumulator_witnesses[0] * x_witnesses[0] + op_code + v_witnesses[0] * p_x_witnesses[0] +
        v_squared_witnesses[0] * p_y_witnesses[0] + v_cubed_witnesses[0] * z_1_lo + v_quarted_witnesses[0] * z_2_lo +
        quotient_witnesses[0] * neg_modulus_limbs[0] - remainder_witnesses[0]; // This covers the lowest limb
    info("LW1:", low_wide_relation_limb_part_1);
    Fr low_wide_relation_limb =
        low_wide_relation_limb_part_1 +
        (previous_accumulator_witnesses[1] * x_witnesses[0] + previous_accumulator_witnesses[0] * x_witnesses[1] +
         v_witnesses[1] * p_x_witnesses[0] + p_x_witnesses[1] * v_witnesses[0] +
         v_squared_witnesses[1] * p_y_witnesses[0] + v_squared_witnesses[0] * p_y_witnesses[1] +
         v_cubed_witnesses[1] * z_1_lo + z_1_hi * v_cubed_witnesses[0] + v_quarted_witnesses[1] * z_2_lo +
         v_quarted_witnesses[0] * z_2_hi + quotient_witnesses[0] * neg_modulus_limbs[1] +
         quotient_witnesses[1] * neg_modulus_limbs[0] - remainder_witnesses[1]) *
            shift_1; // And this covers the limb shifted by 68
    for (auto& limb : quotient_witnesses) {
        info("Q: ", limb);
    }
    // Treating accumulator as 254-bit constrained value
    constexpr auto max_limb_size = (uint512_t(1) << NUM_LIMB_BITS) - 1;
    constexpr auto shift_1_u512 = uint512_t(shift_1);
    constexpr auto op_max_size = uint512_t(4);
    constexpr uint512_t low_wide_limb_maximum_value =
        op_max_size + (max_limb_size * max_limb_size) * ((shift_1_u512 * 12) + 6);
    constexpr uint512_t low_wide_limb_maximum_value_constraint =
        (low_wide_limb_maximum_value >> (2 * NUM_LIMB_BITS)).lo +
        uint256_t(uint64_t((low_wide_limb_maximum_value % uint512_t(1) << (2 * NUM_LIMB_BITS)) != 0));
    constexpr auto low_wide_limb_range_consraint_size = low_wide_limb_maximum_value_constraint.get_msb() + 1;
    info("Low limb range constraint: ", low_wide_limb_range_consraint_size);
    // Low bits have to be zero
    ASSERT(uint256_t(low_wide_relation_limb).slice(0, 2 * NUM_LIMB_BITS) == 0);

    Fr low_wide_relation_limb_divided = low_wide_relation_limb * shift_2_inverse;
    // We need to range constrain the low_wide_relation_limb_divided
    constexpr size_t NUM_LAST_BN254_LIMB_BITS = modulus_u512.get_msb() + 1 - NUM_LIMB_BITS * 3;

    constexpr auto max_high_limb_size = (uint512_t(1) << NUM_LAST_BN254_LIMB_BITS) - 1;
    constexpr uint512_t high_wide_limb_maximum_value =
        low_wide_limb_maximum_value_constraint + (max_limb_size * max_limb_size) * 16 +
        (max_limb_size * max_limb_size * 10 + max_limb_size * max_high_limb_size * 10) * shift_1_u512;
    constexpr uint512_t high_wide_limb_maximum_value_constraint =
        (high_wide_limb_maximum_value >> (2 * NUM_LIMB_BITS)).lo +
        uint256_t(uint64_t((high_wide_limb_maximum_value % uint512_t(1) << (2 * NUM_LIMB_BITS)) != 0));
    constexpr auto high_wide_limb_range_constraint_size = high_wide_limb_maximum_value_constraint.get_msb() + 1;
    info(high_wide_limb_range_constraint_size);
    // 4 high combinations = 8 ml*ml + 8 ml*last_ml. 2 low combinations = 2*ml*ml + 2*ml*last_ml
    Fr high_wide_relation_limb =
        low_wide_relation_limb_divided + previous_accumulator_witnesses[2] * x_witnesses[0] +
        previous_accumulator_witnesses[1] * x_witnesses[1] + previous_accumulator_witnesses[0] * x_witnesses[2] +
        v_witnesses[2] * p_x_witnesses[0] + v_witnesses[1] * p_x_witnesses[1] + v_witnesses[0] * p_x_witnesses[2] +
        v_squared_witnesses[2] * p_y_witnesses[0] + v_squared_witnesses[1] * p_y_witnesses[1] +
        v_squared_witnesses[0] * p_y_witnesses[2] + v_cubed_witnesses[2] * z_1_lo + v_cubed_witnesses[1] * z_1_hi +
        v_quarted_witnesses[2] * z_2_lo + v_quarted_witnesses[1] * z_2_hi +
        quotient_witnesses[2] * neg_modulus_limbs[0] + quotient_witnesses[1] * neg_modulus_limbs[1] +
        quotient_witnesses[0] * neg_modulus_limbs[2] - remainder_witnesses[2] +
        (previous_accumulator_witnesses[3] * x_witnesses[0] + previous_accumulator_witnesses[2] * x_witnesses[1] +
         previous_accumulator_witnesses[1] * x_witnesses[2] + previous_accumulator_witnesses[0] * x_witnesses[3] +
         v_witnesses[3] * p_x_witnesses[0] + v_witnesses[2] * p_x_witnesses[1] + v_witnesses[1] * p_x_witnesses[2] +
         v_witnesses[0] * p_x_witnesses[3] + v_squared_witnesses[3] * p_y_witnesses[0] +
         v_squared_witnesses[2] * p_y_witnesses[1] + v_squared_witnesses[1] * p_y_witnesses[2] +
         v_squared_witnesses[0] * p_y_witnesses[3] + v_cubed_witnesses[3] * z_1_lo + v_cubed_witnesses[2] * z_1_hi +
         v_quarted_witnesses[3] * z_2_lo + v_quarted_witnesses[2] * z_2_hi +
         quotient_witnesses[3] * neg_modulus_limbs[0] + quotient_witnesses[2] * neg_modulus_limbs[1] +
         quotient_witnesses[1] * neg_modulus_limbs[2] + quotient_witnesses[0] * neg_modulus_limbs[3] -
         remainder_witnesses[3]) *
            shift_1;
    info("Value: ", high_wide_relation_limb);
    info("Value: ", high_wide_relation_limb * shift_2_inverse);
    ASSERT(uint256_t(high_wide_relation_limb).slice(0, 2 * NUM_LIMB_BITS) == 0);

    GoblinTranslatorCircuitBuilder::AccumulationInput input{
        .op_code = op_code,
        .P_x_lo = p_x_lo,
        .P_x_hi = p_x_hi,
        .P_x_limbs = p_x_witnesses,
        .P_x_microlimbs = {},
        .P_y_lo = p_y_lo,
        .P_y_hi = p_y_hi,
        .P_y_limbs = p_y_witnesses,
        .P_y_microlimbs = {},
        .z_1 = z_1,
        .z_1_limbs = { z_1_lo, z_1_hi },
        .z_1_microlimbs = {},
        .z_2 = z_2,
        .z_2_limbs = { z_2_lo, z_2_hi },
        .z_2_microlimbs = {},
        .previous_accumulator = previous_accumulator_witnesses,
        .current_accumulator = remainder_witnesses,
        .current_accumulator_microlimbs = {},
        .quotient_binary_limbs = quotient_witnesses,
        .quotient_microlimbs = {},
        .relation_wide_limbs = { low_wide_relation_limb_divided, high_wide_relation_limb * shift_2_inverse },
        .x_limbs = x_witnesses,
        .v_limbs = v_witnesses,
        .v_squared_limbs = v_squared_witnesses,
        .v_cubed_limbs = v_cubed_witnesses,
        .v_quarted_limbs = v_quarted_witnesses,

    };
    for (size_t i = 0; i < GoblinTranslatorCircuitBuilder::NUM_BINARY_LIMBS; i++) {
        input.P_x_microlimbs[i] = split_standard_limb_into_micro_limbs(input.P_x_limbs[i]);
    }
    for (size_t i = 0; i < GoblinTranslatorCircuitBuilder::NUM_BINARY_LIMBS; i++) {
        input.P_y_microlimbs[i] = split_standard_limb_into_micro_limbs(input.P_y_limbs[i]);
    }

    for (size_t i = 0; i < GoblinTranslatorCircuitBuilder::NUM_Z_LIMBS; i++) {
        input.z_1_microlimbs[i] = split_standard_limb_into_micro_limbs(input.z_1_limbs[i]);
        input.z_2_microlimbs[i] = split_standard_limb_into_micro_limbs(input.z_2_limbs[i]);
    }
    for (size_t i = 0; i < GoblinTranslatorCircuitBuilder::NUM_BINARY_LIMBS; i++) {
        input.current_accumulator_microlimbs[i] = split_standard_limb_into_micro_limbs(input.current_accumulator[i]);
        // info("Stored: ", single_accumulation_step.current_accumulator_microlimbs[i][5], " at ", i);
    }
    for (size_t i = 0; i < GoblinTranslatorCircuitBuilder::NUM_BINARY_LIMBS; i++) {
        input.quotient_microlimbs[i] = split_standard_limb_into_micro_limbs(input.quotient_binary_limbs[i]);
        // info("Stored: ", single_accumulation_step.current_accumulator_microlimbs[i][5], " at ", i);
    }
    return input;
}
TEST(translator_circuit_builder, scoping_out_the_circuit)
{
    // Questions:
    // 1. Do we need 68-bit limbs at all?
    using Fr = ::curve::BN254::ScalarField;
    using Fq = ::curve::BN254::BaseField;

    constexpr size_t NUM_LIMB_BITS = 68;

    constexpr std::array<Fr, 5> neg_modulus_limbs = GoblinTranslatorCircuitBuilder::NEGATIVE_MODULUS_LIMBS;
    // x is the value (challenge) at which we are evaluating the polynomials
    // y is the end result of the whole combination (I don't know why we use y for domain and x for evalutation in
    // the pepe paper) v is the polynomial batching challenge

    // 2 rows:
    // OP | P.xₗₒ | P.xₕᵢ | P.yₗₒ
    // -  | P.yₕᵢ | z₁    | z₂

    // Rows written vertically:
    // 0	 |  -       |   OP      |
    // 1	 |  P.yₕᵢ   |   P.xₗₒ   |
    // 2	 |  z₁      |   P.xₕᵢ   |
    // 3	 |  z₂      |   P.yₗₒ   |
    // 4	 |  p_x_1   |   p_x_0   | 68-bit limbs
    // 5	 |  p_x_1_0 |   p_x_0_0 | 12 bit limbs
    // 6	 |  p_x_1_1 |   p_x_0_1 | 12 bit limbs
    // 7	 |  p_x_1_2 |   p_x_0_2 | 12 bit limbs
    // 8	 |  p_x_1_3 |   p_x_0_3 | 12 bit limbs
    // 9	 |  p_x_1_4 |   p_x_0_4 | 12 bit limbs
    // 10	 |  p_x_1_5 |   p_x_0_5 | 8 bit limns
    // 11	 |  p_x_3   |   p_x_2   | 68-bit limbs
    // 12	 |  p_x_3_0 |   p_x_2_0 | 12 bit limbs
    // 13	 |  p_x_3_1 |   p_x_2_1 | 12 bit limbs
    // 14	 |  p_x_3_2 |   p_x_2_2 | 12 bit limbs
    // 15	 |  p_x_3_3 |   p_x_2_3 | 12 bit limbs
    // 16	 |  p_x_3_4 |   p_x_2_4 | p_x_3_4 is 2 bits and enforced with a relation. p_x_2_4 is 12 bits
    // 17	 |  -       |   p_x_2_5 | 8 bit limb
    // 18	 |  p_y_1   |   p_y_0   | 68-bit limbs
    // 19	 |  p_y_1_0 |   p_y_0_0 | 12 bit limbs
    // 20	 |  p_y_1_1 |   p_y_0_1 | 12 bit limbs
    // 21	 |  p_y_1_2 |   p_y_0_2 | 12 bit limbs
    // 22	 |  p_y_1_3 |   p_y_0_3 | 12 bit limbs
    // 23	 |  p_y_1_4 |   p_y_0_4 | 12 bit limbs
    // 24	 |  p_y_1_5 |   p_y_0_5 | 8 bit limns
    // 25	 |  p_y_3   |   p_y_2   | 68-bit limbs
    // 26	 |  p_y_3_0 |   p_y_2_0 | 12 bit limbs
    // 27	 |  p_y_3_1 |   p_y_2_1 | 12 bit limbs
    // 28	 |  p_y_3_2 |   p_y_2_2 | 12 bit limbs
    // 29	 |  p_y_3_3 |   p_y_2_3 | 12 bit limbs
    // 30	 |  p_y_3_4 |   p_y_2_4 | p_y_3_4 is 2 bits and enforced with a relation. p_y_2_4 is 12 bits
    // 31	 |  -       |   p_y_2_5 | 8 bit limb
    // 32	 |  z_1_hi  |   z_1_lo  | 68 bit limbs
    // 33	 |  z_1_hi_0|   z_1_lo_0| 12 bit limbs
    // 34	 |  z_1_hi_1|   z_1_lo_1| 12 bit limbs
    // 35	 |  z_1_hi_2|   z_1_lo_2| 12 bit limbs
    // 36	 |  z_1_hi_3|   z_1_lo_3| 12 bit limbs
    // 37	 |  z_1_hi_4|   z_1_lo_4| 12 bit limbs
    // 38	 |  z_1_hi_5|   z_1_lo_5| 8 bit limbs
    // 39	 |  z_2_hi  |   z_2_lo  | 68 bit limbs
    // 40	 |  z_2_hi_0|   z_2_lo_0| 12 bit limbs
    // 41	 |  z_2_hi_1|   z_2_lo_1| 12 bit limbs
    // 42	 |  z_2_hi_2|   z_2_lo_2| 12 bit limbs
    // 43	 |  z_2_hi_3|   z_2_lo_3| 12 bit limbs
    // 44	 |  z_2_hi_4|   z_2_lo_4| 12 bit limbs
    // 45	 |  z_2_hi_5|   z_2_lo_5| 8 bit limbs
    // 46	 |  Aₚᵣₑᵥ_₀ |   A₀      | 68
    // 47	 |  Aₚᵣₑᵥ_₁ |   A₁      | 68
    // 48	 |  Aₚᵣₑᵥ_₂ |   A₂      | 68
    // 49	 |  Aₚᵣₑᵥ_₃ |   A₃      | 68
    // 50	 |  A_1_0   |   A_0_0   | 12
    // 51	 |  A_1_1   |   A_0_1   | 12
    // 52	 |  A_1_2   |   A_0_2   | 12
    // 53	 |  A_1_3   |   A_0_3   | 12
    // 54	 |  A_1_4   |   A_0_4   | 12
    // 55	 |  A_1_5   |   A_0_5   | 8
    // 56	 |  A_3_0   |   A_2_0   | 12
    // 57	 |  A_3_1   |   A_2_1   | 12
    // 58	 |  A_3_2   |   A_2_2   | 12
    // 59	 |  A_3_3   |   A_2_3   | 12
    // 60	 |  A_3_4   |   A_2_4   | 2/12
    // 61	 |  -       |   A_2_5   | 12
    // 62    |  Q_1     |   Q_0     | 68
    // 63    |  Q_1_0   |   Q_0_0   | 12
    // 64    |  Q_1_1   |   Q_0_1   | 12
    // 65    |  Q_1_2   |   Q_0_2   | 12
    // 66    |  Q_1_3   |   Q_0_3   | 12
    // 67    |  Q_1_4   |   Q_0_4   | 12
    // 68    |  Q_1_5   |   Q_0_5   | 8
    // 69    |  Q_3     |   Q_2     | 68
    // 70    |  Q_3_0   |   Q_2_0   | 12
    // 71    |  Q_3_1   |   Q_2_1   | 12
    // 72    |  Q_3_2   |   Q_2_2   | 12
    // 73    |  Q_3_3   |   Q_2_3   | 12
    // 74    |  Q_3_4   |   Q_2_4   | 4
    // 75    |  -       |   Q_2_5   | 8
    Fr op;
    Fr p_x_lo;
    Fr p_x_hi;
    Fr p_y_lo;
    Fr p_y_hi;
    Fr z_1;
    Fr z_2;
    op = Fr::random_element();
    auto get_random_wide_limb = []() { return Fr(engine.get_random_uint256() >> (256 - NUM_LIMB_BITS * 2)); };
    auto get_random_shortened_wide_limb = []() { return uint256_t(Fq::random_element()) >> (NUM_LIMB_BITS * 2); };
    p_x_lo = get_random_wide_limb();
    p_x_hi = get_random_shortened_wide_limb();
    p_y_lo = get_random_wide_limb();
    p_y_hi = get_random_shortened_wide_limb();
    z_1 = get_random_wide_limb();
    z_2 = get_random_wide_limb();

    Fq accumulator;
    accumulator = Fq::random_element();
    Fq v = Fq::random_element();
    Fq x = Fq::random_element();
    // p_y_lo = get_random_wide_limb();
    //  Creating a bigfield representation from (binary_limb_0, binary_limb_1, binary_limb_2, binary_limb_3, prime_limb)

    // Range constrain all the individual limbs

    // Low bits have to be zero
    // And we'll need to range constrain it
    // 68 can be treated as 12/12/12/12/12/8
    // 68 can be treated as 12/12/12/12/12/8
    GoblinTranslatorCircuitBuilder::AccumulationInput witnesses =
        generate_witness_values(op, p_x_lo, p_x_hi, p_y_lo, p_y_hi, z_1, z_2, accumulator, v, x);
    // Prime relation
    Fr prime_relation = witnesses.previous_accumulator[4] * witnesses.x_limbs[4] + witnesses.op_code +
                        witnesses.v_limbs[4] * witnesses.P_x_limbs[4] +
                        witnesses.v_squared_limbs[4] * witnesses.P_y_limbs[4] + witnesses.v_cubed_limbs[4] * z_1 +
                        witnesses.v_quarted_limbs[4] * z_2 + witnesses.quotient_binary_limbs[4] * neg_modulus_limbs[4] -
                        witnesses.current_accumulator[4];
    EXPECT_EQ(prime_relation, 0);
}

TEST(translator_circuit_builder, circuit_builder_base_case)
{
    // Questions:
    // 1. Do we need 68-bit limbs at all?
    using Fr = ::curve::BN254::ScalarField;
    using Fq = ::curve::BN254::BaseField;
    // using Fq = ::curve::BN254::BaseField;

    constexpr size_t NUM_LIMB_BITS = GoblinTranslatorCircuitBuilder::NUM_LIMB_BITS;

    Fr op;
    op = Fr(engine.get_random_uint8() & 3);
    auto get_random_wide_limb = []() { return Fr(engine.get_random_uint256().slice(0, 2 * NUM_LIMB_BITS)); };
    //  auto get_random_shortened_wide_limb = []() { return uint256_t(Fq::random_element()) >> (NUM_LIMB_BITS * 2); };
    Fq p_x = Fq::random_element();
    Fr p_x_lo = uint256_t(p_x).slice(0, 2 * NUM_LIMB_BITS);
    Fr p_x_hi = uint256_t(p_x).slice(2 * NUM_LIMB_BITS, 4 * NUM_LIMB_BITS);
    Fq p_y = Fq::random_element();
    Fr p_y_lo = uint256_t(p_y).slice(0, 2 * NUM_LIMB_BITS);
    Fr p_y_hi = uint256_t(p_y).slice(2 * NUM_LIMB_BITS, 4 * NUM_LIMB_BITS);
    Fr z_1 = get_random_wide_limb();
    Fr z_2 = get_random_wide_limb();
    Fq v = Fq::random_element();
    Fq x = Fq::random_element();

    Fq previous_accumulator = Fq::random_element();
    GoblinTranslatorCircuitBuilder::AccumulationInput single_accumulation_step =
        generate_witness_values(op, p_x_lo, p_x_hi, p_y_lo, p_y_hi, z_1, z_2, previous_accumulator, v, x);

    auto circuit_builder = GoblinTranslatorCircuitBuilder();
    circuit_builder.create_accumulation_gate(single_accumulation_step);
    EXPECT_TRUE(circuit_builder.check_circuit(x, v));
}
} // namespace proof_system
#include "goblin_translator_circuit_builder.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include <array>
#include <cstddef>
#include <gtest/gtest.h>

using namespace barretenberg;
namespace {
auto& engine = numeric::random::get_debug_engine();
}
namespace proof_system {

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
#pragma once
/**
 * @file goblin_translator_builder.hpp
 * @author @Rumata888
 * @brief Circuit Logic generation for Goblin Plonk translator (checks equivalence of Queues/Transcripts for ECCVM and
 * Recursive Circuits)
 *
 * @copyright Copyright (c) 2023
 *
 */
#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/ecc/curves/bn254/fq.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/proof_system/arithmetization/arithmetization.hpp"
#include "barretenberg/proof_system/op_queue/ecc_op_queue.hpp"
#include "circuit_builder_base.hpp"
#include <array>
#include <cstddef>
#include <cstdlib>
#include <iterator>
#include <tuple>
namespace bb {
/**
 * @brief GoblinTranslatorCircuitBuilder creates a circuit that evaluates the correctness of the evaluation of
 * EccOpQueue in Fq while operating in the Fr scalar field (r is the modulus of Fr and p is the modulus of Fp)
 *
 * @details Goblin Translator Circuit Builder builds a circuit the purpose of which is to calculate the batched
 * evaluation of 5 polynomials in non-native field represented through coefficients in 4 native polynomials (op,
 * x_lo_y_hi, x_hi_z_1, y_lo_z_2):
 *
 * OP | X_LO | X_HI | Y_LO
 * 0  | Y_HI | Z1   | Z2
 *
 *  OP is supposed to be { 0, 1, 2, 3, 4, 8 }. X_LO and Y_LO need to be < 2¹³⁶, X_HI and Y_LO < 2¹¹⁸, Z1 and Z2 < 2¹²⁸.
 *  X_* and Y_* are supposed to be the decompositions of bn254 base fields elements P.x and P.y and are split into two
 * chunks each because the scalar field we are operating on can't fit them
 *
 * Goblin Translator calculates the result of evaluation of a polynomial op + P.x⋅v +P.y⋅v² + z1 ⋅ v³ + z2⋅v⁴ at the
 * given challenge x (evaluation_input_x). For this it uses logic similar to the stdlib bigfield class. We operate in Fr
 * while trying to calculate values in Fq. To show that a⋅b=c mod p, we:
 * 1) Compute a⋅b in integers
 * 2) Compute quotient=a⋅b/p
 * 3) Show that a⋅b - quotient⋅p - c = 0 mod 2²⁷²
 * 4) Show that a⋅b - quotient⋅p - c = 0 mod r (scalar field modulus)
 * This ensures that the logic is sound modulo 2²⁷²⋅r, which means it's correct in integers, if all the values are
 * sufficiently constrained (there is no way to undeflow or overflow)
 *
 * Concretely, Goblin Translator computes one accumulation every two gates:
 * previous_accumulator⋅x + op + P.x⋅v +P.y⋅v² + z1⋅v³ + z2⋅v⁴ = current_accumulator mod p. Because of the nature of
 * polynomial commitment, previous_accumulator is located at higher index than the current_accumulator. Values of x
 * (evaluation_input_x) and v (batching_challenge_v) are precomputed and considered inputs to the relations.
 *
 * P.x and P.y are deconstructed into 4 limbs (3 68-bit and 1 50-bit) for non-native arithmetic
 * z1 and z2 are deconstructed into 2 limbs each (68 and 60 bits)
 * op is small and doesn't have to be deconstructed
 *
 * To show the accumulation is correct we also need to provide the quotient and accumulators as witnesses. Accumulator
 * is split the same way as P.x and P.y, but quotient is 256 bits,so the top limb is 52 bits.
 *
 * Ensuring that the relation mod 2²⁷² is correct is done through splitting this check into two checks modulo 2¹³⁶.
 * First, we check that a proper combination of the values in the lower limbs gives the correct result modulo 2¹³⁶ (by
 * dividing the result by 2¹³⁶ and range constraining it). Then we use the overlow and higher limbs to prove the same
 * modulo 2¹³⁶ again and as a result we get correctness modulo 2²⁷².
 *
 * One big issue are range constraints. In Goblin Translator we check ranges by decomposing LIMBS into special other
 * range constrained MICROLIMBS (have "_CONSTRAINT_" in the name of their wires). These wires always have the range of
 * 14 bits, so when we need to constrain something further we use two wires at once and scale the values (for example,
 * 68 bits are decomposed into 5 14-bit limbs + 1 shifted limb, which is equal to the highest microlimb multiplied by
 * 4). The shifted wires usually have "_TAIL" in the name, but that is not a strict rule. To save space and because of
 * the proving system requirements we put some of the decomposed values from relation limbs (limbs which compute the
 * result of computation modulo 2²⁷² divided by shifts) into constraint wires named after P.x, P.y, accumulator and
 * quotient. This is due to the fact that the highest limb in these four is less than 56 bits, which frees up an extra
 * microlimb.
 *
 */
class GoblinTranslatorCircuitBuilder : public CircuitBuilderBase<bb::fr> {
    // We don't need templating for Goblin
    using Fr = bb::fr;
    using Fq = bb::fq;
    using Arithmetization = arithmetization::GoblinTranslator;

  public:
    static constexpr size_t NUM_WIRES = Arithmetization::NUM_WIRES;

    /**
     * We won't need these standard gates that are defined as virtual in circuit builder base
     *
     */
    void create_add_gate(const add_triple_<Fr>&) override{};
    void create_mul_gate(const mul_triple_<Fr>&) override{};
    void create_bool_gate(const uint32_t) override{};
    void create_poly_gate(const poly_triple_<Fr>&) override{};
    [[nodiscard]] size_t get_num_constant_gates() const override { return 0; };

    /**
     * @brief There are so many wires that naming them has no sense, it is easier to access them with enums
     */
    enum WireIds : size_t {
        OP, // The first 4 wires contain the standard values from the EccQueue wire
        X_LOW_Y_HI,
        X_HIGH_Z_1,
        Y_LOW_Z_2,
        P_X_LOW_LIMBS,                    // P.xₗₒ split into 2 68 bit limbs
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_0, // Low limbs split further into smaller chunks for range constraints
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_1,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_2,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_3,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_4,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_TAIL,
        P_X_HIGH_LIMBS,                    // P.xₕᵢ split into a 68 and a 50 bit limb
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_0, // High limbs split into chunks for range constraints
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_1,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_2,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_3,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_4,
        P_X_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL,
        P_Y_LOW_LIMBS,                    // P.yₗₒ split into 2 68 bit limbs
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_0, // Low limbs split into chunks for range constraints
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_1,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_2,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_3,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_4,
        P_Y_LOW_LIMBS_RANGE_CONSTRAINT_TAIL,
        P_Y_HIGH_LIMBS,                    // P.yₕᵢ split into a 68 and a 50 bit limb
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0, // High limbs split into chunks for range constraints
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_1,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_2,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_3,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_4,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL,
        Z_LOW_LIMBS,                    // Low limbs of z_1 and z_2 (68 bits each)
        Z_LOW_LIMBS_RANGE_CONSTRAINT_0, // Range constraints for low limbs of z_1 and z_2
        Z_LOW_LIMBS_RANGE_CONSTRAINT_1,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_2,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_3,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_4,
        Z_LOW_LIMBS_RANGE_CONSTRAINT_TAIL,
        Z_HIGH_LIMBS,                    // High Limbs of z_1 and z_2 (60 bits each)
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_0, // Range constraints for high limbs of z_1 and z_2
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_1,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_2,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_3,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_4,
        Z_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL,
        ACCUMULATORS_BINARY_LIMBS_0, // Contain 68-bit limbs of current and previous accumulator (previous at higher
                                     // indices because of the nuances of KZG commitment).
        ACCUMULATORS_BINARY_LIMBS_1,
        ACCUMULATORS_BINARY_LIMBS_2,
        ACCUMULATORS_BINARY_LIMBS_3,              // Highest limb is 50 bits (254 mod 68)
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_0, // Range constraints for the current accumulator limbs (no need to
                                                  // redo previous accumulator)
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_1,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_2,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_3,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_4,
        ACCUMULATOR_LOW_LIMBS_RANGE_CONSTRAINT_TAIL,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_0,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_1,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_2,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_3,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_4,
        ACCUMULATOR_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL,
        QUOTIENT_LOW_BINARY_LIMBS, // Quotient limbs
        QUOTIENT_HIGH_BINARY_LIMBS,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAIN_0, // Range constraints for quotient
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAIN_1,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAIN_2,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAIN_3,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAIN_4,
        QUOTIENT_LOW_LIMBS_RANGE_CONSTRAIN_TAIL,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAIN_0,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAIN_1,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAIN_2,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAIN_3,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAIN_4,
        QUOTIENT_HIGH_LIMBS_RANGE_CONSTRAIN_TAIL,
        RELATION_WIDE_LIMBS, // Limbs for checking the correctness of  mod 2²⁷² relations.
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_0,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_1,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_2,
        RELATION_WIDE_LIMBS_RANGE_CONSTRAINT_3,

        TOTAL_COUNT

    };

    // Basic goblin translator has the minicircuit size of 2048, so optimize for that case
    // For context, minicircuit is the part of the final polynomials fed into the proving system, where we have all the
    // arithmetic logic. However, the full circuit is several times larger (we use a trick to bring down the degree of
    // the permutation argument)
    static constexpr size_t DEFAULT_TRANSLATOR_VM_LENGTH = 2048;

    // Maximum size of a single limb is 68 bits
    static constexpr size_t NUM_LIMB_BITS = 68;

    // For soundness we need to constrain the highest limb so that the whole value is at most 50 bits
    static constexpr size_t NUM_LAST_LIMB_BITS = Fq::modulus.get_msb() + 1 - 3 * NUM_LIMB_BITS;

    // 128-bit z_1 and z_2 are split into 2 limbs each
    static constexpr size_t NUM_Z_LIMBS = 2;

    // Number of bits in the quotient representation
    static constexpr size_t NUM_QUOTIENT_BITS = 256;

    // Number of bits in the quotient highest limb
    static constexpr size_t NUM_LAST_QUOTIENT_LIMB_BITS = 256 - 3 * NUM_LIMB_BITS;

    // Number of bits in Z scalars
    static constexpr size_t NUM_Z_BITS = 128;
    // The circuit builder has a default range constraint mechanism that is used throughout. It range cosntrains the
    // values to < 2¹⁴
    static constexpr size_t MICRO_LIMB_BITS = 14;

    // Maximum size of a micro limb used for range constraints
    static constexpr auto MAX_MICRO_LIMB_SIZE = (uint256_t(1) << MICRO_LIMB_BITS) - 1;

    // To range constrain a limb to 68 bits we need 6 limbs: 5 for 14-bit windowed chunks and 1 to range constrain the
    // highest to a more restrictive range (0 <= a < 2¹⁴ && 0 <= 4*a < 2¹⁴ ) ~ ( 0 <= a < 2¹² )
    static constexpr size_t NUM_MICRO_LIMBS = 6;

    // How many limbs we split the 254-bit value in
    static constexpr size_t NUM_BINARY_LIMBS = 4;

    // How many limbs we use for computation of result modulo 2²⁷²
    static constexpr size_t NUM_RELATION_WIDE_LIMBS = 2;

    // Range constraint of relation limbs
    static constexpr size_t RELATION_WIDE_LIMB_BITS = 84;

    // Maximum size of each relation limb (in accordance with range constraints)
    static constexpr uint256_t MAX_RELATION_WIDE_LIMB_SIZE = uint256_t(1) << RELATION_WIDE_LIMB_BITS;

    // Shift of a single micro (range constraint) limb
    static constexpr auto MICRO_SHIFT = uint256_t(1) << MICRO_LIMB_BITS;

    // Maximum size of 2 lower limbs concatenated
    static constexpr auto MAX_LOW_WIDE_LIMB_SIZE = (uint256_t(1) << (NUM_LIMB_BITS * 2)) - 1;

    // Maximum size of 2 higher limbs concatenated
    static constexpr auto MAX_HIGH_WIDE_LIMB_SIZE = (uint256_t(1) << (NUM_LIMB_BITS + NUM_LAST_LIMB_BITS)) - 1;

    // How much you'd need to multiply a value by to perform a shift to a higher binary limb
    static constexpr auto SHIFT_1 = uint256_t(1) << NUM_LIMB_BITS;

    // Shift by 2 binary limbs
    static constexpr auto SHIFT_2 = uint256_t(1) << (NUM_LIMB_BITS << 1);

    // Precomputed inverse to easily divide by the shift by 2 limbs
    static constexpr auto SHIFT_2_INVERSE = Fr(SHIFT_2).invert();

    // Shift by 3 binary limbs
    static constexpr auto SHIFT_3 = uint256_t(1) << (NUM_LIMB_BITS * 3);

    // The modulus of the target emulated field as a 512-bit integer
    static constexpr uint512_t MODULUS_U512 = uint512_t(Fq::modulus);

    // The binary modulus used in the CRT computation
    static constexpr uint512_t BINARY_BASIS_MODULUS = uint512_t(1) << (NUM_LIMB_BITS << 2);

    // Negated modulus of the target emulated field in the binary modulus
    static constexpr uint512_t NEGATIVE_PRIME_MODULUS = BINARY_BASIS_MODULUS - MODULUS_U512;

    // Negated modulus of the target emulated field in the binary modulus split into 4 binary limbs + the final limb is
    // the negated modulus of the target emulated field in the scalar field
    static constexpr std::array<Fr, 5> NEGATIVE_MODULUS_LIMBS = {
        Fr(NEGATIVE_PRIME_MODULUS.slice(0, NUM_LIMB_BITS).lo),
        Fr(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2).lo),
        Fr(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3).lo),
        Fr(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4).lo),
        -Fr(Fq::modulus)
    };
    /**
     * @brief The accumulation input structure contains all the necessary values to initalize an accumulation gate as
     * well as additional values for checking its correctness
     *
     * @details For example, we don't really nead the prime limbs, but they serve to check the correctness of over
     * values. We also don't need the values of x's and v's limbs during circuit construction, since they are added to
     * relations directly, but this allows us to check correctness of the computed accumulator
     */
    struct AccumulationInput {
        // Members necessary for the gate creation
        Fr op_code; // Operator
        Fr P_x_lo;
        Fr P_x_hi;
        std::array<Fr, NUM_BINARY_LIMBS> P_x_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> P_x_microlimbs;
        Fr P_y_lo;
        Fr P_y_hi;
        std::array<Fr, NUM_BINARY_LIMBS> P_y_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> P_y_microlimbs;

        Fr z_1;
        std::array<Fr, NUM_Z_LIMBS> z_1_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_Z_LIMBS> z_1_microlimbs;
        Fr z_2;
        std::array<Fr, NUM_Z_LIMBS> z_2_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_Z_LIMBS> z_2_microlimbs;

        std::array<Fr, NUM_BINARY_LIMBS> previous_accumulator;
        std::array<Fr, NUM_BINARY_LIMBS> current_accumulator;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> current_accumulator_microlimbs;
        std::array<Fr, NUM_BINARY_LIMBS> quotient_binary_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> quotient_microlimbs;
        std::array<Fr, NUM_RELATION_WIDE_LIMBS> relation_wide_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, 2> relation_wide_microlimbs;

        // Additional
        std::array<Fr, NUM_BINARY_LIMBS> x_limbs;
        std::array<Fr, NUM_BINARY_LIMBS> v_limbs;
        std::array<Fr, NUM_BINARY_LIMBS> v_squared_limbs = { 0 };
        std::array<Fr, NUM_BINARY_LIMBS> v_cubed_limbs = { 0 };
        std::array<Fr, NUM_BINARY_LIMBS> v_quarted_limbs = { 0 };
    };
    struct RelationInputs {
        std::array<Fr, NUM_BINARY_LIMBS> x_limbs;
        std::array<Fr, NUM_BINARY_LIMBS> v_limbs;
        std::array<Fr, NUM_BINARY_LIMBS> v_squared_limbs = { 0 };
        std::array<Fr, NUM_BINARY_LIMBS> v_cubed_limbs = { 0 };
        std::array<Fr, NUM_BINARY_LIMBS> v_quarted_limbs = { 0 };
    };
    static constexpr std::string_view NAME_STRING = "GoblinTranslatorArithmetization";

    // The challenge that is used for batching together evaluations of several polynomials
    Fq batching_challenge_v;

    // The input we evaluate polynomials on
    Fq evaluation_input_x;

    std::array<std::vector<uint32_t, bb::ContainerSlabAllocator<uint32_t>>, NUM_WIRES> wires;

    /**
     * @brief Construct a new Goblin Translator Circuit Builder object
     *
     * @details Goblin Translator Circuit builder has to be initializaed with evaluation input and batching challenge
     * (they are used to compute witness and to store the value for the prover)
     *
     * @param batching_challenge_v_
     * @param evaluation_input_x_
     */
    GoblinTranslatorCircuitBuilder(Fq batching_challenge_v_, Fq evaluation_input_x_)
        : CircuitBuilderBase(DEFAULT_TRANSLATOR_VM_LENGTH)
        , batching_challenge_v(batching_challenge_v_)
        , evaluation_input_x(evaluation_input_x_)
    {
        add_variable(Fr::zero());
        for (auto& wire : wires) {
            wire.emplace_back(0);
        }
        num_gates++;
    };

    /**
     * @brief Construct a new Goblin Translator Circuit Builder object and feed op_queue inside
     *
     * @details Goblin Translator Circuit builder has to be initialized with evaluation input and batching challenge
     * (they are used to compute witness and to store the value for the prover)
     *
     * @param batching_challenge_v_
     * @param evaluation_input_x_
     * @param op_queue
     */
    GoblinTranslatorCircuitBuilder(Fq batching_challenge_v_,
                                   Fq evaluation_input_x_,
                                   std::shared_ptr<ECCOpQueue> op_queue)
        : GoblinTranslatorCircuitBuilder(batching_challenge_v_, evaluation_input_x_)
    {
        feed_ecc_op_queue_into_circuit(op_queue);
    }

    GoblinTranslatorCircuitBuilder() = default;
    GoblinTranslatorCircuitBuilder(const GoblinTranslatorCircuitBuilder& other) = delete;
    GoblinTranslatorCircuitBuilder(GoblinTranslatorCircuitBuilder&& other) noexcept
        : CircuitBuilderBase(std::move(other)){};
    GoblinTranslatorCircuitBuilder& operator=(const GoblinTranslatorCircuitBuilder& other) = delete;
    GoblinTranslatorCircuitBuilder& operator=(GoblinTranslatorCircuitBuilder&& other) noexcept
    {
        CircuitBuilderBase::operator=(std::move(other));
        return *this;
    };
    ~GoblinTranslatorCircuitBuilder() override = default;

    /**
     * @brief Create limb representations of x and powers of v that are needed to compute the witness or check
     * circuit correctness
     *
     * @param evaluation_input_x The point at which the polynomials are being evaluated
     * @param batching_challenge_v The batching challenge
     * @return RelationInputs
     */
    static RelationInputs compute_relation_inputs_limbs(Fq batching_challenge_v, Fq evaluation_input_x)
    {
        /**
         * @brief A small function to transform a native element Fq into its bigfield representation  in Fr scalars
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
        Fq& v = batching_challenge_v;
        Fq& x = evaluation_input_x;
        Fq v_squared;
        Fq v_cubed;
        Fq v_quarted;
        v_squared = v * v;
        v_cubed = v_squared * v;
        v_quarted = v_cubed * v;
        RelationInputs result;
        result.x_limbs = base_element_to_limbs(x);
        result.v_limbs = base_element_to_limbs(v);
        result.v_squared_limbs = base_element_to_limbs(v_squared);
        result.v_cubed_limbs = base_element_to_limbs(v_cubed);
        result.v_quarted_limbs = base_element_to_limbs(v_quarted);
        return result;
    }

    /**
     * @brief Create a single accumulation gate
     *
     * @details An accumulation gate adds 2 rows from the op queue computing the accumulation of a single EccOpQueue
     * step
     *
     * @param acc_step
     */
    void create_accumulation_gate(AccumulationInput acc_step);

    /**
     * @brief Get the result of accumulation
     *
     * @return bb::fq
     */
    bb::fq get_computation_result()
    {
        const size_t RESULT_ROW = 1;
        ASSERT(num_gates > RESULT_ROW);
        return (uint256_t(get_variable(wires[WireIds::ACCUMULATORS_BINARY_LIMBS_0][RESULT_ROW])) +
                uint256_t(get_variable(wires[WireIds::ACCUMULATORS_BINARY_LIMBS_1][RESULT_ROW])) * SHIFT_1 +
                uint256_t(get_variable(wires[WireIds::ACCUMULATORS_BINARY_LIMBS_2][RESULT_ROW])) * SHIFT_2 +
                uint256_t(get_variable(wires[WireIds::ACCUMULATORS_BINARY_LIMBS_3][RESULT_ROW])) * SHIFT_3);
    }
    /**
     * @brief Generate all the gates required to prove the correctness of batched evalution of polynomials representing
     * commitments to ECCOpQueue
     *
     * @param ecc_op_queue The queue
     */
    void feed_ecc_op_queue_into_circuit(std::shared_ptr<ECCOpQueue> ecc_op_queue);

    /**
     * @brief Check the witness satisifies the circuit
     *
     * @details Goes through each gate and checks the correctness of accumulation
     *
     * @return true
     * @return false
     */
    bool check_circuit();
};
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
                                                                          Fq evaluation_input_x);
} // namespace bb

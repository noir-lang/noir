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
#include "barretenberg/ecc/curves/bn254/fq.hpp"
#include "barretenberg/proof_system/arithmetization/arithmetization.hpp"
#include "circuit_builder_base.hpp"
#include <array>
#include <cstddef>
namespace proof_system {
class GoblinTranslatorCircuitBuilder : CircuitBuilderBase<arithmetization::GoblinTranslator> {
    // We don't need templating for Goblin
    using Fr = barretenberg::fr;
    using Fq = barretenberg::fq;

  public:
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
     *
     */
    enum WireIds : size_t {
        OP, // The first 4 wires contain the standard values from the EccQueue wire
        X_LO_Y_HI,
        X_HI_Z_1,
        Y_LO_Z_2,
        P_X_LOW_LIMBS,                    // P.xₗₒ split into 2 68 bit limbs
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_0, // Low limbs split further into smaller chunks for range constraints
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_1,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_2,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_3,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_4,
        P_X_LOW_LIMBS_RANGE_CONSTRAINT_TAIL,
        P_X_HIGH_LIMBS,                    // P.xₕᵢ split into 2 68 bit limbs
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
        P_Y_HIGH_LIMBS,                    // P.yₕᵢ split into 2 68 bit limbs
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0, // High limbs split into chunks for range constraints
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_1,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_2,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_3,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_4,
        P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_TAIL,
        Z_LO_LIMBS,                    // Low limbs of z_1 and z_2
        Z_LO_LIMBS_RANGE_CONSTRAINT_0, // Range constraints for low limbs of z_1 and z_2
        Z_LO_LIMBS_RANGE_CONSTRAINT_1,
        Z_LO_LIMBS_RANGE_CONSTRAINT_2,
        Z_LO_LIMBS_RANGE_CONSTRAINT_3,
        Z_LO_LIMBS_RANGE_CONSTRAINT_4,
        Z_LO_LIMBS_RANGE_CONSTRAINT_TAIL,
        Z_HI_LIMBS,                    // Hi Limbs of z_1 and z_2
        Z_HI_LIMBS_RANGE_CONSTRAINT_0, // Range constraints for high limbs of z_1 and z_2
        Z_HI_LIMBS_RANGE_CONSTRAINT_1,
        Z_HI_LIMBS_RANGE_CONSTRAINT_2,
        Z_HI_LIMBS_RANGE_CONSTRAINT_3,
        Z_HI_LIMBS_RANGE_CONSTRAINT_4,
        Z_HI_LIMBS_RANGE_CONSTRAINT_TAIL,
        ACCUMULATORS_BINARY_LIMBS_0, // Contain 68-bit limbs of current and previous accumulator (previous at higher
                                     // indices because of the nuances of KZG commitment)
        ACCUMULATORS_BINARY_LIMBS_1,
        ACCUMULATORS_BINARY_LIMBS_2,
        ACCUMULATORS_BINARY_LIMBS_3,
        ACCUMULATOR_LO_LIMBS_RANGE_CONSTRAINT_0, // Range constraints for the current accumulator limbs (no need to redo
                                                 // previous accumulator)
        ACCUMULATOR_LO_LIMBS_RANGE_CONSTRAINT_1,
        ACCUMULATOR_LO_LIMBS_RANGE_CONSTRAINT_2,
        ACCUMULATOR_LO_LIMBS_RANGE_CONSTRAINT_3,
        ACCUMULATOR_LO_LIMBS_RANGE_CONSTRAINT_4,
        ACCUMULATOR_LO_LIMBS_RANGE_CONSTRAINT_TAIL,
        ACCUMULATOR_HI_LIMBS_RANGE_CONSTRAINT_0,
        ACCUMULATOR_HI_LIMBS_RANGE_CONSTRAINT_1,
        ACCUMULATOR_HI_LIMBS_RANGE_CONSTRAINT_2,
        ACCUMULATOR_HI_LIMBS_RANGE_CONSTRAINT_3,
        ACCUMULATOR_HI_LIMBS_RANGE_CONSTRAINT_4,
        ACCUMULATOR_HI_LIMBS_RANGE_CONSTRAINT_TAIL,
        QUOTIENT_LO_BINARY_LIMBS, // Quotient limbs
        QUOTIENT_HI_BINARY_LIMBS,
        QUOTIENT_LO_LIMBS_RANGE_CONSTRAIN_0, // Range constraints for quotient
        QUOTIENT_LO_LIMBS_RANGE_CONSTRAIN_1,
        QUOTIENT_LO_LIMBS_RANGE_CONSTRAIN_2,
        QUOTIENT_LO_LIMBS_RANGE_CONSTRAIN_3,
        QUOTIENT_LO_LIMBS_RANGE_CONSTRAIN_4,
        QUOTIENT_LO_LIMBS_RANGE_CONSTRAIN_TAIL,
        QUOTIENT_HI_LIMBS_RANGE_CONSTRAIN_0,
        QUOTIENT_HI_LIMBS_RANGE_CONSTRAIN_1,
        QUOTIENT_HI_LIMBS_RANGE_CONSTRAIN_2,
        QUOTIENT_HI_LIMBS_RANGE_CONSTRAIN_3,
        QUOTIENT_HI_LIMBS_RANGE_CONSTRAIN_4,
        QUOTIENT_HI_LIMBS_RANGE_CONSTRAIN_TAIL,
        RELATION_WIDE_LIMBS, // Limbs for checking the correctness of  mod 2²⁷² relations. TODO(kesha): add range
                             // constraints

    };
    static constexpr size_t MAX_OPERAND = 3;
    static constexpr size_t NUM_LIMB_BITS = 68;
    static constexpr size_t NUM_Z_LIMBS = 2;
    static constexpr size_t MICRO_LIMB_BITS = 12;
    static constexpr size_t LEFTOVER_CHUNK_BITS = 8;
    static constexpr size_t NUM_MICRO_LIMBS = 6;
    static constexpr size_t NUM_BINARY_LIMBS = 4;
    static constexpr size_t WIDE_RELATION_LIMB_BITS = 72;
    static constexpr auto MICRO_SHIFT = uint256_t(1) << MICRO_LIMB_BITS;
    static constexpr auto MAXIMUM_LEFTOVER_LIMB_SIZE = (uint256_t(1) << LEFTOVER_CHUNK_BITS) - 1;
    static constexpr size_t NUM_LAST_LIMB_BITS = Fq::modulus.get_msb() + 1 - 3 * NUM_LIMB_BITS;
    static constexpr auto MAX_LOW_WIDE_LIMB_SIZE = (uint256_t(1) << (NUM_LIMB_BITS * 2)) - 1;
    static constexpr auto MAX_HIGH_WIDE_LIMB_SIZE = (uint256_t(1) << (NUM_LIMB_BITS + NUM_LAST_LIMB_BITS)) - 1;
    static constexpr auto SHIFT_1 = uint256_t(1) << NUM_LIMB_BITS;
    static constexpr auto SHIFT_2 = uint256_t(1) << (NUM_LIMB_BITS << 1);
    static constexpr auto SHIFT_2_INVERSE = Fr(SHIFT_2).invert();
    static constexpr uint512_t MODULUS_U512 = uint512_t(Fq::modulus);
    static constexpr uint512_t BINARY_BASIS_MODULUS = uint512_t(1) << (NUM_LIMB_BITS << 2);
    static constexpr uint512_t NEGATIVE_PRIME_MODULUS = BINARY_BASIS_MODULUS - MODULUS_U512;
    static constexpr std::array<Fr, 5> NEGATIVE_MODULUS_LIMBS = {
        Fr(NEGATIVE_PRIME_MODULUS.slice(0, NUM_LIMB_BITS).lo),
        Fr(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS, NUM_LIMB_BITS * 2).lo),
        Fr(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS * 2, NUM_LIMB_BITS * 3).lo),
        Fr(NEGATIVE_PRIME_MODULUS.slice(NUM_LIMB_BITS * 3, NUM_LIMB_BITS * 4).lo),
        -Fr(Fq::modulus)
    };
    static constexpr std::string_view NAME_STRING = "GoblinTranslatorArithmetization";
    // TODO(kesha): fix size hints
    GoblinTranslatorCircuitBuilder()
        : CircuitBuilderBase({}, 0)
    {
        // We'll have to shift all wires, so we need the starting element to be zero
        for (auto& wire : wires) {
            wire.push_back(0);
        }
    };
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
        std::array<Fr, NUM_BINARY_LIMBS + 1> P_x_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> P_x_microlimbs;
        Fr P_y_lo;
        Fr P_y_hi;
        std::array<Fr, NUM_BINARY_LIMBS + 1> P_y_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> P_y_microlimbs;

        Fr z_1;
        std::array<Fr, NUM_Z_LIMBS> z_1_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_Z_LIMBS> z_1_microlimbs;
        Fr z_2;
        std::array<Fr, NUM_Z_LIMBS> z_2_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_Z_LIMBS> z_2_microlimbs;

        std::array<Fr, NUM_BINARY_LIMBS + 1> previous_accumulator;
        std::array<Fr, NUM_BINARY_LIMBS + 1> current_accumulator;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> current_accumulator_microlimbs;
        std::array<Fr, NUM_BINARY_LIMBS + 1> quotient_binary_limbs;
        std::array<std::array<Fr, NUM_MICRO_LIMBS>, NUM_BINARY_LIMBS> quotient_microlimbs;
        std::array<Fr, 2> relation_wide_limbs;

        // Additional
        std::array<Fr, NUM_BINARY_LIMBS + 1> x_limbs;
        std::array<Fr, NUM_BINARY_LIMBS + 1> v_limbs;
        std::array<Fr, NUM_BINARY_LIMBS + 1> v_squared_limbs = { 0 };
        std::array<Fr, NUM_BINARY_LIMBS + 1> v_cubed_limbs = { 0 };
        std::array<Fr, NUM_BINARY_LIMBS + 1> v_quarted_limbs = { 0 };
    };
    struct RelationInputs {
        std::array<Fr, NUM_BINARY_LIMBS + 1> x_limbs;
        std::array<Fr, NUM_BINARY_LIMBS + 1> v_limbs;
        std::array<Fr, NUM_BINARY_LIMBS + 1> v_squared_limbs = { 0 };
        std::array<Fr, NUM_BINARY_LIMBS + 1> v_cubed_limbs = { 0 };
        std::array<Fr, NUM_BINARY_LIMBS + 1> v_quarted_limbs = { 0 };
    };

    /**
     * @brief Create bigfield representations of x and powers of v
     *
     * @param x The point at which the polynomials are being evaluated
     * @param v The batching challenge
     * @return RelationInputs
     */
    static RelationInputs compute_relation_inputs_limbs(Fq x, Fq v)
    {
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
        Fq v_squared;
        Fq v_cubed;
        Fq v_quarted;
        v_squared = v * v;
        v_cubed = v_squared * v;
        v_quarted = v_cubed * v;
        RelationInputs result;
        result.x_limbs = base_element_to_bigfield(x);
        result.v_limbs = base_element_to_bigfield(v);
        result.v_squared_limbs = base_element_to_bigfield(v_squared);
        result.v_cubed_limbs = base_element_to_bigfield(v_cubed);
        result.v_quarted_limbs = base_element_to_bigfield(v_quarted);
        return result;
    }

    /**
     * @brief Create a single accumulation gate
     *
     * @param acc_step
     */
    void create_accumulation_gate(const AccumulationInput acc_step)
    {
        // The first wires OpQueue/Transcript wires
        ASSERT(uint256_t(acc_step.op_code) <= MAX_OPERAND);
        auto& op_wire = std::get<WireIds::OP>(wires);
        op_wire.push_back(add_variable(acc_step.op_code));
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
        insert_pair_into_wire(WireIds::X_LO_Y_HI, acc_step.P_x_lo, acc_step.P_y_hi);

        // Check and insert P_x_hi and z_1 into wire 2
        ASSERT(uint256_t(acc_step.P_x_hi) <= MAX_HIGH_WIDE_LIMB_SIZE);
        ASSERT(uint256_t(acc_step.z_1) <= MAX_LOW_WIDE_LIMB_SIZE);
        insert_pair_into_wire(WireIds::X_HI_Z_1, acc_step.P_x_hi, acc_step.z_1);

        // Check and insert P_y_lo and z_2 into wire 3
        ASSERT(uint256_t(acc_step.P_y_lo) <= MAX_LOW_WIDE_LIMB_SIZE);
        ASSERT(uint256_t(acc_step.z_2) <= MAX_LOW_WIDE_LIMB_SIZE);
        insert_pair_into_wire(WireIds::Y_LO_Z_2, acc_step.P_y_lo, acc_step.z_2);

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
                                                                        bool relaxed_last_limb = false) {
            if constexpr (total_limbs == (NUM_BINARY_LIMBS + 1)) {
                for (size_t i = 0; i < NUM_BINARY_LIMBS - 1; i++) {
                    ASSERT(uint256_t(limbs[i]) < SHIFT_1);
                }
                if (!relaxed_last_limb) {
                    ASSERT(uint256_t(limbs[NUM_BINARY_LIMBS - 1]) < (uint256_t(1) << NUM_LAST_LIMB_BITS));
                } else {

                    ASSERT(uint256_t(limbs[NUM_BINARY_LIMBS - 1]) < (SHIFT_1));
                }
            } else {
                for (size_t i = 0; i < total_limbs; i++) {
                    ASSERT(uint256_t(limbs[i]) < SHIFT_1);
                }
            }
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

        // Check limb values are in range
        check_binary_limbs_maximum_values(acc_step.P_x_limbs);
        check_binary_limbs_maximum_values(acc_step.P_y_limbs);
        check_binary_limbs_maximum_values(acc_step.z_1_limbs);
        check_binary_limbs_maximum_values(acc_step.z_2_limbs);
        check_binary_limbs_maximum_values(acc_step.previous_accumulator);
        check_binary_limbs_maximum_values(acc_step.current_accumulator);
        check_binary_limbs_maximum_values(acc_step.quotient_binary_limbs, /*relaxed_last_limb=*/true);

        // Insert limbs used in bigfield evaluations
        insert_pair_into_wire(P_X_LOW_LIMBS, acc_step.P_x_limbs[0], acc_step.P_x_limbs[1]);
        insert_pair_into_wire(P_X_HIGH_LIMBS, acc_step.P_x_limbs[2], acc_step.P_x_limbs[3]);
        insert_pair_into_wire(P_Y_LOW_LIMBS, acc_step.P_y_limbs[0], acc_step.P_y_limbs[1]);
        insert_pair_into_wire(P_Y_HIGH_LIMBS, acc_step.P_y_limbs[2], acc_step.P_y_limbs[3]);
        insert_pair_into_wire(Z_LO_LIMBS, acc_step.z_1_limbs[0], acc_step.z_2_limbs[0]);
        insert_pair_into_wire(Z_HI_LIMBS, acc_step.z_1_limbs[1], acc_step.z_2_limbs[1]);
        insert_pair_into_wire(
            QUOTIENT_LO_BINARY_LIMBS, acc_step.quotient_binary_limbs[0], acc_step.quotient_binary_limbs[1]);
        insert_pair_into_wire(
            QUOTIENT_HI_BINARY_LIMBS, acc_step.quotient_binary_limbs[2], acc_step.quotient_binary_limbs[3]);
        insert_pair_into_wire(RELATION_WIDE_LIMBS, acc_step.relation_wide_limbs[0], acc_step.relation_wide_limbs[1]);

        // Check limbs used in range constraints are in range
        check_micro_limbs_maximum_values(acc_step.P_x_microlimbs);
        check_micro_limbs_maximum_values(acc_step.P_y_microlimbs);
        check_micro_limbs_maximum_values(acc_step.z_1_microlimbs);
        check_micro_limbs_maximum_values(acc_step.z_2_microlimbs);
        check_micro_limbs_maximum_values(acc_step.current_accumulator_microlimbs);

        // Check that relation limbs are in range
        ASSERT(uint256_t(acc_step.relation_wide_limbs[0]).get_msb() < WIDE_RELATION_LIMB_BITS);
        ASSERT(uint256_t(acc_step.relation_wide_limbs[1]).get_msb() < WIDE_RELATION_LIMB_BITS);

        /**
         * @brief Put several values in sequential wires
         *
         */
        auto lay_limbs_in_row = [this]<size_t array_size>(std::array<Fr, array_size> input,
                                                          WireIds starting_wire,
                                                          size_t number_of_elements) {
            ASSERT(number_of_elements <= array_size);
            for (size_t i = 0; i < number_of_elements; i++) {
                wires[starting_wire + i].push_back(add_variable(input[i]));
            }
        };
        lay_limbs_in_row(acc_step.P_x_microlimbs[0], P_X_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.P_x_microlimbs[1], P_X_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.P_x_microlimbs[2], P_X_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.P_x_microlimbs[3], P_X_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.P_y_microlimbs[0], P_Y_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.P_y_microlimbs[1], P_Y_LOW_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.P_y_microlimbs[2], P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.P_y_microlimbs[3], P_Y_HIGH_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.z_1_microlimbs[0], Z_LO_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.z_2_microlimbs[0], Z_LO_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.z_1_microlimbs[1], Z_HI_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.z_2_microlimbs[1], Z_HI_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.current_accumulator, ACCUMULATORS_BINARY_LIMBS_0, NUM_BINARY_LIMBS);
        lay_limbs_in_row(acc_step.previous_accumulator, ACCUMULATORS_BINARY_LIMBS_0, NUM_BINARY_LIMBS);
        lay_limbs_in_row(
            acc_step.current_accumulator_microlimbs[0], ACCUMULATOR_LO_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(
            acc_step.current_accumulator_microlimbs[1], ACCUMULATOR_LO_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(
            acc_step.current_accumulator_microlimbs[2], ACCUMULATOR_HI_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(
            acc_step.current_accumulator_microlimbs[3], ACCUMULATOR_HI_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.quotient_microlimbs[0], QUOTIENT_LO_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.quotient_microlimbs[1], QUOTIENT_LO_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.quotient_microlimbs[2], QUOTIENT_HI_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS);
        lay_limbs_in_row(acc_step.quotient_microlimbs[3], QUOTIENT_HI_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS);

        num_gates += 2;
    }

    /**
     * @brief Check the witness satisifies the circuit
     *
     * @details Does one gate for now
     *
     * @param x
     * @param v
     * @return true
     * @return false
     */
    bool check_circuit(Fq x, Fq v)
    {
        // Compute the limbs of x and powers of v (these go into the relation)
        RelationInputs relation_inputs = compute_relation_inputs_limbs(x, v);

        // Get the wires
        auto& op_wire = std::get<OP>(wires);
        auto& x_lo_y_hi_wire = std::get<X_LO_Y_HI>(wires);
        auto& x_hi_z_1_wire = std::get<X_HI_Z_1>(wires);
        auto& y_lo_z_2_wire = std::get<Y_LO_Z_2>(wires);
        auto& p_x_0_p_x_1_wire = std::get<P_X_LOW_LIMBS>(wires);
        auto& p_x_2_p_x_3_wire = std::get<P_X_HIGH_LIMBS>(wires);
        auto& p_y_0_p_y_1_wire = std::get<P_Y_LOW_LIMBS>(wires);
        auto& p_y_2_p_y_3_wire = std::get<P_Y_HIGH_LIMBS>(wires);
        auto& z_lo_wire = std::get<Z_LO_LIMBS>(wires);
        auto& z_hi_wire = std::get<Z_HI_LIMBS>(wires);
        auto& accumulators_binary_limbs_0_wire = std::get<ACCUMULATORS_BINARY_LIMBS_0>(wires);
        auto& accumulators_binary_limbs_1_wire = std::get<ACCUMULATORS_BINARY_LIMBS_1>(wires);
        auto& accumulators_binary_limbs_2_wire = std::get<ACCUMULATORS_BINARY_LIMBS_2>(wires);
        auto& accumulators_binary_limbs_3_wire = std::get<ACCUMULATORS_BINARY_LIMBS_3>(wires);
        auto& quotient_low_binary_limbs = std::get<QUOTIENT_LO_BINARY_LIMBS>(wires);
        auto& quotient_high_binary_limbs = std::get<QUOTIENT_HI_BINARY_LIMBS>(wires);
        auto& relation_wide_limbs_wire = std::get<RELATION_WIDE_LIMBS>(wires);

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
         */
        auto accumulate_limb_from_micro_chunks = [](const std::vector<Fr>& chunks) {
            Fr mini_accumulator(0);
            for (auto it = chunks.end(); it != chunks.begin();) {
                --it;
                mini_accumulator = mini_accumulator * MICRO_SHIFT + *it;
            }
            return mini_accumulator;
        };
        /**
         * @brief Enumerate through the gates
         *
         */
        for (size_t i = 0; i < num_gates; i++) {
            // The main relation is computed between odd and the next even indices. For example, 1 and 2
            if (i & 1) {
                // Get the values
                Fr op_code = get_variable(op_wire[i]);
                Fr p_x_lo = get_variable(x_lo_y_hi_wire[i]);
                Fr p_x_hi = get_variable(x_hi_z_1_wire[i]);
                Fr p_x_0 = get_variable(p_x_0_p_x_1_wire[i]);
                Fr p_x_1 = get_variable(p_x_0_p_x_1_wire[i + 1]);
                Fr p_x_2 = get_variable(p_x_2_p_x_3_wire[i]);
                Fr p_x_3 = get_variable(p_x_2_p_x_3_wire[i + 1]);
                const std::vector p_x_binary_limbs = { p_x_0, p_x_1, p_x_2, p_x_3 };
                Fr p_y_lo = get_variable(y_lo_z_2_wire[i]);
                Fr p_y_hi = get_variable(x_lo_y_hi_wire[i + 1]);
                Fr p_y_0 = get_variable(p_y_0_p_y_1_wire[i]);
                Fr p_y_1 = get_variable(p_y_0_p_y_1_wire[i + 1]);
                Fr p_y_2 = get_variable(p_y_2_p_y_3_wire[i]);
                Fr p_y_3 = get_variable(p_y_2_p_y_3_wire[i + 1]);
                const std::vector p_y_binary_limbs = { p_y_0, p_y_1, p_y_2, p_y_3 };
                Fr z_1 = get_variable(x_hi_z_1_wire[i + 1]);
                Fr z_2 = get_variable(y_lo_z_2_wire[i + 1]);
                Fr z_1_lo = get_variable(z_lo_wire[i]);
                Fr z_2_lo = get_variable(z_lo_wire[i + 1]);
                Fr z_1_hi = get_variable(z_hi_wire[i]);
                Fr z_2_hi = get_variable(z_hi_wire[i + 1]);
                Fr low_wide_relation_limb = get_variable(relation_wide_limbs_wire[i]);
                Fr high_wide_relation_limb = get_variable(relation_wide_limbs_wire[i + 1]);
                const std::vector z_1_binary_limbs = { z_1_lo, z_1_hi };
                const std::vector z_2_binary_limbs = { z_2_lo, z_2_hi };
                const std::vector current_accumulator_binary_limbs = {
                    get_variable(accumulators_binary_limbs_0_wire[i]),
                    get_variable(accumulators_binary_limbs_1_wire[i]),
                    get_variable(accumulators_binary_limbs_2_wire[i]),
                    get_variable(accumulators_binary_limbs_3_wire[i]),
                };
                const std::vector previous_accumulator_binary_limbs = {
                    get_variable(accumulators_binary_limbs_0_wire[i + 1]),
                    get_variable(accumulators_binary_limbs_1_wire[i + 1]),
                    get_variable(accumulators_binary_limbs_2_wire[i + 1]),
                    get_variable(accumulators_binary_limbs_3_wire[i + 1]),
                };
                const std::vector quotient_binary_limbs = {
                    get_variable(quotient_low_binary_limbs[i]),
                    get_variable(quotient_low_binary_limbs[i + 1]),
                    get_variable(quotient_high_binary_limbs[i]),
                    get_variable(quotient_high_binary_limbs[i + 1]),
                };

                // These need to be range constrained, but that logic is not present yet
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
                    get_sequential_micro_chunks(i, Z_LO_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),

                    get_sequential_micro_chunks(i, Z_HI_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                };

                auto z_2_micro_chunks = {

                    get_sequential_micro_chunks(i + 1, Z_LO_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                    get_sequential_micro_chunks(i + 1, Z_HI_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS)
                };

                auto current_accumulator_micro_chunks = {
                    get_sequential_micro_chunks(i, ACCUMULATOR_LO_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                    get_sequential_micro_chunks(i + 1, ACCUMULATOR_LO_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                    get_sequential_micro_chunks(i, ACCUMULATOR_HI_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                    get_sequential_micro_chunks(i + 1, ACCUMULATOR_HI_LIMBS_RANGE_CONSTRAINT_0, NUM_MICRO_LIMBS),
                };
                auto quotient_micro_chunks = {
                    get_sequential_micro_chunks(i, QUOTIENT_LO_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS),
                    get_sequential_micro_chunks(i + 1, QUOTIENT_LO_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS),
                    get_sequential_micro_chunks(i, QUOTIENT_HI_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS),
                    get_sequential_micro_chunks(i + 1, QUOTIENT_HI_LIMBS_RANGE_CONSTRAIN_0, NUM_MICRO_LIMBS),
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

                // Check that limbs have been decomposed into microlimbs correctly
                // value = ∑ (2ˡ)ⁱ⋅ chunkᵢ, where 2ˡ is the shift
                auto check_micro_limb_decomposition_correctness =
                    [&accumulate_limb_from_micro_chunks](const std::vector<Fr>& binary_limbs,
                                                         const std::vector<std::vector<Fr>>& micro_limbs) {
                        ASSERT(binary_limbs.size() == micro_limbs.size());
                        for (size_t i = 0; i < binary_limbs.size(); i++) {
                            if (binary_limbs[i] != accumulate_limb_from_micro_chunks(micro_limbs[i])) {
                                return false;
                            }
                        }
                        return true;
                    };
                // Check all micro limb decompositions
                if (!check_micro_limb_decomposition_correctness(p_x_binary_limbs, p_x_micro_chunks)) {
                    return false;
                }
                if (!check_micro_limb_decomposition_correctness(p_y_binary_limbs, p_y_micro_chunks)) {
                    return false;
                }
                if (!check_micro_limb_decomposition_correctness(z_1_binary_limbs, z_1_micro_chunks)) {
                    return false;
                }
                if (!check_micro_limb_decomposition_correctness(z_2_binary_limbs, z_2_micro_chunks)) {
                    return false;
                }
                if (!check_micro_limb_decomposition_correctness(current_accumulator_binary_limbs,
                                                                current_accumulator_micro_chunks)) {
                    return false;
                }
                if (!check_micro_limb_decomposition_correctness(quotient_binary_limbs, quotient_micro_chunks)) {
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
                    previous_accumulator_binary_limbs[0] * relation_inputs.x_limbs[2] +
                    relation_inputs.v_limbs[2] * p_x_0 + relation_inputs.v_limbs[1] * p_x_1 +
                    relation_inputs.v_limbs[0] * p_x_2 + relation_inputs.v_squared_limbs[2] * p_y_0 +
                    relation_inputs.v_squared_limbs[1] * p_y_1 + relation_inputs.v_squared_limbs[0] * p_y_2 +
                    relation_inputs.v_cubed_limbs[2] * z_1_lo + relation_inputs.v_cubed_limbs[1] * z_1_hi +
                    relation_inputs.v_quarted_limbs[2] * z_2_lo + relation_inputs.v_quarted_limbs[1] * z_2_hi +
                    quotient_binary_limbs[2] * NEGATIVE_MODULUS_LIMBS[0] +
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
            }
        }
        return true;
    }
};
template <typename Fq, typename Fr>
GoblinTranslatorCircuitBuilder::AccumulationInput generate_witness_values(
    Fr op_code, Fr p_x_lo, Fr p_x_hi, Fr p_y_lo, Fr p_y_hi, Fr z_1, Fr z_2, Fq previous_accumulator, Fq v, Fq x);
extern template GoblinTranslatorCircuitBuilder::AccumulationInput generate_witness_values(
    barretenberg::fr op_code,
    barretenberg::fr p_x_lo,
    barretenberg::fr p_x_hi,
    barretenberg::fr p_y_lo,
    barretenberg::fr p_y_hi,
    barretenberg::fr z_1,
    barretenberg::fr z_2,
    barretenberg::fq previous_accumulator,
    barretenberg::fq v,
    barretenberg::fq x);
} // namespace proof_system
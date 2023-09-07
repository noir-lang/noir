/**
 * @file ultra_relation_consistency.test.cpp
 * @brief Show that relation arithmetic has a simple form.
 * @details The purpose of this test suite is to show that the identity arithmetic implemented in the Relations is
 * equivalent to a simpler unoptimized version implemented in the tests themselves. This is useful 1) as documentation
 * since the simple implementations here should make the underlying arithmetic easier to see, and 2) as a check that
 * optimizations introduced into the Relations have not changed the result.
 *
 * For this purpose, we simply feed (the same) random inputs into each of the two implementations and confirm that
 * the outputs match. This does not confirm the correctness of the identity arithmetic (the identities will not be
 * satisfied in general by random inputs) only that the two implementations are equivalent.
 *
 */
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/proof_system/relations/auxiliary_relation.hpp"
#include "barretenberg/proof_system/relations/elliptic_relation.hpp"
#include "barretenberg/proof_system/relations/gen_perm_sort_relation.hpp"
#include "barretenberg/proof_system/relations/lookup_relation.hpp"
#include "barretenberg/proof_system/relations/permutation_relation.hpp"
#include "barretenberg/proof_system/relations/ultra_arithmetic_relation.hpp"
#include <gtest/gtest.h>

using namespace proof_system;

namespace proof_system::ultra_relation_consistency_tests {

using FF = barretenberg::fr;
struct InputElements {
    static constexpr size_t NUM_ELEMENTS = 43;
    std::array<FF, NUM_ELEMENTS> _data;

    static InputElements get_random()
    {
        InputElements result;
        std::generate(result._data.begin(), result._data.end(), [] { return FF::random_element(); });
        return result;
    }

    static InputElements get_special() // use non-random values
    {
        InputElements result;
        FF idx = 0;
        std::generate(result._data.begin(), result._data.end(), [&] {
            idx += FF(1);
            return idx;
        });
        return result;
    }

    FF& q_c = std::get<0>(_data);
    FF& q_l = std::get<1>(_data);
    FF& q_r = std::get<2>(_data);
    FF& q_o = std::get<3>(_data);
    FF& q_4 = std::get<4>(_data);
    FF& q_m = std::get<5>(_data);
    FF& q_arith = std::get<6>(_data);
    FF& q_sort = std::get<7>(_data);
    FF& q_elliptic = std::get<8>(_data);
    FF& q_aux = std::get<9>(_data);
    FF& q_lookup = std::get<10>(_data);
    FF& sigma_1 = std::get<11>(_data);
    FF& sigma_2 = std::get<12>(_data);
    FF& sigma_3 = std::get<13>(_data);
    FF& sigma_4 = std::get<14>(_data);
    FF& id_1 = std::get<15>(_data);
    FF& id_2 = std::get<16>(_data);
    FF& id_3 = std::get<17>(_data);
    FF& id_4 = std::get<18>(_data);
    FF& table_1 = std::get<19>(_data);
    FF& table_2 = std::get<20>(_data);
    FF& table_3 = std::get<21>(_data);
    FF& table_4 = std::get<22>(_data);
    FF& lagrange_first = std::get<23>(_data);
    FF& lagrange_last = std::get<24>(_data);
    FF& w_l = std::get<25>(_data);
    FF& w_r = std::get<26>(_data);
    FF& w_o = std::get<27>(_data);
    FF& w_4 = std::get<28>(_data);
    FF& sorted_accum = std::get<29>(_data);
    FF& z_perm = std::get<30>(_data);
    FF& z_lookup = std::get<31>(_data);
    FF& table_1_shift = std::get<32>(_data);
    FF& table_2_shift = std::get<33>(_data);
    FF& table_3_shift = std::get<34>(_data);
    FF& table_4_shift = std::get<35>(_data);
    FF& w_l_shift = std::get<36>(_data);
    FF& w_r_shift = std::get<37>(_data);
    FF& w_o_shift = std::get<38>(_data);
    FF& w_4_shift = std::get<39>(_data);
    FF& sorted_accum_shift = std::get<40>(_data);
    FF& z_perm_shift = std::get<41>(_data);
    FF& z_lookup_shift = std::get<42>(_data);
};

class UltraRelationConsistency : public testing::Test {
  public:
    template <typename Relation>
    static void validate_relation_execution(const auto& expected_values,
                                            const InputElements& input_elements,
                                            const auto& parameters)
    {
        typename Relation::RelationValues accumulator;
        std::fill(accumulator.begin(), accumulator.end(), FF(0));
        Relation::add_full_relation_value_contribution(accumulator, input_elements, parameters);
        EXPECT_EQ(accumulator, expected_values);
    };
};

TEST_F(UltraRelationConsistency, UltraArithmeticRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = UltraArithmeticRelation<FF>;
        using RelationValues = typename Relation::RelationValues;

        const InputElements input_elements = random_inputs ? InputElements::get_random() : InputElements::get_special();
        const auto& w_1 = input_elements.w_l;
        const auto& w_1_shift = input_elements.w_l_shift;
        const auto& w_2 = input_elements.w_r;
        const auto& w_3 = input_elements.w_o;
        const auto& w_4 = input_elements.w_4;
        const auto& w_4_shift = input_elements.w_4_shift;
        const auto& q_m = input_elements.q_m;
        const auto& q_l = input_elements.q_l;
        const auto& q_r = input_elements.q_r;
        const auto& q_o = input_elements.q_o;
        const auto& q_4 = input_elements.q_4;
        const auto& q_c = input_elements.q_c;
        const auto& q_arith = input_elements.q_arith;

        RelationValues expected_values;
        static const FF neg_half = FF(-2).invert();

        // Contribution 1
        auto contribution_1 = (q_arith - 3) * (q_m * w_2 * w_1) * neg_half;
        contribution_1 += (q_l * w_1) + (q_r * w_2) + (q_o * w_3) + (q_4 * w_4) + q_c;
        contribution_1 += (q_arith - 1) * w_4_shift;
        contribution_1 *= q_arith;
        expected_values[0] = contribution_1;

        // Contribution 2
        auto contribution_2 = (w_1 + w_4 - w_1_shift + q_m);
        contribution_2 *= (q_arith - 2) * (q_arith - 1) * q_arith;
        expected_values[1] = contribution_2;

        const auto parameters = RelationParameters<FF>::get_random();

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(UltraRelationConsistency, UltraPermutationRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = UltraPermutationRelation<FF>;
        using RelationValues = typename Relation::RelationValues;

        const InputElements input_elements = random_inputs ? InputElements::get_random() : InputElements::get_special();
        const auto& w_1 = input_elements.w_l;
        const auto& w_2 = input_elements.w_r;
        const auto& w_3 = input_elements.w_o;
        const auto& w_4 = input_elements.w_4;
        const auto& sigma_1 = input_elements.sigma_1;
        const auto& sigma_2 = input_elements.sigma_2;
        const auto& sigma_3 = input_elements.sigma_3;
        const auto& sigma_4 = input_elements.sigma_4;
        const auto& id_1 = input_elements.id_1;
        const auto& id_2 = input_elements.id_2;
        const auto& id_3 = input_elements.id_3;
        const auto& id_4 = input_elements.id_4;
        const auto& z_perm = input_elements.z_perm;
        const auto& z_perm_shift = input_elements.z_perm_shift;
        const auto& lagrange_first = input_elements.lagrange_first;
        const auto& lagrange_last = input_elements.lagrange_last;

        RelationValues expected_values;

        const auto parameters = RelationParameters<FF>::get_random();
        const auto& beta = parameters.beta;
        const auto& gamma = parameters.gamma;
        const auto& public_input_delta = parameters.public_input_delta;

        // Contribution 1
        auto contribution_1 = (z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) *
                                  (w_3 + id_3 * beta + gamma) * (w_4 + id_4 * beta + gamma) -
                              (z_perm_shift + lagrange_last * public_input_delta) * (w_1 + sigma_1 * beta + gamma) *
                                  (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma) *
                                  (w_4 + sigma_4 * beta + gamma);
        expected_values[0] = contribution_1;

        // Contribution 2
        auto contribution_2 = z_perm_shift * lagrange_last;
        expected_values[1] = contribution_2;

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(UltraRelationConsistency, LookupRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = LookupRelation<FF>;
        using RelationValues = typename Relation::RelationValues;

        const InputElements input_elements = random_inputs ? InputElements::get_random() : InputElements::get_special();
        const auto& w_1 = input_elements.w_l;
        const auto& w_2 = input_elements.w_r;
        const auto& w_3 = input_elements.w_o;

        const auto& w_1_shift = input_elements.w_l_shift;
        const auto& w_2_shift = input_elements.w_r_shift;
        const auto& w_3_shift = input_elements.w_o_shift;

        const auto& table_1 = input_elements.table_1;
        const auto& table_2 = input_elements.table_2;
        const auto& table_3 = input_elements.table_3;
        const auto& table_4 = input_elements.table_4;

        const auto& table_1_shift = input_elements.table_1_shift;
        const auto& table_2_shift = input_elements.table_2_shift;
        const auto& table_3_shift = input_elements.table_3_shift;
        const auto& table_4_shift = input_elements.table_4_shift;

        const auto& s_accum = input_elements.sorted_accum;
        const auto& s_accum_shift = input_elements.sorted_accum_shift;
        const auto& z_lookup = input_elements.z_lookup;
        const auto& z_lookup_shift = input_elements.z_lookup_shift;

        const auto& table_index = input_elements.q_o;
        const auto& column_1_step_size = input_elements.q_r;
        const auto& column_2_step_size = input_elements.q_m;
        const auto& column_3_step_size = input_elements.q_c;
        const auto& q_lookup = input_elements.q_lookup;

        const auto& lagrange_first = input_elements.lagrange_first;
        const auto& lagrange_last = input_elements.lagrange_last;

        RelationValues expected_values;

        const auto parameters = RelationParameters<FF>::get_random();

        const auto eta = parameters.eta;
        const auto beta = parameters.beta;
        const auto gamma = parameters.gamma;
        auto grand_product_delta = parameters.lookup_grand_product_delta;

        // Extract the extended edges for manual computation of relation contribution
        auto one_plus_beta = FF::one() + beta;
        auto gamma_by_one_plus_beta = gamma * one_plus_beta;
        auto eta_sqr = eta * eta;
        auto eta_cube = eta_sqr * eta;

        auto wire_accum = (w_1 + column_1_step_size * w_1_shift) + (w_2 + column_2_step_size * w_2_shift) * eta +
                          (w_3 + column_3_step_size * w_3_shift) * eta_sqr + table_index * eta_cube;

        auto table_accum = table_1 + table_2 * eta + table_3 * eta_sqr + table_4 * eta_cube;
        auto table_accum_shift =
            table_1_shift + table_2_shift * eta + table_3_shift * eta_sqr + table_4_shift * eta_cube;

        // Contribution 1
        auto contribution_1 = (z_lookup + lagrange_first) * (q_lookup * wire_accum + gamma) *
                              (table_accum + table_accum_shift * beta + gamma_by_one_plus_beta) * one_plus_beta;
        contribution_1 -= (z_lookup_shift + lagrange_last * grand_product_delta) *
                          (s_accum + s_accum_shift * beta + gamma_by_one_plus_beta);
        expected_values[0] = contribution_1;

        // Contribution 2
        auto contribution_2 = z_lookup_shift * lagrange_last;
        expected_values[1] = contribution_2;

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(UltraRelationConsistency, GenPermSortRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = GenPermSortRelation<FF>;
        using RelationValues = typename Relation::RelationValues;

        const InputElements input_elements = random_inputs ? InputElements::get_random() : InputElements::get_special();
        const auto& w_1 = input_elements.w_l;
        const auto& w_2 = input_elements.w_r;
        const auto& w_3 = input_elements.w_o;
        const auto& w_4 = input_elements.w_4;
        const auto& w_1_shift = input_elements.w_l_shift;
        const auto& q_sort = input_elements.q_sort;

        auto delta_1 = w_2 - w_1;
        auto delta_2 = w_3 - w_2;
        auto delta_3 = w_4 - w_3;
        auto delta_4 = w_1_shift - w_4;

        auto contribution_1 = delta_1 * (delta_1 - 1) * (delta_1 - 2) * (delta_1 - 3);
        auto contribution_2 = delta_2 * (delta_2 - 1) * (delta_2 - 2) * (delta_2 - 3);
        auto contribution_3 = delta_3 * (delta_3 - 1) * (delta_3 - 2) * (delta_3 - 3);
        auto contribution_4 = delta_4 * (delta_4 - 1) * (delta_4 - 2) * (delta_4 - 3);

        RelationValues expected_values;

        expected_values[0] = contribution_1 * q_sort;
        expected_values[1] = contribution_2 * q_sort;
        expected_values[2] = contribution_3 * q_sort;
        expected_values[3] = contribution_4 * q_sort;

        const auto parameters = RelationParameters<FF>::get_random();

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(UltraRelationConsistency, EllipticRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = EllipticRelation<FF>;
        using RelationValues = typename Relation::RelationValues;

        const InputElements input_elements = random_inputs ? InputElements::get_random() : InputElements::get_special();
        const auto& x_1 = input_elements.w_r;
        const auto& y_1 = input_elements.w_o;

        const auto& x_2 = input_elements.w_l_shift;
        const auto& y_2 = input_elements.w_4_shift;
        const auto& x_3 = input_elements.w_r_shift;
        const auto& y_3 = input_elements.w_o_shift;

        const auto& q_sign = input_elements.q_l;
        const auto& q_beta = input_elements.q_o;
        const auto& q_beta_sqr = input_elements.q_4;
        const auto& q_elliptic = input_elements.q_elliptic;

        RelationValues expected_values;
        // Compute x/y coordinate identities

        // Contribution 1
        auto x_identity = q_sign * (y_1 * y_2 * 2);
        x_identity += q_beta * (x_1 * x_2 * x_3 * 2 + x_1 * x_1 * x_2) * FF(-1);
        x_identity += q_beta_sqr * (x_2 * x_2 * x_3 - x_1 * x_2 * x_2);
        x_identity += (x_1 * x_1 * x_3 - y_2 * y_2 - y_1 * y_1 + x_2 * x_2 * x_2 + x_1 * x_1 * x_1);

        // Contribution 2
        auto y_identity = q_sign * (y_2 * x_3 - y_2 * x_1);
        y_identity += q_beta * (x_2 * y_3 + y_1 * x_2);
        y_identity += (x_1 * y_1 - x_1 * y_3 - y_1 * x_3 - x_1 * y_1);

        expected_values[0] = x_identity * q_elliptic;
        expected_values[1] = y_identity * q_elliptic;

        const auto parameters = RelationParameters<FF>::get_random();

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(UltraRelationConsistency, AuxiliaryRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = AuxiliaryRelation<FF>;
        using RelationValues = typename Relation::RelationValues;

        const InputElements input_elements = random_inputs ? InputElements::get_random() : InputElements::get_special();
        const auto& w_1 = input_elements.w_l;
        const auto& w_2 = input_elements.w_r;
        const auto& w_3 = input_elements.w_o;
        const auto& w_4 = input_elements.w_4;
        const auto& w_1_shift = input_elements.w_l_shift;
        const auto& w_2_shift = input_elements.w_r_shift;
        const auto& w_3_shift = input_elements.w_o_shift;
        const auto& w_4_shift = input_elements.w_4_shift;

        const auto& q_1 = input_elements.q_l;
        const auto& q_2 = input_elements.q_r;
        const auto& q_3 = input_elements.q_o;
        const auto& q_4 = input_elements.q_4;
        const auto& q_m = input_elements.q_m;
        const auto& q_c = input_elements.q_c;
        const auto& q_arith = input_elements.q_arith;
        const auto& q_aux = input_elements.q_aux;

        constexpr FF LIMB_SIZE(uint256_t(1) << 68);
        constexpr FF SUBLIMB_SHIFT(uint256_t(1) << 14);
        constexpr FF SUBLIMB_SHIFT_2(SUBLIMB_SHIFT * SUBLIMB_SHIFT);
        constexpr FF SUBLIMB_SHIFT_3(SUBLIMB_SHIFT_2 * SUBLIMB_SHIFT);
        constexpr FF SUBLIMB_SHIFT_4(SUBLIMB_SHIFT_3 * SUBLIMB_SHIFT);

        const auto parameters = RelationParameters<FF>::get_random();
        const auto& eta = parameters.eta;

        RelationValues expected_values;
        /**
         * Non native field arithmetic gate 2
         *
         *             _                                                                               _
         *            /   _                   _                               _       14                \
         * q_2 . q_4 |   (w_1 . w_2) + (w_1 . w_2) + (w_1 . w_4 + w_2 . w_3 - w_3) . 2    - w_3 - w_4   |
         *            \_                                                                               _/
         *
         **/
        auto limb_subproduct = w_1 * w_2_shift + w_1_shift * w_2;
        auto non_native_field_gate_2 = (w_1 * w_4 + w_2 * w_3 - w_3_shift);
        non_native_field_gate_2 *= LIMB_SIZE;
        non_native_field_gate_2 -= w_4_shift;
        non_native_field_gate_2 += limb_subproduct;

        limb_subproduct *= LIMB_SIZE;
        limb_subproduct += (w_1_shift * w_2_shift);
        auto non_native_field_gate_1 = limb_subproduct;
        non_native_field_gate_1 -= (w_3 + w_4);

        auto non_native_field_gate_3 = limb_subproduct;
        non_native_field_gate_3 += w_4;
        non_native_field_gate_3 -= (w_3_shift + w_4_shift);

        auto non_native_field_identity = q_2 * q_3 * non_native_field_gate_1;
        non_native_field_identity += q_2 * q_4 * non_native_field_gate_2;
        non_native_field_identity += q_2 * q_m * non_native_field_gate_3;

        auto limb_accumulator_1 = w_1 + w_2 * SUBLIMB_SHIFT + w_3 * SUBLIMB_SHIFT_2 + w_1_shift * SUBLIMB_SHIFT_3 +
                                  w_2_shift * SUBLIMB_SHIFT_4 - w_4;

        auto limb_accumulator_2 = w_3 + w_4 * SUBLIMB_SHIFT + w_1_shift * SUBLIMB_SHIFT_2 +
                                  w_2_shift * SUBLIMB_SHIFT_3 + w_3_shift * SUBLIMB_SHIFT_4 - w_4_shift;

        auto limb_accumulator_identity = q_3 * q_4 * limb_accumulator_1;
        limb_accumulator_identity += q_3 * q_m * limb_accumulator_2;

        /**
         * MEMORY
         **/

        /**
         * Memory Record Check
         */
        auto memory_record_check = w_3;
        memory_record_check *= eta;
        memory_record_check += w_2;
        memory_record_check *= eta;
        memory_record_check += w_1;
        memory_record_check *= eta;
        memory_record_check += q_c;
        auto partial_record_check = memory_record_check; // used in RAM consistency check
        memory_record_check = memory_record_check - w_4;

        /**
         * ROM Consistency Check
         */
        auto index_delta = w_1_shift - w_1;
        auto record_delta = w_4_shift - w_4;

        auto index_is_monotonically_increasing = index_delta * index_delta - index_delta;

        // auto adjacent_values_match_if_adjacent_indices_match = (FF(1) - index_delta) * record_delta;
        auto adjacent_values_match_if_adjacent_indices_match = (index_delta * FF(-1) + FF(1)) * record_delta;

        expected_values[1] = adjacent_values_match_if_adjacent_indices_match * (q_1 * q_2);
        expected_values[2] = index_is_monotonically_increasing * (q_1 * q_2);
        auto ROM_consistency_check_identity = memory_record_check * (q_1 * q_2);

        /**
         * RAM Consistency Check
         */
        auto access_type = (w_4 - partial_record_check);             // will be 0 or 1 for honest Prover
        auto access_check = access_type * access_type - access_type; // check value is 0 or 1

        auto next_gate_access_type = w_3_shift;
        next_gate_access_type *= eta;
        next_gate_access_type += w_2_shift;
        next_gate_access_type *= eta;
        next_gate_access_type += w_1_shift;
        next_gate_access_type *= eta;
        next_gate_access_type = w_4_shift - next_gate_access_type;

        auto value_delta = w_3_shift - w_3;
        auto adjacent_values_match_if_adjacent_indices_match_and_next_access_is_a_read_operation =
            (index_delta * FF(-1) + FF(1)) * value_delta * (next_gate_access_type * FF(-1) + FF(1));

        // We can't apply the RAM consistency check identity on the final entry in the sorted list (the wires in the
        // next gate would make the identity fail). We need to validate that its 'access type' bool is correct. Can't do
        // with an arithmetic gate because of the `eta` factors. We need to check that the *next* gate's access type is
        // correct, to cover this edge case
        auto next_gate_access_type_is_boolean = next_gate_access_type * next_gate_access_type - next_gate_access_type;

        // Putting it all together...
        expected_values[3] =
            adjacent_values_match_if_adjacent_indices_match_and_next_access_is_a_read_operation * (q_arith);
        expected_values[4] = index_is_monotonically_increasing * (q_arith);
        expected_values[5] = next_gate_access_type_is_boolean * (q_arith);
        auto RAM_consistency_check_identity = access_check * (q_arith);

        /**
         * RAM/ROM access check gate
         */
        memory_record_check *= (q_1 * q_m);

        /**
         * RAM Timestamp Consistency Check
         */
        auto timestamp_delta = w_2_shift - w_2;
        auto RAM_timestamp_check_identity = (index_delta * FF(-1) + FF(1)) * timestamp_delta - w_3;
        RAM_timestamp_check_identity *= (q_1 * q_4);

        /**
         * The complete RAM/ROM memory identity
         */
        auto memory_identity = ROM_consistency_check_identity;
        memory_identity += RAM_timestamp_check_identity;
        memory_identity += memory_record_check;
        memory_identity += RAM_consistency_check_identity;

        expected_values[0] = memory_identity + non_native_field_identity + limb_accumulator_identity;
        expected_values[0] *= q_aux;
        expected_values[1] *= q_aux;
        expected_values[2] *= q_aux;
        expected_values[3] *= q_aux;
        expected_values[4] *= q_aux;
        expected_values[5] *= q_aux;

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

} // namespace proof_system::ultra_relation_consistency_tests

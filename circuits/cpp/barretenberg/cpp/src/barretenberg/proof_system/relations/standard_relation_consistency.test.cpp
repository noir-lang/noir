/**
 * @file standard_relation_consistency.test.cpp
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
#include "barretenberg/proof_system/relations/arithmetic_relation.hpp"
#include "barretenberg/proof_system/relations/permutation_relation.hpp"
#include <gtest/gtest.h>

using namespace proof_system;

namespace proof_system::standard_relation_consistency_tests {

using FF = barretenberg::fr;
struct InputElements {
    static constexpr size_t NUM_ELEMENTS = 18;
    std::array<FF, NUM_ELEMENTS> _data;

    static InputElements get_random()
    {
        InputElements result;
        std::generate(result._data.begin(), result._data.end(), [] { return FF::random_element(); });
        return result;
    }

    static InputElements get_special() // Use non-random values
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
    FF& q_m = std::get<4>(_data);
    FF& sigma_1 = std::get<5>(_data);
    FF& sigma_2 = std::get<6>(_data);
    FF& sigma_3 = std::get<7>(_data);
    FF& id_1 = std::get<8>(_data);
    FF& id_2 = std::get<9>(_data);
    FF& id_3 = std::get<10>(_data);
    FF& lagrange_first = std::get<11>(_data);
    FF& lagrange_last = std::get<12>(_data);
    FF& w_l = std::get<13>(_data);
    FF& w_r = std::get<14>(_data);
    FF& w_o = std::get<15>(_data);
    FF& z_perm = std::get<16>(_data);
    FF& z_perm_shift = std::get<17>(_data);
};

class StandardRelationConsistency : public testing::Test {
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

TEST_F(StandardRelationConsistency, ArithmeticRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = ArithmeticRelation<FF>;
        using RelationValues = typename Relation::RelationValues;

        const InputElements input_elements = random_inputs ? InputElements::get_random() : InputElements::get_special();
        const auto& w_l = input_elements.w_l;
        const auto& w_r = input_elements.w_r;
        const auto& w_o = input_elements.w_o;
        const auto& q_m = input_elements.q_m;
        const auto& q_l = input_elements.q_l;
        const auto& q_r = input_elements.q_r;
        const auto& q_o = input_elements.q_o;
        const auto& q_c = input_elements.q_c;

        RelationValues expected_values;
        expected_values[0] = (q_m * w_r * w_l) + (q_r * w_r) + (q_l * w_l) + (q_o * w_o) + (q_c);

        const auto parameters = RelationParameters<FF>::get_random();

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

TEST_F(StandardRelationConsistency, PermutationRelation)
{
    const auto run_test = [](bool random_inputs) {
        using Relation = PermutationRelation<FF>;
        using RelationValues = typename Relation::RelationValues;

        const InputElements input_elements = random_inputs ? InputElements::get_random() : InputElements::get_special();
        const auto& w_1 = input_elements.w_l;
        const auto& w_2 = input_elements.w_r;
        const auto& w_3 = input_elements.w_o;
        const auto& sigma_1 = input_elements.sigma_1;
        const auto& sigma_2 = input_elements.sigma_2;
        const auto& sigma_3 = input_elements.sigma_3;
        const auto& id_1 = input_elements.id_1;
        const auto& id_2 = input_elements.id_2;
        const auto& id_3 = input_elements.id_3;
        const auto& z_perm = input_elements.z_perm;
        const auto& z_perm_shift = input_elements.z_perm_shift;
        const auto& lagrange_first = input_elements.lagrange_first;
        const auto& lagrange_last = input_elements.lagrange_last;

        RelationValues expected_values;
        const auto parameters = RelationParameters<FF>::get_random();
        const auto& beta = parameters.beta;
        const auto& gamma = parameters.gamma;
        const auto& public_input_delta = parameters.public_input_delta;

        expected_values[0] = (z_perm + lagrange_first) * (w_1 + id_1 * beta + gamma) * (w_2 + id_2 * beta + gamma) *
                                 (w_3 + id_3 * beta + gamma) -
                             (z_perm_shift + lagrange_last * public_input_delta) * (w_1 + sigma_1 * beta + gamma) *
                                 (w_2 + sigma_2 * beta + gamma) * (w_3 + sigma_3 * beta + gamma);

        expected_values[1] = z_perm_shift * lagrange_last;

        validate_relation_execution<Relation>(expected_values, input_elements, parameters);
    };
    run_test(/*random_inputs=*/false);
    run_test(/*random_inputs=*/true);
};

} // namespace proof_system::standard_relation_consistency_tests

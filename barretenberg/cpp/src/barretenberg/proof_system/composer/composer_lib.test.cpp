#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/common/slab_allocator.hpp"
#include "barretenberg/honk/flavor/standard.hpp" // TODO: needed?
#include "barretenberg/proof_system/types/circuit_type.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include <array>
#include <gtest/gtest.h>

namespace proof_system::test_composer_lib {

class ComposerLibTests : public ::testing::Test {
  protected:
    using Flavor = honk::flavor::Standard;
    using FF = typename Flavor::FF;
    Flavor::CircuitBuilder circuit_constructor;
    Flavor::ProvingKey proving_key = []() {
        auto crs_factory = barretenberg::srs::factories::CrsFactory<curve::BN254>();
        auto crs = crs_factory.get_prover_crs(4);
        return Flavor::ProvingKey(/*circuit_size=*/4, /*num_public_inputs=*/0);
    }();
};

TEST_F(ComposerLibTests, ConstructSelectors)
{
    circuit_constructor.q_m = { 1, 2, 3, 4 };
    circuit_constructor.q_1 = { 5, 6, 7, 8 };
    circuit_constructor.q_2 = { 9, 10, 11, 12 };
    circuit_constructor.q_3 = { 13, 14, 15, 16 };
    circuit_constructor.q_c = { 17, 18, 19, 20 };

    construct_selector_polynomials<Flavor>(circuit_constructor, &proving_key);

    EXPECT_EQ(proving_key.q_m[0], 1);
    EXPECT_EQ(proving_key.q_m[1], 2);
    EXPECT_EQ(proving_key.q_m[2], 3);
    EXPECT_EQ(proving_key.q_m[3], 4);

    EXPECT_EQ(proving_key.q_l[0], 5);
    EXPECT_EQ(proving_key.q_l[1], 6);
    EXPECT_EQ(proving_key.q_l[2], 7);
    EXPECT_EQ(proving_key.q_l[3], 8);

    EXPECT_EQ(proving_key.q_r[0], 9);
    EXPECT_EQ(proving_key.q_r[1], 10);
    EXPECT_EQ(proving_key.q_r[2], 11);
    EXPECT_EQ(proving_key.q_r[3], 12);

    EXPECT_EQ(proving_key.q_o[0], 13);
    EXPECT_EQ(proving_key.q_o[1], 14);
    EXPECT_EQ(proving_key.q_o[2], 15);
    EXPECT_EQ(proving_key.q_o[3], 16);

    EXPECT_EQ(proving_key.q_c[0], 17);
    EXPECT_EQ(proving_key.q_c[1], 18);
    EXPECT_EQ(proving_key.q_c[2], 19);
    EXPECT_EQ(proving_key.q_c[3], 20);
}

TEST_F(ComposerLibTests, ConstructWitnessPolynomialsBase)
{
    circuit_constructor.add_public_variable(1024);
    circuit_constructor.add_public_variable(1025);

    uint32_t v_1 = circuit_constructor.add_variable(16 + 1);
    uint32_t v_2 = circuit_constructor.add_variable(16 + 2);
    uint32_t v_3 = circuit_constructor.add_variable(16 + 3);
    uint32_t v_4 = circuit_constructor.add_variable(16 + 4);
    uint32_t v_5 = circuit_constructor.add_variable(16 + 5);
    uint32_t v_6 = circuit_constructor.add_variable(16 + 6);
    uint32_t v_7 = circuit_constructor.add_variable(16 + 7);
    uint32_t v_8 = circuit_constructor.add_variable(16 + 8);
    uint32_t v_9 = circuit_constructor.add_variable(16 + 9);
    uint32_t v_10 = circuit_constructor.add_variable(16 + 10);
    uint32_t v_11 = circuit_constructor.add_variable(16 + 11);
    uint32_t v_12 = circuit_constructor.add_variable(16 + 12);

    circuit_constructor.create_add_gate({ v_1, v_5, v_9, 0, 0, 0, 0 });
    circuit_constructor.create_add_gate({ v_2, v_6, v_10, 0, 0, 0, 0 });
    circuit_constructor.create_add_gate({ v_3, v_7, v_11, 0, 0, 0, 0 });
    circuit_constructor.create_add_gate({ v_4, v_8, v_12, 0, 0, 0, 0 });

    /* Execution trace:
           w_l        w_r       w_o
        ------------------------------
        pub1_idx | pub1_idx |    0     <-- public inputs
        pub2_idx | pub2_idx |    0     <-/
        zero_idx | zero_idx | zero_idx <-- fix witness for 0
        one_idx  | zero_idx | zero_idx <-- fix witness for 1
        one_idx  | one_idx  | one_idx  <-- ensure nonzero selectors... TODO(Cody): redundant now
          v_1    |   v_5    |    v_9
          v_2    |   v_6    |    v_10
          v_3    |   v_7    |    v_11
          v_4    |   v_8    |    v_12

     */

    const size_t dyadic_circuit_size = circuit_constructor.get_circuit_subgroup_size(
        circuit_constructor.num_gates + circuit_constructor.public_inputs.size());

    auto wires = construct_wire_polynomials_base<Flavor>(circuit_constructor, dyadic_circuit_size);
    auto& w_l = wires[0];
    auto& w_r = wires[1];
    auto& w_o = wires[2];
    auto& zero_idx = circuit_constructor.zero_idx;
    auto& one_idx = circuit_constructor.one_idx;

    EXPECT_EQ(w_l[0], 1024);
    EXPECT_EQ(w_l[1], 1025);
    EXPECT_EQ(w_l[2], zero_idx);
    EXPECT_EQ(w_l[3], one_idx);
    EXPECT_EQ(w_l[4], one_idx);
    EXPECT_EQ(w_l[5], 17);
    EXPECT_EQ(w_l[6], 18);
    EXPECT_EQ(w_l[7], 19);
    EXPECT_EQ(w_l[8], 20);

    EXPECT_EQ(w_r[0], 1024);
    EXPECT_EQ(w_r[1], 1025);
    EXPECT_EQ(w_r[2], zero_idx);
    EXPECT_EQ(w_r[3], zero_idx);
    EXPECT_EQ(w_r[4], one_idx);
    EXPECT_EQ(w_r[5], 21);
    EXPECT_EQ(w_r[6], 22);
    EXPECT_EQ(w_r[7], 23);
    EXPECT_EQ(w_r[8], 24);

    EXPECT_EQ(w_o[0], 0);
    EXPECT_EQ(w_o[1], 0);
    EXPECT_EQ(w_o[2], zero_idx);
    EXPECT_EQ(w_o[3], zero_idx);
    EXPECT_EQ(w_o[4], one_idx);
    EXPECT_EQ(w_o[5], 25);
    EXPECT_EQ(w_o[6], 26);
    EXPECT_EQ(w_o[7], 27);
    EXPECT_EQ(w_o[8], 28);
}

} // namespace proof_system::test_composer_lib

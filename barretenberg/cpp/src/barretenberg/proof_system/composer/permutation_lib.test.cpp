#include "barretenberg/proof_system/composer/permutation_lib.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/proof_system/types/circuit_type.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include <array>
#include <gtest/gtest.h>

using namespace bb;

class PermutationHelperTests : public ::testing::Test {
  protected:
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    using ProvingKey = Flavor::ProvingKey;
    Flavor::CircuitBuilder circuit_constructor;
    srs::factories::CrsFactory<curve::BN254> crs_factory = srs::factories::CrsFactory<curve::BN254>();
    std::shared_ptr<Flavor::ProvingKey> proving_key;

    virtual void SetUp()
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

        size_t num_public_inputs = circuit_constructor.public_inputs.size();
        size_t dyadic_circuit_size =
            circuit_constructor.get_circuit_subgroup_size(circuit_constructor.num_gates + num_public_inputs);
        proving_key = std::make_shared<ProvingKey>(dyadic_circuit_size, num_public_inputs);

        // construct_selector_polynomials<Flavor>(circuit_constructor, proving_key.get());
    }
};

TEST_F(PermutationHelperTests, ComputeWireCopyCycles)
{
    // TODO(#425) Flesh out these tests
    compute_wire_copy_cycles<Flavor>(circuit_constructor);
}

TEST_F(PermutationHelperTests, ComputePermutationMapping)
{
    // TODO(#425) Flesh out these tests
    compute_permutation_mapping<Flavor, /*generalized=*/false>(circuit_constructor, proving_key.get());
}

TEST_F(PermutationHelperTests, ComputeHonkStyleSigmaLagrangePolynomialsFromMapping)
{
    // TODO(#425) Flesh out these tests
    auto mapping = compute_permutation_mapping<Flavor, /*generalized=*/false>(circuit_constructor, proving_key.get());
    compute_honk_style_permutation_lagrange_polynomials_from_mapping<Flavor>(
        proving_key->get_sigma_polynomials(), mapping.sigmas, proving_key.get());
}

TEST_F(PermutationHelperTests, ComputeStandardAuxPolynomials)
{
    // TODO(#425) Flesh out these tests
    compute_first_and_last_lagrange_polynomials<Flavor>(proving_key);
}

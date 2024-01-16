#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/common/slab_allocator.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/proof_system/types/circuit_type.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include <array>
#include <gtest/gtest.h>

namespace proof_system::test_composer_lib {

class ComposerLibTests : public ::testing::Test {
  protected:
    using Flavor = honk::flavor::Ultra;
    using FF = typename Flavor::FF;
    Flavor::CircuitBuilder circuit_constructor;
    Flavor::ProvingKey proving_key = []() {
        auto crs_factory = bb::srs::factories::CrsFactory<curve::BN254>();
        auto crs = crs_factory.get_prover_crs(4);
        return Flavor::ProvingKey(/*circuit_size=*/8, /*num_public_inputs=*/0);
    }();
};

TEST_F(ComposerLibTests, ConstructSelectors)
{
    circuit_constructor.q_m() = { 1, 2, 3, 4 };
    circuit_constructor.q_1() = { 5, 6, 7, 8 };
    circuit_constructor.q_2() = { 9, 10, 11, 12 };
    circuit_constructor.q_3() = { 13, 14, 15, 16 };
    circuit_constructor.q_c() = { 17, 18, 19, 20 };

    construct_selector_polynomials<Flavor>(circuit_constructor, &proving_key);
    size_t offset = 0;
    if (Flavor::has_zero_row) {
        offset += 1;
    }

    EXPECT_EQ(proving_key.q_m[0 + offset], 1);
    EXPECT_EQ(proving_key.q_m[1 + offset], 2);
    EXPECT_EQ(proving_key.q_m[2 + offset], 3);
    EXPECT_EQ(proving_key.q_m[3 + offset], 4);

    EXPECT_EQ(proving_key.q_l[0 + offset], 5);
    EXPECT_EQ(proving_key.q_l[1 + offset], 6);
    EXPECT_EQ(proving_key.q_l[2 + offset], 7);
    EXPECT_EQ(proving_key.q_l[3 + offset], 8);

    EXPECT_EQ(proving_key.q_r[0 + offset], 9);
    EXPECT_EQ(proving_key.q_r[1 + offset], 10);
    EXPECT_EQ(proving_key.q_r[2 + offset], 11);
    EXPECT_EQ(proving_key.q_r[3 + offset], 12);

    EXPECT_EQ(proving_key.q_o[0 + offset], 13);
    EXPECT_EQ(proving_key.q_o[1 + offset], 14);
    EXPECT_EQ(proving_key.q_o[2 + offset], 15);
    EXPECT_EQ(proving_key.q_o[3 + offset], 16);

    EXPECT_EQ(proving_key.q_c[0 + offset], 17);
    EXPECT_EQ(proving_key.q_c[1 + offset], 18);
    EXPECT_EQ(proving_key.q_c[2 + offset], 19);
    EXPECT_EQ(proving_key.q_c[3 + offset], 20);
}

} // namespace proof_system::test_composer_lib

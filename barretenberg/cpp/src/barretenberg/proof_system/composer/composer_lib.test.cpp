#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/common/slab_allocator.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/proof_system/types/circuit_type.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include <array>
#include <gtest/gtest.h>

using namespace bb;

class ComposerLibTests : public ::testing::Test {
  protected:
    using Flavor = UltraFlavor;
    using FF = typename Flavor::FF;
    Flavor::CircuitBuilder circuit_constructor;
    Flavor::ProvingKey proving_key = []() {
        auto crs_factory = srs::factories::CrsFactory<bb::curve::BN254>();
        auto crs = crs_factory.get_prover_crs(4);
        return Flavor::ProvingKey(/*circuit_size=*/8, /*num_public_inputs=*/0);
    }();
};
#include "./honk_prover.hpp"
#include "../../proof_system/proving_key/proving_key.hpp"
#include "../../srs/reference_string/file_reference_string.hpp"
#include <gtest/gtest.h>
#include <common/mem.hpp>
#include <cstddef>

#pragma GCC diagnostic ignored "-Wunused-variable"
#pragma GCC diagnostic ignored "-Wunused-parameter"

using namespace honk;

namespace test_honk_prover {

TEST(HonkProver, HonkProverInstantiation)
{
    // Define some mock inputs for ProvingKey constructor
    size_t num_gates = 8;
    size_t num_public_inputs = 0;
    auto reference_string = std::make_shared<waffle::FileReferenceString>(num_gates + 1, "../srs_db/ignition");

    // Instatiate a proving_key and make a pointer to it
    auto proving_key =
        std::make_shared<waffle::proving_key>(num_gates, num_public_inputs, reference_string, waffle::STANDARD);

    // Instantiate a HonkProver with the proving_key pointer
    auto honk_prover = Prover(proving_key);
}

} // namespace test_honk_prover
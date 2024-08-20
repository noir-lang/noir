#include "barretenberg/goblin/goblin.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"

#include <gtest/gtest.h>

using namespace bb;

class GoblinTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite()
    {
        srs::init_crs_factory("../srs_db/ignition");
        srs::init_grumpkin_crs_factory("../srs_db/grumpkin");
    }

    using Builder = MegaCircuitBuilder;
    using ECCVMVerificationKey = bb::ECCVMFlavor::VerificationKey;
    using TranslatorVerificationKey = bb::TranslatorFlavor::VerificationKey;

    static Builder construct_mock_circuit(std::shared_ptr<ECCOpQueue> op_queue)
    {
        Builder circuit{ op_queue };
        MockCircuits::construct_arithmetic_circuit(circuit, /*target_log2_dyadic_size=*/8);
        MockCircuits::construct_goblin_ecc_op_circuit(circuit);
        return circuit;
    }
};

/**
 * @brief A simple test demonstrating goblin proof construction / verification based on operations from a collection of
 * circuits
 *
 */
TEST_F(GoblinTests, MultipleCircuits)
{
    GoblinProver goblin;

    // Construct and accumulate multiple circuits
    size_t NUM_CIRCUITS = 3;
    for (size_t idx = 0; idx < NUM_CIRCUITS; ++idx) {
        auto circuit = construct_mock_circuit(goblin.op_queue);
        goblin.merge(circuit); // appends a recurisve merge verifier if a merge proof exists
    }

    // Construct a goblin proof which consists of a merge proof and ECCVM/Translator proofs
    GoblinProof proof = goblin.prove();

    // Verify the goblin proof (eccvm, translator, merge); (Construct ECCVM/Translator verification keys from their
    // respective proving keys)
    auto eccvm_vkey = std::make_shared<ECCVMVerificationKey>(goblin.get_eccvm_proving_key());
    auto translator_vkey = std::make_shared<TranslatorVerificationKey>(goblin.get_translator_proving_key());
    GoblinVerifier goblin_verifier{ eccvm_vkey, translator_vkey };
    bool verified = goblin_verifier.verify(proof);

    EXPECT_TRUE(verified);
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/787) Expand these tests.

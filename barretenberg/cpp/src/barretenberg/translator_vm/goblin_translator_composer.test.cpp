#include "barretenberg/translator_vm/goblin_translator_composer.hpp"
#include "barretenberg/common/log.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/proof_system/circuit_builder/goblin_translator_circuit_builder.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/sumcheck/sumcheck_round.hpp"
#include "barretenberg/translator_vm/goblin_translator_prover.hpp"

#include <gtest/gtest.h>
using namespace bb;

namespace {
using CircuitBuilder = GoblinTranslatorFlavor::CircuitBuilder;
using Transcript = GoblinTranslatorFlavor::Transcript;
using OpQueue = ECCOpQueue;
auto& engine = numeric::get_debug_randomness();

std::vector<uint32_t> add_variables(auto& circuit_constructor, std::vector<bb::fr> variables)
{
    std::vector<uint32_t> res;
    for (fr& variable : variables) {
        res.emplace_back(circuit_constructor.add_variable(variable));
    }
    return res;
}

void ensure_non_zero(auto& polynomial)
{
    bool has_non_zero_coefficient = false;
    for (auto& coeff : polynomial) {
        has_non_zero_coefficient |= !coeff.is_zero();
    }
    ASSERT_TRUE(has_non_zero_coefficient);
}

class GoblinTranslatorComposerTests : public ::testing::Test {
  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};
} // namespace

/**
 * @brief Test simple circuit with public inputs
 *
 */
TEST_F(GoblinTranslatorComposerTests, Basic)
{
    using G1 = g1::affine_element;
    using Fr = fr;
    using Fq = fq;

    // Add an element and scalar the accumulation of which leaves no Point-at-Infinity commitments
    const auto x = uint256_t(0xd3c208c16d87cfd3, 0xd97816a916871ca8, 0x9b85045b68181585, 0x30644e72e131a02);
    const auto y = uint256_t(0x3ce1cc9c7e645a83, 0x2edac647851e3ac5, 0xd0cbe61fced2bc53, 0x1a76dae6d3272396);
    auto padding_element = G1(x, y);
    auto padding_scalar = -Fr::one();

    auto P1 = G1::random_element();
    auto P2 = G1::random_element();
    auto z = Fr::random_element();

    // Add the same operations to the ECC op queue; the native computation is performed under the hood.
    auto op_queue = std::make_shared<bb::ECCOpQueue>();

    // Accumulate padding so that we don't produce Point-at-Infinity commitments. Currently our transcript can't handle
    // them
    op_queue->mul_accumulate(padding_element, padding_scalar);

    // Push everything else
    for (size_t i = 0; i < 500; i++) {
        op_queue->add_accumulate(P1);
        op_queue->mul_accumulate(P2, z);
    }

    auto prover_transcript = std::make_shared<Transcript>();
    prover_transcript->send_to_verifier("init", Fq::random_element());
    prover_transcript->export_proof();
    Fq translation_batching_challenge = prover_transcript->template get_challenge<Fq>("Translation:batching_challenge");
    Fq translation_evaluation_challenge = Fq::random_element();
    auto circuit_builder = CircuitBuilder(translation_batching_challenge, translation_evaluation_challenge, op_queue);
    EXPECT_TRUE(circuit_builder.check_circuit());

    auto composer = GoblinTranslatorComposer();
    auto prover = composer.create_prover(circuit_builder, prover_transcript);
    auto proof = prover.construct_proof();

    auto verifier_transcript = std::make_shared<Transcript>(prover_transcript->proof_data);
    verifier_transcript->template receive_from_prover<Fq>("init");
    auto verifier = composer.create_verifier(circuit_builder, verifier_transcript);
    bool verified = verifier.verify_proof(proof);
    EXPECT_TRUE(verified);
}

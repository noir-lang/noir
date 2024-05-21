#include "barretenberg/translator_vm/translator.fuzzer.hpp"
#include "barretenberg/translator_vm/translator_prover.hpp"
#include "barretenberg/translator_vm/translator_verifier.hpp"
extern "C" void LLVMFuzzerInitialize(int*, char***)
{
    srs::init_crs_factory("../srs_db/ignition");
}
/**
 * @brief A very primitive fuzzer for the composer
 *
 * @details Super-slow. Shouldn't be run on its own. First you need to run the circuit builder fuzzer, then minimize the
 * corpus and then just use that corpus
 *
 */
extern "C" int LLVMFuzzerTestOneInput(const unsigned char* data, size_t size)
{
    // Parse challenges and opqueue from data
    auto parsing_result = parse_and_construct_opqueue(data, size);
    if (!parsing_result.has_value()) {
        return 0;
    }
    auto [batching_challenge_init, x, op_queue] = parsing_result.value();
    auto prover_transcript = std::make_shared<bb::TranslatorFlavor::Transcript>();
    prover_transcript->send_to_verifier("init", batching_challenge_init);
    prover_transcript->export_proof();
    Fq translation_batching_challenge = prover_transcript->template get_challenge<Fq>("Translation:batching_challenge");

    // Construct circuit
    auto circuit_builder = TranslatorCircuitBuilder(translation_batching_challenge, x, op_queue);

    // Check that the circuit passes
    bool checked = circuit_builder.check_circuit();

    // Construct proof
    TranslatorProver prover(circuit_builder, prover_transcript);
    auto proof = prover.construct_proof();

    // Verify proof
    auto verifier_transcript = std::make_shared<bb::TranslatorFlavor::Transcript>(prover_transcript->proof_data);
    verifier_transcript->template receive_from_prover<Fq>("init");
    TranslatorVerifier verifier(prover.key, verifier_transcript);
    bool verified = verifier.verify_proof(proof);
    (void)checked;
    (void)verified;
    return 0;
}
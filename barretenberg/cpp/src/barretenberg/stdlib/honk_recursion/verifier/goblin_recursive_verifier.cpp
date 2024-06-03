#include "barretenberg/stdlib/honk_recursion/verifier/goblin_recursive_verifier.hpp"

namespace bb::stdlib::recursion::honk {

void GoblinRecursiveVerifier::verify(const GoblinProof& proof)
{
    // Run the ECCVM recursive verifier
    ECCVMVerifier eccvm_verifier{ builder, verification_keys.eccvm_verification_key };
    eccvm_verifier.verify_proof(proof.eccvm_proof);

    // Run the Translator recursive verifier
    TranslatorVerifier translator_verifier{ builder,
                                            verification_keys.translator_verification_key,
                                            eccvm_verifier.transcript };
    translator_verifier.verify_proof(proof.translator_proof);

    // Verify the consistency between the ECCVM and Translator transcript polynomial evaluations
    // In reality the Goblin Proof is going to already be a stdlib proof and this conversion is not going to happen here
    // (see https://github.com/AztecProtocol/barretenberg/issues/991)
    auto native_translation_evaluations = proof.translation_evaluations;
    auto translation_evaluations =
        TranslationEvaluations{ TranslatorBF::from_witness(builder, native_translation_evaluations.op),
                                TranslatorBF::from_witness(builder, native_translation_evaluations.Px),
                                TranslatorBF::from_witness(builder, native_translation_evaluations.Py),
                                TranslatorBF::from_witness(builder, native_translation_evaluations.z1),
                                TranslatorBF::from_witness(builder, native_translation_evaluations.z2)

        };
    translator_verifier.verify_translation(translation_evaluations);

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/1024): Perform recursive merge verification once it
    // works with Ultra arithmetization
    // MergeVerifier merge_verified{ builder };
    // [[maybe_unused]] auto merge_pairing_points = merge_verifier.verify_proof(proof.merge_proof);
}
} // namespace bb::stdlib::recursion::honk
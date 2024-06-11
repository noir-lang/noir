#pragma once

#include "barretenberg/eccvm/eccvm_prover.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"

namespace bb {
struct GoblinAccumulationOutput {
    HonkProof proof;
    std::shared_ptr<MegaFlavor::VerificationKey> verification_key;
};

struct GoblinProof {
    using TranslationEvaluations = bb::ECCVMProver::TranslationEvaluations;
    using FF = MegaFlavor::FF;

    HonkProof merge_proof;
    HonkProof eccvm_proof;
    HonkProof translator_proof;
    ECCVMProver::TranslationEvaluations translation_evaluations;

    size_t size() const
    {
        return merge_proof.size() + eccvm_proof.size() + translator_proof.size() + TranslationEvaluations::size();
    };

    MSGPACK_FIELDS(merge_proof, eccvm_proof, translator_proof, translation_evaluations);
};
} // namespace bb

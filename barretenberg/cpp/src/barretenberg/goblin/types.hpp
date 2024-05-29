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

    std::vector<FF> to_buffer() const
    {
        // ACIRHACK: so much copying and duplication added here and elsewhere
        std::vector<FF> result;
        result.reserve(size());
        const auto insert = [&result](const std::vector<FF>& buf) {
            result.insert(result.end(), buf.begin(), buf.end());
        };
        insert(merge_proof);
        insert(eccvm_proof);
        insert(translator_proof);
        insert(translation_evaluations.to_buffer());
        return result;
    }
};
} // namespace bb

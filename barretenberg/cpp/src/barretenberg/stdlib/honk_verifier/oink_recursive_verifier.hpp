#pragma once
#include "barretenberg/stdlib/protogalaxy_verifier/recursive_verifier_instance.hpp"
#include "barretenberg/stdlib/transcript/transcript.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_recursive_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_recursive_flavor.hpp"

namespace bb::stdlib::recursion::honk {

template <typename Flavor> class OinkRecursiveVerifier_ {
  public:
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using GroupElement = typename Flavor::GroupElement;
    using Instance = RecursiveVerifierInstance_<Flavor>;
    using VerificationKey = typename Flavor::VerificationKey;
    using Builder = typename Flavor::CircuitBuilder;
    using RelationSeparator = typename Flavor::RelationSeparator;
    using Transcript = bb::BaseTranscript<bb::stdlib::recursion::honk::StdlibTranscriptParams<Builder>>;
    using WitnessCommitments = typename Flavor::WitnessCommitments;
    using OinkProof = std::vector<FF>;

    /**
     * @brief Constructs an Oink Recursive Verifier with a transcript that has been instantiated externally.
     * @details Used when oink recursive verification is part of a larger protocol for which a transcript already
     * exists, e.g. Honk recursive verification.
     *
     * @param builder
     * @param instance Incomplete verifier instance to be completed during verification
     * @param transcript Transcript instantiated with an Oink proof (or a proof that contains an Oink proof).
     * @param domain_separator string used for differentiating instances in the transcript (PG only)
     */
    explicit OinkRecursiveVerifier_(Builder* builder,
                                    const std::shared_ptr<Instance>& instance,
                                    std::shared_ptr<Transcript> transcript,
                                    std::string domain_separator = "");

    /**
     * @brief Constructs an Oink Recursive Verifier
     *
     * @param builder
     * @param instance Incomplete verifier instance to be completed during verification
     * @param domain_separator string used for differentiating instances in the transcript (PG only)
     */
    explicit OinkRecursiveVerifier_(Builder* builder,
                                    const std::shared_ptr<Instance>& instance,
                                    std::string domain_separator = "");

    /**
     * @brief Constructs an oink recursive verifier circuit for an oink proof assumed to be contained in the transcript.
     *
     */
    void verify();

    /**
     * @brief Constructs an oink recursive verifier circuit for a provided oink proof.
     *
     */
    void verify_proof(OinkProof& proof);

    std::shared_ptr<Instance> instance;
    Builder* builder;
    std::shared_ptr<Transcript> transcript;
    std::string domain_separator; // used in PG to distinguish between instances in transcript
};

} // namespace bb::stdlib::recursion::honk

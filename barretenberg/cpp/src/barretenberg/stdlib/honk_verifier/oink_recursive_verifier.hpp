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

    explicit OinkRecursiveVerifier_(Builder* builder,
                                    const std::shared_ptr<Instance>& instance,
                                    std::shared_ptr<Transcript> transcript,
                                    std::string domain_separator = "");

    void verify();

    std::shared_ptr<Instance> instance;
    Builder* builder;
    std::shared_ptr<Transcript> transcript;
    std::string domain_separator; // used in PG to distinguish between instances in transcript
};

} // namespace bb::stdlib::recursion::honk

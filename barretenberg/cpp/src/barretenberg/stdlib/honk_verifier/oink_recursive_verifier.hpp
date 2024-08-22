#pragma once
#include "barretenberg/stdlib/transcript/transcript.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_recursive_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_recursive_flavor.hpp"

namespace bb::stdlib::recursion::honk {

template <typename Flavor> class OinkRecursiveVerifier_ {
  public:
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using GroupElement = typename Flavor::GroupElement;
    using VerificationKey = typename Flavor::VerificationKey;
    using Builder = typename Flavor::CircuitBuilder;
    using RelationSeparator = typename Flavor::RelationSeparator;
    using Transcript = bb::BaseTranscript<bb::stdlib::recursion::honk::StdlibTranscriptParams<Builder>>;
    using WitnessCommitments = typename Flavor::WitnessCommitments;

    struct Output {
        bb::RelationParameters<typename Flavor::FF> relation_parameters;
        WitnessCommitments commitments;
        std::vector<typename Flavor::FF> public_inputs;
        typename Flavor::RelationSeparator alphas;
    };

    explicit OinkRecursiveVerifier_(Builder* builder,
                                    const std::shared_ptr<VerificationKey>& vkey,
                                    std::shared_ptr<Transcript> transcript,
                                    std::string domain_separator = "");

    Output verify();

    std::shared_ptr<VerificationKey> key;
    Builder* builder;
    std::shared_ptr<Transcript> transcript;
    std::string domain_separator; // used in PG to distinguish between instances in transcript
};

} // namespace bb::stdlib::recursion::honk

#pragma once
#include "barretenberg/honk/flavor/goblin_ultra.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/honk/flavor/ultra_grumpkin.hpp"
#include "barretenberg/honk/flavor/ultra_recursive.hpp"
#include "barretenberg/honk/sumcheck/sumcheck.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/stdlib/recursion/honk/transcript/transcript.hpp"

namespace proof_system::plonk::stdlib::recursion::honk {
template <typename Flavor, bool goblin_flag = false> class UltraRecursiveVerifier_ {
    using FF = typename Flavor::FF;
    using Commitment = typename Flavor::Commitment;
    using GroupElement = typename Flavor::GroupElement;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Builder = typename Flavor::CircuitBuilder;
    using PairingPoints = std::array<GroupElement, 2>;

  public:
    explicit UltraRecursiveVerifier_(Builder* builder, std::shared_ptr<VerificationKey> verifier_key = nullptr);
    UltraRecursiveVerifier_(UltraRecursiveVerifier_&& other) = delete;
    UltraRecursiveVerifier_(const UltraRecursiveVerifier_& other) = delete;
    UltraRecursiveVerifier_& operator=(const UltraRecursiveVerifier_& other) = delete;
    UltraRecursiveVerifier_& operator=(UltraRecursiveVerifier_&& other) = delete;
    ~UltraRecursiveVerifier_() = default;

    // TODO(luke): Eventually this will return something like aggregation_state but I'm simplifying for now until we
    // determine the exact interface. Simply returns the two pairing points.
    PairingPoints verify_proof(const plonk::proof& proof);

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::map<std::string, FF> pcs_fr_elements;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    Builder* builder;
    Transcript<Builder> transcript;
};

extern template class UltraRecursiveVerifier_<proof_system::honk::flavor::UltraRecursive, /*goblin_flag*/ false>;
extern template class UltraRecursiveVerifier_<proof_system::honk::flavor::UltraRecursive, /*goblin_flag*/ true>;

using UltraRecursiveVerifier =
    UltraRecursiveVerifier_<proof_system::honk::flavor::UltraRecursive, /*goblin_flag*/ false>;

} // namespace proof_system::plonk::stdlib::recursion::honk

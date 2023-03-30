#pragma once

#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/srs/reference_string/file_reference_string.hpp"
#include "barretenberg/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/honk/proof_system/prover.hpp"
#include "barretenberg/honk/proof_system/verifier.hpp"
#include "barretenberg/proof_system/circuit_constructors/standard_circuit_constructor.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/plonk/proof_system/verifier/verifier.hpp"
#include "barretenberg/proof_system/composer/composer_base.hpp"
#include "barretenberg/proof_system/composer/composer_helper_lib.hpp"
#include "barretenberg/proof_system/composer/permutation_helper.hpp"

#include <utility>

namespace honk {
// TODO(Kesha): change initializations to specify this parameter
// Cody: What does this mean?
template <typename CircuitConstructor> class StandardHonkComposerHelper {
  public:
    static constexpr size_t NUM_RANDOMIZED_GATES = 2; // equal to the number of multilinear evaluations leaked
    static constexpr size_t program_width = CircuitConstructor::program_width;
    std::shared_ptr<bonk::proving_key> circuit_proving_key;
    std::vector<barretenberg::polynomial> wire_polynomials;
    std::shared_ptr<bonk::verification_key> circuit_verification_key;
    // TODO(#218)(kesha): we need to put this into the commitment key, so that the composer doesn't have to handle srs
    // at all
    std::shared_ptr<bonk::ReferenceStringFactory> crs_factory_;
    bool computed_witness = false;
    StandardHonkComposerHelper()
        : StandardHonkComposerHelper(
              std::shared_ptr<bonk::ReferenceStringFactory>(new bonk::FileReferenceStringFactory("../srs_db/ignition")))
    {}
    StandardHonkComposerHelper(std::shared_ptr<bonk::ReferenceStringFactory> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}

    StandardHonkComposerHelper(std::unique_ptr<bonk::ReferenceStringFactory>&& crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}
    StandardHonkComposerHelper(std::shared_ptr<bonk::proving_key> p_key, std::shared_ptr<bonk::verification_key> v_key)
        : circuit_proving_key(std::move(p_key))
        , circuit_verification_key(std::move(v_key))
    {}
    StandardHonkComposerHelper(StandardHonkComposerHelper&& other) noexcept = default;
    StandardHonkComposerHelper(const StandardHonkComposerHelper& other) = delete;
    StandardHonkComposerHelper& operator=(StandardHonkComposerHelper&& other) noexcept = default;
    StandardHonkComposerHelper& operator=(const StandardHonkComposerHelper& other) = delete;
    ~StandardHonkComposerHelper() = default;

    std::shared_ptr<bonk::proving_key> compute_proving_key(const CircuitConstructor& circuit_constructor);
    std::shared_ptr<bonk::verification_key> compute_verification_key(const CircuitConstructor& circuit_constructor);

    StandardVerifier create_verifier(const CircuitConstructor& circuit_constructor);

    template <typename Flavor> StandardProver create_prover(const CircuitConstructor& circuit_constructor);

    // TODO(#216)(Adrian): Seems error prone to provide the number of randomized gates
    // Cody: Where should this go? In the flavor (or whatever that becomes)?
    std::shared_ptr<bonk::proving_key> compute_proving_key_base(
        const CircuitConstructor& circuit_constructor,
        const size_t minimum_ciricut_size = 0,
        const size_t num_randomized_gates = NUM_RANDOMIZED_GATES);
    // This needs to be static as it may be used only to compute the selector commitments.

    static std::shared_ptr<bonk::verification_key> compute_verification_key_base(
        std::shared_ptr<bonk::proving_key> const& proving_key,
        std::shared_ptr<bonk::VerifierReferenceString> const& vrs);

    void compute_witness(const CircuitConstructor& circuit_constructor, const size_t minimum_circuit_size = 0);
};

} // namespace honk

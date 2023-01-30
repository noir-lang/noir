#pragma once

#include <srs/reference_string/file_reference_string.hpp>
#include <proof_system/proving_key/proving_key.hpp>
#include <honk/proof_system/prover.hpp>
#include <honk/proof_system/verifier.hpp>
#include <honk/circuit_constructors/standard_circuit_constructor.hpp>
#include <honk/pcs/commitment_key.hpp>
#include <proof_system/verification_key/verification_key.hpp>
#include <plonk/proof_system/verifier/verifier.hpp>
#include <proof_system/composer/composer_base.hpp>

#include <utility>

namespace honk {
// TODO(Kesha): change initializations to specify this parameter
// Cody: What does this mean?
template <typename CircuitConstructor> class ComposerHelper {
  public:
    static constexpr size_t NUM_RANDOMIZED_GATES = 2; // equal to the number of multilinear evaluations leaked
    static constexpr size_t program_width = CircuitConstructor::program_width;
    std::shared_ptr<waffle::proving_key> circuit_proving_key;
    std::shared_ptr<waffle::verification_key> circuit_verification_key;
    // TODO(kesha): we need to put this into the commitment key, so that the composer doesn't have to handle srs at all
    std::shared_ptr<waffle::ReferenceStringFactory> crs_factory_;
    bool computed_witness = false;
    ComposerHelper()
        : ComposerHelper(std::shared_ptr<waffle::ReferenceStringFactory>(
              new waffle::FileReferenceStringFactory("../srs_db/ignition")))
    {}
    ComposerHelper(std::shared_ptr<waffle::ReferenceStringFactory> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}

    ComposerHelper(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}
    ComposerHelper(std::shared_ptr<waffle::proving_key> p_key, std::shared_ptr<waffle::verification_key> v_key)
        : circuit_proving_key(std::move(p_key))
        , circuit_verification_key(std::move(v_key))
    {}
    ComposerHelper(ComposerHelper&& other) noexcept = default;
    ComposerHelper(const ComposerHelper& other) = delete;
    ComposerHelper& operator=(ComposerHelper&& other) noexcept = default;
    ComposerHelper& operator=(const ComposerHelper& other) = delete;
    ~ComposerHelper() = default;

    std::shared_ptr<waffle::proving_key> compute_proving_key(const CircuitConstructor& circuit_constructor);
    std::shared_ptr<waffle::verification_key> compute_verification_key(const CircuitConstructor& circuit_constructor);

    void compute_witness(const CircuitConstructor& circuit_constructor)
    {
        compute_witness_base<program_width>(circuit_constructor);
    }

    StandardVerifier create_verifier(const CircuitConstructor& circuit_constructor);
    /**
     * Preprocess the circuit. Delegates to create_prover.
     *
     * @return A new initialized prover.
     */
    StandardProver preprocess(const CircuitConstructor& circuit_constructor)
    {
        return create_prover(circuit_constructor);
    };
    StandardProver create_prover(const CircuitConstructor& circuit_constructor);

    StandardUnrolledVerifier create_unrolled_verifier(const CircuitConstructor& circuit_constructor);

    template <typename Flavor>
    StandardUnrolledProver create_unrolled_prover(const CircuitConstructor& circuit_constructor);

    // TODO(Adrian): Seems error prone to provide the number of randomized gates
    // Cody: Where should this go? In the flavor (or whatever that becomes)?
    std::shared_ptr<waffle::proving_key> compute_proving_key_base(
        const CircuitConstructor& circuit_constructor,
        const size_t minimum_ciricut_size = 0,
        const size_t num_randomized_gates = NUM_RANDOMIZED_GATES);
    // This needs to be static as it may be used only to compute the selector commitments.

    static std::shared_ptr<waffle::verification_key> compute_verification_key_base(
        std::shared_ptr<waffle::proving_key> const& proving_key,
        std::shared_ptr<waffle::VerifierReferenceString> const& vrs);

    template <size_t program_width>
    void compute_witness_base(const CircuitConstructor& circuit_constructor, const size_t minimum_circuit_size = 0);
};

} // namespace honk
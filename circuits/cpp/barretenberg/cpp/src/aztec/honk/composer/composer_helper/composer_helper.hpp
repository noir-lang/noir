#include <srs/reference_string/file_reference_string.hpp>
#include <proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/proof_system/prover/prover.hpp>
#include <plonk/proof_system/verifier/verifier.hpp>
#
namespace waffle {
// TODO: change initializations to specify this parameter
#define NUM_RESERVED_GATES 4
class ComposerHelper {
  public:
    std::shared_ptr<proving_key> circuit_proving_key;
    std::shared_ptr<verification_key> circuit_verification_key;
    std::shared_ptr<ReferenceStringFactory> crs_factory_;
    bool computed_witness = false;
    ComposerHelper() {}
    ComposerHelper(std::shared_ptr<ReferenceStringFactory> const& crs_factory)
        : crs_factory_(crs_factory)
    {}

    ComposerHelper(std::unique_ptr<ReferenceStringFactory>&& crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}
    ComposerHelper(std::shared_ptr<proving_key> const& p_key, std::shared_ptr<verification_key> const& v_key)
        : circuit_proving_key(p_key)
        , circuit_verification_key(v_key)
    {}
    ComposerHelper(ComposerHelper&& other) = default;
    ComposerHelper& operator=(ComposerHelper&& other) = default;
    ~ComposerHelper() {}

    template <typename CircuitConstructor>
    std::shared_ptr<proving_key> compute_proving_key(CircuitConstructor& circuit_constructor);
    template <typename CircuitConstructor>
    std::shared_ptr<verification_key> compute_verification_key(CircuitConstructor& circuit_constructor);

    template <typename CircuitConstructor> void compute_witness(CircuitConstructor& circuit_constructor)
    {
        compute_witness_base<4, CircuitConstructor>(circuit_constructor);
    }
    template <typename CircuitConstructor> Verifier create_verifier(CircuitConstructor& circuit_constructor);
    /**
     * Preprocess the circuit. Delegates to create_prover.
     *
     * @return A new initialized prover.
     */
    template <typename CircuitConstructor> Prover preprocess(CircuitConstructor& circuit_constructor)
    {
        return create_prover(circuit_constructor);
    };
    template <typename CircuitConstructor> Prover create_prover(CircuitConstructor& circuit_constructor);
    template <typename CircuitConstructor>
    UnrolledVerifier create_unrolled_verifier(CircuitConstructor& circuit_constructor);
    template <typename CircuitConstructor>
    UnrolledProver create_unrolled_prover(CircuitConstructor& circuit_constructor);
    template <typename CircuitConstructor>
    std::shared_ptr<proving_key> compute_proving_key_base(CircuitConstructor& circuit_constructor,
                                                          const size_t minimum_ciricut_size = 0,
                                                          const size_t num_reserved_gates = NUM_RESERVED_GATES);
    // This needs to be static as it may be used only to compute the selector commitments.

    static std::shared_ptr<verification_key> compute_verification_key_base(
        std::shared_ptr<proving_key> const& proving_key, std::shared_ptr<VerifierReferenceString> const& vrs);
    std::shared_ptr<proving_key> compute_proving_key();
    std::shared_ptr<verification_key> compute_verification_key();
    void compute_witness();
    template <size_t program_width, typename CircuitConstructor>
    void compute_witness_base(CircuitConstructor& circuit_constructor, const size_t minimum_circuit_size = 0);
};
} // namespace waffle
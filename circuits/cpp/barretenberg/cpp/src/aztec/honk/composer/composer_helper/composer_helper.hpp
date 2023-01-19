#include <srs/reference_string/file_reference_string.hpp>
#include <proof_system/proving_key/proving_key.hpp>
#include <honk/proof_system/prover.hpp>
#include <honk/proof_system/verifier.hpp>
#include <honk/pcs/commitment_key.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>
#include <plonk/proof_system/verifier/verifier.hpp>
#include "permutation_helper.hpp"

#include <honk/circuit_constructors/standard_circuit_constructor.hpp>
namespace honk {
// TODO: change initializations to specify this parameter
template <typename CircuitConstructor> class ComposerHelper {
  public:
    static constexpr size_t NUM_RESERVED_GATES = 4; // this must be >= num_roots_cut_out_of_vanishing_polynomial
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
    ComposerHelper(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory)
        : crs_factory_(crs_factory)
    {}

    ComposerHelper(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}
    ComposerHelper(std::shared_ptr<waffle::proving_key> const& p_key,
                   std::shared_ptr<waffle::verification_key> const& v_key)
        : circuit_proving_key(p_key)
        , circuit_verification_key(v_key)
    {}
    ComposerHelper(ComposerHelper&& other) = default;
    ComposerHelper& operator=(ComposerHelper&& other) = default;
    ~ComposerHelper() {}

    std::shared_ptr<waffle::proving_key> compute_proving_key(CircuitConstructor& circuit_constructor);
    std::shared_ptr<waffle::verification_key> compute_verification_key(CircuitConstructor& circuit_constructor);

    void compute_witness(CircuitConstructor& circuit_constructor)
    {
        compute_witness_base<program_width>(circuit_constructor);
    }
    waffle::Verifier create_verifier(CircuitConstructor& circuit_constructor);
    /**
     * Preprocess the circuit. Delegates to create_prover.
     *
     * @return A new initialized prover.
     */
    StandardProver preprocess(CircuitConstructor& circuit_constructor) { return create_prover(circuit_constructor); };
    StandardProver create_prover(CircuitConstructor& circuit_constructor);

    waffle::UnrolledVerifier create_unrolled_verifier(CircuitConstructor& circuit_constructor);

    template <typename Flavor> StandardUnrolledProver create_unrolled_prover(CircuitConstructor& circuit_constructor);

    std::shared_ptr<waffle::proving_key> compute_proving_key_base(CircuitConstructor& circuit_constructor,
                                                                  const size_t minimum_ciricut_size = 0,
                                                                  const size_t num_reserved_gates = NUM_RESERVED_GATES);
    // This needs to be static as it may be used only to compute the selector commitments.

    static std::shared_ptr<waffle::verification_key> compute_verification_key_base(
        std::shared_ptr<waffle::proving_key> const& proving_key,
        std::shared_ptr<waffle::VerifierReferenceString> const& vrs);

    template <size_t program_width>
    void compute_witness_base(CircuitConstructor& circuit_constructor, const size_t minimum_circuit_size = 0);
};

} // namespace honk
#pragma once

#include "barretenberg/proof_system/composer/composer_helper_lib.hpp"
#include "barretenberg/plonk/composer/splitting_tmp/composer_helper/composer_helper_lib.hpp"
#include "barretenberg/srs/reference_string/file_reference_string.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/honk/proof_system/ultra_verifier.hpp"

#include <cstddef>
#include <memory>
#include <utility>
#include <vector>

namespace proof_system::honk {
class UltraHonkComposerHelper {
  public:
    using Flavor = flavor::Ultra;
    using CircuitConstructor = Flavor::CircuitConstructor;
    using ProvingKey = Flavor::ProvingKey;
    using VerificationKey = Flavor::VerificationKey;

    // TODO(#340)(luke): In the split composers, NUM_RANDOMIZED_GATES has replaced NUM_RESERVED_GATES (in some places)
    // to determine the next-power-of-2 circuit size. (There are some places in this composer that still use
    // NUM_RESERVED_GATES). Therefore for consistency within this composer itself, and consistency with the original
    // Ultra Composer, this value must match that of NUM_RESERVED_GATES. This issue needs to be reconciled
    // simultaneously here and in the other split composers.
    static constexpr size_t NUM_RANDOMIZED_GATES = 4; // equal to the number of multilinear evaluations leaked
    static constexpr size_t NUM_WIRES = CircuitConstructor::NUM_WIRES;
    std::shared_ptr<ProvingKey> proving_key;
    std::shared_ptr<VerificationKey> verification_key;
    // TODO(#218)(kesha): we need to put this into the commitment key, so that the composer doesn't have to handle srs
    // at all
    std::shared_ptr<ReferenceStringFactory> crs_factory_;

    std::vector<uint32_t> recursive_proof_public_input_indices;
    bool contains_recursive_proof = false;
    bool computed_witness = false;

    // This variable controls the amount with which the lookup table and witness values need to be shifted
    // above to make room for adding randomness into the permutation and witness polynomials in the plookup widget.
    // This must be (num_roots_cut_out_of_the_vanishing_polynomial - 1), since the variable num_roots_cut_out_of_
    // vanishing_polynomial cannot be trivially fetched here, I am directly setting this to 4 - 1 = 3.
    static constexpr size_t s_randomness = 3;

    explicit UltraHonkComposerHelper(std::shared_ptr<ReferenceStringFactory> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}

    UltraHonkComposerHelper(std::shared_ptr<ProvingKey> p_key, std::shared_ptr<VerificationKey> v_key)
        : proving_key(std::move(p_key))
        , verification_key(std::move(v_key))
    {}

    UltraHonkComposerHelper(UltraHonkComposerHelper&& other) noexcept = default;
    UltraHonkComposerHelper(UltraHonkComposerHelper const& other) noexcept = default;
    UltraHonkComposerHelper& operator=(UltraHonkComposerHelper&& other) noexcept = default;
    UltraHonkComposerHelper& operator=(UltraHonkComposerHelper const& other) noexcept = default;
    ~UltraHonkComposerHelper() = default;

    void finalize_circuit(CircuitConstructor& circuit_constructor) { circuit_constructor.finalize_circuit(); };

    std::shared_ptr<ProvingKey> compute_proving_key(const CircuitConstructor& circuit_constructor);
    std::shared_ptr<VerificationKey> compute_verification_key(const CircuitConstructor& circuit_constructor);

    void compute_witness(CircuitConstructor& circuit_constructor);

    UltraProver create_prover(CircuitConstructor& circuit_constructor);
    UltraVerifier create_verifier(const CircuitConstructor& circuit_constructor);

    void add_table_column_selector_poly_to_proving_key(polynomial& small, const std::string& tag);
};

} // namespace proof_system::honk

#pragma once

#include "barretenberg/flavor/plonk_flavors.hpp"
#include "barretenberg/plonk/composer/composer_lib.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/verifier/verifier.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"

#include <cstddef>
#include <utility>

namespace bb::plonk {
class UltraComposer {
  public:
    using Flavor = flavor::Ultra;
    using CircuitBuilder = UltraCircuitBuilder;
    using Curve = Flavor::Curve;

    static constexpr std::string_view NAME_STRING = "UltraPlonk";
    static constexpr CircuitType type = CircuitType::ULTRA;
    static constexpr size_t NUM_RESERVED_GATES = 4; // equal to the number of multilinear evaluations leaked
    static constexpr size_t program_width = CircuitBuilder::NUM_WIRES;
    std::shared_ptr<plonk::proving_key> circuit_proving_key;
    std::shared_ptr<plonk::verification_key> circuit_verification_key;

    bool computed_witness = false;

    // This variable controls the amount with which the lookup table and witness values need to be shifted
    // above to make room for adding randomness into the permutation and witness polynomials in the plookup widget.
    // This must be (num_roots_cut_out_of_the_vanishing_polynomial - 1), since the variable num_roots_cut_out_of_
    // vanishing_polynomial cannot be trivially fetched here, I am directly setting this to 4 - 1 = 3.
    static constexpr size_t s_randomness = 3;

    UltraComposer() = default;

    UltraComposer(std::shared_ptr<proving_key> p_key, std::shared_ptr<verification_key> v_key)
        : circuit_proving_key(std::move(p_key))
        , circuit_verification_key(std::move(v_key))
    {}

    UltraComposer(UltraComposer&& other) noexcept = default;
    UltraComposer(UltraComposer const& other) noexcept = default;
    UltraComposer& operator=(UltraComposer&& other) noexcept = default;
    UltraComposer& operator=(UltraComposer const& other) noexcept = default;
    ~UltraComposer() = default;

    std::vector<SelectorProperties> ultra_selector_properties()
    {
        // When reading and writing the proving key from a buffer we must precompute the Lagrange form of certain
        // selector polynomials. In order to avoid a new selector type and definitions in the polynomial manifest, we
        // can instead store the Lagrange forms of all the selector polynomials.
        //
        // This workaround increases the memory footprint of the prover, and is a possible place of improvement in the
        // future. Below is the previous state showing where the Lagrange form is necessary for a selector:
        //     { "q_m", true },         { "q_c", true },    { "q_1", true },        { "q_2", true },
        //     { "q_3", true },         { "q_4", false },   { "q_arith", false },   { "q_sort", false },
        //     { "q_elliptic", false }, { "q_aux", false }, { "table_type", true },
        std::vector<SelectorProperties> result{
            { "q_m", true },        { "q_c", true },   { "q_1", true },        { "q_2", true },
            { "q_3", true },        { "q_4", true },   { "q_arith", true },    { "q_sort", true },
            { "q_elliptic", true }, { "q_aux", true }, { "table_type", true },
        };
        return result;
    }

    [[nodiscard]] size_t get_num_selectors() { return ultra_selector_properties().size(); }

    std::shared_ptr<plonk::proving_key> compute_proving_key(CircuitBuilder& circuit_constructor);
    std::shared_ptr<plonk::verification_key> compute_verification_key(CircuitBuilder& circuit_constructor);

    void compute_witness(CircuitBuilder& circuit_constructor);

    UltraProver create_prover(CircuitBuilder& circuit_constructor);
    UltraVerifier create_verifier(CircuitBuilder& circuit_constructor);

    UltraToStandardProver create_ultra_to_standard_prover(CircuitBuilder& circuit_constructor);
    UltraToStandardVerifier create_ultra_to_standard_verifier(CircuitBuilder& circuit_constructor);

    UltraWithKeccakProver create_ultra_with_keccak_prover(CircuitBuilder& circuit_constructor);
    UltraWithKeccakVerifier create_ultra_with_keccak_verifier(CircuitBuilder& circuit_constructor);

    void add_table_column_selector_poly_to_proving_key(polynomial& small, const std::string& tag);

    /**
     * @brief Create a manifest object
     *
     * @note UltraPlonk manifest does not use linearisation trick
     * @param num_public_inputs
     * @return transcript::Manifest
     */
    static transcript::Manifest create_manifest(const size_t num_public_inputs)
    {
        return Flavor::create_manifest(num_public_inputs);
    }
};

} // namespace bb::plonk

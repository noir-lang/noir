#pragma once

#include "barretenberg/plonk/flavor/flavor.hpp"
#include "barretenberg/proof_system/composer/composer_helper_lib.hpp"
#include "barretenberg/plonk/composer/splitting_tmp/composer_helper/composer_helper_lib.hpp"
#include "barretenberg/srs/reference_string/file_reference_string.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/plonk/proof_system/verifier/verifier.hpp"
#include "barretenberg/proof_system/circuit_constructors/ultra_circuit_constructor.hpp"

#include <cstddef>
#include <utility>

namespace proof_system::plonk {
class UltraPlonkComposerHelper {
  public:
    using Flavor = flavor::Ultra;
    using CircuitConstructor = UltraCircuitConstructor;

    // TODO(luke): In the split composers, NUM_RANDOMIZED_GATES has replaced NUM_RESERVED_GATES (in some places) to
    // determine the next-power-of-2 circuit size. (There are some places in this composer that still use
    // NUM_RESERVED_GATES). Therefore for consistency within this composer itself, and consistency with the original
    // Ultra Composer, this value must match that of NUM_RESERVED_GATES. This issue needs to be reconciled
    // simultaneously here and in the other split composers.
    static constexpr size_t NUM_RESERVED_GATES = 4; // equal to the number of multilinear evaluations leaked
    static constexpr size_t program_width = CircuitConstructor::NUM_WIRES;
    std::shared_ptr<plonk::proving_key> circuit_proving_key;
    std::shared_ptr<plonk::verification_key> circuit_verification_key;
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

    explicit UltraPlonkComposerHelper(std::shared_ptr<ReferenceStringFactory> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}

    UltraPlonkComposerHelper(std::shared_ptr<proving_key> p_key, std::shared_ptr<verification_key> v_key)
        : circuit_proving_key(std::move(p_key))
        , circuit_verification_key(std::move(v_key))
    {}

    UltraPlonkComposerHelper(UltraPlonkComposerHelper&& other) noexcept = default;
    UltraPlonkComposerHelper(UltraPlonkComposerHelper const& other) noexcept = default;
    UltraPlonkComposerHelper& operator=(UltraPlonkComposerHelper&& other) noexcept = default;
    UltraPlonkComposerHelper& operator=(UltraPlonkComposerHelper const& other) noexcept = default;
    ~UltraPlonkComposerHelper() = default;

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

    void finalize_circuit(CircuitConstructor& circuit_constructor) { circuit_constructor.finalize_circuit(); };

    std::shared_ptr<plonk::proving_key> compute_proving_key(const CircuitConstructor& circuit_constructor);
    std::shared_ptr<plonk::verification_key> compute_verification_key(const CircuitConstructor& circuit_constructor);

    void compute_witness(CircuitConstructor& circuit_constructor);

    UltraProver create_prover(CircuitConstructor& circuit_constructor);
    UltraVerifier create_verifier(const CircuitConstructor& circuit_constructor);

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
        // add public inputs....
        constexpr size_t g1_size = 64;
        constexpr size_t fr_size = 32;
        const size_t public_input_size = fr_size * num_public_inputs;
        transcript::Manifest output = transcript::Manifest(

            { transcript::Manifest::RoundManifest(
                  { // { name, num_bytes, derived_by_verifier }
                    { "circuit_size", 4, true },
                    { "public_input_size", 4, true } },
                  "init", // challenge_name
                  1       // num_challenges_in
                  ),

              transcript::Manifest::RoundManifest(
                  { // { name, num_bytes, derived_by_verifier }
                    { "public_inputs", public_input_size, false },
                    { "W_1", g1_size, false },
                    { "W_2", g1_size, false },
                    { "W_3", g1_size, false } },
                  "eta", // challenge_name
                  1      // num_challenges_in
                  ),

              transcript::Manifest::RoundManifest(
                  { // { name, num_bytes, derived_by_verifier }
                    { "W_4", g1_size, false },
                    { "S", g1_size, false } },
                  "beta", // challenge_name
                  2       // num_challenges_in
                  ),

              transcript::Manifest::RoundManifest(
                  { // { name, num_bytes, derived_by_verifier }
                    { "Z_PERM", g1_size, false },
                    { "Z_LOOKUP", g1_size, false } },
                  "alpha", // challenge_name
                  1        // num_challenges_in
                  ),

              transcript::Manifest::RoundManifest(
                  { // { name, num_bytes, derived_by_verifier }
                    { "T_1", g1_size, false },
                    { "T_2", g1_size, false },
                    { "T_3", g1_size, false },
                    { "T_4", g1_size, false } },
                  "z", // challenge_name
                  1    // num_challenges_in
                  ),

              // N.B. THE SHFITED EVALS (_omega) MUST HAVE THE SAME CHALLENGE INDEX AS THE NON SHIFTED VALUES
              transcript::Manifest::RoundManifest(
                  {
                      // { name, num_bytes, derived_by_verifier, challenge_map_index }
                      { "t", fr_size, true, -1 }, // *
                      { "w_1", fr_size, false, 0 },
                      { "w_2", fr_size, false, 1 },
                      { "w_3", fr_size, false, 2 },
                      { "w_4", fr_size, false, 3 },
                      { "s", fr_size, false, 4 },
                      { "z_perm", fr_size, false, 5 }, // *
                      { "z_lookup", fr_size, false, 6 },
                      { "q_1", fr_size, false, 7 },
                      { "q_2", fr_size, false, 8 },
                      { "q_3", fr_size, false, 9 },
                      { "q_4", fr_size, false, 10 },
                      { "q_m", fr_size, false, 11 },
                      { "q_c", fr_size, false, 12 },
                      { "q_arith", fr_size, false, 13 },
                      { "q_sort", fr_size, false, 14 },     // *
                      { "q_elliptic", fr_size, false, 15 }, // *
                      { "q_aux", fr_size, false, 16 },
                      { "sigma_1", fr_size, false, 17 },
                      { "sigma_2", fr_size, false, 18 },
                      { "sigma_3", fr_size, false, 19 },
                      { "sigma_4", fr_size, false, 20 },
                      { "table_value_1", fr_size, false, 21 },
                      { "table_value_2", fr_size, false, 22 },
                      { "table_value_3", fr_size, false, 23 },
                      { "table_value_4", fr_size, false, 24 },
                      { "table_type", fr_size, false, 25 },
                      { "id_1", fr_size, false, 26 },
                      { "id_2", fr_size, false, 27 },
                      { "id_3", fr_size, false, 28 },
                      { "id_4", fr_size, false, 29 },
                      { "w_1_omega", fr_size, false, 0 },
                      { "w_2_omega", fr_size, false, 1 },
                      { "w_3_omega", fr_size, false, 2 },
                      { "w_4_omega", fr_size, false, 3 },
                      { "s_omega", fr_size, false, 4 },
                      { "z_perm_omega", fr_size, false, 5 },
                      { "z_lookup_omega", fr_size, false, 6 },
                      { "table_value_1_omega", fr_size, false, 21 },
                      { "table_value_2_omega", fr_size, false, 22 },
                      { "table_value_3_omega", fr_size, false, 23 },
                      { "table_value_4_omega", fr_size, false, 24 },
                  },
                  "nu",                // challenge_name
                  ULTRA_MANIFEST_SIZE, // num_challenges_in
                  true                 // map_challenges_in
                  ),

              transcript::Manifest::RoundManifest(
                  { // { name, num_bytes, derived_by_verifier, challenge_map_index }
                    { "PI_Z", g1_size, false },
                    { "PI_Z_OMEGA", g1_size, false } },
                  "separator", // challenge_name
                  3            // num_challenges_in
                  ) });

        return output;
    }
};

} // namespace proof_system::plonk

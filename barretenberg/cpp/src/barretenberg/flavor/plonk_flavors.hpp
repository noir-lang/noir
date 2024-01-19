#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/transcript/transcript.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"

namespace bb::plonk::flavor {
class Standard {
  public:
    using CircuitBuilder = bb::StandardCircuitBuilder;
    using ProvingKey = plonk::proving_key;
    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Polynomial = bb::Polynomial<FF>;
    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;
    // Whether or not the first row of the execution trace is reserved for 0s to enable shifts
    static constexpr bool has_zero_row = false;
};

class Ultra {
  public:
    using CircuitBuilder = bb::UltraCircuitBuilder;
    using ProvingKey = plonk::proving_key;
    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Polynomial = bb::Polynomial<FF>;
    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;
    // Whether or not the first row of the execution trace is reserved for 0s to enable shifts
    static constexpr bool has_zero_row = false;

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
} // namespace bb::plonk::flavor
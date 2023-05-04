#pragma once

#include "barretenberg/plonk/flavor/flavor.hpp"
#include "barretenberg/plonk/composer/splitting_tmp/composer_helper/composer_helper_lib.hpp"
#include "barretenberg/proof_system/composer/composer_helper_lib.hpp"
#include "barretenberg/srs/reference_string/file_reference_string.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/plonk/proof_system/verifier/verifier.hpp"

namespace proof_system::plonk {
class TurboPlonkComposerHelper {
  public:
    using Flavor = plonk::flavor::Turbo;
    using CircuitConstructor = TurboCircuitConstructor;

    static constexpr size_t NUM_RANDOMIZED_GATES = 2; // equal to the number of multilinear evaluations leaked
    static constexpr size_t program_width = CircuitConstructor::program_width;

    std::shared_ptr<plonk::proving_key> circuit_proving_key;
    std::shared_ptr<plonk::verification_key> circuit_verification_key;

    // TODO(#218)(kesha): we need to put this into the commitment key, so that the composer doesn't have to handle srs
    // at all
    std::shared_ptr<ReferenceStringFactory> crs_factory_;

    std::vector<uint32_t> recursive_proof_public_input_indices;
    bool contains_recursive_proof = false;

    bool computed_witness = false;
    TurboPlonkComposerHelper()
        : TurboPlonkComposerHelper(std::shared_ptr<ReferenceStringFactory>(
              new proof_system::FileReferenceStringFactory("../srs_db/ignition")))
    {}

    TurboPlonkComposerHelper(std::shared_ptr<ReferenceStringFactory> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}
    TurboPlonkComposerHelper(std::unique_ptr<ReferenceStringFactory>&& crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}
    TurboPlonkComposerHelper(std::shared_ptr<plonk::proving_key> p_key, std::shared_ptr<plonk::verification_key> v_key)
        : circuit_proving_key(std::move(p_key))
        , circuit_verification_key(std::move(v_key))
    {}
    TurboPlonkComposerHelper(TurboPlonkComposerHelper&& other) = default;
    TurboPlonkComposerHelper& operator=(TurboPlonkComposerHelper&& other) noexcept = default;
    ~TurboPlonkComposerHelper() {}

    std::shared_ptr<proving_key> compute_proving_key(const CircuitConstructor& circuit_constructor);
    std::shared_ptr<verification_key> compute_verification_key(const CircuitConstructor& circuit_constructor);
    void compute_witness(const CircuitConstructor& circuit_constructor, const size_t minimum_circuit_size = 0);

    TurboProver create_prover(const CircuitConstructor& circuit_constructor);
    TurboVerifier create_verifier(const CircuitConstructor& circuit_constructor);
    inline std::vector<SelectorProperties> turbo_selector_properties()
    {
        const std::vector<SelectorProperties> result{
            { "q_m", false },          { "q_c", false },     { "q_1", false },     { "q_2", false },
            { "q_3", false },          { "q_4", false },     { "q_5", false },     { "q_arith", false },
            { "q_fixed_base", false }, { "q_range", false }, { "q_logic", false },
        };
        return result;
    }
    void add_recursive_proof(CircuitConstructor& circuit_constructor,
                             const std::vector<uint32_t>& proof_output_witness_indices)
    {

        if (contains_recursive_proof) {
            circuit_constructor.failure("added recursive proof when one already exists");
        }
        contains_recursive_proof = true;

        for (const auto& idx : proof_output_witness_indices) {
            circuit_constructor.set_public_input(idx);
            recursive_proof_public_input_indices.push_back((uint32_t)(circuit_constructor.public_inputs.size() - 1));
        }
    }
    static transcript::Manifest create_manifest(const size_t num_public_inputs)
    {
        // add public inputs....
        constexpr size_t g1_size = 64;
        constexpr size_t fr_size = 32;
        const size_t public_input_size = fr_size * num_public_inputs;
        const transcript::Manifest output = transcript::Manifest(
            { transcript::Manifest::RoundManifest(
                  { { "circuit_size", 4, true }, { "public_input_size", 4, true } }, "init", 1),

              transcript::Manifest::RoundManifest({}, "eta", 0),

              transcript::Manifest::RoundManifest(
                  {
                      { "public_inputs", public_input_size, false },
                      { "W_1", g1_size, false },
                      { "W_2", g1_size, false },
                      { "W_3", g1_size, false },
                      { "W_4", g1_size, false },
                  },
                  "beta",
                  2),
              transcript::Manifest::RoundManifest({ { "Z_PERM", g1_size, false } }, "alpha", 1),
              transcript::Manifest::RoundManifest(
                  {
                      { "T_1", g1_size, false },
                      { "T_2", g1_size, false },
                      { "T_3", g1_size, false },
                      { "T_4", g1_size, false },
                  },
                  "z",
                  1),

              transcript::Manifest::RoundManifest(
                  {
                      { "t", fr_size, true, -1 },         { "w_1", fr_size, false, 0 },
                      { "w_2", fr_size, false, 1 },       { "w_3", fr_size, false, 2 },
                      { "w_4", fr_size, false, 3 },       { "sigma_1", fr_size, false, 4 },
                      { "sigma_2", fr_size, false, 5 },   { "sigma_3", fr_size, false, 6 },
                      { "sigma_4", fr_size, false, 7 },   { "q_1", fr_size, false, 8 },
                      { "q_2", fr_size, false, 9 },       { "q_3", fr_size, false, 10 },
                      { "q_4", fr_size, false, 11 },      { "q_5", fr_size, false, 12 },
                      { "q_m", fr_size, false, 13 },      { "q_c", fr_size, false, 14 },
                      { "q_arith", fr_size, false, 15 },  { "q_logic", fr_size, false, 16 },
                      { "q_range", fr_size, false, 17 },  { "q_fixed_base", fr_size, false, 18 },
                      { "z_perm", fr_size, false, 19 },   { "z_perm_omega", fr_size, false, 19 },
                      { "w_1_omega", fr_size, false, 0 }, { "w_2_omega", fr_size, false, 1 },
                      { "w_3_omega", fr_size, false, 2 }, { "w_4_omega", fr_size, false, 3 },
                  },
                  "nu",
                  TURBO_MANIFEST_SIZE,
                  true),

              transcript::Manifest::RoundManifest(
                  { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 3) });

        return output;
    }
};
} // namespace proof_system::plonk

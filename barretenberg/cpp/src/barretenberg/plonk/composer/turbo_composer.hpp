#pragma once

#include "barretenberg/plonk/composer/composer_lib.hpp"
#include "barretenberg/plonk/flavor/flavor.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/verifier/verifier.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"

namespace proof_system::plonk {
class TurboComposer {
  public:
    using Flavor = plonk::flavor::Turbo;
    using CircuitBuilder = TurboCircuitBuilder;

    static constexpr std::string_view NAME_STRING = "TurboPlonk";
    static constexpr size_t NUM_RESERVED_GATES = 4; // equal to the number of evaluations leaked
    static constexpr size_t program_width = CircuitBuilder::program_width;

    std::shared_ptr<plonk::proving_key> circuit_proving_key;
    std::shared_ptr<plonk::verification_key> circuit_verification_key;

    // The crs_factory holds the path to the srs and exposes methods to extract the srs elements
    std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> crs_factory_;

    bool computed_witness = false;
    TurboComposer()
        : TurboComposer(std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>>(
              new barretenberg::srs::factories::FileCrsFactory<curve::BN254>("../srs_db/ignition")))
    {}

    TurboComposer(std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}
    TurboComposer(std::unique_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>>&& crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}
    TurboComposer(std::shared_ptr<plonk::proving_key> p_key, std::shared_ptr<plonk::verification_key> v_key)
        : circuit_proving_key(std::move(p_key))
        , circuit_verification_key(std::move(v_key))
    {}
    TurboComposer(TurboComposer&& other) = default;
    TurboComposer& operator=(TurboComposer&& other) noexcept = default;
    ~TurboComposer() {}

    std::shared_ptr<proving_key> compute_proving_key(const CircuitBuilder& circuit_constructor);
    std::shared_ptr<verification_key> compute_verification_key(const CircuitBuilder& circuit_constructor);
    void compute_witness(const CircuitBuilder& circuit_constructor, const size_t minimum_circuit_size = 0);

    TurboProver create_prover(const CircuitBuilder& circuit_constructor);
    TurboVerifier create_verifier(const CircuitBuilder& circuit_constructor);
    inline std::vector<SelectorProperties> turbo_selector_properties()
    {
        const std::vector<SelectorProperties> result{
            { "q_m", false },          { "q_c", false },     { "q_1", false },     { "q_2", false },
            { "q_3", false },          { "q_4", false },     { "q_5", false },     { "q_arith", false },
            { "q_fixed_base", false }, { "q_range", false }, { "q_logic", false },
        };
        return result;
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

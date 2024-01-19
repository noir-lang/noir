#pragma once

#include "barretenberg/flavor/plonk_flavors.hpp"
#include "barretenberg/plonk/composer/composer_lib.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/plonk/proof_system/verifier/verifier.hpp"
#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"
#include <utility>

namespace bb::plonk {
class StandardComposer {
  public:
    using Flavor = plonk::flavor::Standard;

    using CircuitBuilder = StandardCircuitBuilder;

    static constexpr std::string_view NAME_STRING = "StandardPlonk";
    static constexpr size_t NUM_RESERVED_GATES = 4; // equal to the number of evaluations leaked
    static constexpr size_t program_width = CircuitBuilder::program_width;
    std::shared_ptr<plonk::proving_key> circuit_proving_key;
    std::shared_ptr<plonk::verification_key> circuit_verification_key;

    // The crs_factory holds the path to the srs and exposes methods to extract the srs elements
    std::shared_ptr<bb::srs::factories::CrsFactory<curve::BN254>> crs_factory_;

    bool computed_witness = false;

    StandardComposer() { crs_factory_ = bb::srs::get_crs_factory(); }
    StandardComposer(std::shared_ptr<bb::srs::factories::CrsFactory<curve::BN254>> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}

    StandardComposer(std::unique_ptr<bb::srs::factories::CrsFactory<curve::BN254>>&& crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}
    StandardComposer(std::shared_ptr<plonk::proving_key> p_key, std::shared_ptr<plonk::verification_key> v_key)
        : circuit_proving_key(std::move(p_key))
        , circuit_verification_key(std::move(v_key))
    {}

    StandardComposer(StandardComposer&& other) noexcept = default;
    StandardComposer(const StandardComposer& other) = delete;
    StandardComposer& operator=(StandardComposer&& other) noexcept = default;
    StandardComposer& operator=(const StandardComposer& other) = delete;
    ~StandardComposer() = default;

    inline std::vector<SelectorProperties> standard_selector_properties()
    {
        std::vector<SelectorProperties> result{
            { "q_m", false }, { "q_c", false }, { "q_1", false }, { "q_2", false }, { "q_3", false },
        };
        return result;
    }
    std::shared_ptr<plonk::proving_key> compute_proving_key(const CircuitBuilder& circuit_constructor);
    std::shared_ptr<plonk::verification_key> compute_verification_key(const CircuitBuilder& circuit_constructor);

    plonk::Verifier create_verifier(const CircuitBuilder& circuit_constructor);
    plonk::Prover create_prover(const CircuitBuilder& circuit_constructor);

    void compute_witness(const CircuitBuilder& circuit_constructor, const size_t minimum_circuit_size = 0);
    /**
     * Create a manifest, which specifies proof rounds, elements and who supplies them.
     *
     * @param num_public_inputs The number of public inputs.
     *
     * @return Constructed manifest.
     * */
    static transcript::Manifest create_manifest(const size_t num_public_inputs)
    {
        constexpr size_t g1_size = 64;
        constexpr size_t fr_size = 32;
        const size_t public_input_size = fr_size * num_public_inputs;
        /*  A RoundManifest describes data that will be put in or extracted from a transcript.
            Here we have 7 RoundManifests. */
        const transcript::Manifest output = transcript::Manifest(
            { // clang-format off

              // Round 0
              transcript::Manifest::RoundManifest(
                {
                  { .name = "circuit_size",      .num_bytes = 4, .derived_by_verifier = true },
                  { .name = "public_input_size", .num_bytes = 4, .derived_by_verifier = true }
                },
                /* challenge_name = */ "init",
                /* num_challenges_in = */ 1),

              // Round 1
              transcript::Manifest::RoundManifest(
                {},
                /* challenge_name = */ "eta",
                /* num_challenges_in = */ 0),

              // Round 2
              transcript::Manifest::RoundManifest(
                {
                    { .name = "public_inputs", .num_bytes = public_input_size, .derived_by_verifier = false },
                    { .name = "W_1",           .num_bytes = g1_size,           .derived_by_verifier = false },
                    { .name = "W_2",           .num_bytes = g1_size,           .derived_by_verifier = false },
                    { .name = "W_3",           .num_bytes = g1_size,           .derived_by_verifier = false },
                },
                /* challenge_name = */ "beta",
                /* num_challenges_in = */ 2),

              // Round 3
              transcript::Manifest::RoundManifest(
                { { .name = "Z_PERM", .num_bytes = g1_size, .derived_by_verifier = false } },
                /* challenge_name = */ "alpha",
                /* num_challenges_in = */ 1),

              // Round 4
              transcript::Manifest::RoundManifest(
                { { .name = "T_1", .num_bytes = g1_size, .derived_by_verifier = false },
                  { .name = "T_2", .num_bytes = g1_size, .derived_by_verifier = false },
                  { .name = "T_3", .num_bytes = g1_size, .derived_by_verifier = false } },
                /* challenge_name = */ "z",
                /* num_challenges_in = */ 1),

              // Round 5
              transcript::Manifest::RoundManifest(
                {
                    { .name = "t",            .num_bytes = fr_size, .derived_by_verifier = true,  .challenge_map_index = -1 },
                    { .name = "w_1",          .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 0 },
                    { .name = "w_2",          .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 1 },
                    { .name = "w_3",          .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 2 },
                    { .name = "sigma_1",      .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 3 },
                    { .name = "sigma_2",      .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 4 },
                    { .name = "sigma_3",      .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 5 },
                    { .name = "q_1",          .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 6 },
                    { .name = "q_2",          .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 7 },
                    { .name = "q_3",          .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 8 },
                    { .name = "q_m",          .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 9 },
                    { .name = "q_c",          .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 10 },
                    { .name = "z_perm",       .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = 11 },
                    { .name = "z_perm_omega", .num_bytes = fr_size, .derived_by_verifier = false, .challenge_map_index = -1 },
                },
                /* challenge_name = */ "nu",
                /* num_challenges_in = */ STANDARD_MANIFEST_SIZE,
                /* map_challenges_in = */ true),

              // Round 6
              transcript::Manifest::RoundManifest(
                { { .name = "PI_Z",       .num_bytes = g1_size, .derived_by_verifier = false },
                  { .name = "PI_Z_OMEGA", .num_bytes = g1_size, .derived_by_verifier = false } },
                /* challenge_name = */ "separator",
                /* num_challenges_in = */ 1) }

            // clang-format off
    );
        return output;
    }
};

} // namespace bb::plonk

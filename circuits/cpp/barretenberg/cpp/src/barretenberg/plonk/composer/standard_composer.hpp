#pragma once
#include "composer_base.hpp"
#include "barretenberg/proof_system/types/merkle_hash_type.hpp"
#include "barretenberg/proof_system/types/pedersen_commitment_type.hpp"
#include "barretenberg/transcript/manifest.hpp"
#include "barretenberg/srs/reference_string/file_reference_string.hpp"

namespace proof_system::plonk {
enum StandardSelectors { QM, QC, Q1, Q2, Q3, NUM };

inline std::vector<ComposerBase::SelectorProperties> standard_selector_properties()
{
    std::vector<ComposerBase::SelectorProperties> result{
        { "q_m", false }, { "q_c", false }, { "q_1", false }, { "q_2", false }, { "q_3", false },
    };
    return result;
}

class StandardComposer : public ComposerBase {
  public:
    static constexpr ComposerType type = ComposerType::STANDARD;
    static constexpr merkle::HashType merkle_hash_type = merkle::HashType::FIXED_BASE_PEDERSEN;
    static constexpr pedersen::CommitmentType commitment_type = pedersen::CommitmentType::FIXED_BASE_PEDERSEN;
    static constexpr size_t UINT_LOG2_BASE = 2;

    StandardComposer(const size_t size_hint = 0)
        : ComposerBase(StandardSelectors::NUM, size_hint, standard_selector_properties())
    {
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        zero_idx = put_constant_variable(barretenberg::fr::zero());
    };

    StandardComposer(const size_t num_selectors,
                     const size_t size_hint,
                     const std::vector<SelectorProperties> selector_properties)
        : ComposerBase(num_selectors, size_hint, selector_properties)
    {
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        zero_idx = put_constant_variable(barretenberg::fr::zero());
    };

    StandardComposer(std::string const& crs_path, const size_t size_hint = 0)
        : StandardComposer(std::unique_ptr<ReferenceStringFactory>(new FileReferenceStringFactory(crs_path)),
                           size_hint){};

    StandardComposer(std::shared_ptr<ReferenceStringFactory> const& crs_factory, const size_t size_hint = 0)
        : ComposerBase(crs_factory, StandardSelectors::NUM, size_hint, standard_selector_properties())
    {
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);

        zero_idx = put_constant_variable(fr::zero());
    }

    StandardComposer(std::unique_ptr<ReferenceStringFactory>&& crs_factory, const size_t size_hint = 0)
        : ComposerBase(std::move(crs_factory), StandardSelectors::NUM, size_hint, standard_selector_properties())
    {
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        zero_idx = put_constant_variable(barretenberg::fr::zero());
    }

    StandardComposer(std::shared_ptr<proving_key> const& p_key,
                     std::shared_ptr<verification_key> const& v_key,
                     size_t size_hint = 0)
        : ComposerBase(p_key, v_key, StandardSelectors::NUM, size_hint, standard_selector_properties())
    {
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        zero_idx = put_constant_variable(barretenberg::fr::zero());
    }

    StandardComposer(StandardComposer&& other) = default;
    StandardComposer& operator=(StandardComposer&& other) = default;
    ~StandardComposer() {}

    virtual size_t get_total_circuit_size() const override { return num_gates; };

    void assert_equal_constant(uint32_t const a_idx,
                               barretenberg::fr const& b,
                               std::string const& msg = "assert equal constant");

    virtual std::shared_ptr<proving_key> compute_proving_key() override;
    virtual std::shared_ptr<verification_key> compute_verification_key() override;
    virtual void compute_witness() override;
    Verifier create_verifier();
    Prover create_prover();

    void create_add_gate(const add_triple& in) override;
    void create_mul_gate(const mul_triple& in) override;
    void create_bool_gate(const uint32_t a) override;
    void create_poly_gate(const poly_triple& in) override;
    void create_big_add_gate(const add_quad& in);
    void create_big_add_gate_with_bit_extraction(const add_quad& in);
    void create_big_mul_gate(const mul_quad& in);
    void create_balanced_add_gate(const add_quad& in);

    void fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value);

    std::vector<uint32_t> decompose_into_base4_accumulators(const uint32_t witness_index,
                                                            const size_t num_bits,
                                                            std::string const& msg = "create_range_constraint");

    void create_range_constraint(const uint32_t variable_index,
                                 const size_t num_bits,
                                 std::string const& msg = "create_range_constraint")
    {
        decompose_into_base4_accumulators(variable_index, num_bits, msg);
    }

    void add_recursive_proof(const std::vector<uint32_t>& proof_output_witness_indices)
    {
        if (contains_recursive_proof) {
            failure("added recursive proof when one already exists");
        }
        contains_recursive_proof = true;

        for (const auto& idx : proof_output_witness_indices) {
            set_public_input(idx);
            recursive_proof_public_input_indices.push_back((uint32_t)(public_inputs.size() - 1));
        }
    }

    std::vector<uint32_t> recursive_proof_public_input_indices;
    bool contains_recursive_proof = false;

    accumulator_triple create_logic_constraint(const uint32_t a,
                                               const uint32_t b,
                                               const size_t num_bits,
                                               bool is_xor_gate);
    accumulator_triple create_and_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);
    accumulator_triple create_xor_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);

    uint32_t put_constant_variable(const barretenberg::fr& variable);

    size_t get_num_constant_gates() const override { return 0; }

    // These are variables that we have used a gate on, to enforce that they are
    // equal to a defined value.
    std::map<barretenberg::fr, uint32_t> constant_variable_indices;

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

    bool check_circuit();
};
} // namespace proof_system::plonk

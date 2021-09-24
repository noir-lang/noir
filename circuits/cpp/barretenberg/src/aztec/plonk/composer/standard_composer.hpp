#pragma once
#include "composer_base.hpp"
#include <plonk/reference_string/file_reference_string.hpp>
#include <plonk/transcript/manifest.hpp>

namespace waffle {
enum StandardSelectors {
    QM = 0,
    QC = 1,
    Q1 = 2,
    Q2 = 3,
    Q3 = 4,
};

inline std::vector<ComposerBase::SelectorProperties> standard_sel_props()
{
    // We set the use_quotient_mid variable to false in composer settings so as to
    // disallow fft computations of size 2n as the degrees of polynomials slighly change
    // on introducing the new vanishing polynomial with some roots cut out.
    std::vector<ComposerBase::SelectorProperties> result{
        { "q_m", false, false }, { "q_c", false, false }, { "q_1", false, false },
        { "q_2", false, false }, { "q_3", false, false },
    };
    return result;
}

class StandardComposer : public ComposerBase {
  public:
    static constexpr ComposerType type = ComposerType::STANDARD;
    static constexpr size_t UINT_LOG2_BASE = 2;

    StandardComposer(const size_t size_hint = 0)
        : ComposerBase(5, size_hint, standard_sel_props())
    {
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        zero_idx = put_constant_variable(barretenberg::fr::zero());
    };

    StandardComposer(const size_t selector_num,
                     const size_t size_hint,
                     const std::vector<SelectorProperties> selector_properties)
        : ComposerBase(selector_num, size_hint, selector_properties)
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
        : ComposerBase(crs_factory, 5, size_hint, standard_sel_props())
    {
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);

        zero_idx = put_constant_variable(fr::zero());
    }

    StandardComposer(std::unique_ptr<ReferenceStringFactory>&& crs_factory, const size_t size_hint = 0)
        : ComposerBase(std::move(crs_factory), 5, size_hint, standard_sel_props())
    {
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        zero_idx = put_constant_variable(barretenberg::fr::zero());
    }

    StandardComposer(std::shared_ptr<proving_key> const& p_key,
                     std::shared_ptr<verification_key> const& v_key,
                     size_t size_hint = 0)
        : ComposerBase(p_key, v_key, 5, size_hint, standard_sel_props())
    {
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        zero_idx = put_constant_variable(barretenberg::fr::zero());
    }

    StandardComposer(StandardComposer&& other) = default;
    StandardComposer& operator=(StandardComposer&& other) = default;
    ~StandardComposer() {}

    void assert_equal_constant(uint32_t const a_idx,
                               barretenberg::fr const& b,
                               std::string const& msg = "assert equal constant");

    virtual std::shared_ptr<proving_key> compute_proving_key() override;
    virtual std::shared_ptr<verification_key> compute_verification_key() override;
    virtual std::shared_ptr<program_witness> compute_witness() override;
    Verifier create_verifier();
    Prover preprocess() { return create_prover(); };
    Prover create_prover();
    UnrolledVerifier create_unrolled_verifier();
    UnrolledProver create_unrolled_prover();

    void create_add_gate(const add_triple& in) override;
    void create_mul_gate(const mul_triple& in) override;
    void create_bool_gate(const uint32_t a) override;
    void create_poly_gate(const poly_triple& in) override;
    void create_big_add_gate(const add_quad& in);
    void create_big_add_gate_with_bit_extraction(const add_quad& in);
    void create_big_mul_gate(const mul_quad& in);
    void create_balanced_add_gate(const add_quad& in);
    void create_fixed_group_add_gate(const fixed_group_add_quad& in);
    void create_fixed_group_add_gate_with_init(const fixed_group_add_quad& in, const fixed_group_init_quad& init);
    void create_fixed_group_add_gate_final(const add_quad& in);

    fixed_group_add_quad previous_add_quad;

    void fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value);

    std::vector<uint32_t> decompose_into_base4_accumulators(const uint32_t witness_index,
                                                            const size_t num_bits,
                                                            std::string const& msg = "create_range_constraint");

    std::vector<uint32_t> create_range_constraint(const uint32_t witness_index,
                                                  const size_t num_bits,
                                                  std::string const& msg = "create_range_constraint");
    void add_recursive_proof(const std::vector<uint32_t>& proof_output_witness_indices)
    {
        if (contains_recursive_proof) {
            failed = true;
            err = "added recursive proof when one already exists";
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

    // these are variables that we have used a gate on, to enforce that they are
    // equal to a defined value
    std::map<barretenberg::fr, uint32_t> constant_variables;

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
                  },
                  "beta",
                  2),
              transcript::Manifest::RoundManifest({ { "Z", g1_size, false } }, "alpha", 1),
              transcript::Manifest::RoundManifest(
                  { { "T_1", g1_size, false }, { "T_2", g1_size, false }, { "T_3", g1_size, false } }, "z", 1),
              transcript::Manifest::RoundManifest(
                  {
                      { "t", fr_size, true, -1 },
                      { "w_1", fr_size, false, 0 },
                      { "w_2", fr_size, false, 1 },
                      { "w_3", fr_size, false, 2 },
                      { "sigma_1", fr_size, false, 3 },
                      { "sigma_2", fr_size, false, 4 },
                      { "r", fr_size, false, 5 },
                      { "z_omega", fr_size, false, -1 },
                  },
                  "nu",
                  6,
                  true),
              transcript::Manifest::RoundManifest(
                  { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 1) });
        return output;
    }

    static transcript::Manifest create_unrolled_manifest(const size_t num_public_inputs)
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
                  },
                  "beta",
                  2),
              transcript::Manifest::RoundManifest({ { "Z", g1_size, false } }, "alpha", 1),
              transcript::Manifest::RoundManifest(
                  { { "T_1", g1_size, false }, { "T_2", g1_size, false }, { "T_3", g1_size, false } }, "z", 1),
              transcript::Manifest::RoundManifest(
                  {
                      { "t", fr_size, true, -1 },
                      { "w_1", fr_size, false, 0 },
                      { "w_2", fr_size, false, 1 },
                      { "w_3", fr_size, false, 2 },
                      { "sigma_1", fr_size, false, 3 },
                      { "sigma_2", fr_size, false, 4 },
                      { "sigma_3", fr_size, false, 5 },
                      { "q_1", fr_size, false, 6 },
                      { "q_2", fr_size, false, 7 },
                      { "q_3", fr_size, false, 8 },
                      { "q_m", fr_size, false, 9 },
                      { "q_c", fr_size, false, 10 },
                      { "z", fr_size, false, 11 },
                      { "z_omega", fr_size, false, -1 },
                  },
                  "nu",
                  12,
                  true),
              transcript::Manifest::RoundManifest(
                  { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 1) });
        return output;
    }
};
} // namespace waffle

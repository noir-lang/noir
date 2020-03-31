#pragma once
#include "composer_base.hpp"
#include <plonk/reference_string/file_reference_string.hpp>
#include <plonk/transcript/manifest.hpp>

namespace waffle {
class StandardComposer : public ComposerBase {
  public:
    StandardComposer(const size_t size_hint = 0)
    {
        features |= static_cast<size_t>(Features::BASIC_ARITHMETISATION);
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        q_m.reserve(size_hint);
        q_1.reserve(size_hint);
        q_2.reserve(size_hint);
        q_3.reserve(size_hint);
        q_c.reserve(size_hint);
        zero_idx = put_constant_variable(barretenberg::fr::zero());
    };

    StandardComposer(std::string const& crs_path, const size_t size_hint = 0)
        : StandardComposer(std::unique_ptr<ReferenceStringFactory>(new FileReferenceStringFactory(crs_path)),
                           size_hint){};

    StandardComposer(std::unique_ptr<ReferenceStringFactory>&& crs_factory, const size_t size_hint = 0)
        : ComposerBase(std::move(crs_factory))
    {
        features |= static_cast<size_t>(Features::BASIC_ARITHMETISATION);
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        q_m.reserve(size_hint);
        q_1.reserve(size_hint);
        q_2.reserve(size_hint);
        q_3.reserve(size_hint);
        q_c.reserve(size_hint);
        zero_idx = put_constant_variable(barretenberg::fr::zero());
    }

    StandardComposer(std::shared_ptr<proving_key> const& p_key,
                     std::shared_ptr<verification_key> const& v_key,
                     size_t size_hint = 0)
        : ComposerBase(p_key, v_key)
    {
        features |= static_cast<size_t>(Features::BASIC_ARITHMETISATION);
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        q_m.reserve(size_hint);
        q_1.reserve(size_hint);
        q_2.reserve(size_hint);
        q_3.reserve(size_hint);
        q_c.reserve(size_hint);
        zero_idx = put_constant_variable(barretenberg::fr::zero());
    }

    StandardComposer(StandardComposer&& other) = default;
    StandardComposer& operator=(StandardComposer&& other) = default;
    ~StandardComposer() {}

    void assert_equal_constant(uint32_t const a_idx, barretenberg::fr const& b);

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
    void fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value);

    std::vector<uint32_t> create_range_constraint(const uint32_t witness_index, const size_t num_bits);
    accumulator_triple create_logic_constraint(const uint32_t a,
                                               const uint32_t b,
                                               const size_t num_bits,
                                               bool is_xor_gate);
    accumulator_triple create_and_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);
    accumulator_triple create_xor_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);

    uint32_t put_constant_variable(const barretenberg::fr& variable);

    void create_dummy_gates();
    size_t get_num_constant_gates() const override { return 0; }

    uint32_t zero_idx = 0;

    // these are variables that we have used a gate on, to enforce that they are equal to a defined value
    std::map<barretenberg::fr, uint32_t> constant_variables;

    std::vector<barretenberg::fr> q_m;
    std::vector<barretenberg::fr> q_1;
    std::vector<barretenberg::fr> q_2;
    std::vector<barretenberg::fr> q_3;
    std::vector<barretenberg::fr> q_c;

    static transcript::Manifest create_manifest(const size_t num_public_inputs)
    {
        // add public inputs....
        constexpr size_t g1_size = 64;
        constexpr size_t fr_size = 32;
        const size_t public_input_size = fr_size * num_public_inputs;
        const transcript::Manifest output = transcript::Manifest(
            { transcript::Manifest::RoundManifest(
                  { { "circuit_size", 4, true }, { "public_input_size", 4, true } }, "init", 1),
              transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
                                                    { "W_1", g1_size, false },
                                                    { "W_2", g1_size, false },
                                                    { "W_3", g1_size, false } },
                                                  "beta",
                                                  2),
              transcript::Manifest::RoundManifest({ { "Z", g1_size, false } }, "alpha", 1),
              transcript::Manifest::RoundManifest(
                  { { "T_1", g1_size, false }, { "T_2", g1_size, false }, { "T_3", g1_size, false } }, "z", 1),
              transcript::Manifest::RoundManifest({ { "w_1", fr_size, false },
                                                    { "w_2", fr_size, false },
                                                    { "w_3", fr_size, false },
                                                    { "w_3_omega", fr_size, false },
                                                    { "z_omega", fr_size, false },
                                                    { "sigma_1", fr_size, false },
                                                    { "sigma_2", fr_size, false },
                                                    { "r", fr_size, false },
                                                    { "t", fr_size, true } },
                                                  "nu",
                                                  10),
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
              transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
                                                    { "W_1", g1_size, false },
                                                    { "W_2", g1_size, false },
                                                    { "W_3", g1_size, false } },
                                                  "beta",
                                                  2),
              transcript::Manifest::RoundManifest({ { "Z", g1_size, false } }, "alpha", 1),
              transcript::Manifest::RoundManifest(
                  { { "T_1", g1_size, false }, { "T_2", g1_size, false }, { "T_3", g1_size, false } }, "z", 1),
              transcript::Manifest::RoundManifest({ { "w_1", fr_size, false },
                                                    { "w_2", fr_size, false },
                                                    { "w_3", fr_size, false },
                                                    { "w_3_omega", fr_size, false },
                                                    { "z", fr_size, false },
                                                    { "z_omega", fr_size, false },
                                                    { "sigma_1", fr_size, false },
                                                    { "sigma_2", fr_size, false },
                                                    { "sigma_3", fr_size, false },
                                                    { "q_1", fr_size, false },
                                                    { "q_2", fr_size, false },
                                                    { "q_3", fr_size, false },
                                                    { "q_m", fr_size, false },
                                                    { "q_c", fr_size, false },
                                                    { "t", fr_size, true } },
                                                  "nu",
                                                  18),
              transcript::Manifest::RoundManifest(
                  { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 1) });
        return output;
    }
};
} // namespace waffle

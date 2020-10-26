#pragma once
#include "standard_composer.hpp"
#include <plonk/transcript/manifest.hpp>
enum MimcSelectors {
    QMIMC_COEFF = 5,
    QMIMC_SELEC = 6,
};

namespace waffle {
inline std::vector<ComposerBase::SelectorProperties> mimc_sel_props()
{
    std::vector<ComposerBase::SelectorProperties> result{

        // We set the use_quotient_mid variable to false in composer settings so as to 
        // disallow fft computations of size 2n as the degrees of polynomials slightly change
        // on introducing the new vanishing polynomial with some roots cut out.
        { "q_m", false, false },
        { "q_c", false, false },
        { "q_1", false, false },
        { "q_2", false, false },
        { "q_3", false, false },
        { "q_mimc_coefficient", false, false },
        { "q_mimc_selector", false, false },
    };
    return result;
}

struct mimc_quadruplet {
    uint32_t x_in_idx;
    uint32_t x_cubed_idx;
    uint32_t k_idx;
    uint32_t x_out_idx;
    barretenberg::fr mimc_constant;
};

class MiMCComposer : public StandardComposer {
  public:
    static constexpr ComposerType type = ComposerType::STANDARD;
    static constexpr size_t UINT_LOG2_BASE = 2;

    MiMCComposer(const size_t size_hint = 0)
        : StandardComposer(7, size_hint, mimc_sel_props())
    {
        auto& q_mimc_coefficient = selectors[MimcSelectors::QMIMC_COEFF];
        auto& q_mimc_selector = selectors[MimcSelectors::QMIMC_SELEC];
        q_mimc_coefficient.push_back(barretenberg::fr::zero());
        q_mimc_selector.push_back(barretenberg::fr::zero());
    };
    MiMCComposer(MiMCComposer&& other) = default;
    MiMCComposer& operator=(MiMCComposer&& other) = default;

    ~MiMCComposer() {}

    std::shared_ptr<proving_key> compute_proving_key() override;
    std::shared_ptr<verification_key> compute_verification_key() override;
    std::shared_ptr<program_witness> compute_witness() override;
    MiMCVerifier create_verifier();
    Prover preprocess();

    void create_add_gate(const add_triple& in) override;
    void create_mul_gate(const mul_triple& in) override;
    void create_bool_gate(const uint32_t a) override;
    void create_poly_gate(const poly_triple& in) override;
    void create_mimc_gate(const mimc_quadruplet& in);
    void create_noop_gate();
    void create_dummy_gates();
    size_t get_num_constant_gates() const override { return StandardComposer::get_num_constant_gates(); }

    std::vector<uint32_t> create_range_constraint(const uint32_t witness_index, const size_t num_bits)
    {
        auto& q_mimc_coefficient = selectors[MimcSelectors::QMIMC_COEFF];
        auto& q_mimc_selector = selectors[MimcSelectors::QMIMC_SELEC];
        if (current_output_wire != static_cast<uint32_t>(-1)) {
            create_noop_gate();
        }
        const size_t old_n = n;
        std::vector<uint32_t> out = StandardComposer::create_range_constraint(witness_index, num_bits);
        const size_t new_n = n;
        const size_t diff = new_n - old_n;
        for (size_t i = 0; i < diff; ++i) {
            q_mimc_coefficient.emplace_back(0);
            q_mimc_selector.emplace_back(0);
        }
        current_output_wire = static_cast<uint32_t>(-1);
        return out;
    };

    accumulator_triple create_logic_constraint(const uint32_t a,
                                               const uint32_t b,
                                               const size_t num_bits,
                                               bool is_xor_gate)
    {
        auto& q_mimc_coefficient = selectors[MimcSelectors::QMIMC_COEFF];
        auto& q_mimc_selector = selectors[MimcSelectors::QMIMC_SELEC];
        if (current_output_wire != static_cast<uint32_t>(-1)) {
            create_noop_gate();
        }
        const size_t old_n = n;
        accumulator_triple out = StandardComposer::create_logic_constraint(a, b, num_bits, is_xor_gate);
        const size_t new_n = n;
        const size_t diff = new_n - old_n;
        for (size_t i = 0; i < diff; ++i) {
            q_mimc_coefficient.emplace_back(0);
            q_mimc_selector.emplace_back(0);
        }
        current_output_wire = static_cast<uint32_t>(-1);
        return out;
    };

    void create_big_add_gate(const add_quad& in)
    {
        auto& q_mimc_coefficient = selectors[MimcSelectors::QMIMC_COEFF];
        auto& q_mimc_selector = selectors[MimcSelectors::QMIMC_SELEC];
        if (current_output_wire != static_cast<uint32_t>(-1)) {
            create_noop_gate();
        }
        const size_t old_n = n;
        StandardComposer::create_big_add_gate(in);
        const size_t new_n = n;
        const size_t diff = new_n - old_n;
        for (size_t i = 0; i < diff; ++i) {
            q_mimc_coefficient.emplace_back(0);
            q_mimc_selector.emplace_back(0);
        }
        current_output_wire = static_cast<uint32_t>(-1);
    }
    void create_big_add_gate_with_bit_extraction(const add_quad& in)
    {
        auto& q_mimc_coefficient = selectors[MimcSelectors::QMIMC_COEFF];
        auto& q_mimc_selector = selectors[MimcSelectors::QMIMC_SELEC];
        if (current_output_wire != static_cast<uint32_t>(-1)) {
            create_noop_gate();
        }
        const size_t old_n = n;
        StandardComposer::create_big_add_gate_with_bit_extraction(in);
        const size_t new_n = n;
        const size_t diff = new_n - old_n;
        for (size_t i = 0; i < diff; ++i) {
            q_mimc_coefficient.emplace_back(0);
            q_mimc_selector.emplace_back(0);
        }
        current_output_wire = static_cast<uint32_t>(-1);
    }
    void create_big_mul_gate(const mul_quad& in)
    {
        auto& q_mimc_coefficient = selectors[MimcSelectors::QMIMC_COEFF];
        auto& q_mimc_selector = selectors[MimcSelectors::QMIMC_SELEC];
        if (current_output_wire != static_cast<uint32_t>(-1)) {
            create_noop_gate();
        }
        const size_t old_n = n;
        StandardComposer::create_big_mul_gate(in);
        const size_t new_n = n;
        const size_t diff = new_n - old_n;
        for (size_t i = 0; i < diff; ++i) {
            q_mimc_coefficient.emplace_back(0);
            q_mimc_selector.emplace_back(0);
        }
        current_output_wire = static_cast<uint32_t>(-1);
    }
    void create_balanced_add_gate(const add_quad& in)
    {
        auto& q_mimc_coefficient = selectors[MimcSelectors::QMIMC_COEFF];
        auto& q_mimc_selector = selectors[MimcSelectors::QMIMC_SELEC];
        if (current_output_wire != static_cast<uint32_t>(-1)) {
            create_noop_gate();
        }
        const size_t old_n = n;
        StandardComposer::create_balanced_add_gate(in);
        const size_t new_n = n;
        const size_t diff = new_n - old_n;
        for (size_t i = 0; i < diff; ++i) {
            q_mimc_coefficient.emplace_back(0);
            q_mimc_selector.emplace_back(0);
        }
        current_output_wire = static_cast<uint32_t>(-1);
    }

    void fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value)
    {
        auto& q_mimc_coefficient = selectors[MimcSelectors::QMIMC_COEFF];
        auto& q_mimc_selector = selectors[MimcSelectors::QMIMC_SELEC];
        if (current_output_wire != static_cast<uint32_t>(-1)) {
            create_noop_gate();
        }
        const size_t old_n = n;
        StandardComposer::fix_witness(witness_index, witness_value);
        const size_t new_n = n;
        const size_t diff = new_n - old_n;
        for (size_t i = 0; i < diff; ++i) {
            q_mimc_coefficient.emplace_back(0);
            q_mimc_selector.emplace_back(0);
        }
        current_output_wire = static_cast<uint32_t>(-1);
    }

    void assert_equal_constant(uint32_t const a_idx, barretenberg::fr const& b)
    {
        const add_triple gate_coefficients{
            a_idx, a_idx, a_idx, barretenberg::fr::one(), barretenberg::fr::zero(), barretenberg::fr::zero(), -b,
        };
        create_add_gate(gate_coefficients);
    }

    uint32_t current_output_wire = static_cast<uint32_t>(-1);

    static transcript::Manifest create_manifest(const size_t num_public_inputs = 0)
    {
        // add public inputs....
        constexpr size_t g1_size = 64;
        constexpr size_t fr_size = 32;
        const size_t public_input_size = fr_size * num_public_inputs;
        const transcript::Manifest output = transcript::Manifest(
            { transcript::Manifest::RoundManifest(
                  { { "circuit_size", 4, true }, { "public_input_size", 4, true } }, "init", 1),
              transcript::Manifest::RoundManifest({}, "eta", 0),
              transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
                                                    { "W_1", g1_size, false },
                                                    { "W_2", g1_size, false },
                                                    { "W_3", g1_size, false } },
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
                      { "q_mimc_coefficient", fr_size, false, 6 },
                      { "z_omega", fr_size, false, -1 },
                      { "w_3_omega", fr_size, false, 0 },

                  },
                  "nu",
                  10,
                  true),
              transcript::Manifest::RoundManifest(
                  { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 1) });
        return output;
    }
};
} // namespace waffle

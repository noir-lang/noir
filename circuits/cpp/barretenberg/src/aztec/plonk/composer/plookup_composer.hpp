#pragma once
#include "composer_base.hpp"
#include "plookup_tables/plookup_tables.hpp"

namespace waffle {

class PLookupComposer : public ComposerBase {

  public:
    enum PLookupSelectors {
        QM = 0,
        QC = 1,
        Q1 = 2,
        Q2 = 3,
        Q3 = 4,
        Q4 = 5,
        Q5 = 6,
        QARITH = 7,
        QECC_1 = 8,
        QRANGE = 9,
        QLOGIC = 10,
        QLOOKUPINDEX = 11,
        QLOOKUPTYPE = 12,
    };
    PLookupComposer();
    PLookupComposer(std::string const& crs_path, const size_t size_hint = 0);
    PLookupComposer(std::unique_ptr<ReferenceStringFactory>&& crs_factory, const size_t size_hint = 0);
    PLookupComposer(std::shared_ptr<proving_key> const& p_key,
                    std::shared_ptr<verification_key> const& v_key,
                    size_t size_hint = 0);
    PLookupComposer(PLookupComposer&& other) = default;
    PLookupComposer& operator=(PLookupComposer&& other) = default;
    ~PLookupComposer() {}

    void add_lookup_selector(polynomial& small, const std::string& tag);

    std::shared_ptr<proving_key> compute_proving_key() override;
    std::shared_ptr<verification_key> compute_verification_key() override;
    std::shared_ptr<program_witness> compute_witness() override;

    PLookupProver create_prover();
    PLookupVerifier create_verifier();

    UnrolledPLookupProver create_unrolled_prover();
    UnrolledPLookupVerifier create_unrolled_verifier();

    void initialize_precomputed_table(
        const PLookupTableId id,
        bool (*generator)(std::vector<barretenberg::fr>&,
                          std::vector<barretenberg::fr>&,
                          std::vector<barretenberg::fr>&),
        std::array<barretenberg::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>));

    PLookupTable& get_table(const PLookupTableId id);
    PLookupMultiTable& get_multi_table(const PLookupMultiTableId id);

    std::array<std::vector<uint32_t>, 3> read_sequence_from_multi_table(const PLookupMultiTable& multi_table,
                                                                        const uint32_t key_index);

    std::array<uint32_t, 2> read_from_table(const PLookupTableId id, const uint32_t key_idx);
    uint32_t read_from_table(const PLookupTableId id, const uint32_t key_a, const uint32_t key_b);
    // std::pair<uint32_t, uint32_t> read_from_table(const PLookupTableId id, const uint32_t key);

    std::vector<uint32_t> read_sequence_from_table(const PLookupTableId id,
                                                   const std::vector<std::array<uint32_t, 2>>& key_indices);

    std::array<std::vector<uint32_t>, 3> read_sequence_from_table(const PLookupTableId id,
                                                                  const uint32_t key_index_a,
                                                                  const uint32_t key_index_b,
                                                                  const size_t num_lookups);

    PLookupReadData get_multi_table_values(const PLookupMultiTableId id, const barretenberg::fr key);

    std::array<std::vector<uint32_t>, 3> read_sequence_from_multi_table(const PLookupMultiTableId& id,
                                                                        const PLookupReadData& read_values,
                                                                        const uint32_t key_index);

    void validate_lookup(const PLookupTableId id, const std::array<uint32_t, 3> keys);
    void create_dummy_gate();
    void create_add_gate(const add_triple& in) override;

    void create_big_add_gate(const add_quad& in);
    void create_big_add_gate_with_bit_extraction(const add_quad& in);
    void create_big_mul_gate(const mul_quad& in);
    void create_balanced_add_gate(const add_quad& in);

    void create_mul_gate(const mul_triple& in) override;
    void create_bool_gate(const uint32_t a) override;
    void create_poly_gate(const poly_triple& in) override;
    void create_fixed_group_add_gate(const fixed_group_add_quad& in);
    void create_fixed_group_add_gate_with_init(const fixed_group_add_quad& in, const fixed_group_init_quad& init);
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

    void assert_equal_constant(const uint32_t a_idx, const barretenberg::fr& b)
    {
        ASSERT(variables[a_idx] == b);
        const add_triple gate_coefficients{
            a_idx, a_idx, a_idx, barretenberg::fr::one(), barretenberg::fr::zero(), barretenberg::fr::zero(), -b,
        };
        create_add_gate(gate_coefficients);
    }

    uint32_t zero_idx = 0;

    // these are variables that we have used a gate on, to enforce that they are equal to a defined value
    std::map<barretenberg::fr, uint32_t> constant_variables;

    std::vector<PLookupTable> lookup_tables;
    std::vector<PLookupMultiTable> lookup_multi_tables;

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
                                                    { "W_3", g1_size, false },
                                                    { "W_4", g1_size, false } },
                                                  "eta",
                                                  1),
              transcript::Manifest::RoundManifest({ { "S", g1_size, false } }, "beta", 2),
              transcript::Manifest::RoundManifest(
                  { { "Z", g1_size, false }, { "Z_LOOKUP", g1_size, false } }, "alpha", 1),
              transcript::Manifest::RoundManifest({ { "T_1", g1_size, false },
                                                    { "T_2", g1_size, false },
                                                    { "T_3", g1_size, false },
                                                    { "T_4", g1_size, false } },
                                                  "z",
                                                  1),
              transcript::Manifest::RoundManifest({ { "w_1", fr_size, false },
                                                    { "w_2", fr_size, false },
                                                    { "w_3", fr_size, false },
                                                    { "w_4", fr_size, false },
                                                    { "z_omega", fr_size, false },
                                                    { "sigma_1", fr_size, false },
                                                    { "sigma_2", fr_size, false },
                                                    { "sigma_3", fr_size, false },
                                                    { "q_arith", fr_size, false },
                                                    { "q_ecc_1", fr_size, false },
                                                    { "q_2", fr_size, false },
                                                    { "q_m", fr_size, false },
                                                    { "q_c", fr_size, false },
                                                    { "table_value_1", fr_size, false },
                                                    { "table_value_2", fr_size, false },
                                                    { "table_value_3", fr_size, false },
                                                    { "table_value_4", fr_size, false },
                                                    { "table_index", fr_size, false },
                                                    { "table_type", fr_size, false },
                                                    { "s", fr_size, false },
                                                    { "z_lookup", fr_size, false },
                                                    { "r", fr_size, false },
                                                    { "w_1_omega", fr_size, false },
                                                    { "w_2_omega", fr_size, false },
                                                    { "w_3_omega", fr_size, false },
                                                    { "w_4_omega", fr_size, false },
                                                    { "table_value_1_omega", fr_size, false },
                                                    { "table_value_2_omega", fr_size, false },
                                                    { "table_value_3_omega", fr_size, false },
                                                    { "table_value_4_omega", fr_size, false },
                                                    { "s_omega", fr_size, false },
                                                    { "z_lookup_omega", fr_size, false },
                                                    { "t", fr_size, true } },
                                                  "nu",
                                                  22,
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
              transcript::Manifest::RoundManifest({ { "public_inputs", public_input_size, false },
                                                    { "W_1", g1_size, false },
                                                    { "W_2", g1_size, false },
                                                    { "W_3", g1_size, false },
                                                    { "W_4", g1_size, false } },
                                                  "eta",
                                                  1),
              transcript::Manifest::RoundManifest({ { "S", g1_size, false } }, "beta", 2),
              transcript::Manifest::RoundManifest(
                  { { "Z", g1_size, false }, { "Z_LOOKUP", g1_size, false } }, "alpha", 1),
              transcript::Manifest::RoundManifest({ { "T_1", g1_size, false },
                                                    { "T_2", g1_size, false },
                                                    { "T_3", g1_size, false },
                                                    { "T_4", g1_size, false } },
                                                  "z",
                                                  1),
              transcript::Manifest::RoundManifest(
                  {
                      { "w_1", fr_size, false },
                      { "w_2", fr_size, false },
                      { "w_3", fr_size, false },
                      { "w_4", fr_size, false },
                      { "z_omega", fr_size, false },
                      { "sigma_1", fr_size, false },
                      { "sigma_2", fr_size, false },
                      { "sigma_3", fr_size, false },
                      { "sigma_4", fr_size, false },
                      { "q_1", fr_size, false },
                      { "q_2", fr_size, false },
                      { "q_3", fr_size, false },
                      { "q_4", fr_size, false },
                      { "q_5", fr_size, false },
                      { "q_m", fr_size, false },
                      { "q_c", fr_size, false },
                      { "q_arith", fr_size, false },
                      { "q_logic", fr_size, false },
                      { "q_range", fr_size, false },
                      { "q_ecc_1", fr_size, false },
                      { "table_value_1", fr_size, false },
                      { "table_value_2", fr_size, false },
                      { "table_value_3", fr_size, false },
                      { "table_value_4", fr_size, false },
                      { "table_index", fr_size, false },
                      { "table_type", fr_size, false },
                      { "s", fr_size, false },
                      { "z_lookup", fr_size, false },
                      { "w_1_omega", fr_size, false },
                      { "w_2_omega", fr_size, false },
                      { "w_3_omega", fr_size, false },
                      { "w_4_omega", fr_size, false },
                      { "z", fr_size, false },
                      { "table_value_1_omega", fr_size, false },
                      { "table_value_2_omega", fr_size, false },
                      { "table_value_3_omega", fr_size, false },
                      { "table_value_4_omega", fr_size, false },
                      { "s_omega", fr_size, false },
                      { "z_lookup_omega", fr_size, false },
                      { "t", fr_size, true },
                  },
                  "nu",
                  28,
                  true),
              transcript::Manifest::RoundManifest(
                  { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 1) });
        return output;
    }
};
} // namespace waffle

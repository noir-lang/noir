#pragma once
#include "composer_base.hpp"
#include "plookup_tables/plookup_tables.hpp"

namespace waffle {

class PlookupComposer : public ComposerBase {

  public:
    static constexpr ComposerType type = ComposerType::PLOOKUP;
    static constexpr size_t NUM_PLOOKUP_SELECTORS = 15;
    static constexpr size_t NUM_RESERVED_GATES = 4; // this must be >= num_roots_cut_out_of_vanishing_polynomial
    static constexpr size_t UINT_LOG2_BASE = 6;
    // the plookup range proof requires work linear in range size, thus cannot be used directly for
    // large ranges such as 2^64. For such ranges the element will be decomposed into smaller
    // chuncks according to the parameter below
    static constexpr size_t DEFAULT_PLOOKUP_RANGE_BITNUM = 17;
    static constexpr size_t DEFAULT_PLOOKUP_RANGE_SIZE = (1 << DEFAULT_PLOOKUP_RANGE_BITNUM) - 1;

    struct RangeList {
        uint64_t target_range;
        uint32_t range_tag;
        uint32_t tau_tag;
        std::vector<uint32_t> variable_indices;
    };

    enum PlookupSelectors {
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
        QSORT = 10,
        QLOGIC = 11,
        QELLIPTIC = 12,
        QLOOKUPINDEX = 13,
        QLOOKUPTYPE = 14,
    };
    PlookupComposer();
    PlookupComposer(std::string const& crs_path, const size_t size_hint = 0);
    PlookupComposer(std::unique_ptr<ReferenceStringFactory>&& crs_factory, const size_t size_hint = 0);
    PlookupComposer(std::shared_ptr<proving_key> const& p_key,
                    std::shared_ptr<verification_key> const& v_key,
                    size_t size_hint = 0);
    PlookupComposer(PlookupComposer&& other) = default;
    PlookupComposer& operator=(PlookupComposer&& other) = default;
    ~PlookupComposer() {}

    std::shared_ptr<proving_key> compute_proving_key() override;
    std::shared_ptr<verification_key> compute_verification_key() override;
    std::shared_ptr<program_witness> compute_witness() override;

    PlookupProver create_prover();
    PlookupVerifier create_verifier();

    UnrolledPlookupProver create_unrolled_prover();
    UnrolledPlookupVerifier create_unrolled_verifier();

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
    void create_fixed_group_add_gate_final(const add_quad& in);

    void create_ecc_add_gate(const ecc_add_gate& in);

    void fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value);

    std::vector<uint32_t> decompose_into_base4_accumulators(const uint32_t witness_index,
                                                            const size_t num_bits,
                                                            std::string const& msg = "create_range_constraint");
    accumulator_triple create_logic_constraint(const uint32_t a,
                                               const uint32_t b,
                                               const size_t num_bits,
                                               bool is_xor_gate);
    accumulator_triple create_and_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);
    accumulator_triple create_xor_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);

    uint32_t put_constant_variable(const barretenberg::fr& variable);

    void create_dummy_gates();
    size_t get_num_constant_gates() const override { return 0; }

    void assert_equal_constant(const uint32_t a_idx,
                               const barretenberg::fr& b,
                               std::string const& msg = "assert equal constant")
    {
        if (variables[a_idx] != b && !failed) {
            failed = true;
            err = msg;
        }
        auto b_idx = put_constant_variable(b);
        assert_equal(a_idx, b_idx, msg);
    }

    /**
     * Plookup Methods
     **/
    void add_lookup_selector(polynomial& small, const std::string& tag);
    void initialize_precomputed_table(
        const PlookupBasicTableId id,
        bool (*generator)(std::vector<barretenberg::fr>&,
                          std::vector<barretenberg::fr>&,
                          std::vector<barretenberg::fr>&),
        std::array<barretenberg::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>));

    PlookupBasicTable& get_table(const PlookupBasicTableId id);
    PlookupMultiTable& create_table(const PlookupMultiTableId id);

    std::array<std::vector<uint32_t>, 3> read_sequence_from_multi_table(const PlookupMultiTableId& id,
                                                                        const PlookupReadData& read_values,
                                                                        const uint32_t key_a_index,
                                                                        const uint32_t key_b_index = IS_CONSTANT);

    /**
     * Generalized Permutation Methods
     **/
    std::vector<uint32_t> decompose_into_default_range(const uint32_t variable_index,
                                                       const size_t num_bits,
                                                       std::string const& msg = "decompose_into_default_range");
    std::vector<uint32_t> decompose_into_default_range_better_for_oddlimbnum(
        const uint32_t variable_index,
        const size_t num_bits,
        std::string const& msg = "decompose_into_default_range_better_for_oddlimbnum");
    void create_dummy_constraints(const std::vector<uint32_t>& variable_index);
    void create_sort_constraint(const std::vector<uint32_t>& variable_index);
    void create_sort_constraint_with_edges(const std::vector<uint32_t>& variable_index,
                                           const barretenberg::fr&,
                                           const barretenberg::fr&);
    void assign_tag(const uint32_t variable_index, const uint32_t tag)
    {
        ASSERT(tag <= current_tag);
        ASSERT(variable_tags[real_variable_index[variable_index]] == DUMMY_TAG);
        variable_tags[real_variable_index[variable_index]] = tag;
    }

    uint32_t create_tag(const uint32_t tag_index, const uint32_t tau_index)
    {
        tau.insert({ tag_index, tau_index });
        current_tag++;
        return current_tag;
    }
    uint32_t get_new_tag()
    {
        current_tag++;
        return current_tag;
    }

    RangeList create_range_list(const uint64_t target_range);
    void create_new_range_constraint(const uint32_t variable_index, const uint64_t target_range);
    void process_range_list(const RangeList& list);
    void process_range_lists();

    /**
     * Member Variables
     **/
    uint32_t zero_idx = 0;

    // This variable controls the amount with which the lookup table and witness values need to be shifted
    // above to make room for adding randomness into the permutation and witness polynomials in plookup widget.
    // This must be (num_roots_cut_out_of_the_vanishing_polynomial - 1), since the variable num_roots_cut_out_of_
    // vanishing_polynomial cannot be trivially fetched here, I am directly setting this to 4 - 1 = 3.
    static constexpr size_t s_randomness = 3;

    // these are variables that we have used a gate on, to enforce that they are equal to a defined value
    std::map<barretenberg::fr, uint32_t> constant_variables;

    std::vector<PlookupBasicTable> lookup_tables;
    std::vector<PlookupMultiTable> lookup_multi_tables;
    std::map<uint64_t, RangeList> range_lists;

    /**
     * Program Manifests
     **/
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
              transcript::Manifest::RoundManifest(
                  {
                      { "w_1", fr_size, false, 0 },
                      { "w_2", fr_size, false, 1 },
                      { "w_3", fr_size, false, 2 },
                      { "w_4", fr_size, false, 3 },
                      { "sigma_1", fr_size, false, 4 },
                      { "sigma_2", fr_size, false, 5 },
                      { "sigma_3", fr_size, false, 6 },
                      { "q_arith", fr_size, false, 7 },
                      { "q_ecc_1", fr_size, false, 8 },
                      { "q_2", fr_size, false, 9 },
                      { "q_3", fr_size, false, 10 },
                      { "q_4", fr_size, false, 11 },
                      { "q_5", fr_size, false, 12 },
                      { "q_m", fr_size, false, 13 },
                      { "q_c", fr_size, false, 14 },
                      { "table_value_1", fr_size, false, 15 },
                      { "table_value_2", fr_size, false, 16 },
                      { "table_value_3", fr_size, false, 17 },
                      { "table_value_4", fr_size, false, 18 },
                      { "table_index", fr_size, false, 19 },
                      { "table_type", fr_size, false, 20 },
                      { "s", fr_size, false, 21 },
                      { "z_lookup", fr_size, false, 22 },
                      { "id_1", fr_size, false, 24 },
                      { "id_2", fr_size, false, 25 },
                      { "id_3", fr_size, false, 26 },
                      { "id_4", fr_size, false, 27 },
                      { "z_omega", fr_size, false, -1 },
                      { "w_1_omega", fr_size, false, 0 },
                      { "w_2_omega", fr_size, false, 1 },
                      { "w_3_omega", fr_size, false, 2 },
                      { "w_4_omega", fr_size, false, 3 },
                      { "table_value_1_omega", fr_size, false, 4 },
                      { "table_value_2_omega", fr_size, false, 5 },
                      { "table_value_3_omega", fr_size, false, 6 },
                      { "table_value_4_omega", fr_size, false, 7 },
                      { "s_omega", fr_size, false, 8 },
                      { "z_lookup_omega", fr_size, false, 9 },
                  },
                  "nu",
                  28,
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
                      { "t", fr_size, true, -1 },
                      { "w_1", fr_size, false, 0 },
                      { "w_2", fr_size, false, 1 },
                      { "w_3", fr_size, false, 2 },
                      { "w_4", fr_size, false, 3 },
                      { "sigma_1", fr_size, false, 4 },
                      { "sigma_2", fr_size, false, 5 },
                      { "sigma_3", fr_size, false, 6 },
                      { "sigma_4", fr_size, false, 7 },
                      { "q_1", fr_size, false, 8 },
                      { "q_2", fr_size, false, 9 },
                      { "q_3", fr_size, false, 10 },
                      { "q_4", fr_size, false, 11 },
                      { "q_5", fr_size, false, 12 },
                      { "q_m", fr_size, false, 13 },
                      { "q_c", fr_size, false, 14 },
                      { "q_arith", fr_size, false, 15 },
                      { "q_logic", fr_size, false, 16 },
                      { "q_range", fr_size, false, 17 },
                      { "q_sort", fr_size, false, 18 },
                      { "q_ecc_1", fr_size, false, 19 },
                      { "q_elliptic", fr_size, false, 20 },
                      { "table_index", fr_size, false, 21 },
                      { "table_type", fr_size, false, 22 },
                      { "s", fr_size, false, 23 },
                      { "z_lookup", fr_size, false, 24 },
                      { "table_value_1", fr_size, false, 25 },
                      { "table_value_2", fr_size, false, 26 },
                      { "table_value_3", fr_size, false, 27 },
                      { "table_value_4", fr_size, false, 28 },
                      { "z", fr_size, false, 29 },
                      { "id_1", fr_size, false, 30 },
                      { "id_2", fr_size, false, 31 },
                      { "id_3", fr_size, false, 32 },
                      { "id_4", fr_size, false, 33 },
                      { "z_omega", fr_size, false, -1 },
                      { "w_1_omega", fr_size, false, 0 },
                      { "w_2_omega", fr_size, false, 1 },
                      { "w_3_omega", fr_size, false, 2 },
                      { "w_4_omega", fr_size, false, 3 },
                      { "s_omega", fr_size, false, 4 },
                      { "z_lookup_omega", fr_size, false, 5 },
                      { "table_value_1_omega", fr_size, false, 6 },
                      { "table_value_2_omega", fr_size, false, 7 },
                      { "table_value_3_omega", fr_size, false, 8 },
                      { "table_value_4_omega", fr_size, false, 9 },
                  },
                  "nu",
                  34,
                  true),
              transcript::Manifest::RoundManifest(
                  { { "PI_Z", g1_size, false }, { "PI_Z_OMEGA", g1_size, false } }, "separator", 3) });
        return output;
    }
};
} // namespace waffle

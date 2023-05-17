#pragma once
#include "barretenberg/proof_system/plookup_tables/plookup_tables.hpp"
#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/honk/proof_system/ultra_verifier.hpp"
#include "barretenberg/proof_system/circuit_constructors/ultra_circuit_constructor.hpp"
#include "barretenberg/honk/composer/composer_helper/ultra_honk_composer_helper.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
namespace proof_system::honk {

class UltraHonkComposer {

  public:
    // An instantiation of the circuit constructor that only depends on arithmetization, not  on the proof system
    UltraCircuitConstructor circuit_constructor;
    // Composer helper contains all proof-related material that is separate from circuit creation such as:
    // 1) Proving and verification keys
    // 2) CRS
    // 3) Converting variables to witness vectors/polynomials
    using Flavor = honk::flavor::Ultra;
    using CircuitConstructor = UltraCircuitConstructor;
    using ProvingKey = typename Flavor::ProvingKey;
    using VerificationKey = typename Flavor::VerificationKey;

    UltraHonkComposerHelper composer_helper;
    size_t& num_gates;

    UltraHonkComposer()
        : UltraHonkComposer("../srs_db/ignition", 0){};

    UltraHonkComposer(std::string const& crs_path, const size_t size_hint)
        : UltraHonkComposer(std::unique_ptr<ReferenceStringFactory>(new FileReferenceStringFactory(crs_path)),
                            size_hint){};

    UltraHonkComposer(std::shared_ptr<ReferenceStringFactory> const& crs_factory, const size_t size_hint)
        : circuit_constructor(size_hint)
        , composer_helper(crs_factory)
        , num_gates(circuit_constructor.num_gates){};

    UltraHonkComposer(std::shared_ptr<ProvingKey> const& p_key,
                      std::shared_ptr<VerificationKey> const& v_key,
                      size_t size_hint = 0);
    UltraHonkComposer(UltraHonkComposer&& other) = default;
    UltraHonkComposer& operator=(UltraHonkComposer&& other) = delete;
    ~UltraHonkComposer() = default;

    uint32_t get_zero_idx() { return circuit_constructor.zero_idx; }

    uint32_t add_variable(const barretenberg::fr& in) { return circuit_constructor.add_variable(in); }

    barretenberg::fr get_variable(const uint32_t index) const { return circuit_constructor.get_variable(index); }

    void finalize_circuit() { circuit_constructor.finalize_circuit(); };

    UltraProver create_prover() { return composer_helper.create_prover(circuit_constructor); };
    UltraVerifier create_verifier() { return composer_helper.create_verifier(circuit_constructor); };

    void add_gates_to_ensure_all_polys_are_non_zero()
    {
        circuit_constructor.add_gates_to_ensure_all_polys_are_non_zero();
    }

    void create_add_gate(const add_triple& in) { circuit_constructor.create_add_gate(in); }

    void create_big_add_gate(const add_quad& in, const bool use_next_gate_w_4 = false)
    {
        circuit_constructor.create_big_add_gate(in, use_next_gate_w_4);
    };

    // void create_big_add_gate_with_bit_extraction(const add_quad& in);
    // void create_big_mul_gate(const mul_quad& in);
    // void create_balanced_add_gate(const add_quad& in);

    // void create_mul_gate(const mul_triple& in) override;
    // void create_bool_gate(const uint32_t a) override;
    // void create_poly_gate(const poly_triple& in) override;
    void create_ecc_add_gate(const ecc_add_gate& in) { circuit_constructor.create_ecc_add_gate(in); };

    // void fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value);

    // void add_recursive_proof(const std::vector<uint32_t>& proof_output_witness_indices)
    // {
    //     if (contains_recursive_proof) {
    //         failure("added recursive proof when one already exists");
    //     }
    //     contains_recursive_proof = true;

    //     for (const auto& idx : proof_output_witness_indices) {
    //         set_public_input(idx);
    //         recursive_proof_public_input_indices.push_back((uint32_t)(public_inputs.size() - 1));
    //     }
    // }

    void create_new_range_constraint(const uint32_t variable_index,
                                     const uint64_t target_range,
                                     std::string const msg = "create_new_range_constraint")
    {
        circuit_constructor.create_new_range_constraint(variable_index, target_range, msg);
    };
    // void create_range_constraint(const uint32_t variable_index, const size_t num_bits, std::string const& msg)
    // {
    //     if (num_bits <= DEFAULT_PLOOKUP_RANGE_BITNUM) {
    //         /**
    //          * N.B. if `variable_index` is not used in any arithmetic constraints, this will create an unsatisfiable
    //          *      circuit!
    //          *      this range constraint will increase the size of the 'sorted set' of range-constrained integers
    //          by 1.
    //          *      The 'non-sorted set' of range-constrained integers is a subset of the wire indices of all
    //          arithmetic
    //          *      gates. No arithemtic gate => size imbalance between sorted and non-sorted sets. Checking for this
    //          *      and throwing an error would require a refactor of the Composer to catelog all 'orphan' variables
    //          not
    //          *      assigned to gates.
    //          **/
    //         create_new_range_constraint(variable_index, 1ULL << num_bits, msg);
    //     } else {
    //         decompose_into_default_range(variable_index, num_bits, DEFAULT_PLOOKUP_RANGE_BITNUM, msg);
    //     }
    // }

    // accumulator_triple create_logic_constraint(const uint32_t a,
    //                                            const uint32_t b,
    //                                            const size_t num_bits,
    //                                            bool is_xor_gate);
    // accumulator_triple create_and_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);
    // accumulator_triple create_xor_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);

    uint32_t put_constant_variable(const barretenberg::fr& variable)
    {
        return circuit_constructor.put_constant_variable(variable);
    };

    // size_t get_num_constant_gates() const override { return 0; }

    // /**
    //  * @brief Get the final number of gates in a circuit, which consists of the sum of:
    //  * 1) Current number number of actual gates
    //  * 2) Number of public inputs, as we'll need to add a gate for each of them
    //  * 3) Number of Rom array-associated gates
    //  * 4) NUmber of range-list associated gates
    //  *
    //  *
    //  * @param count return arument, number of existing gates
    //  * @param rangecount return argument, extra gates due to range checks
    //  * @param romcount return argument, extra gates due to rom reads
    //  * @param ramcount return argument, extra gates due to ram read/writes
    //  */
    // void get_num_gates_split_into_components(size_t& count,
    //                                          size_t& rangecount,
    //                                          size_t& romcount,
    //                                          size_t& ramcount) const
    // {
    //     count = num_gates;
    //     // each ROM gate adds +1 extra gate due to the rom reads being copied to a sorted list set
    //     for (size_t i = 0; i < rom_arrays.size(); ++i) {
    //         for (size_t j = 0; j < rom_arrays[i].state.size(); ++j) {
    //             if (rom_arrays[i].state[j][0] == UNINITIALIZED_MEMORY_RECORD) {
    //                 romcount += 2;
    //             }
    //         }
    //         romcount += (rom_arrays[i].records.size());
    //         romcount += 1; // we add an addition gate after procesing a rom array
    //     }

    //     constexpr size_t gate_width = ultra_settings::program_width;
    //     // each RAM gate adds +2 extra gates due to the ram reads being copied to a sorted list set,
    //     // as well as an extra gate to validate timestamps
    //     std::vector<size_t> ram_timestamps;
    //     std::vector<size_t> ram_range_sizes;
    //     std::vector<size_t> ram_range_exists;
    //     for (size_t i = 0; i < ram_arrays.size(); ++i) {
    //         for (size_t j = 0; j < ram_arrays[i].state.size(); ++j) {
    //             if (ram_arrays[i].state[j] == UNINITIALIZED_MEMORY_RECORD) {
    //                 ramcount += NUMBER_OF_GATES_PER_RAM_ACCESS;
    //             }
    //         }
    //         ramcount += (ram_arrays[i].records.size() * NUMBER_OF_GATES_PER_RAM_ACCESS);
    //         ramcount += NUMBER_OF_ARITHMETIC_GATES_PER_RAM_ARRAY; // we add an addition gate after procesing a ram
    //         array

    //         // there will be 'max_timestamp' number of range checks, need to calculate.
    //         const auto max_timestamp = ram_arrays[i].access_count - 1;

    //         // if a range check of length `max_timestamp` already exists, we are double counting.
    //         // We record `ram_timestamps` to detect and correct for this error when we process range lists.
    //         ram_timestamps.push_back(max_timestamp);
    //         size_t padding = (gate_width - (max_timestamp % gate_width)) % gate_width;
    //         if (max_timestamp == gate_width)
    //             padding += gate_width;
    //         const size_t ram_range_check_list_size = max_timestamp + padding;

    //         size_t ram_range_check_gate_count = (ram_range_check_list_size / gate_width);
    //         ram_range_check_gate_count += 1; // we need to add 1 extra addition gates for every distinct range list

    //         ram_range_sizes.push_back(ram_range_check_gate_count);
    //         ram_range_exists.push_back(false);
    //         // rangecount += ram_range_check_gate_count;
    //     }
    //     for (const auto& list : range_lists) {
    //         auto list_size = list.second.variable_indices.size();
    //         size_t padding = (gate_width - (list.second.variable_indices.size() % gate_width)) % gate_width;
    //         if (list.second.variable_indices.size() == gate_width)
    //             padding += gate_width;
    //         list_size += padding;

    //         for (size_t i = 0; i < ram_timestamps.size(); ++i) {
    //             if (list.second.target_range == ram_timestamps[i]) {
    //                 ram_range_exists[i] = true;
    //             }
    //         }
    //         rangecount += (list_size / gate_width);
    //         rangecount += 1; // we need to add 1 extra addition gates for every distinct range list
    //     }
    //     // update rangecount to include the ram range checks the composer will eventually be creating
    //     for (size_t i = 0; i < ram_range_sizes.size(); ++i) {
    //         if (!ram_range_exists[i]) {
    //             rangecount += ram_range_sizes[i];
    //         }
    //     }
    // }

    // /**
    //  * @brief Get the final number of gates in a circuit, which consists of the sum of:
    //  * 1) Current number number of actual gates
    //  * 2) Number of public inputs, as we'll need to add a gate for each of them
    //  * 3) Number of Rom array-associated gates
    //  * 4) NUmber of range-list associated gates
    //  *
    //  * @return size_t
    //  */
    // virtual size_t get_num_gates() const override
    // {
    //     // if circuit finalised already added extra gates
    //     if (circuit_finalised) {
    //         return num_gates;
    //     }
    //     size_t count = 0;
    //     size_t rangecount = 0;
    //     size_t romcount = 0;
    //     size_t ramcount = 0;
    //     get_num_gates_split_into_components(count, rangecount, romcount, ramcount);
    //     return count + romcount + ramcount + rangecount;
    // }

    // virtual void print_num_gates() const override
    // {
    //     size_t count = 0;
    //     size_t rangecount = 0;
    //     size_t romcount = 0;
    //     size_t ramcount = 0;

    //     get_num_gates_split_into_components(count, rangecount, romcount, ramcount);

    //     size_t total = count + romcount + ramcount + rangecount;
    //     std::cout << "gates = " << total << " (arith " << count << ", rom " << romcount << ", ram " << ramcount
    //               << ", range " << rangecount << "), pubinp = " << public_inputs.size() << std::endl;
    // }

    void assert_equal(const uint32_t a_variable_idx,
                      const uint32_t b_variable_idx,
                      std::string const& msg = "assert_equal")
    {
        circuit_constructor.assert_equal(a_variable_idx, b_variable_idx, msg);
    }

    // void assert_equal_constant(const uint32_t a_idx,
    //                            const barretenberg::fr& b,
    //                            std::string const& msg = "assert equal constant")
    // {
    //     if (variables[a_idx] != b && !failed()) {
    //         failure(msg);
    //     }
    //     auto b_idx = put_constant_variable(b);
    //     assert_equal(a_idx, b_idx, msg);
    // }

    // /**
    //  * Plookup Methods
    //  **/
    // void add_table_column_selector_poly_to_proving_key(polynomial& small, const std::string& tag);
    // void initialize_precomputed_table(
    //     const plookup::BasicTableId id,
    //     bool (*generator)(std::vector<barretenberg::fr>&,
    //                       std::vector<barretenberg::fr>&,
    //                       std::vector<barretenberg::fr>&),
    //     std::array<barretenberg::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>));

    // plookup::BasicTable& get_table(const plookup::BasicTableId id);
    // plookup::MultiTable& create_table(const plookup::MultiTableId id);

    plookup::ReadData<uint32_t> create_gates_from_plookup_accumulators(
        const plookup::MultiTableId& id,
        const plookup::ReadData<barretenberg::fr>& read_values,
        const uint32_t key_a_index,
        std::optional<uint32_t> key_b_index = std::nullopt)
    {
        return circuit_constructor.create_gates_from_plookup_accumulators(id, read_values, key_a_index, key_b_index);
    };

    // /**
    //  * Generalized Permutation Methods
    //  **/
    std::vector<uint32_t> decompose_into_default_range(
        const uint32_t variable_index,
        const uint64_t num_bits,
        const uint64_t target_range_bitnum = UltraCircuitConstructor::DEFAULT_PLOOKUP_RANGE_BITNUM,
        std::string const& msg = "decompose_into_default_range")
    {
        return circuit_constructor.decompose_into_default_range(variable_index, num_bits, target_range_bitnum, msg);
    };
    // std::vector<uint32_t> decompose_into_default_range_better_for_oddlimbnum(
    //     const uint32_t variable_index,
    //     const size_t num_bits,
    //     std::string const& msg = "decompose_into_default_range_better_for_oddlimbnum");
    void create_dummy_constraints(const std::vector<uint32_t>& variable_index)
    {
        circuit_constructor.create_dummy_constraints(variable_index);
    };
    void create_sort_constraint(const std::vector<uint32_t>& variable_index)
    {
        circuit_constructor.create_sort_constraint(variable_index);
    };
    void create_sort_constraint_with_edges(const std::vector<uint32_t>& variable_index,
                                           const barretenberg::fr& start,
                                           const barretenberg::fr& end)
    {
        circuit_constructor.create_sort_constraint_with_edges(variable_index, start, end);
    };

    void assign_tag(const uint32_t variable_index, const uint32_t tag)
    {
        circuit_constructor.assign_tag(variable_index, tag);
    }

    // void assign_tag(const uint32_t variable_index, const uint32_t tag)
    // {
    //     ASSERT(tag <= current_tag);
    //     ASSERT(real_variable_tags[real_variable_index[variable_index]] == DUMMY_TAG);
    //     real_variable_tags[real_variable_index[variable_index]] = tag;
    // }

    uint32_t create_tag(const uint32_t tag_index, const uint32_t tau_index)
    {
        return circuit_constructor.create_tag(tag_index, tau_index);
    }

    // uint32_t get_new_tag()
    // {
    //     current_tag++;
    //     return current_tag;
    // }

    // RangeList create_range_list(const uint64_t target_range);
    // void process_range_list(const RangeList& list);
    // void process_range_lists();

    // /**
    //  * Custom Gate Selectors
    //  **/
    // void apply_aux_selectors(const AUX_SELECTORS type);

    // /**
    //  * Non Native Field Arithmetic
    //  **/
    void range_constrain_two_limbs(
        const uint32_t lo_idx,
        const uint32_t hi_idx,
        const size_t lo_limb_bits = UltraCircuitConstructor::DEFAULT_NON_NATIVE_FIELD_LIMB_BITS,
        const size_t hi_limb_bits = UltraCircuitConstructor::DEFAULT_NON_NATIVE_FIELD_LIMB_BITS)
    {
        circuit_constructor.range_constrain_two_limbs(lo_idx, hi_idx, lo_limb_bits, hi_limb_bits);
    };
    // std::array<uint32_t, 2> decompose_non_native_field_double_width_limb(
    //     const uint32_t limb_idx, const size_t num_limb_bits = (2 * DEFAULT_NON_NATIVE_FIELD_LIMB_BITS));
    std::array<uint32_t, 2> evaluate_non_native_field_multiplication(
        const UltraCircuitConstructor::non_native_field_witnesses& input,
        const bool range_constrain_quotient_and_remainder = true)
    {
        return circuit_constructor.evaluate_non_native_field_multiplication(input,
                                                                            range_constrain_quotient_and_remainder);
    };
    // std::array<uint32_t, 2> evaluate_partial_non_native_field_multiplication(const non_native_field_witnesses&
    // input); typedef std::pair<uint32_t, barretenberg::fr> scaled_witness; typedef std::tuple<scaled_witness,
    // scaled_witness, barretenberg::fr> add_simple; std::array<uint32_t, 5> evaluate_non_native_field_subtraction(
    //     add_simple limb0,
    //     add_simple limb1,
    //     add_simple limb2,
    //     add_simple limb3,
    //     std::tuple<uint32_t, uint32_t, barretenberg::fr> limbp);
    // std::array<uint32_t, 5> evaluate_non_native_field_addition(add_simple limb0,
    //                                                            add_simple limb1,
    //                                                            add_simple limb2,
    //                                                            add_simple limb3,
    //                                                            std::tuple<uint32_t, uint32_t, barretenberg::fr>
    //                                                            limbp);

    // /**
    //  * Memory
    //  **/

    size_t create_RAM_array(const size_t array_size) { return circuit_constructor.create_RAM_array(array_size); };
    size_t create_ROM_array(const size_t array_size) { return circuit_constructor.create_ROM_array(array_size); };

    void set_ROM_element(const size_t rom_id, const size_t index_value, const uint32_t value_witness)
    {
        circuit_constructor.set_ROM_element(rom_id, index_value, value_witness);
    };
    // void set_ROM_element_pair(const size_t rom_id,
    //                           const size_t index_value,
    //                           const std::array<uint32_t, 2>& value_witnesses);
    uint32_t read_ROM_array(const size_t rom_id, const uint32_t index_witness)
    {
        return circuit_constructor.read_ROM_array(rom_id, index_witness);
    };
    // std::array<uint32_t, 2> read_ROM_array_pair(const size_t rom_id, const uint32_t index_witness);
    // void create_ROM_gate(RomRecord& record);
    // void create_sorted_ROM_gate(RomRecord& record);
    // void process_ROM_array(const size_t rom_id, const size_t gate_offset_from_public_inputs);
    // void process_ROM_arrays(const size_t gate_offset_from_public_inputs);

    // void create_RAM_gate(RamRecord& record);
    // void create_sorted_RAM_gate(RamRecord& record);
    // void create_final_sorted_RAM_gate(RamRecord& record, const size_t ram_array_size);

    // size_t create_RAM_array(const size_t array_size);
    void init_RAM_element(const size_t ram_id, const size_t index_value, const uint32_t value_witness)
    {
        circuit_constructor.init_RAM_element(ram_id, index_value, value_witness);
    };
    uint32_t read_RAM_array(const size_t ram_id, const uint32_t index_witness)
    {
        return circuit_constructor.read_RAM_array(ram_id, index_witness);
    };
    void write_RAM_array(const size_t ram_id, const uint32_t index_witness, const uint32_t value_witness)
    {
        circuit_constructor.write_RAM_array(ram_id, index_witness, value_witness);
    };
    // void process_RAM_array(const size_t ram_id, const size_t gate_offset_from_public_inputs);
    // void process_RAM_arrays(const size_t gate_offset_from_public_inputs);

    // /**
    //  * Member Variables
    //  **/

    // uint32_t zero_idx = 0;
    bool circuit_finalised = false;

    // // This variable controls the amount with which the lookup table and witness values need to be shifted
    // // above to make room for adding randomness into the permutation and witness polynomials in the plookup widget.
    // // This must be (num_roots_cut_out_of_the_vanishing_polynomial - 1), since the variable num_roots_cut_out_of_
    // // vanishing_polynomial cannot be trivially fetched here, I am directly setting this to 4 - 1 = 3.
    // static constexpr size_t s_randomness = 3;

    // // these are variables that we have used a gate on, to enforce that they are equal to a defined value
    // std::map<barretenberg::fr, uint32_t> constant_variable_indices;

    // std::vector<plookup::BasicTable> lookup_tables;
    // std::vector<plookup::MultiTable> lookup_multi_tables;
    // std::map<uint64_t, RangeList> range_lists; // DOCTODO: explain this.

    // /**
    //  * @brief Each entry in ram_arrays represents an independent RAM table.
    //  * RamTranscript tracks the current table state,
    //  * as well as the 'records' produced by each read and write operation.
    //  * Used in `compute_proving_key` to generate consistency check gates required to validate the RAM read/write
    //  history
    //  */
    // std::vector<RamTranscript> ram_arrays;

    // /**
    //  * @brief Each entry in ram_arrays represents an independent ROM table.
    //  * RomTranscript tracks the current table state,
    //  * as well as the 'records' produced by each read operation.
    //  * Used in `compute_proving_key` to generate consistency check gates required to validate the ROM read history
    //  */
    // std::vector<RomTranscript> rom_arrays;

    // // Stores gate index of ROM and RAM reads (required by proving key)
    // std::vector<uint32_t> memory_read_records;
    // // Stores gate index of RAM writes (required by proving key)
    // std::vector<uint32_t> memory_write_records;

    // std::vector<uint32_t> recursive_proof_public_input_indices;
    // bool contains_recursive_proof = false;
};
} // namespace proof_system::honk

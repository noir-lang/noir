#pragma once
#include "barretenberg/proof_system/arithmetization/arithmetization.hpp"
#include "barretenberg/plonk/proof_system/types/polynomial_manifest.hpp"
#include "circuit_constructor_base.hpp"
#include "barretenberg/plonk/proof_system/constants.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/proof_system/types/merkle_hash_type.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/plonk/composer/plookup_tables/types.hpp"
#include "barretenberg/plonk/composer/plookup_tables/plookup_tables.hpp"
#include "barretenberg/plonk/proof_system/types/prover_settings.hpp"
#include <optional>

namespace proof_system {

static constexpr ComposerType type = ComposerType::PLOOKUP;
static constexpr merkle::HashType merkle_hash_type = merkle::HashType::LOOKUP_PEDERSEN;
static constexpr size_t NUM_RESERVED_GATES = 4; // This must be >= num_roots_cut_out_of_vanishing_polynomial
                                                // See the comment in plonk/proof_system/prover/prover.cpp
                                                // ProverBase::compute_quotient_commitments() for why 4 exactly.
static constexpr size_t UINT_LOG2_BASE = 6;     // DOCTODO: explain what this is, or rename.
// The plookup range proof requires work linear in range size, thus cannot be used directly for
// large ranges such as 2^64. For such ranges the element will be decomposed into smaller
// chuncks according to the parameter below
static constexpr size_t DEFAULT_PLOOKUP_RANGE_BITNUM = 14;
static constexpr size_t DEFAULT_PLOOKUP_RANGE_STEP_SIZE = 3;
static constexpr size_t DEFAULT_PLOOKUP_RANGE_SIZE = (1 << DEFAULT_PLOOKUP_RANGE_BITNUM) - 1;
static constexpr size_t DEFAULT_NON_NATIVE_FIELD_LIMB_BITS = 68;
static constexpr uint32_t UNINITIALIZED_MEMORY_RECORD = UINT32_MAX;
static constexpr size_t NUMBER_OF_GATES_PER_RAM_ACCESS = 2;
static constexpr size_t NUMBER_OF_ARITHMETIC_GATES_PER_RAM_ARRAY = 1;

struct non_native_field_witnesses {
    // first 4 array elements = limbs
    // 5th element = prime basis limb
    std::array<uint32_t, 5> a;
    std::array<uint32_t, 5> b;
    std::array<uint32_t, 5> q;
    std::array<uint32_t, 5> r;
    std::array<barretenberg::fr, 5> neg_modulus;
    barretenberg::fr modulus;
};

enum AUX_SELECTORS {
    NONE,
    LIMB_ACCUMULATE_1,
    LIMB_ACCUMULATE_2,
    NON_NATIVE_FIELD_1,
    NON_NATIVE_FIELD_2,
    NON_NATIVE_FIELD_3,
    RAM_CONSISTENCY_CHECK,
    ROM_CONSISTENCY_CHECK,
    RAM_TIMESTAMP_CHECK,
    ROM_READ,
    RAM_READ,
    RAM_WRITE,
};

struct RangeList {
    uint64_t target_range;
    uint32_t range_tag;
    uint32_t tau_tag;
    std::vector<uint32_t> variable_indices;
};

/**
 * @brief A ROM memory record that can be ordered
 *
 *
 */
struct RomRecord {
    uint32_t index_witness = 0;
    uint32_t value_column1_witness = 0;
    uint32_t value_column2_witness = 0;
    uint32_t index = 0;
    uint32_t record_witness = 0;
    size_t gate_index = 0;
    bool operator<(const RomRecord& other) const { return index < other.index; }
};

/**
 * @brief A RAM memory record that can be ordered
 *
 *
 */
struct RamRecord {
    enum AccessType {
        READ,
        WRITE,
    };
    uint32_t index_witness = 0;
    uint32_t timestamp_witness = 0;
    uint32_t value_witness = 0;
    uint32_t index = 0;
    uint32_t timestamp = 0;
    AccessType access_type = AccessType::READ; // read or write?
    uint32_t record_witness = 0;
    size_t gate_index = 0;
    bool operator<(const RamRecord& other) const
    {
        bool index_test = (index) < (other.index);
        return index_test || (index == other.index && timestamp < other.timestamp);
    }
};

/**
 * @brief Each ram array is an instance of memory transcript. It saves values and indexes for a particular memory
 * array
 *
 *
 */
struct RamTranscript {
    // Contains the value of each index of the array
    std::vector<uint32_t> state;

    // A vector of records, each of which contains:
    // + The constant witness with the index
    // + The value in the memory slot
    // + The actual index value
    std::vector<RamRecord> records;

    // used for RAM records, to compute the timestamp when performing a read/write
    size_t access_count = 0;
};

/**
 * @brief Each rom array is an instance of memory transcript. It saves values and indexes for a particular memory
 * array
 *
 *
 */
struct RomTranscript {
    // Contains the value of each index of the array
    std::vector<std::array<uint32_t, 2>> state;

    // A vector of records, each of which contains:
    // + The constant witness with the index
    // + The value in the memory slot
    // + The actual index value
    std::vector<RomRecord> records;
};

inline std::vector<std::string> ultra_selector_names()
{
    std::vector<std::string> result{ "q_m",     "q_c",    "q_1",        "q_2",   "q_3",       "q_4",
                                     "q_arith", "q_sort", "q_elliptic", "q_aux", "table_type" };
    return result;
}

class UltraCircuitConstructor : public CircuitConstructorBase<arithmetization::Ultra> {
  public:
    std::vector<uint32_t>& w_l = std::get<0>(wires);
    std::vector<uint32_t>& w_r = std::get<1>(wires);
    std::vector<uint32_t>& w_o = std::get<2>(wires);
    std::vector<uint32_t>& w_4 = std::get<3>(wires);

    std::vector<barretenberg::fr>& q_m = std::get<0>(selectors);
    std::vector<barretenberg::fr>& q_c = std::get<1>(selectors);
    std::vector<barretenberg::fr>& q_1 = std::get<2>(selectors);
    std::vector<barretenberg::fr>& q_2 = std::get<3>(selectors);
    std::vector<barretenberg::fr>& q_3 = std::get<4>(selectors);
    std::vector<barretenberg::fr>& q_4 = std::get<5>(selectors);
    std::vector<barretenberg::fr>& q_arith = std::get<6>(selectors);
    std::vector<barretenberg::fr>& q_sort = std::get<7>(selectors);
    std::vector<barretenberg::fr>& q_elliptic = std::get<8>(selectors);
    std::vector<barretenberg::fr>& q_aux = std::get<9>(selectors);
    std::vector<barretenberg::fr>& q_lookup_type = std::get<10>(selectors);

    // TODO(#216)(Kesha): replace this with Honk enums after we have a verifier and no longer depend on plonk
    // prover/verifier
    static constexpr ComposerType type = ComposerType::STANDARD_HONK;
    static constexpr size_t UINT_LOG2_BASE = 2;

    // These are variables that we have used a gate on, to enforce that they are
    // equal to a defined value.
    // TODO(#216)(Adrian): Why is this not in CircuitConstructorBase
    std::map<barretenberg::fr, uint32_t> constant_variable_indices;

    std::vector<plookup::BasicTable> lookup_tables;
    std::vector<plookup::MultiTable> lookup_multi_tables;
    std::map<uint64_t, RangeList> range_lists; // DOCTODO: explain this.

    /**
     * @brief Each entry in ram_arrays represents an independent RAM table.
     * RamTranscript tracks the current table state,
     * as well as the 'records' produced by each read and write operation.
     * Used in `compute_proving_key` to generate consistency check gates required to validate the RAM read/write history
     */
    std::vector<RamTranscript> ram_arrays;

    /**
     * @brief Each entry in ram_arrays represents an independent ROM table.
     * RomTranscript tracks the current table state,
     * as well as the 'records' produced by each read operation.
     * Used in `compute_proving_key` to generate consistency check gates required to validate the ROM read history
     */
    std::vector<RomTranscript> rom_arrays;

    // Stores gate index of ROM and RAM reads (required by proving key)
    std::vector<uint32_t> memory_read_records;
    // Stores gate index of RAM writes (required by proving key)
    std::vector<uint32_t> memory_write_records;

    bool circuit_finalised = false;

    UltraCircuitConstructor(const size_t size_hint = 0)
        : CircuitConstructorBase(ultra_selector_names(), size_hint)
    {
        w_l.reserve(size_hint);
        w_r.reserve(size_hint);
        w_o.reserve(size_hint);
        w_4.reserve(size_hint);
        zero_idx = put_constant_variable(0);
        tau.insert({ DUMMY_TAG, DUMMY_TAG });
    };

    UltraCircuitConstructor(const UltraCircuitConstructor& other) = delete;
    UltraCircuitConstructor(UltraCircuitConstructor&& other) = default;
    UltraCircuitConstructor& operator=(const UltraCircuitConstructor& other) = delete;
    UltraCircuitConstructor& operator=(UltraCircuitConstructor&& other) = delete;
    ~UltraCircuitConstructor() override = default;

    void finalize_circuit();

    void create_add_gate(const add_triple& in) override;

    void create_big_add_gate(const add_quad& in, const bool use_next_gate_w_4 = false);
    void create_big_add_gate_with_bit_extraction(const add_quad& in);
    void create_big_mul_gate(const mul_quad& in);
    void create_balanced_add_gate(const add_quad& in);

    void create_mul_gate(const mul_triple& in) override;
    void create_bool_gate(const uint32_t a) override;
    void create_poly_gate(const poly_triple& in) override;
    void create_ecc_add_gate(const ecc_add_gate& in);

    void fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value);

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
                                     std::string const msg = "create_new_range_constraint");
    void create_range_constraint(const uint32_t variable_index, const size_t num_bits, std::string const& msg)
    {
        if (num_bits <= DEFAULT_PLOOKUP_RANGE_BITNUM) {
            /**
             * N.B. if `variable_index` is not used in any arithmetic constraints, this will create an unsatisfiable
             *      circuit!
             *      this range constraint will increase the size of the 'sorted set' of range-constrained integers by 1.
             *      The 'non-sorted set' of range-constrained integers is a subset of the wire indices of all arithmetic
             *      gates. No arithemtic gate => size imbalance between sorted and non-sorted sets. Checking for this
             *      and throwing an error would require a refactor of the Composer to catelog all 'orphan' variables not
             *      assigned to gates.
             **/
            create_new_range_constraint(variable_index, 1ULL << num_bits, msg);
        } else {
            decompose_into_default_range(variable_index, num_bits, DEFAULT_PLOOKUP_RANGE_BITNUM, msg);
        }
    }

    accumulator_triple create_logic_constraint(const uint32_t a,
                                               const uint32_t b,
                                               const size_t num_bits,
                                               bool is_xor_gate);
    accumulator_triple create_and_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);
    accumulator_triple create_xor_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);

    uint32_t put_constant_variable(const barretenberg::fr& variable);

    size_t get_num_constant_gates() const override { return 0; }

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

    void assert_equal_constant(const uint32_t a_idx,
                               const barretenberg::fr& b,
                               std::string const& msg = "assert equal constant")
    {
        if (variables[a_idx] != b && !failed()) {
            failure(msg);
        }
        auto b_idx = put_constant_variable(b);
        assert_equal(a_idx, b_idx, msg);
    }

    /**
     * Plookup Methods
     **/
    void add_table_column_selector_poly_to_proving_key(barretenberg::polynomial& small, const std::string& tag);
    void initialize_precomputed_table(
        const plookup::BasicTableId id,
        bool (*generator)(std::vector<barretenberg::fr>&,
                          std::vector<barretenberg::fr>&,
                          std::vector<barretenberg::fr>&),
        std::array<barretenberg::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>));

    plookup::BasicTable& get_table(const plookup::BasicTableId id);
    plookup::MultiTable& create_table(const plookup::MultiTableId id);

    plookup::ReadData<uint32_t> create_gates_from_plookup_accumulators(
        const plookup::MultiTableId& id,
        const plookup::ReadData<barretenberg::fr>& read_values,
        const uint32_t key_a_index,
        std::optional<uint32_t> key_b_index = std::nullopt);

    /**
     * Generalized Permutation Methods
     **/
    std::vector<uint32_t> decompose_into_default_range(
        const uint32_t variable_index,
        const uint64_t num_bits,
        const uint64_t target_range_bitnum = DEFAULT_PLOOKUP_RANGE_BITNUM,
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
        ASSERT(real_variable_tags[real_variable_index[variable_index]] == DUMMY_TAG);
        real_variable_tags[real_variable_index[variable_index]] = tag;
    }

    uint32_t create_tag(const uint32_t tag_index, const uint32_t tau_index)
    {
        tau.insert({ tag_index, tau_index });
        current_tag++; // Why exactly?
        return current_tag;
    }

    uint32_t get_new_tag()
    {
        current_tag++;
        return current_tag;
    }

    RangeList create_range_list(const uint64_t target_range);
    void process_range_list(const RangeList& list);
    void process_range_lists();

    /**
     * Custom Gate Selectors
     **/
    void apply_aux_selectors(const AUX_SELECTORS type);

    /**
     * Non Native Field Arithmetic
     **/
    void range_constrain_two_limbs(const uint32_t lo_idx,
                                   const uint32_t hi_idx,
                                   const size_t lo_limb_bits = DEFAULT_NON_NATIVE_FIELD_LIMB_BITS,
                                   const size_t hi_limb_bits = DEFAULT_NON_NATIVE_FIELD_LIMB_BITS);
    std::array<uint32_t, 2> decompose_non_native_field_double_width_limb(
        const uint32_t limb_idx, const size_t num_limb_bits = (2 * DEFAULT_NON_NATIVE_FIELD_LIMB_BITS));
    std::array<uint32_t, 2> evaluate_non_native_field_multiplication(
        const non_native_field_witnesses& input, const bool range_constrain_quotient_and_remainder = true);
    std::array<uint32_t, 2> evaluate_partial_non_native_field_multiplication(const non_native_field_witnesses& input);
    typedef std::pair<uint32_t, barretenberg::fr> scaled_witness;
    typedef std::tuple<scaled_witness, scaled_witness, barretenberg::fr> add_simple;
    std::array<uint32_t, 5> evaluate_non_native_field_subtraction(
        add_simple limb0,
        add_simple limb1,
        add_simple limb2,
        add_simple limb3,
        std::tuple<uint32_t, uint32_t, barretenberg::fr> limbp);
    std::array<uint32_t, 5> evaluate_non_native_field_addition(add_simple limb0,
                                                               add_simple limb1,
                                                               add_simple limb2,
                                                               add_simple limb3,
                                                               std::tuple<uint32_t, uint32_t, barretenberg::fr> limbp);

    /**
     * Memory
     **/

    // size_t create_RAM_array(const size_t array_size);
    size_t create_ROM_array(const size_t array_size);

    void set_ROM_element(const size_t rom_id, const size_t index_value, const uint32_t value_witness);
    void set_ROM_element_pair(const size_t rom_id,
                              const size_t index_value,
                              const std::array<uint32_t, 2>& value_witnesses);
    uint32_t read_ROM_array(const size_t rom_id, const uint32_t index_witness);
    std::array<uint32_t, 2> read_ROM_array_pair(const size_t rom_id, const uint32_t index_witness);
    void create_ROM_gate(RomRecord& record);
    void create_sorted_ROM_gate(RomRecord& record);
    void process_ROM_array(const size_t rom_id, const size_t gate_offset_from_public_inputs);
    void process_ROM_arrays(const size_t gate_offset_from_public_inputs);

    void create_RAM_gate(RamRecord& record);
    void create_sorted_RAM_gate(RamRecord& record);
    void create_final_sorted_RAM_gate(RamRecord& record, const size_t ram_array_size);

    size_t create_RAM_array(const size_t array_size);
    void init_RAM_element(const size_t ram_id, const size_t index_value, const uint32_t value_witness);
    uint32_t read_RAM_array(const size_t ram_id, const uint32_t index_witness);
    void write_RAM_array(const size_t ram_id, const uint32_t index_witness, const uint32_t value_witness);
    void process_RAM_array(const size_t ram_id, const size_t gate_offset_from_public_inputs);
    void process_RAM_arrays(const size_t gate_offset_from_public_inputs);
};
} // namespace proof_system

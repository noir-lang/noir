#pragma once
// #include "barretenberg/plonk/proof_system/constants.hpp"
// #include "barretenberg/plonk/proof_system/types/polynomial_manifest.hpp"
// #include "barretenberg/plonk/proof_system/types/prover_settings.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/proof_system/op_queue/ecc_op_queue.hpp"
#include "barretenberg/proof_system/plookup_tables/plookup_tables.hpp"
#include "barretenberg/proof_system/plookup_tables/types.hpp"
#include "barretenberg/proof_system/types/circuit_type.hpp"
#include "barretenberg/proof_system/types/merkle_hash_type.hpp"
#include "barretenberg/proof_system/types/pedersen_commitment_type.hpp"
#include "circuit_builder_base.hpp"
#include <optional>

namespace bb {

template <typename FF> struct non_native_field_witnesses {
    // first 4 array elements = limbs
    // 5th element = prime basis limb
    std::array<uint32_t, 5> a;
    std::array<uint32_t, 5> b;
    std::array<uint32_t, 5> q;
    std::array<uint32_t, 5> r;
    std::array<FF, 5> neg_modulus;
    FF modulus;
};

using namespace bb;

template <typename Arithmetization>
class UltraCircuitBuilder_ : public CircuitBuilderBase<typename Arithmetization::FF> {
  public:
    using FF = typename Arithmetization::FF;
    static constexpr size_t NUM_WIRES = Arithmetization::NUM_WIRES;
    // Keeping NUM_WIRES, at least temporarily, for backward compatibility
    static constexpr size_t program_width = Arithmetization::NUM_WIRES;
    static constexpr size_t num_selectors = Arithmetization::NUM_SELECTORS;
    std::vector<std::string> selector_names = Arithmetization::selector_names;

    static constexpr std::string_view NAME_STRING = "UltraArithmetization";
    static constexpr CircuitType CIRCUIT_TYPE = CircuitType::ULTRA;
    static constexpr merkle::HashType merkle_hash_type = merkle::HashType::LOOKUP_PEDERSEN;
    static constexpr pedersen::CommitmentType commitment_type = pedersen::CommitmentType::FIXED_BASE_PEDERSEN;
    static constexpr size_t UINT_LOG2_BASE = 6; // DOCTODO: explain what this is, or rename.
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
    static constexpr size_t NUM_RESERVED_GATES = 4;
    // number of gates created per non-native field operation in process_non_native_field_multiplications
    static constexpr size_t GATES_PER_NON_NATIVE_FIELD_MULTIPLICATION_ARITHMETIC = 7;

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
        bool operator==(const RangeList& other) const noexcept
        {
            return target_range == other.target_range && range_tag == other.range_tag && tau_tag == other.tau_tag &&
                   variable_indices == other.variable_indices;
        }
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
        bool operator==(const RomRecord& other) const noexcept
        {
            return index_witness == other.index_witness && value_column1_witness == other.value_column1_witness &&
                   value_column2_witness == other.value_column2_witness && index == other.index &&
                   record_witness == other.record_witness && gate_index == other.gate_index;
        }
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
        bool operator==(const RamRecord& other) const noexcept
        {
            return index_witness == other.index_witness && timestamp_witness == other.timestamp_witness &&
                   value_witness == other.value_witness && index == other.index && timestamp == other.timestamp &&
                   access_type == other.access_type && record_witness == other.record_witness &&
                   gate_index == other.gate_index;
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
        // Used to check that the state hasn't changed in tests
        bool operator==(const RamTranscript& other) const noexcept
        {
            return (state == other.state && records == other.records && access_count == other.access_count);
        }
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

        // Used to check that the state hasn't changed in tests
        bool operator==(const RomTranscript& other) const noexcept
        {
            return (state == other.state && records == other.records);
        }
    };

    /**
     * @brief Used to store instructions to create partial_non_native_field_multiplication gates.
     *        We want to cache these (and remove duplicates) as the stdlib code can end up multiplying the same inputs
     * repeatedly.
     */
    struct cached_partial_non_native_field_multiplication {
        std::array<uint32_t, 5> a;
        std::array<uint32_t, 5> b;
        FF lo_0;
        FF hi_0;
        FF hi_1;

        bool operator==(const cached_partial_non_native_field_multiplication& other) const
        {
            bool valid = true;
            for (size_t i = 0; i < 5; ++i) {
                valid = valid && (a[i] == other.a[i]);
                valid = valid && (b[i] == other.b[i]);
            }
            return valid;
        }

        static void deduplicate(std::vector<cached_partial_non_native_field_multiplication>& vec)
        {
            std::unordered_set<cached_partial_non_native_field_multiplication, Hash, std::equal_to<>> seen;

            std::vector<cached_partial_non_native_field_multiplication> uniqueVec;

            for (const auto& item : vec) {
                if (seen.insert(item).second) {
                    uniqueVec.push_back(item);
                }
            }

            vec.swap(uniqueVec);
        }

        bool operator<(const cached_partial_non_native_field_multiplication& other) const
        {
            if (a < other.a) {
                return true;
            }
            if (other.a < a) {
                return false;
            }
            if (b < other.b) {
                return true;
            }
            return other.b < b;
        }

        struct Hash {
            size_t operator()(const cached_partial_non_native_field_multiplication& obj) const
            {
                size_t combined_hash = 0;

                // C++ does not have a standard way to hash values, so we use the
                // common algorithm that boot uses.
                // You can search for 'cpp hash_combine' to find more information.
                // Here is one reference:
                // https://stackoverflow.com/questions/2590677/how-do-i-combine-hash-values-in-c0x
                auto hash_combiner = [](size_t lhs, size_t rhs) {
                    return lhs ^ (rhs + 0x9e3779b9 + (lhs << 6) + (lhs >> 2));
                };

                for (const auto& elem : obj.a) {
                    combined_hash = hash_combiner(combined_hash, std::hash<uint32_t>()(elem));
                }
                for (const auto& elem : obj.b) {
                    combined_hash = hash_combiner(combined_hash, std::hash<uint32_t>()(elem));
                }

                return combined_hash;
            }
        };
    };

    struct non_native_field_multiplication_cross_terms {
        uint32_t lo_0_idx;
        uint32_t lo_1_idx;
        uint32_t hi_0_idx;
        uint32_t hi_1_idx;
        uint32_t hi_2_idx;
        uint32_t hi_3_idx;
    };

    /**
     * @brief CircuitDataBackup is a structure we use to store all the information about the circuit that is needed
     * to restore it back to a pre-finalized state
     * @details In check_circuit method in UltraCircuitBuilder we want to check that the whole circuit works,
     * but ultra circuits need to have ram, rom and range gates added in the end for the check to be complete as
     * well as the set permutation check, so we finalize the circuit when we check it. This structure allows us to
     * restore the circuit to the state before the finalization.
     */
    struct CircuitDataBackup {
        using WireVector = std::vector<uint32_t, bb::ContainerSlabAllocator<uint32_t>>;
        using SelectorVector = std::vector<FF, bb::ContainerSlabAllocator<FF>>;

        std::vector<uint32_t> public_inputs;
        std::vector<FF> variables;
        // index of next variable in equivalence class (=REAL_VARIABLE if you're last)
        std::vector<uint32_t> next_var_index;
        // index of  previous variable in equivalence class (=FIRST if you're in a cycle alone)
        std::vector<uint32_t> prev_var_index;
        // indices of corresponding real variables
        std::vector<uint32_t> real_variable_index;
        std::vector<uint32_t> real_variable_tags;
        std::map<FF, uint32_t> constant_variable_indices;
        WireVector w_l;
        WireVector w_r;
        WireVector w_o;
        WireVector w_4;
        SelectorVector q_m;
        SelectorVector q_c;
        SelectorVector q_1;
        SelectorVector q_2;
        SelectorVector q_3;
        SelectorVector q_4;
        SelectorVector q_arith;
        SelectorVector q_sort;
        SelectorVector q_elliptic;
        SelectorVector q_aux;
        SelectorVector q_lookup_type;
        uint32_t current_tag = DUMMY_TAG;
        std::map<uint32_t, uint32_t> tau;

        std::vector<RamTranscript> ram_arrays;
        std::vector<RomTranscript> rom_arrays;

        std::vector<uint32_t> memory_read_records;
        std::vector<uint32_t> memory_write_records;
        std::map<uint64_t, RangeList> range_lists;

        std::vector<UltraCircuitBuilder_::cached_partial_non_native_field_multiplication>
            cached_partial_non_native_field_multiplications;

        size_t num_gates;
        bool circuit_finalized = false;
        /**
         * @brief Stores the state of everything logic-related in the builder.
         *
         * @details We need this function for tests. Specifically, to ensure that we are not changing anything in
         * check_circuit
         *
         * @param builder
         * @return CircuitDataBackup
         */
        template <typename CircuitBuilder> static CircuitDataBackup store_full_state(const CircuitBuilder& builder)
        {
            CircuitDataBackup stored_state;
            stored_state.public_inputs = builder.public_inputs;
            stored_state.variables = builder.variables;

            stored_state.next_var_index = builder.next_var_index;

            stored_state.prev_var_index = builder.prev_var_index;

            stored_state.real_variable_index = builder.real_variable_index;
            stored_state.real_variable_tags = builder.real_variable_tags;
            stored_state.constant_variable_indices = builder.constant_variable_indices;
            stored_state.w_l = builder.w_l();
            stored_state.w_r = builder.w_r();
            stored_state.w_o = builder.w_o();
            stored_state.w_4 = builder.w_4();
            stored_state.q_m = builder.q_m();
            stored_state.q_c = builder.q_c();
            stored_state.q_1 = builder.q_1();
            stored_state.q_2 = builder.q_2();
            stored_state.q_3 = builder.q_3();
            stored_state.q_4 = builder.q_4();
            stored_state.q_arith = builder.q_arith();
            stored_state.q_sort = builder.q_sort();
            stored_state.q_elliptic = builder.q_elliptic();
            stored_state.q_aux = builder.q_aux();
            stored_state.q_lookup_type = builder.q_lookup_type();
            stored_state.current_tag = builder.current_tag;
            stored_state.tau = builder.tau;

            stored_state.ram_arrays = builder.ram_arrays;
            stored_state.rom_arrays = builder.rom_arrays;

            stored_state.memory_read_records = builder.memory_read_records;
            stored_state.memory_write_records = builder.memory_write_records;
            stored_state.range_lists = builder.range_lists;
            stored_state.circuit_finalized = builder.circuit_finalized;
            stored_state.num_gates = builder.num_gates;
            stored_state.cached_partial_non_native_field_multiplications =
                builder.cached_partial_non_native_field_multiplications;
            return stored_state;
        }

        /**
         * @brief Stores the state of all members of the circuit constructor that are needed to restore the state
         * after finalizing the circuit.
         *
         * @param builder
         * @return CircuitDataBackup
         */
        template <typename CircuitBuilder>
        static CircuitDataBackup store_prefinilized_state(const CircuitBuilder* builder)
        {
            CircuitDataBackup stored_state;
            stored_state.public_inputs = builder->public_inputs;
            stored_state.variables = builder->variables;

            stored_state.next_var_index = builder->next_var_index;

            stored_state.prev_var_index = builder->prev_var_index;

            stored_state.real_variable_index = builder->real_variable_index;
            stored_state.real_variable_tags = builder->real_variable_tags;
            stored_state.constant_variable_indices = builder->constant_variable_indices;
            stored_state.current_tag = builder->current_tag;
            stored_state.tau = builder->tau;

            stored_state.ram_arrays = builder->ram_arrays;
            stored_state.rom_arrays = builder->rom_arrays;

            stored_state.memory_read_records = builder->memory_read_records;
            stored_state.memory_write_records = builder->memory_write_records;
            stored_state.range_lists = builder->range_lists;
            stored_state.circuit_finalized = builder->circuit_finalized;
            stored_state.num_gates = builder->num_gates;
            stored_state.cached_partial_non_native_field_multiplications =
                builder->cached_partial_non_native_field_multiplications;

            return stored_state;
        }

        /**
         * @brief Restores circuit constructor to a prefinilized state.
         *
         * @param builder
         * @return CircuitDataBackup
         */
        template <typename CircuitBuilder> void restore_prefinilized_state(CircuitBuilder* builder)
        {
            builder->public_inputs = public_inputs;
            builder->variables = variables;

            builder->next_var_index = next_var_index;

            builder->prev_var_index = prev_var_index;

            builder->real_variable_index = real_variable_index;
            builder->real_variable_tags = real_variable_tags;
            builder->constant_variable_indices = constant_variable_indices;
            builder->current_tag = current_tag;
            builder->tau = tau;

            builder->ram_arrays = ram_arrays;
            builder->rom_arrays = rom_arrays;

            builder->memory_read_records = memory_read_records;
            builder->memory_write_records = memory_write_records;
            builder->range_lists = range_lists;
            builder->circuit_finalized = circuit_finalized;
            builder->num_gates = num_gates;
            builder->cached_partial_non_native_field_multiplications = cached_partial_non_native_field_multiplications;
            builder->w_l().resize(num_gates);
            builder->w_r().resize(num_gates);
            builder->w_o().resize(num_gates);
            builder->w_4().resize(num_gates);
            builder->q_m().resize(num_gates);
            builder->q_c().resize(num_gates);
            builder->q_1().resize(num_gates);
            builder->q_2().resize(num_gates);
            builder->q_3().resize(num_gates);
            builder->q_4().resize(num_gates);
            builder->q_arith().resize(num_gates);
            builder->q_sort().resize(num_gates);
            builder->q_elliptic().resize(num_gates);
            builder->q_aux().resize(num_gates);
            builder->q_lookup_type().resize(num_gates);
        }
        /**
         * @brief Checks that the circuit state is the same as the stored circuit's one
         *
         * @param builder
         * @return true
         * @return false
         */
        template <typename CircuitBuilder> bool is_same_state(const CircuitBuilder& builder)
        {
            if (!(public_inputs == builder.public_inputs)) {
                return false;
            }
            if (!(variables == builder.variables)) {
                return false;
            }
            if (!(next_var_index == builder.next_var_index)) {
                return false;
            }
            if (!(prev_var_index == builder.prev_var_index)) {
                return false;
            }
            if (!(real_variable_index == builder.real_variable_index)) {
                return false;
            }
            if (!(real_variable_tags == builder.real_variable_tags)) {
                return false;
            }
            if (!(constant_variable_indices == builder.constant_variable_indices)) {
                return false;
            }
            if (!(w_l == builder.w_l())) {
                return false;
            }
            if (!(w_r == builder.w_r())) {
                return false;
            }
            if (!(w_o == builder.w_o())) {
                return false;
            }
            if (!(w_4 == builder.w_4())) {
                return false;
            }
            if (!(q_m == builder.q_m())) {
                return false;
            }
            if (!(q_c == builder.q_c())) {
                return false;
            }
            if (!(q_1 == builder.q_1())) {
                return false;
            }
            if (!(q_2 == builder.q_2())) {
                return false;
            }
            if (!(q_3 == builder.q_3())) {
                return false;
            }
            if (!(q_4 == builder.q_4())) {
                return false;
            }
            if (!(q_arith == builder.q_arith())) {
                return false;
            }
            if (!(q_sort == builder.q_sort())) {
                return false;
            }
            if (!(q_elliptic == builder.q_elliptic())) {
                return false;
            }
            if (!(q_aux == builder.q_aux())) {
                return false;
            }
            if (!(q_lookup_type == builder.q_lookup_type())) {
                return false;
            }
            if (!(current_tag == builder.current_tag)) {
                return false;
            }
            if (!(tau == builder.tau)) {
                return false;
            }
            if (!(ram_arrays == builder.ram_arrays)) {
                return false;
            }
            if (!(rom_arrays == builder.rom_arrays)) {
                return false;
            }
            if (!(memory_read_records == builder.memory_read_records)) {
                return false;
            }
            if (!(memory_write_records == builder.memory_write_records)) {
                return false;
            }
            if (!(range_lists == builder.range_lists)) {
                return false;
            }
            if (!(cached_partial_non_native_field_multiplications ==
                  builder.cached_partial_non_native_field_multiplications)) {
                return false;
            }
            if (!(num_gates == builder.num_gates)) {
                return false;
            }
            if (!(circuit_finalized == builder.circuit_finalized)) {
                return false;
            }
            return true;
        }
    };

    std::array<std::vector<uint32_t, bb::ContainerSlabAllocator<uint32_t>>, NUM_WIRES> wires;
    Arithmetization selectors;

    using WireVector = std::vector<uint32_t, ContainerSlabAllocator<uint32_t>>;
    using SelectorVector = std::vector<FF, ContainerSlabAllocator<FF>>;

    WireVector& w_l() { return std::get<0>(wires); };
    WireVector& w_r() { return std::get<1>(wires); };
    WireVector& w_o() { return std::get<2>(wires); };
    WireVector& w_4() { return std::get<3>(wires); };

    const WireVector& w_l() const { return std::get<0>(wires); };
    const WireVector& w_r() const { return std::get<1>(wires); };
    const WireVector& w_o() const { return std::get<2>(wires); };
    const WireVector& w_4() const { return std::get<3>(wires); };

    SelectorVector& q_m() { return selectors.q_m(); };
    SelectorVector& q_c() { return selectors.q_c(); };
    SelectorVector& q_1() { return selectors.q_1(); };
    SelectorVector& q_2() { return selectors.q_2(); };
    SelectorVector& q_3() { return selectors.q_3(); };
    SelectorVector& q_4() { return selectors.q_4(); };
    SelectorVector& q_arith() { return selectors.q_arith(); };
    SelectorVector& q_sort() { return selectors.q_sort(); };
    SelectorVector& q_elliptic() { return selectors.q_elliptic(); };
    SelectorVector& q_aux() { return selectors.q_aux(); };
    SelectorVector& q_lookup_type() { return selectors.q_lookup_type(); };

    const SelectorVector& q_c() const { return selectors.q_c(); };
    const SelectorVector& q_1() const { return selectors.q_1(); };
    const SelectorVector& q_2() const { return selectors.q_2(); };
    const SelectorVector& q_3() const { return selectors.q_3(); };
    const SelectorVector& q_4() const { return selectors.q_4(); };
    const SelectorVector& q_arith() const { return selectors.q_arith(); };
    const SelectorVector& q_sort() const { return selectors.q_sort(); };
    const SelectorVector& q_elliptic() const { return selectors.q_elliptic(); };
    const SelectorVector& q_aux() const { return selectors.q_aux(); };
    const SelectorVector& q_lookup_type() const { return selectors.q_lookup_type(); };
    const SelectorVector& q_m() const { return selectors.q_m(); };

    // These are variables that we have used a gate on, to enforce that they are
    // equal to a defined value.
    // TODO(#216)(Adrian): Why is this not in CircuitBuilderBase
    std::map<FF, uint32_t> constant_variable_indices;

    std::vector<plookup::BasicTable> lookup_tables;
    std::vector<plookup::MultiTable> lookup_multi_tables;
    std::map<uint64_t, RangeList> range_lists; // DOCTODO: explain this.

    /**
     * @brief Each entry in ram_arrays represents an independent RAM table.
     * RamTranscript tracks the current table state,
     * as well as the 'records' produced by each read and write operation.
     * Used in `compute_proving_key` to generate consistency check gates required to validate the RAM read/write
     * history
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

    std::vector<cached_partial_non_native_field_multiplication> cached_partial_non_native_field_multiplications;

    bool circuit_finalized = false;

    void process_non_native_field_multiplications();
    UltraCircuitBuilder_(const size_t size_hint = 0)
        : CircuitBuilderBase<FF>(size_hint)
    {
        selectors.reserve(size_hint);
        w_l().reserve(size_hint);
        w_r().reserve(size_hint);
        w_o().reserve(size_hint);
        w_4().reserve(size_hint);
        this->zero_idx = put_constant_variable(FF::zero());
        this->tau.insert({ DUMMY_TAG, DUMMY_TAG }); // TODO(luke): explain this
    };
    /**
     * @brief Constructor from data generated from ACIR
     *
     * @param size_hint
     * @param witness_values witnesses values known to acir
     * @param public_inputs indices of public inputs in witness array
     * @param varnum number of known witness
     *
     * @note The size of witness_values may be less than varnum. The former is the set of actual witness values known at
     * the time of acir generation. The former may be larger and essentially acounts for placeholders for witnesses that
     * we know will exist but whose values are not known during acir generation. Both are in general less than the total
     * number of variables/witnesses that might be present for a circuit generated from acir, since many gates will
     * depend on the details of the bberg implementation (or more generally on the backend used to process acir).
     */
    UltraCircuitBuilder_(const size_t size_hint,
                         auto& witness_values,
                         const std::vector<uint32_t>& public_inputs,
                         size_t varnum)
        : CircuitBuilderBase<FF>(size_hint)
    {
        selectors.reserve(size_hint);
        w_l().reserve(size_hint);
        w_r().reserve(size_hint);
        w_o().reserve(size_hint);
        w_4().reserve(size_hint);

        for (size_t idx = 0; idx < varnum; ++idx) {
            // Zeros are added for variables whose existence is known but whose values are not yet known. The values may
            // be "set" later on via the assert_equal mechanism.
            auto value = idx < witness_values.size() ? witness_values[idx] : 0;
            this->add_variable(value);
        }

        // Add the public_inputs from acir
        this->public_inputs = public_inputs;

        // Add the const zero variable after the acir witness has been
        // incorporated into variables.
        this->zero_idx = put_constant_variable(FF::zero());
        this->tau.insert({ DUMMY_TAG, DUMMY_TAG }); // TODO(luke): explain this
    };
    UltraCircuitBuilder_(const UltraCircuitBuilder_& other) = default;
    UltraCircuitBuilder_(UltraCircuitBuilder_&& other)
        : CircuitBuilderBase<FF>(std::move(other))
    {
        wires = other.wires;
        selectors = other.selectors;
        constant_variable_indices = other.constant_variable_indices;

        lookup_tables = other.lookup_tables;
        lookup_multi_tables = other.lookup_multi_tables;
        range_lists = other.range_lists;
        ram_arrays = other.ram_arrays;
        rom_arrays = other.rom_arrays;
        memory_read_records = other.memory_read_records;
        memory_write_records = other.memory_write_records;
        cached_partial_non_native_field_multiplications = other.cached_partial_non_native_field_multiplications;
        circuit_finalized = other.circuit_finalized;
    };
    UltraCircuitBuilder_& operator=(const UltraCircuitBuilder_& other) = default;
    UltraCircuitBuilder_& operator=(UltraCircuitBuilder_&& other)
    {
        CircuitBuilderBase<FF>::operator=(std::move(other));
        wires = other.wires;
        selectors = other.selectors;
        constant_variable_indices = other.constant_variable_indices;

        lookup_tables = other.lookup_tables;
        lookup_multi_tables = other.lookup_multi_tables;
        range_lists = other.range_lists;
        ram_arrays = other.ram_arrays;
        rom_arrays = other.rom_arrays;
        memory_read_records = other.memory_read_records;
        memory_write_records = other.memory_write_records;
        cached_partial_non_native_field_multiplications = other.cached_partial_non_native_field_multiplications;
        circuit_finalized = other.circuit_finalized;
        return *this;
    };
    ~UltraCircuitBuilder_() override = default;

    /**
     * @brief Debug helper method for ensuring all selectors have the same size
     * @details Each gate construction method manually appends values to the selectors. Failing to update one of the
     * selectors will lead to an unsatisfiable circuit. This method provides a mechanism for ensuring that each selector
     * has been updated as expected. Its logic is only active in debug mode.
     *
     */
    void check_selector_length_consistency()
    {
#if NDEBUG
        // do nothing
#else
        size_t nominal_size = selectors.get()[0].size();
        for (size_t idx = 1; idx < selectors.get().size(); ++idx) {
            ASSERT(selectors.get()[idx].size() == nominal_size);
        }
#endif // NDEBUG
    }

    void finalize_circuit();

    void add_gates_to_ensure_all_polys_are_non_zero();

    void create_add_gate(const add_triple_<FF>& in) override;

    void create_big_add_gate(const add_quad_<FF>& in, const bool use_next_gate_w_4 = false);
    void create_big_add_gate_with_bit_extraction(const add_quad_<FF>& in);
    void create_big_mul_gate(const mul_quad_<FF>& in);
    void create_balanced_add_gate(const add_quad_<FF>& in);

    void create_mul_gate(const mul_triple_<FF>& in) override;
    void create_bool_gate(const uint32_t a) override;
    void create_poly_gate(const poly_triple_<FF>& in) override;
    void create_ecc_add_gate(const ecc_add_gate_<FF>& in);
    void create_ecc_dbl_gate(const ecc_dbl_gate_<FF>& in);

    void fix_witness(const uint32_t witness_index, const FF& witness_value);

    void create_new_range_constraint(const uint32_t variable_index,
                                     const uint64_t target_range,
                                     std::string const msg = "create_new_range_constraint");
    void create_range_constraint(const uint32_t variable_index, const size_t num_bits, std::string const& msg)
    {
        if (num_bits == 1) {
            create_bool_gate(variable_index);
        } else if (num_bits <= DEFAULT_PLOOKUP_RANGE_BITNUM) {
            /**
             * N.B. if `variable_index` is not used in any arithmetic constraints, this will create an unsatisfiable
             *      circuit!
             *      this range constraint will increase the size of the 'sorted set' of range-constrained integers by 1.
             *      The 'non-sorted set' of range-constrained integers is a subset of the wire indices of all arithmetic
             *      gates. No arithmetic gate => size imbalance between sorted and non-sorted sets. Checking for this
             *      and throwing an error would require a refactor of the Composer to catelog all 'orphan' variables not
             *      assigned to gates.
             *
             * TODO(Suyash):
             *    The following is a temporary fix to make sure the range constraints on numbers with
             *    num_bits <= DEFAULT_PLOOKUP_RANGE_BITNUM is correctly enforced in the circuit.
             *    Longer term, as Zac says, we would need to refactor the composer to fix this.
             **/
            create_poly_gate(poly_triple_<FF>{
                .a = variable_index,
                .b = variable_index,
                .c = variable_index,
                .q_m = 0,
                .q_l = 1,
                .q_r = -1,
                .q_o = 0,
                .q_c = 0,
            });
            create_new_range_constraint(variable_index, 1ULL << num_bits, msg);
        } else {
            decompose_into_default_range(variable_index, num_bits, DEFAULT_PLOOKUP_RANGE_BITNUM, msg);
        }
    }

    accumulator_triple_<FF> create_logic_constraint(const uint32_t a,
                                                    const uint32_t b,
                                                    const size_t num_bits,
                                                    bool is_xor_gate);
    accumulator_triple_<FF> create_and_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);
    accumulator_triple_<FF> create_xor_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);

    uint32_t put_constant_variable(const FF& variable);

  public:
    size_t get_num_constant_gates() const override { return 0; }
    /**
     * @brief Get the final number of gates in a circuit, which consists of the sum of:
     * 1) Current number number of actual gates
     * 2) Number of public inputs, as we'll need to add a gate for each of them
     * 3) Number of Rom array-associated gates
     * 4) Number of range-list associated gates
     * 5) Number of non-native field multiplication gates.
     *
     *
     * @param count return arument, number of existing gates
     * @param rangecount return argument, extra gates due to range checks
     * @param romcount return argument, extra gates due to rom reads
     * @param ramcount return argument, extra gates due to ram read/writes
     * @param nnfcount return argument, extra gates due to queued non native field gates
     */
    void get_num_gates_split_into_components(
        size_t& count, size_t& rangecount, size_t& romcount, size_t& ramcount, size_t& nnfcount) const
    {
        count = this->num_gates;
        // each ROM gate adds +1 extra gate due to the rom reads being copied to a sorted list set
        for (size_t i = 0; i < rom_arrays.size(); ++i) {
            for (size_t j = 0; j < rom_arrays[i].state.size(); ++j) {
                if (rom_arrays[i].state[j][0] == UNINITIALIZED_MEMORY_RECORD) {
                    romcount += 2;
                }
            }
            romcount += (rom_arrays[i].records.size());
            romcount += 1; // we add an addition gate after procesing a rom array
        }

        // each RAM gate adds +2 extra gates due to the ram reads being copied to a sorted list set,
        // as well as an extra gate to validate timestamps
        std::vector<size_t> ram_timestamps;
        std::vector<size_t> ram_range_sizes;
        std::vector<size_t> ram_range_exists;
        for (size_t i = 0; i < ram_arrays.size(); ++i) {
            for (size_t j = 0; j < ram_arrays[i].state.size(); ++j) {
                if (ram_arrays[i].state[j] == UNINITIALIZED_MEMORY_RECORD) {
                    ramcount += NUMBER_OF_GATES_PER_RAM_ACCESS;
                }
            }
            ramcount += (ram_arrays[i].records.size() * NUMBER_OF_GATES_PER_RAM_ACCESS);
            ramcount += NUMBER_OF_ARITHMETIC_GATES_PER_RAM_ARRAY; // we add an addition gate after procesing a ram array

            // there will be 'max_timestamp' number of range checks, need to calculate.
            const auto max_timestamp = ram_arrays[i].access_count - 1;

            // if a range check of length `max_timestamp` already exists, we are double counting.
            // We record `ram_timestamps` to detect and correct for this error when we process range lists.
            ram_timestamps.push_back(max_timestamp);
            size_t padding = (NUM_WIRES - (max_timestamp % NUM_WIRES)) % NUM_WIRES;
            if (max_timestamp == NUM_WIRES)
                padding += NUM_WIRES;
            const size_t ram_range_check_list_size = max_timestamp + padding;

            size_t ram_range_check_gate_count = (ram_range_check_list_size / NUM_WIRES);
            ram_range_check_gate_count += 1; // we need to add 1 extra addition gates for every distinct range list

            ram_range_sizes.push_back(ram_range_check_gate_count);
            ram_range_exists.push_back(false);
        }
        for (const auto& list : range_lists) {
            auto list_size = list.second.variable_indices.size();
            size_t padding = (NUM_WIRES - (list.second.variable_indices.size() % NUM_WIRES)) % NUM_WIRES;
            if (list.second.variable_indices.size() == NUM_WIRES)
                padding += NUM_WIRES;
            list_size += padding;

            for (size_t i = 0; i < ram_timestamps.size(); ++i) {
                if (list.second.target_range == ram_timestamps[i]) {
                    ram_range_exists[i] = true;
                }
            }
            rangecount += (list_size / NUM_WIRES);
            rangecount += 1; // we need to add 1 extra addition gates for every distinct range list
        }
        // update rangecount to include the ram range checks the composer will eventually be creating
        for (size_t i = 0; i < ram_range_sizes.size(); ++i) {
            if (!ram_range_exists[i]) {
                rangecount += ram_range_sizes[i];
            }
        }
        std::vector<cached_partial_non_native_field_multiplication> nnf_copy(
            cached_partial_non_native_field_multiplications);
        // update nnfcount
        std::sort(nnf_copy.begin(), nnf_copy.end());

        auto last = std::unique(nnf_copy.begin(), nnf_copy.end());
        const size_t num_nnf_ops = static_cast<size_t>(std::distance(nnf_copy.begin(), last));
        nnfcount = num_nnf_ops * GATES_PER_NON_NATIVE_FIELD_MULTIPLICATION_ARITHMETIC;
    }

    /**
     * @brief Get the final number of gates in a circuit, which consists of the sum of:
     * 1) Current number number of actual gates
     * 2) Number of public inputs, as we'll need to add a gate for each of them
     * 3) Number of Rom array-associated gates
     * 4) Number of range-list associated gates
     * 5) Number of non-native field multiplication gates.
     *
     * @return size_t
     */
    size_t get_num_gates() const override
    {
        // if circuit finalized already added extra gates
        if (circuit_finalized) {
            return this->num_gates;
        }
        size_t count = 0;
        size_t rangecount = 0;
        size_t romcount = 0;
        size_t ramcount = 0;
        size_t nnfcount = 0;
        get_num_gates_split_into_components(count, rangecount, romcount, ramcount, nnfcount);
        return count + romcount + ramcount + rangecount + nnfcount;
    }

    /**
     * @brief Get the size of the circuit if it was finalized now
     *
     * @details This method estimates the size of the circuit without rounding up to the next power of 2. It takes into
     * account the possibility that the tables will dominate the size and checks both the estimated plookup argument
     * size and the general circuit size
     *
     * @return size_t
     */
    size_t get_total_circuit_size() const
    {
        size_t tables_size = 0;
        size_t lookups_size = 0;
        for (const auto& table : lookup_tables) {
            tables_size += table.size;
            lookups_size += table.lookup_gates.size();
        }

        auto minimum_circuit_size = tables_size + lookups_size;
        auto num_filled_gates = get_num_gates() + this->public_inputs.size();
        return std::max(minimum_circuit_size, num_filled_gates) + NUM_RESERVED_GATES;
    }

    /**x
     * @brief Print the number and composition of gates in the circuit
     *
     */
    virtual void print_num_gates() const override
    {
        size_t count = 0;
        size_t rangecount = 0;
        size_t romcount = 0;
        size_t ramcount = 0;
        size_t nnfcount = 0;
        get_num_gates_split_into_components(count, rangecount, romcount, ramcount, nnfcount);

        size_t total = count + romcount + ramcount + rangecount;
        std::cout << "gates = " << total << " (arith " << count << ", rom " << romcount << ", ram " << ramcount
                  << ", range " << rangecount << ", non native field gates " << nnfcount
                  << "), pubinp = " << this->public_inputs.size() << std::endl;
    }

    void assert_equal_constant(const uint32_t a_idx, const FF& b, std::string const& msg = "assert equal constant")
    {
        if (this->variables[a_idx] != b && !this->failed()) {
            this->failure(msg);
        }
        auto b_idx = put_constant_variable(b);
        this->assert_equal(a_idx, b_idx, msg);
    }

    /**
     * Plookup Methods
     **/
    void add_table_column_selector_poly_to_proving_key(bb::polynomial& small, const std::string& tag);
    void initialize_precomputed_table(const plookup::BasicTableId id,
                                      bool (*generator)(std::vector<FF>&, std::vector<FF>&, std::vector<FF>&),
                                      std::array<FF, 2> (*get_values_from_key)(const std::array<uint64_t, 2>));

    plookup::BasicTable& get_table(const plookup::BasicTableId id);
    plookup::MultiTable& create_table(const plookup::MultiTableId id);

    plookup::ReadData<uint32_t> create_gates_from_plookup_accumulators(
        const plookup::MultiTableId& id,
        const plookup::ReadData<FF>& read_values,
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
    void create_sort_constraint_with_edges(const std::vector<uint32_t>& variable_index, const FF&, const FF&);
    void assign_tag(const uint32_t variable_index, const uint32_t tag)
    {
        ASSERT(tag <= this->current_tag);
        // If we've already assigned this tag to this variable, return (can happen due to copy constraints)
        if (this->real_variable_tags[this->real_variable_index[variable_index]] == tag) {
            return;
        }
        ASSERT(this->real_variable_tags[this->real_variable_index[variable_index]] == DUMMY_TAG);
        this->real_variable_tags[this->real_variable_index[variable_index]] = tag;
    }

    uint32_t create_tag(const uint32_t tag_index, const uint32_t tau_index)
    {
        this->tau.insert({ tag_index, tau_index });
        this->current_tag++; // Why exactly?
        return this->current_tag;
    }

    uint32_t get_new_tag()
    {
        this->current_tag++;
        return this->current_tag;
    }

    RangeList create_range_list(const uint64_t target_range);
    void process_range_list(RangeList& list);
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
        const non_native_field_witnesses<FF>& input, const bool range_constrain_quotient_and_remainder = true);
    std::array<uint32_t, 2> queue_partial_non_native_field_multiplication(const non_native_field_witnesses<FF>& input);
    typedef std::pair<uint32_t, FF> scaled_witness;
    typedef std::tuple<scaled_witness, scaled_witness, FF> add_simple;
    std::array<uint32_t, 5> evaluate_non_native_field_subtraction(add_simple limb0,
                                                                  add_simple limb1,
                                                                  add_simple limb2,
                                                                  add_simple limb3,
                                                                  std::tuple<uint32_t, uint32_t, FF> limbp);
    std::array<uint32_t, 5> evaluate_non_native_field_addition(add_simple limb0,
                                                               add_simple limb1,
                                                               add_simple limb2,
                                                               add_simple limb3,
                                                               std::tuple<uint32_t, uint32_t, FF> limbp);

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
    void process_ROM_array(const size_t rom_id);
    void process_ROM_arrays();

    void create_RAM_gate(RamRecord& record);
    void create_sorted_RAM_gate(RamRecord& record);
    void create_final_sorted_RAM_gate(RamRecord& record, const size_t ram_array_size);

    size_t create_RAM_array(const size_t array_size);
    void init_RAM_element(const size_t ram_id, const size_t index_value, const uint32_t value_witness);
    uint32_t read_RAM_array(const size_t ram_id, const uint32_t index_witness);
    void write_RAM_array(const size_t ram_id, const uint32_t index_witness, const uint32_t value_witness);
    void process_RAM_array(const size_t ram_id);
    void process_RAM_arrays();

    // Circuit evaluation methods

    FF compute_arithmetic_identity(FF q_arith_value,
                                   FF q_1_value,
                                   FF q_2_value,
                                   FF q_3_value,
                                   FF q_4_value,
                                   FF q_m_value,
                                   FF q_c_value,
                                   FF w_1_value,
                                   FF w_2_value,
                                   FF w_3_value,
                                   FF w_4_value,
                                   FF w_1_shifted_value,
                                   FF w_4_shifted_value,
                                   const FF alpha_base,
                                   const FF alpha) const;
    FF compute_auxilary_identity(FF q_aux_value,
                                 FF q_arith_value,
                                 FF q_1_value,
                                 FF q_2_value,
                                 FF q_3_value,
                                 FF q_4_value,
                                 FF q_m_value,
                                 FF q_c_value,
                                 FF w_1_value,
                                 FF w_2_value,
                                 FF w_3_value,
                                 FF w_4_value,
                                 FF w_1_shifted_value,
                                 FF w_2_shifted_value,
                                 FF w_3_shifted_value,
                                 FF w_4_shifted_value,
                                 FF alpha_base,
                                 FF alpha,
                                 FF eta) const;
    FF compute_elliptic_identity(FF q_elliptic_value,
                                 FF q_1_value,
                                 FF q_m_value,
                                 FF w_2_value,
                                 FF w_3_value,
                                 FF w_1_shifted_value,
                                 FF w_2_shifted_value,
                                 FF w_3_shifted_value,
                                 FF w_4_shifted_value,
                                 FF alpha_base,
                                 FF alpha) const;
    FF compute_genperm_sort_identity(FF q_sort_value,
                                     FF w_1_value,
                                     FF w_2_value,
                                     FF w_3_value,
                                     FF w_4_value,
                                     FF w_1_shifted_value,
                                     FF alpha_base,
                                     FF alpha) const;

    bool check_circuit();
};
using UltraCircuitBuilder = UltraCircuitBuilder_<arithmetization::Ultra<bb::fr>>;
} // namespace bb

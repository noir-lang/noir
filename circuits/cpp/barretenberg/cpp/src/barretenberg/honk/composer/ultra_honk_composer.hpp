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
        , num_gates(circuit_constructor.num_gates)
    {
        // TODO(#217/#423): Related to issue of ensuring no identically 0 polynomials
        add_gates_to_ensure_all_polys_are_non_zero();
    };

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

    void create_ecc_add_gate(const ecc_add_gate& in) { circuit_constructor.create_ecc_add_gate(in); };

    void create_new_range_constraint(const uint32_t variable_index,
                                     const uint64_t target_range,
                                     std::string const msg = "create_new_range_constraint")
    {
        circuit_constructor.create_new_range_constraint(variable_index, target_range, msg);
    };

    uint32_t put_constant_variable(const barretenberg::fr& variable)
    {
        return circuit_constructor.put_constant_variable(variable);
    };

    void assert_equal(const uint32_t a_variable_idx,
                      const uint32_t b_variable_idx,
                      std::string const& msg = "assert_equal")
    {
        circuit_constructor.assert_equal(a_variable_idx, b_variable_idx, msg);
    }

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
    std::array<uint32_t, 2> evaluate_non_native_field_multiplication(
        const UltraCircuitConstructor::non_native_field_witnesses& input,
        const bool range_constrain_quotient_and_remainder = true)
    {
        return circuit_constructor.evaluate_non_native_field_multiplication(input,
                                                                            range_constrain_quotient_and_remainder);
    };

    // /**
    //  * Memory
    //  **/

    size_t create_RAM_array(const size_t array_size) { return circuit_constructor.create_RAM_array(array_size); };
    size_t create_ROM_array(const size_t array_size) { return circuit_constructor.create_ROM_array(array_size); };

    void set_ROM_element(const size_t rom_id, const size_t index_value, const uint32_t value_witness)
    {
        circuit_constructor.set_ROM_element(rom_id, index_value, value_witness);
    };

    uint32_t read_ROM_array(const size_t rom_id, const uint32_t index_witness)
    {
        return circuit_constructor.read_ROM_array(rom_id, index_witness);
    };

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
};
} // namespace proof_system::honk

#pragma once
#include "barretenberg/proof_system/plookup_tables/plookup_tables.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/proof_system/circuit_constructors/ultra_circuit_constructor.hpp"
#include "barretenberg/proof_system/types/merkle_hash_type.hpp"
#include "barretenberg/proof_system/types/pedersen_commitment_type.hpp"
#include "barretenberg/plonk/composer/composer_helper/ultra_plonk_composer_helper.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"
#include <optional>

namespace proof_system::plonk {

class UltraPlonkComposer {

  public:
    using ComposerHelper = UltraPlonkComposerHelper;
    using CircuitConstructor = UltraCircuitConstructor;

    static constexpr ComposerType type = ComposerType::PLOOKUP;
    static_assert(type == CircuitConstructor::type);
    static constexpr merkle::HashType merkle_hash_type = CircuitConstructor::merkle_hash_type;
    static constexpr pedersen::CommitmentType commitment_type = CircuitConstructor::commitment_type;
    static constexpr size_t DEFAULT_PLOOKUP_RANGE_BITNUM = UltraCircuitConstructor::DEFAULT_PLOOKUP_RANGE_BITNUM;
    // An instantiation of the circuit constructor that only depends on arithmetization, not  on the proof system
    UltraCircuitConstructor circuit_constructor;

    // References to circuit_constructor's members for convenience
    size_t& num_gates;
    std::vector<barretenberg::fr>& variables;
    // While we always have it set to zero, feels wrong to have a potentially broken dependency
    uint32_t& zero_idx;

    // Composer helper contains all proof-related material that is separate from circuit creation such as:
    // 1) Proving and verification keys
    // 2) CRS
    // 3) Converting variables to witness vectors/polynomials
    UltraPlonkComposerHelper composer_helper;

    // References to composer helper's members for convenience
    bool& contains_recursive_proof;
    std::vector<uint32_t>& recursive_proof_public_input_indices;

    UltraPlonkComposer()
        : UltraPlonkComposer("../srs_db/ignition", 0){};

    UltraPlonkComposer(std::string const& crs_path, const size_t size_hint = 0)
        : UltraPlonkComposer(std::make_unique<barretenberg::srs::factories::FileCrsFactory>(crs_path), size_hint){};

    UltraPlonkComposer(std::shared_ptr<barretenberg::srs::factories::CrsFactory> const& crs_factory,
                       const size_t size_hint = 0)
        : circuit_constructor(size_hint)
        , num_gates(circuit_constructor.num_gates)
        , variables(circuit_constructor.variables)
        , zero_idx(circuit_constructor.zero_idx)
        , composer_helper(crs_factory)
        , contains_recursive_proof(circuit_constructor.contains_recursive_proof)
        , recursive_proof_public_input_indices(circuit_constructor.recursive_proof_public_input_indices){};

    UltraPlonkComposer(std::shared_ptr<proving_key> const& p_key,
                       std::shared_ptr<verification_key> const& v_key,
                       size_t size_hint = 0)
        : circuit_constructor(size_hint)
        , num_gates(circuit_constructor.num_gates)
        , variables(circuit_constructor.variables)
        , zero_idx(circuit_constructor.zero_idx)
        , composer_helper(p_key, v_key)
        , contains_recursive_proof(circuit_constructor.contains_recursive_proof)
        , recursive_proof_public_input_indices(circuit_constructor.recursive_proof_public_input_indices){};

    UltraPlonkComposer(UltraPlonkComposer&& other)
        : circuit_constructor(std::move(other.circuit_constructor))
        , num_gates(circuit_constructor.num_gates)
        , variables(circuit_constructor.variables)
        , zero_idx(circuit_constructor.zero_idx)
        , composer_helper(std::move(other.composer_helper))
        , contains_recursive_proof(circuit_constructor.contains_recursive_proof)
        , recursive_proof_public_input_indices(circuit_constructor.recursive_proof_public_input_indices){};
    UltraPlonkComposer& operator=(UltraPlonkComposer&& other)
    {
        circuit_constructor = std::move(other.circuit_constructor);
        composer_helper = std::move(other.composer_helper);
        num_gates = circuit_constructor.num_gates;
        variables = circuit_constructor.variables;
        zero_idx = circuit_constructor.zero_idx;
        contains_recursive_proof = circuit_constructor.contains_recursive_proof;
        recursive_proof_public_input_indices = circuit_constructor.recursive_proof_public_input_indices;
        return *this;
    };
    ~UltraPlonkComposer() = default;

    /**Methods related to circuit construction
     * They simply get proxied to the circuit constructor
     */
    void assert_equal(const uint32_t a_variable_idx,
                      const uint32_t b_variable_idx,
                      std::string const& msg = "assert equal")
    {
        circuit_constructor.assert_equal(a_variable_idx, b_variable_idx, msg);
    }
    void assert_equal_constant(uint32_t const a_idx,
                               barretenberg::fr const& b,
                               std::string const& msg = "assert equal constant")
    {
        circuit_constructor.assert_equal_constant(a_idx, b, msg);
    }

    void create_add_gate(const add_triple& in) { circuit_constructor.create_add_gate(in); }
    void create_mul_gate(const mul_triple& in) { circuit_constructor.create_mul_gate(in); }
    void create_bool_gate(const uint32_t a) { circuit_constructor.create_bool_gate(a); }
    void create_poly_gate(const poly_triple& in) { circuit_constructor.create_poly_gate(in); }
    void create_big_add_gate(const add_quad& in, const bool include_next_gate_w_4 = false)
    {
        circuit_constructor.create_big_add_gate(in, include_next_gate_w_4);
    }
    void create_big_add_gate_with_bit_extraction(const add_quad& in)
    {
        circuit_constructor.create_big_add_gate_with_bit_extraction(in);
    }
    void create_big_mul_gate(const mul_quad& in) { circuit_constructor.create_big_mul_gate(in); }
    void create_balanced_add_gate(const add_quad& in) { circuit_constructor.create_balanced_add_gate(in); }
    void create_ecc_add_gate(const ecc_add_gate& in) { circuit_constructor.create_ecc_add_gate(in); }

    void fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value)
    {
        circuit_constructor.fix_witness(witness_index, witness_value);
    }

    void create_range_constraint(const uint32_t variable_index,
                                 const size_t num_bits,
                                 std::string const& msg = "create_range_constraint")
    {
        circuit_constructor.create_range_constraint(variable_index, num_bits, msg);
    }
    accumulator_triple create_logic_constraint(const uint32_t a,
                                               const uint32_t b,
                                               const size_t num_bits,
                                               bool is_xor_gate)
    {
        return circuit_constructor.create_logic_constraint(a, b, num_bits, is_xor_gate);
    }

    accumulator_triple create_and_constraint(const uint32_t a, const uint32_t b, const size_t num_bits)
    {
        return circuit_constructor.create_and_constraint(a, b, num_bits);
    }

    accumulator_triple create_xor_constraint(const uint32_t a, const uint32_t b, const size_t num_bits)
    {
        return circuit_constructor.create_xor_constraint(a, b, num_bits);
    }
    size_t get_num_gates() const { return circuit_constructor.get_num_gates(); }
    uint32_t get_zero_idx() { return circuit_constructor.zero_idx; }

    uint32_t add_variable(const barretenberg::fr& in) { return circuit_constructor.add_variable(in); }

    uint32_t add_public_variable(const barretenberg::fr& in) { return circuit_constructor.add_public_variable(in); }

    void set_public_input(const uint32_t witness_index) { return circuit_constructor.set_public_input(witness_index); }

    uint32_t get_public_input_index(const uint32_t witness_index) const
    {
        return circuit_constructor.get_public_input_index(witness_index);
    }

    uint32_t put_constant_variable(const barretenberg::fr& variable)
    {
        return circuit_constructor.put_constant_variable(variable);
    }

    size_t get_num_constant_gates() const { return circuit_constructor.get_num_constant_gates(); }

    size_t get_total_circuit_size() const { return circuit_constructor.get_total_circuit_size(); }

    size_t get_circuit_subgroup_size(size_t gates) const
    {
        return circuit_constructor.get_circuit_subgroup_size(gates);
    }

    bool check_circuit() { return circuit_constructor.check_circuit(); }

    barretenberg::fr get_variable(const uint32_t index) const { return circuit_constructor.get_variable(index); }

    std::vector<barretenberg::fr> get_public_inputs() const { return circuit_constructor.get_public_inputs(); }
    void finalize_circuit() { circuit_constructor.finalize_circuit(); };

    void print_num_gates() { circuit_constructor.print_num_gates(); }

    /**Proof and verification-related methods*/

    std::shared_ptr<plonk::proving_key> compute_proving_key()
    {
        return composer_helper.compute_proving_key(circuit_constructor);
    }

    std::shared_ptr<plonk::verification_key> compute_verification_key()
    {
        return composer_helper.compute_verification_key(circuit_constructor);
    }

    UltraProver create_prover() { return composer_helper.create_prover(circuit_constructor); };
    UltraVerifier create_verifier() { return composer_helper.create_verifier(circuit_constructor); };

    UltraToStandardProver create_ultra_to_standard_prover()
    {
        return composer_helper.create_ultra_to_standard_prover(circuit_constructor);
    };

    UltraToStandardVerifier create_ultra_to_standard_verifier()
    {
        return composer_helper.create_ultra_to_standard_verifier(circuit_constructor);
    };

    UltraWithKeccakProver create_ultra_with_keccak_prover()
    {
        return composer_helper.create_ultra_with_keccak_prover(circuit_constructor);
    };
    UltraWithKeccakVerifier create_ultra_with_keccak_verifier()
    {

        return composer_helper.create_ultra_with_keccak_verifier(circuit_constructor);
    };

    static transcript::Manifest create_manifest(const size_t num_public_inputs)
    {
        return UltraPlonkComposerHelper::create_manifest(num_public_inputs);
    }
    void add_recursive_proof(const std::vector<uint32_t>& proof_output_witness_indices)
    {
        circuit_constructor.add_recursive_proof(proof_output_witness_indices);
    }

    void set_recursive_proof(const std::vector<uint32_t>& proof_output_witness_indices)
    {
        circuit_constructor.set_recursive_proof(proof_output_witness_indices);
    }

    void create_new_range_constraint(const uint32_t variable_index,
                                     const uint64_t target_range,
                                     std::string const msg = "create_new_range_constraint")
    {
        circuit_constructor.create_new_range_constraint(variable_index, target_range, msg);
    };

    // /**
    //  * Plookup Methods
    //  **/

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
    std::array<uint32_t, 2> decompose_non_native_field_double_width_limb(
        const uint32_t limb_idx,
        const size_t num_limb_bits = (2 * UltraCircuitConstructor::DEFAULT_NON_NATIVE_FIELD_LIMB_BITS))
    {
        return circuit_constructor.decompose_non_native_field_double_width_limb(limb_idx, num_limb_bits);
    }
    std::array<uint32_t, 2> evaluate_non_native_field_multiplication(
        const UltraCircuitConstructor::non_native_field_witnesses& input,
        const bool range_constrain_quotient_and_remainder = true)
    {
        return circuit_constructor.evaluate_non_native_field_multiplication(input,
                                                                            range_constrain_quotient_and_remainder);
    };

    std::array<uint32_t, 2> queue_partial_non_native_field_multiplication(
        const proof_system::UltraCircuitConstructor::non_native_field_witnesses& input)
    {
        return circuit_constructor.queue_partial_non_native_field_multiplication(input);
    }
    using add_simple = proof_system::UltraCircuitConstructor::add_simple;
    std::array<uint32_t, 5> evaluate_non_native_field_subtraction(
        add_simple limb0,
        add_simple limb1,
        add_simple limb2,
        add_simple limb3,
        std::tuple<uint32_t, uint32_t, barretenberg::fr> limbp)
    {
        return circuit_constructor.evaluate_non_native_field_subtraction(limb0, limb1, limb2, limb3, limbp);
    }
    std::array<uint32_t, 5> evaluate_non_native_field_addition(add_simple limb0,
                                                               add_simple limb1,
                                                               add_simple limb2,
                                                               add_simple limb3,
                                                               std::tuple<uint32_t, uint32_t, barretenberg::fr> limbp)
    {
        return circuit_constructor.evaluate_non_native_field_addition(limb0, limb1, limb2, limb3, limbp);
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
    void set_ROM_element_pair(const size_t rom_id,
                              const size_t index_value,
                              const std::array<uint32_t, 2>& value_witnesses)
    {
        circuit_constructor.set_ROM_element_pair(rom_id, index_value, value_witnesses);
    }
    uint32_t read_ROM_array(const size_t rom_id, const uint32_t index_witness)
    {
        return circuit_constructor.read_ROM_array(rom_id, index_witness);
    };
    std::array<uint32_t, 2> read_ROM_array_pair(const size_t rom_id, const uint32_t index_witness)
    {
        return circuit_constructor.read_ROM_array_pair(rom_id, index_witness);
    }

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

    bool failed() const { return circuit_constructor.failed(); };
    const std::string& err() const { return circuit_constructor.err(); };
    void failure(std::string msg) { circuit_constructor.failure(msg); }
};
} // namespace proof_system::plonk

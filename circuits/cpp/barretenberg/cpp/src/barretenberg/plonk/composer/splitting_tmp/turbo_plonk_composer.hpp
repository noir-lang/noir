#pragma once
#include "composer_helper/turbo_plonk_composer_helper.hpp"
#include "barretenberg/proof_system/circuit_constructors/turbo_circuit_constructor.hpp"
#include "barretenberg/proof_system/types/merkle_hash_type.hpp"
#include "barretenberg/proof_system/types/pedersen_commitment_type.hpp"

namespace proof_system::plonk {
/**
 * @brief Standard Plonk Composer has everything required to construct a prover and verifier, just as the legacy
 * classes.
 *
 * @details However, it has a lot of its logic separated into subclasses and simply proxies the calls.
 *
 */
class TurboPlonkComposer {
  public:
    static constexpr ComposerType type = ComposerType::STANDARD;
    static constexpr merkle::HashType merkle_hash_type = merkle::HashType::FIXED_BASE_PEDERSEN;
    static constexpr pedersen::CommitmentType commitment_type = pedersen::CommitmentType::FIXED_BASE_PEDERSEN;

    static constexpr size_t UINT_LOG2_BASE = 2;

    // An instantiation of the circuit constructor that only depends on arithmetization, not  on the proof system
    TurboCircuitConstructor circuit_constructor;
    // Composer helper contains all proof-related material that is separate from circuit creation such as:
    // 1) Proving and verification keys
    // 2) CRS
    // 3) Converting variables to witness vectors/polynomials
    TurboPlonkComposerHelper composer_helper;
    size_t& num_gates;
    std::vector<barretenberg::fr>& variables;

    // Leaving it in for now just in case
    bool contains_recursive_proof = false;
    static constexpr size_t program_width = TurboCircuitConstructor::program_width;

    /**Standard methods*/

    TurboPlonkComposer(const size_t size_hint = 0)
        : circuit_constructor(size_hint)
        , num_gates(circuit_constructor.num_gates)
        , variables(circuit_constructor.variables){};

    TurboPlonkComposer(std::string const& crs_path, const size_t size_hint = 0)
        : TurboPlonkComposer(
              std::unique_ptr<ReferenceStringFactory>(new proof_system::FileReferenceStringFactory(crs_path)),
              size_hint){};

    TurboPlonkComposer(std::shared_ptr<ReferenceStringFactory> const& crs_factory, const size_t size_hint = 0)
        : circuit_constructor(size_hint)
        , composer_helper(crs_factory)
        , num_gates(circuit_constructor.num_gates)
        , variables(circuit_constructor.variables){};

    TurboPlonkComposer(std::unique_ptr<ReferenceStringFactory>&& crs_factory, const size_t size_hint = 0)
        : circuit_constructor(size_hint)
        , composer_helper(std::move(crs_factory))
        , num_gates(circuit_constructor.num_gates)
        , variables(circuit_constructor.variables){};

    TurboPlonkComposer(std::shared_ptr<plonk::proving_key> const& p_key,
                       std::shared_ptr<plonk::verification_key> const& v_key,
                       size_t size_hint = 0)
        : circuit_constructor(size_hint)
        , composer_helper(p_key, v_key)
        , num_gates(circuit_constructor.num_gates)
        , variables(circuit_constructor.variables){};

    TurboPlonkComposer(const TurboPlonkComposer& other) = delete;
    TurboPlonkComposer(TurboPlonkComposer&& other) = default;
    TurboPlonkComposer& operator=(const TurboPlonkComposer& other) = delete;
    // TODO(#230)(Cody): This constructor started to be implicitly deleted when I added `n` and `variables` members.
    // This is a temporary measure until we can rewrite Plonk and all tests using circuit builder methods in place of
    // composer methods, where appropriate. TurboPlonkComposer& operator=(TurboPlonkComposer&& other) = default;
    ~TurboPlonkComposer() = default;

    /**Methods related to circuit construction
     * They simply get proxied to the circuit constructor
     */

    size_t get_num_gates() const { return circuit_constructor.get_num_gates(); }

    void assert_equal(const uint32_t a_variable_idx, const uint32_t b_variable_idx, std::string const& msg)
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
    void create_fixed_group_add_gate(const fixed_group_add_quad& in)
    {
        circuit_constructor.create_fixed_group_add_gate(in);
    }
    void create_fixed_group_add_gate_with_init(const fixed_group_add_quad& in, const fixed_group_init_quad& init)
    {
        circuit_constructor.create_fixed_group_add_gate_with_init(in, init);
    }
    void create_fixed_group_add_gate_final(const add_quad& in)
    {
        circuit_constructor.create_fixed_group_add_gate_final(in);
    }
    void create_big_add_gate(const add_quad& in) { circuit_constructor.create_big_add_gate(in); }
    void create_big_add_gate_with_bit_extraction(const add_quad& in)
    {
        circuit_constructor.create_big_add_gate_with_bit_extraction(in);
    }
    void create_big_mul_gate(const mul_quad& in) { circuit_constructor.create_big_mul_gate(in); }
    void create_balanced_add_gate(const add_quad& in) { circuit_constructor.create_balanced_add_gate(in); }

    void fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value)
    {
        circuit_constructor.fix_witness(witness_index, witness_value);
    }

    std::vector<uint32_t> decompose_into_base4_accumulators(const uint32_t witness_index,

                                                            const size_t num_bits,
                                                            std::string const& msg = "create_range_constraint")
    {
        return circuit_constructor.decompose_into_base4_accumulators(witness_index, num_bits, msg);
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
    uint32_t add_variable(const barretenberg::fr& in) { return circuit_constructor.add_variable(in); }

    uint32_t add_public_variable(const barretenberg::fr& in) { return circuit_constructor.add_public_variable(in); }

    virtual void set_public_input(const uint32_t witness_index)
    {
        return circuit_constructor.set_public_input(witness_index);
    }

    uint32_t put_constant_variable(const barretenberg::fr& variable)
    {
        return circuit_constructor.put_constant_variable(variable);
    }

    size_t get_num_constant_gates() const { return circuit_constructor.get_num_constant_gates(); }

    bool check_circuit() { return circuit_constructor.check_circuit(); }

    barretenberg::fr get_variable(const uint32_t index) const { return circuit_constructor.get_variable(index); }
    /**Proof and verification-related methods*/

    std::shared_ptr<plonk::proving_key> compute_proving_key()
    {
        return composer_helper.compute_proving_key(circuit_constructor);
    }

    std::shared_ptr<plonk::verification_key> compute_verification_key()
    {
        return composer_helper.compute_verification_key(circuit_constructor);
    }

    uint32_t zero_idx = 0;

    void compute_witness() { composer_helper.compute_witness(circuit_constructor); };
    // TODO(#230)(Cody): This will not be needed, but maybe something is required for ComposerHelper to be generic?
    plonk::TurboVerifier create_verifier() { return composer_helper.create_verifier(circuit_constructor); }
    /**
     * Preprocess the circuit. Delegates to create_prover.
     *
     * @return A new initialized prover.
     */
    /**
     * Preprocess the circuit. Delegates to create_unrolled_prover.
     *
     * @return A new initialized prover.
     */
    plonk::TurboProver create_prover() { return composer_helper.create_prover(circuit_constructor); };

    static transcript::Manifest create_manifest(const size_t num_public_inputs)
    {
        return TurboPlonkComposerHelper::create_manifest(num_public_inputs);
    }

    bool failed() const { return circuit_constructor.failed(); };
    const std::string& err() const { return circuit_constructor.err(); };
    void failure(std::string msg) { circuit_constructor.failure(msg); }
};
} // namespace proof_system::plonk

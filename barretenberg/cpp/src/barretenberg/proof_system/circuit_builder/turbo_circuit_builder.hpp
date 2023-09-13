#pragma once
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/proof_system/types/circuit_type.hpp"
#include "barretenberg/proof_system/types/merkle_hash_type.hpp"
#include "barretenberg/proof_system/types/pedersen_commitment_type.hpp"
#include "circuit_builder_base.hpp"
#include <array>

namespace proof_system {

inline std::vector<std::string> turbo_selector_names()
{
    std::vector<std::string> result{ "q_m", "q_c",     "q_1",          "q_2",     "q_3",    "q_4",
                                     "q_5", "q_arith", "q_fixed_base", "q_range", "q_logic" };
    return result;
}
template <typename FF> class TurboCircuitBuilder_ : public CircuitBuilderBase<arithmetization::Turbo<FF>> {

  public:
    static constexpr std::string_view NAME_STRING = "TurboArithmetization";
    static constexpr CircuitType CIRCUIT_TYPE = CircuitType::TURBO;
    // TODO(#563): make issue; these belong in plonk::flavor::Turbo.
    static constexpr merkle::HashType merkle_hash_type = merkle::HashType::FIXED_BASE_PEDERSEN;
    static constexpr pedersen::CommitmentType commitment_type = pedersen::CommitmentType::FIXED_BASE_PEDERSEN;
    static constexpr size_t UINT_LOG2_BASE = 2;

    using WireVector = std::vector<uint32_t, barretenberg::ContainerSlabAllocator<uint32_t>>;
    using SelectorVector = std::vector<FF, barretenberg::ContainerSlabAllocator<FF>>;

    WireVector& w_l = std::get<0>(this->wires);
    WireVector& w_r = std::get<1>(this->wires);
    WireVector& w_o = std::get<2>(this->wires);
    WireVector& w_4 = std::get<3>(this->wires);

    SelectorVector& q_m = this->selectors.q_m;
    SelectorVector& q_c = this->selectors.q_c;
    SelectorVector& q_1 = this->selectors.q_1;
    SelectorVector& q_2 = this->selectors.q_2;
    SelectorVector& q_3 = this->selectors.q_3;
    SelectorVector& q_4 = this->selectors.q_4;
    SelectorVector& q_5 = this->selectors.q_5;
    SelectorVector& q_arith = this->selectors.q_arith;
    SelectorVector& q_fixed_base = this->selectors.q_fixed_base;
    SelectorVector& q_range = this->selectors.q_range;
    SelectorVector& q_logic = this->selectors.q_logic;

    TurboCircuitBuilder_(const size_t size_hint = 0);
    TurboCircuitBuilder_(TurboCircuitBuilder_&& other) = default;
    TurboCircuitBuilder_& operator=(TurboCircuitBuilder_&& other)
    {
        CircuitBuilderBase<arithmetization::Turbo<FF>>::operator=(std::move(other));
        constant_variable_indices = other.constant_variable_indices;
        return *this;
    };
    ~TurboCircuitBuilder_() {}

    void create_add_gate(const add_triple_<FF>& in);

    void create_big_add_gate(const add_quad_<FF>& in);
    void create_big_add_gate_with_bit_extraction(const add_quad_<FF>& in);
    void create_big_mul_gate(const mul_quad_<FF>& in);
    void create_balanced_add_gate(const add_quad_<FF>& in);

    void create_mul_gate(const mul_triple_<FF>& in);
    void create_bool_gate(const uint32_t a);
    void create_poly_gate(const poly_triple_<FF>& in);
    void create_fixed_group_add_gate(const fixed_group_add_quad_<FF>& in);
    void create_fixed_group_add_gate_with_init(const fixed_group_add_quad_<FF>& in,
                                               const fixed_group_init_quad_<FF>& init);
    void create_fixed_group_add_gate_final(const add_quad_<FF>& in);
    void fix_witness(const uint32_t witness_index, const FF& witness_value);

    FF arithmetic_gate_evaluation(const size_t index, const FF alpha_base);
    FF fixed_base_gate_evaluation(const size_t index, const std::vector<FF>& alpha_powers);
    FF logic_gate_evaluation(const size_t index, const FF alpha_bas, const FF alpha);
    FF range_gate_evaluation(const size_t index, const FF alpha_bas, const FF alpha);

    bool lazy_arithmetic_gate_check(const size_t gate_index);
    bool lazy_fixed_base_gate_check(const size_t gate_index);
    bool lazy_logic_gate_check(const size_t gate_index);
    bool lazy_range_gate_check(const size_t gate_index);

    bool check_circuit();

    std::vector<uint32_t> decompose_into_base4_accumulators(const uint32_t witness_index,
                                                            const size_t num_bits,
                                                            std::string const& msg);

    void create_range_constraint(const uint32_t variable_index, const size_t num_bits, std::string const& msg)
    {
        decompose_into_base4_accumulators(variable_index, num_bits, msg);
    }

    accumulator_triple_<FF> create_logic_constraint(const uint32_t a,
                                                    const uint32_t b,
                                                    const size_t num_bits,
                                                    bool is_xor_gate);
    accumulator_triple_<FF> create_and_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);
    accumulator_triple_<FF> create_xor_constraint(const uint32_t a, const uint32_t b, const size_t num_bits);

    uint32_t put_constant_variable(const FF& variable);

    size_t get_num_constant_gates() const { return 0; }

    void assert_equal_constant(const uint32_t a_idx, const FF& b, std::string const& msg = "assert_equal_constant")
    {
        if (this->variables[a_idx] != b && !this->failed()) {
            this->failure(msg);
        }
        auto b_idx = put_constant_variable(b);
        this->assert_equal(a_idx, b_idx, msg);
    }

    /**
     * For any type other than uint32_t (presumed to be a witness index), we call normalize first.
     */
    template <typename T>
    void assert_equal_constant(T const& in, const FF& b, std::string const& msg = "assert_equal_constant")
    {
        assert_equal_constant(in.normalize().witness_index, b, msg);
    }

    // these are variables that we have used a gate on, to enforce that they are equal to a defined value
    std::map<FF, uint32_t> constant_variable_indices;
};
extern template class TurboCircuitBuilder_<barretenberg::fr>;
using TurboCircuitBuilder = TurboCircuitBuilder_<barretenberg::fr>;
} // namespace proof_system

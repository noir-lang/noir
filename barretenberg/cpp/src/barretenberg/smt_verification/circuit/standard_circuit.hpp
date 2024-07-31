#pragma once
#include "circuit_base.hpp"

namespace smt_circuit {

/**
 * @brief Symbolic Circuit class for Standard Circuit Builder.
 *
 * @details Contains all the information about the circuit: gates, variables,
 * symbolic variables, specified names and global solver.
 */
class StandardCircuit : public CircuitBase {
  public:
    std::vector<std::vector<bb::fr>> selectors;    // selectors from the circuit
    std::vector<std::vector<uint32_t>> wires_idxs; // values of the gates' wires

    explicit StandardCircuit(CircuitSchema& circuit_info,
                             Solver* solver,
                             TermType type = TermType::FFTerm,
                             const std::string& tag = "",
                             bool enable_optimizations = true);

    inline size_t get_num_gates() const { return selectors.size(); };

    size_t prepare_gates(size_t cursor);
    bool simulate_circuit_eval(std::vector<bb::fr>& witness) const override;

    void handle_univariate_constraint(bb::fr q_m, bb::fr q_1, bb::fr q_2, bb::fr q_3, bb::fr q_c, uint32_t w);
    size_t handle_logic_constraint(size_t cursor);
    size_t handle_range_constraint(size_t cursor);
    size_t handle_ror_constraint(size_t cursor);
    size_t handle_shr_constraint(size_t cursor);
    size_t handle_shl_constraint(size_t cursor);

    static std::pair<StandardCircuit, StandardCircuit> unique_witness_ext(
        CircuitSchema& circuit_info,
        Solver* s,
        TermType type,
        const std::vector<std::string>& equal = {},
        const std::vector<std::string>& not_equal = {},
        const std::vector<std::string>& equal_at_the_same_time = {},
        const std::vector<std::string>& not_equal_at_the_same_time = {},
        bool enable_optimizations = false);

    static std::pair<StandardCircuit, StandardCircuit> unique_witness(CircuitSchema& circuit_info,
                                                                      Solver* s,
                                                                      TermType type,
                                                                      const std::vector<std::string>& equal = {},
                                                                      bool enable_optimizations = false);
};
}; // namespace smt_circuit
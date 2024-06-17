#pragma once
#include "circuit_base.hpp"

namespace smt_circuit {

/**
 * @brief Symbolic Circuit class for Standard Circuit Builder.
 *
 * @details Contains all the information about the circuit: gates, variables,
 * symbolic variables, specified names and global solver.
 */
class UltraCircuit : public CircuitBase {
  public:
    // TODO(alex): check that there's no actual pub_inputs block
    std::vector<std::vector<std::vector<bb::fr>>> selectors;    // all selectors from the circuit
                                                                // 1st entry are arithmetic selectors
                                                                // 2nd entry are delta_range selectors
                                                                // 3rd entry are elliptic selectors
                                                                // 4th entry are aux selectors
                                                                // 5th entry are lookup selectors
    std::vector<std::vector<std::vector<uint32_t>>> wires_idxs; // values of the gates' wires idxs

    std::vector<std::vector<std::vector<bb::fr>>> lookup_tables;
    std::unordered_map<uint32_t, cvc5::Term> cached_symbolic_tables;
    std::unordered_map<uint64_t, cvc5::Term> cached_range_tables;

    explicit UltraCircuit(CircuitSchema& circuit_info,
                          Solver* solver,
                          TermType type = TermType::FFTerm,
                          const std::string& tag = "",
                          bool optimizations = true);
    UltraCircuit(const UltraCircuit& other) = default;
    UltraCircuit(UltraCircuit&& other) = default;
    UltraCircuit& operator=(const UltraCircuit& other) = default;
    UltraCircuit& operator=(UltraCircuit&& other) = default;
    ~UltraCircuit() override = default;

    inline size_t get_num_gates() const
    {
        return selectors[0].size() + selectors[1].size() + selectors[2].size() + selectors[3].size() +
               selectors[4].size() + selectors[5].size();
    };

    bool simulate_circuit_eval(std::vector<bb::fr>& witness) const override;

    size_t handle_arithmetic_relation(size_t cursor, size_t idx);
    size_t handle_lookup_relation(size_t cursor, size_t idx);
    size_t handle_elliptic_relation(size_t cursor, size_t idx);
    size_t handle_delta_range_relation(size_t cursor, size_t idx);
    void handle_range_constraints();

    static std::pair<UltraCircuit, UltraCircuit> unique_witness_ext(
        CircuitSchema& circuit_info,
        Solver* s,
        TermType type,
        const std::vector<std::string>& equal = {},
        const std::vector<std::string>& not_equal = {},
        const std::vector<std::string>& equal_at_the_same_time = {},
        const std::vector<std::string>& not_equal_at_the_same_time = {});
    static std::pair<UltraCircuit, UltraCircuit> unique_witness(CircuitSchema& circuit_info,
                                                                Solver* s,
                                                                TermType type,
                                                                const std::vector<std::string>& equal = {});
};
}; // namespace smt_circuit
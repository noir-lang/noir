#pragma once
#include <limits>
#include <sstream>
#include <string>
#include <unordered_map>

#include "barretenberg/smt_verification/terms/bool.hpp"
#include "barretenberg/smt_verification/terms/term.hpp"

#include "subcircuits.hpp"

namespace smt_circuit {
using namespace smt_solver;
using namespace smt_terms;
using namespace smt_circuit_schema;
using namespace smt_subcircuits;

enum class SubcircuitType { XOR, AND, RANGE };

/**
 * @brief Symbolic Circuit class.
 *
 * @details Contains all the information about the circuit: gates, variables,
 * symbolic variables, specified names and global solver.
 */
class Circuit {
  private:
    void init();
    size_t prepare_gates(size_t cursor);

    void handle_univariate_constraint(bb::fr q_m, bb::fr q_1, bb::fr q_2, bb::fr q_3, bb::fr q_c, uint32_t w);
    size_t handle_logic_constraint(size_t cursor);
    size_t handle_range_constraint(size_t cursor);

  public:
    std::vector<bb::fr> variables;                                    // circuit witness
    std::vector<uint32_t> public_inps;                                // public inputs from the circuit
    std::unordered_map<uint32_t, std::string> variable_names;         // names of the variables
    std::unordered_map<std::string, uint32_t> variable_names_inverse; // inverse map of the previous memeber
    std::vector<std::vector<bb::fr>> selectors;                       // selectors from the circuit
    std::vector<std::vector<uint32_t>> wires_idxs;                    // values of the gates' wires
    std::unordered_map<uint32_t, STerm> symbolic_vars;                // all the symbolic variables from the circuit
    std::vector<uint32_t> real_variable_index;                        // indexes for assert_equal'd wires
    std::unordered_map<uint32_t, bool> optimized; // keeps track of the variables that were excluded from symbolic
                                                  // circuit during optimizations
    bool optimizations;                           // flags to turn on circuit optimizations
    std::unordered_map<SubcircuitType, std::unordered_map<size_t, CircuitProps>>
        cached_subcircuits; // caches subcircuits during optimization
                            // No need to recompute them each time

    Solver* solver; // pointer to the solver
    TermType type;  // Type of the underlying Symbolic Terms

    std::string tag; // tag of the symbolic circuit.
                     // If not empty, will be added to the names
                     // of symbolic variables to prevent collisions.

    explicit Circuit(CircuitSchema& circuit_info,
                     Solver* solver,
                     TermType type = TermType::FFTerm,
                     const std::string& tag = "",
                     bool optimizations = true);

    STerm operator[](const std::string& name);
    STerm operator[](const uint32_t& idx) { return this->symbolic_vars[this->real_variable_index[idx]]; };
    inline size_t get_num_gates() const { return selectors.size(); };
    inline size_t get_num_real_vars() const { return symbolic_vars.size(); };
    inline size_t get_num_vars() const { return variables.size(); };

    bool simulate_circuit_eval(std::vector<bb::fr>& witness) const;
};

std::pair<Circuit, Circuit> unique_witness_ext(CircuitSchema& circuit_info,
                                               Solver* s,
                                               TermType type,
                                               const std::vector<std::string>& equal = {},
                                               const std::vector<std::string>& not_equal = {},
                                               const std::vector<std::string>& equal_at_the_same_time = {},
                                               const std::vector<std::string>& not_equal_at_the_same_time = {});

std::pair<Circuit, Circuit> unique_witness(CircuitSchema& circuit_info,
                                           Solver* s,
                                           TermType type,
                                           const std::vector<std::string>& equal = {});

}; // namespace smt_circuit

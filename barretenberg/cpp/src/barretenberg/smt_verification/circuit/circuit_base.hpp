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

enum class SubcircuitType { XOR, AND, RANGE, ROR, SHL, SHR };

/**
 * @brief Base class for symbolic circuits
 *
 * @details Contains all the information about the circuit: gates, variables,
 * symbolic variables, specified names, global solver and optimiztaions.
 *
 */
class CircuitBase {
  public:
    std::vector<bb::fr> variables;                                    // circuit witness
    std::vector<uint32_t> public_inps;                                // public inputs from the circuit
    std::unordered_map<uint32_t, std::string> variable_names;         // names of the variables
    std::unordered_map<std::string, uint32_t> variable_names_inverse; // inverse map of the previous memeber
    std::unordered_map<uint32_t, STerm> symbolic_vars;                // all the symbolic variables from the circuit
    std::vector<uint32_t> real_variable_index;                        // indexes for assert_equal'd wires
    std::vector<uint32_t> real_variable_tags;                         // tags of the variables in the circuit
    std::unordered_map<uint32_t, uint64_t> range_tags;                // ranges associated with a certain tag
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

    CircuitBase(std::unordered_map<uint32_t, std::string>& variable_names,
                std::vector<bb::fr>& variables,
                std::vector<uint32_t>& public_inps,
                std::vector<uint32_t>& real_variable_index,
                std::vector<uint32_t>& real_variable_tags,
                std::unordered_map<uint32_t, uint64_t>& range_tags,
                Solver* solver,
                TermType type,
                const std::string& tag = "",
                bool optimizations = true);

    STerm operator[](const std::string& name);
    STerm operator[](const uint32_t& idx) { return this->symbolic_vars[this->real_variable_index[idx]]; };
    inline size_t get_num_real_vars() const { return symbolic_vars.size(); };
    inline size_t get_num_vars() const { return variables.size(); };

    void init();
    virtual bool simulate_circuit_eval(std::vector<bb::fr>& witness) const = 0;

    CircuitBase(const CircuitBase& other) = default;
    CircuitBase(CircuitBase&& other) noexcept = default;
    CircuitBase& operator=(const CircuitBase& other) = default;
    CircuitBase& operator=(CircuitBase&& other) noexcept = default;
    virtual ~CircuitBase() = default;
};

}; // namespace smt_circuit
#pragma once
#include <limits>
#include <sstream>
#include <string>
#include <unordered_map>

#include "barretenberg/smt_verification/terms/bool.hpp"
#include "barretenberg/smt_verification/terms/ffiterm.hpp"
#include "barretenberg/smt_verification/terms/ffterm.hpp"

#include "subcircuits.hpp"

namespace smt_circuit {
using namespace smt_solver;
using namespace smt_terms;
using namespace smt_circuit_schema;
using namespace smt_subcircuits;

/**
 * @brief Symbolic Circuit class.
 *
 * @details Contains all the information about the circuit: gates, variables,
 * symbolic variables, specified names and global solver.
 *
 * @tparam FF FFTerm or FFITerm
 */
template <typename FF> class Circuit {
  private:
    void init();

    size_t prepare_gates(size_t cursor);
    void univariate_handler(bb::fr q_m, bb::fr q_1, bb::fr q_2, bb::fr q_3, bb::fr q_c, uint32_t w);

  public:
    std::vector<bb::fr> variables;                                    // circuit witness
    std::vector<uint32_t> public_inps;                                // public inputs from the circuit
    std::unordered_map<uint32_t, std::string> variable_names;         // names of the variables
    std::unordered_map<std::string, uint32_t> variable_names_inverse; // inverse map of the previous memeber
    std::vector<std::vector<bb::fr>> selectors;                       // selectors from the circuit
    std::vector<std::vector<uint32_t>> wires_idxs;                    // values of the gates' wires
    std::unordered_map<uint32_t, FF> symbolic_vars;                   // all the symbolic variables from the circuit
    std::vector<uint32_t> real_variable_index;                        // indexes for assert_equal'd wires

    Solver* solver;  // pointer to the solver
    std::string tag; // tag of the symbolic circuit.
                     // If not empty, will be added to the names
                     // of symbolic variables to prevent collisions.

    explicit Circuit(CircuitSchema& circuit_info, Solver* solver, const std::string& tag = "");

    FF operator[](const std::string& name);
    FF operator[](const uint32_t& idx) { return symbolic_vars[this->real_variable_index[idx]]; };
    inline size_t get_num_gates() const { return selectors.size(); };
    inline size_t get_num_real_vars() const { return symbolic_vars.size(); };
    inline size_t get_num_vars() const { return variables.size(); };

    bool simulate_circuit_eval(std::vector<bb::fr>& witness) const;
};

/**
 * @brief Construct a new Circuit::Circuit object
 *
 * @param circuit_info CircuitShema object
 * @param solver pointer to the global solver
 * @param tag tag of the circuit. Empty by default.
 */
template <typename FF>
Circuit<FF>::Circuit(CircuitSchema& circuit_info, Solver* solver, const std::string& tag)
    : variables(circuit_info.variables)
    , public_inps(circuit_info.public_inps)
    , variable_names(circuit_info.vars_of_interest)
    , selectors(circuit_info.selectors)
    , wires_idxs(circuit_info.wires)
    , real_variable_index(circuit_info.real_variable_index)
    , solver(solver)
    , tag(tag)
{
    if (!this->tag.empty()) {
        if (this->tag[0] != '_') {
            this->tag = "_" + this->tag;
        }
    }

    for (auto& x : variable_names) {
        variable_names_inverse.insert({ x.second, x.first });
    }

    variable_names.insert({ 0, "zero" });
    variable_names.insert({ 1, "one" });
    variable_names_inverse.insert({ "zero", 0 });
    variable_names_inverse.insert({ "one", 1 });

    this->init();

    // Perform all relaxation for gates or
    // add gate in its normal state to solver
    size_t i = 0;
    while (i < this->get_num_gates()) {
        i = this->prepare_gates(i);
    }
}

/**
 * Creates all the needed symbolic variables and constants
 * which are used in circuit.
 *
 */
template <typename FF> void Circuit<FF>::init()
{
    size_t num_vars = variables.size();

    symbolic_vars.insert({ 0, FF::Var("zero" + this->tag, this->solver) });
    symbolic_vars.insert({ 1, FF::Var("one" + this->tag, this->solver) });

    for (uint32_t i = 2; i < num_vars; i++) {
        uint32_t real_idx = this->real_variable_index[i];
        if (this->symbolic_vars.contains(real_idx)) {
            continue;
        }

        if (variable_names.contains(real_idx)) {
            std::string name = variable_names[real_idx];
            symbolic_vars.insert({ real_idx, FF::Var(name + this->tag, this->solver) });
        } else {
            symbolic_vars.insert({ real_idx, FF::Var("var_" + std::to_string(i) + this->tag, this->solver) });
        }
    }

    symbolic_vars[0] == bb::fr(0);
    symbolic_vars[1] == bb::fr(1);
}

/**
 * @brief Relaxes univariate polynomial constraints.
 * TODO(alex): probably won't be necessary in the nearest future
 * because of new solver
 *
 * @param q_m multiplication selector
 * @param q_1 l selector
 * @param q_2 r selector
 * @param q_3 o selector
 * @param q_c constant
 * @param w   witness index
 */
template <typename FF>
void Circuit<FF>::univariate_handler(bb::fr q_m, bb::fr q_1, bb::fr q_2, bb::fr q_3, bb::fr q_c, uint32_t w)
{
    bb::fr b = q_1 + q_2 + q_3;

    if (q_m == 0) {
        symbolic_vars[w] == -q_c / b;
        return;
    }

    std::pair<bool, bb::fr> d = (b * b - bb::fr(4) * q_m * q_c).sqrt();
    if (!d.first) {
        throw std::invalid_argument("There're no roots of quadratic polynomial");
    }
    bb::fr x1 = (-b + d.second) / (bb::fr(2) * q_m);
    bb::fr x2 = (-b - d.second) / (bb::fr(2) * q_m);

    if (d.second == 0) {
        symbolic_vars[w] == FF(x1, this->solver);
    } else {
        ((Bool(symbolic_vars[w]) == Bool(FF(x1, this->solver))) |
         (Bool(symbolic_vars[w]) == Bool(FF(x2, this->solver))))
            .assert_term();
    }
}

/**
 * @brief Adds all the gate constraints to the solver.
 * Relaxes constraint system for non-ff solver engines
 * via removing subcircuits that were already proved being correct.
 *
 */
template <typename FF> size_t Circuit<FF>::prepare_gates(size_t cursor)
{
    // TODO(alex): Here'll be the operator relaxation that is coming
    // in the next pr

    bb::fr q_m = this->selectors[cursor][0];
    bb::fr q_1 = this->selectors[cursor][1];
    bb::fr q_2 = this->selectors[cursor][2];
    bb::fr q_3 = this->selectors[cursor][3];
    bb::fr q_c = this->selectors[cursor][4];

    uint32_t w_l = this->wires_idxs[cursor][0];
    uint32_t w_r = this->wires_idxs[cursor][1];
    uint32_t w_o = this->wires_idxs[cursor][2];

    bool univariate_flag = w_l == w_r && w_r == w_o;
    univariate_flag |= w_l == w_r && q_3 == 0;
    univariate_flag |= w_l == w_o && q_2 == 0 && q_m == 0;
    univariate_flag |= w_r == w_o && q_1 == 0 && q_m == 0;
    univariate_flag |= q_m == 0 && q_1 == 0 && q_3 == 0;
    univariate_flag |= q_m == 0 && q_2 == 0 && q_3 == 0;
    univariate_flag |= q_m == 0 && q_1 == 0 && q_2 == 0;

    // Univariate gate. Relaxes the solver. Or is it?
    // TODO(alex): Test the effect of this relaxation after the tests are merged.
    if (univariate_flag) {
        if (q_m == 1 && q_1 == 0 && q_2 == 0 && q_3 == -1 && q_c == 0) {
            (Bool(symbolic_vars[w_l]) == Bool(symbolic_vars[0]) | Bool(symbolic_vars[w_l]) == Bool(symbolic_vars[1]))
                .assert_term();
        } else {
            this->univariate_handler(q_m, q_1, q_2, q_3, q_c, w_l);
        }
    } else {
        FF eq = symbolic_vars[0];

        // mul selector
        if (q_m != 0) {
            eq += symbolic_vars[w_l] * symbolic_vars[w_r] * q_m; // TODO(alex): Is there a way to do lmul?
        }
        // left selector
        if (q_1 != 0) {
            eq += symbolic_vars[w_l] * q_1;
        }
        // right selector
        if (q_2 != 0) {
            eq += symbolic_vars[w_r] * q_2;
        }
        // out selector
        if (q_3 != 0) {
            eq += symbolic_vars[w_o] * q_3;
        }
        // constant selector
        if (q_c != 0) {
            eq += q_c;
        }
        eq == symbolic_vars[0];
    }
    return cursor + 1;
}

/**
 * @brief Returns a previously named symbolic variable.
 *
 * @param name
 * @return FF
 */
template <typename FF> FF Circuit<FF>::operator[](const std::string& name)
{
    if (!this->variable_names_inverse.contains(name)) {
        throw std::invalid_argument("No such an item `" + name + "` in vars or it vas not declared as interesting");
    }
    uint32_t idx = this->variable_names_inverse[name];
    return this->symbolic_vars[idx];
}

/**
 * @brief Similar functionality to old .check_circuit() method
 * in standard circuit builder.
 *
 * @param witness
 * @return true
 * @return false
 */
template <typename FF> bool Circuit<FF>::simulate_circuit_eval(std::vector<bb::fr>& witness) const
{
    if (witness.size() != this->get_num_vars()) {
        throw std::invalid_argument("Witness size should be " + std::to_string(this->get_num_vars()) + ", not " +
                                    std::to_string(witness.size()));
    }
    for (size_t i = 0; i < this->selectors.size(); i++) {
        bb::fr res = 0;
        bb::fr x = witness[this->wires_idxs[i][0]];
        bb::fr y = witness[this->wires_idxs[i][1]];
        bb::fr o = witness[this->wires_idxs[i][2]];
        res += this->selectors[i][0] * x * y;
        res += this->selectors[i][1] * x;
        res += this->selectors[i][2] * y;
        res += this->selectors[i][3] * o;
        res += this->selectors[i][4];
        if (res != 0) {
            return false;
        }
    }
    return true;
}

template <typename FF>
std::pair<Circuit<FF>, Circuit<FF>> unique_witness_ext(CircuitSchema& circuit_info,
                                                       Solver* s,
                                                       const std::vector<std::string>& equal = {},
                                                       const std::vector<std::string>& not_equal = {},
                                                       const std::vector<std::string>& equal_at_the_same_time = {},
                                                       const std::vector<std::string>& not_equal_at_the_same_time = {});

extern template std::pair<Circuit<FFTerm>, Circuit<FFTerm>> unique_witness_ext(
    CircuitSchema& circuit_info,
    Solver* s,
    const std::vector<std::string>& equal = {},
    const std::vector<std::string>& not_equal = {},
    const std::vector<std::string>& equal_at_the_same_time = {},
    const std::vector<std::string>& not_equal_at_the_same_time = {});

extern template std::pair<Circuit<FFITerm>, Circuit<FFITerm>> unique_witness_ext(
    CircuitSchema& circuit_info,
    Solver* s,
    const std::vector<std::string>& equal = {},
    const std::vector<std::string>& not_equal = {},
    const std::vector<std::string>& equal_at_the_same_time = {},
    const std::vector<std::string>& not_equal_at_the_same_time = {});

template <typename FF>
std::pair<Circuit<FF>, Circuit<FF>> unique_witness(CircuitSchema& circuit_info,
                                                   Solver* s,
                                                   const std::vector<std::string>& equal = {});

extern template std::pair<Circuit<FFTerm>, Circuit<FFTerm>> unique_witness(CircuitSchema& circuit_info,
                                                                           Solver* s,
                                                                           const std::vector<std::string>& equal = {});

extern template std::pair<Circuit<FFITerm>, Circuit<FFITerm>> unique_witness(
    CircuitSchema& circuit_info, Solver* s, const std::vector<std::string>& equal = {});

}; // namespace smt_circuit

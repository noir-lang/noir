#pragma once
#include <fstream>
#include <limits>
#include <sstream>
#include <string>
#include <unordered_map>

#include "barretenberg/serialize/cbind.hpp"
#include "barretenberg/serialize/msgpack.hpp"

#include "barretenberg/smt_verification/terms/bool.hpp"
#include "barretenberg/smt_verification/terms/ffiterm.hpp"
#include "barretenberg/smt_verification/terms/ffterm.hpp"

namespace smt_circuit {
using namespace smt_solver;
using namespace smt_terms;

struct CircuitSchema {
    std::string modulus;
    std::vector<uint32_t> public_inps;
    std::unordered_map<uint32_t, std::string> vars_of_interest;
    std::vector<bb::fr> variables;
    std::vector<std::vector<bb::fr>> selectors;
    std::vector<std::vector<uint32_t>> wires;
    MSGPACK_FIELDS(modulus, public_inps, vars_of_interest, variables, selectors, wires);
};

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
    void add_gates();

  public:
    std::vector<std::string> variables;                         // circuit witness
    std::vector<uint32_t> public_inps;                          // public inputs from the circuit
    std::unordered_map<uint32_t, std::string> vars_of_interest; // names of the variables
    std::unordered_map<std::string, uint32_t> terms;            // inverse map of the previous memeber
    std::vector<std::vector<std::string>> selectors;            // selectors from the circuit
    std::vector<std::vector<uint32_t>> wires_idxs;              // values of the gates' wires
    std::vector<FF> vars;                                       // all the symbolic variables from the circuit

    Solver* solver;  // pointer to the solver
    std::string tag; // tag of the symbolic circuit.
                     // If not empty, will be added to the names
                     // of symbolic variables to prevent collisions.

    explicit Circuit(CircuitSchema& circuit_info, Solver* solver, const std::string& tag = "");

    FF operator[](const std::string& name);
    FF operator[](const uint32_t& idx) { return vars[idx]; };
    inline uint32_t get_num_gates() const { return static_cast<uint32_t>(selectors.size()); };
    inline uint32_t get_num_vars() const { return static_cast<uint32_t>(vars.size()); };
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
    : public_inps(circuit_info.public_inps)
    , vars_of_interest(circuit_info.vars_of_interest)
    , wires_idxs(circuit_info.wires)
    , solver(solver)
    , tag(tag)
{
    if (!this->tag.empty()) {
        if (this->tag[0] != '_') {
            this->tag = "_" + this->tag;
        }
    }

    for (auto var : circuit_info.variables) {
        std::stringstream buf; // TODO(alex): looks bad. Would be great to create tostring() converter
        buf << var;
        std::string tmp = buf.str();
        tmp[1] = '0'; // avoiding `x` in 0x prefix
        variables.push_back(tmp);
    }

    for (auto& x : vars_of_interest) {
        terms.insert({ x.second, x.first });
    }

    vars_of_interest.insert({ 0, "zero" });
    vars_of_interest.insert({ 1, "one" });
    terms.insert({ "zero", 0 });
    terms.insert({ "one", 1 });

    for (auto sel : circuit_info.selectors) {
        std::vector<std::string> tmp_sel;
        for (size_t j = 0; j < 5; j++) {
            std::stringstream buf; // TODO(alex): #2
            buf << sel[j];
            std::string q_i = buf.str();
            q_i[1] = '0'; // avoiding `x` in 0x prefix
            tmp_sel.push_back(q_i);
        }
        selectors.push_back(tmp_sel);
    }

    this->init();
    this->add_gates();
}

/**
 * Creates all the needed symbolic variables and constants
 * which are used in circuit.
 *
 */
template <typename FF> void Circuit<FF>::init()
{
    size_t num_vars = variables.size();

    vars.push_back(FF::Var("zero" + this->tag, this->solver));
    vars.push_back(FF::Var("one" + this->tag, this->solver));

    for (size_t i = 2; i < num_vars; i++) {
        if (vars_of_interest.contains(static_cast<uint32_t>(i))) {
            std::string name = vars_of_interest[static_cast<uint32_t>(i)];
            vars.push_back(FF::Var(name + this->tag, this->solver));
        } else {
            vars.push_back(FF::Var("var_" + std::to_string(i) + this->tag, this->solver));
        }
    }

    vars[0] == FF::Const("0", this->solver);
    vars[1] == FF::Const("1", this->solver);

    for (auto i : public_inps) {
        vars[i] == FF::Const(variables[i], this->solver);
    }
}

/**
 * @brief Adds all the gate constraints to the solver.
 *
 */
template <typename FF> void Circuit<FF>::add_gates()
{
    for (size_t i = 0; i < get_num_gates(); i++) {
        FF q_m = FF::Const(selectors[i][0], this->solver);
        FF q_1 = FF::Const(selectors[i][1], this->solver);
        FF q_2 = FF::Const(selectors[i][2], this->solver);
        FF q_3 = FF::Const(selectors[i][3], this->solver);
        FF q_c = FF::Const(selectors[i][4], this->solver);

        uint32_t w_l = wires_idxs[i][0];
        uint32_t w_r = wires_idxs[i][1];
        uint32_t w_o = wires_idxs[i][2];

        // Binary gate. Relaxes the solver.
        // TODO(alex): Probably we can add other basic gates here too to relax the stuff.
        // TODO(alex): Theoretically this can be applyed after we ensure that the block of polynomial equations holds
        // and then simplify that block in future to relax the solver constraint system. Seems like a hard one to
        // implement or actually to automate, but I'll think on it for a while. it will probably require to split
        // add_gates and init methods into more complex/generalized parts.
        if (w_l == w_r && w_r == w_o) {
            if (std::string(q_m) == "1" && std::string(q_1) == "0" && std::string(q_2) == "0" &&
                std::string(q_3) == "-1" && std::string(q_c) == "0") { // squaring gate
                (Bool(vars[w_l]) == Bool(vars[0]) | Bool(vars[w_l]) == Bool(vars[1])).assert_term();
            }
        }

        FF eq = vars[0];

        // mult selector
        if (std::string(q_m) != "0") {
            eq += q_m * vars[w_l] * vars[w_r];
        }
        // w_l selector
        if (std::string(q_1) != "0") {
            eq += q_1 * vars[w_l];
        }
        // w_r selector
        if (std::string(q_2) != "0") {
            eq += q_2 * vars[w_r];
        }
        // w_o selector
        if (std::string(q_3) != "0") {
            eq += q_3 * vars[w_o];
        }
        // w_c selector
        if (std::string(q_c) != "0") {
            eq += q_c;
        }
        eq == vars[0];
    }
}

/**
 * @brief Returns a previously named symbolic variable.
 *
 * @param name
 * @return FF
 */
template <typename FF> FF Circuit<FF>::operator[](const std::string& name)
{
    if (!this->terms.contains(name)) {
        throw std::length_error("No such an item " + name + " in vars or it vas not declared as interesting");
    }
    uint32_t idx = this->terms[name];
    return this->vars[idx];
}

CircuitSchema unpack_from_buffer(const msgpack::sbuffer& buf);
CircuitSchema unpack_from_file(const std::string& fname);

template <typename FF>
std::pair<Circuit<FF>, Circuit<FF>> unique_witness(CircuitSchema& circuit_info,
                                                   Solver* s,
                                                   const std::vector<std::string>& equal = {},
                                                   const std::vector<std::string>& not_equal = {},
                                                   const std::vector<std::string>& equal_at_the_same_time = {},
                                                   const std::vector<std::string>& not_eqaul_at_the_same_time = {});

}; // namespace smt_circuit

#include "solver.hpp"
#include <iostream>

namespace smt_solver {

/**
 * Check if the system is solvable.
 *
 * @return true if the system is solvable.
 * */
bool Solver::check()
{
    cvc5::Result result = this->s.checkSat();
    this->res = result.isSat();
    this->checked = true;
    return this->res;
}

/**
 * If the system is solvable, extract the values for the given symbolic variables.
 *
 * @param terms A map containing pairs (name, symbolic term).
 * @return A map containing pairs (name, value).
 * */
std::unordered_map<std::string, std::string> Solver::model(std::unordered_map<std::string, cvc5::Term>& terms) const
{
    if (!this->checked) {
        throw std::length_error("Haven't checked yet");
    }
    if (!this->res) {
        throw std::length_error("There's no solution");
    }
    std::unordered_map<std::string, std::string> resulting_model;
    for (auto& term : terms) {
        std::string str_val = this->s.getValue(term.second).getFiniteFieldValue();
        resulting_model.insert({ term.first, str_val });
    }
    return resulting_model;
}
}; // namespace smt_solver

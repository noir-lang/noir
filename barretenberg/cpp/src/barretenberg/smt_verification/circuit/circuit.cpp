#include "circuit.hpp"

namespace smt_circuit {

/**
 * @brief Check your circuit for witness uniqueness
 *
 * @details Creates two Circuit objects that represent the same
 * circuit, however you can choose which variables should be (not) equal in both cases,
 * and also the variables that should (not) be equal at the same time.
 *
 * @param circuit_info
 * @param s pointer to the global solver
 * @param equal The list of names of variables which should be equal in both circuits(each is equal)
 * @param not_equal The list of names of variables which should not be equal in both circuits(each is not equal)
 * @param equal_at_the_same_time The list of variables, where at least one pair has to be equal
 * @param not_equal_at_the_same_time The list of variables, where at least one pair has to be distinct
 * @return std::pair<Circuit, Circuit>
 */
template <typename FF>
std::pair<Circuit<FF>, Circuit<FF>> unique_witness_ext(CircuitSchema& circuit_info,
                                                       Solver* s,
                                                       const std::vector<std::string>& equal,
                                                       const std::vector<std::string>& not_equal,
                                                       const std::vector<std::string>& equal_at_the_same_time,
                                                       const std::vector<std::string>& not_equal_at_the_same_time)
{
    Circuit<FF> c1(circuit_info, s, "circuit1");
    Circuit<FF> c2(circuit_info, s, "circuit2");

    for (const auto& term : equal) {
        c1[term] == c2[term];
    }
    for (const auto& term : not_equal) {
        c1[term] != c2[term];
    }

    std::vector<Bool> eqs;
    for (const auto& term : equal_at_the_same_time) {
        Bool tmp = Bool(c1[term]) == Bool(c2[term]);
        eqs.push_back(tmp);
    }

    if (eqs.size() > 1) {
        batch_or(eqs).assert_term();
    } else if (eqs.size() == 1) {
        eqs[0].assert_term();
    }

    std::vector<Bool> neqs;
    for (const auto& term : not_equal_at_the_same_time) {
        Bool tmp = Bool(c1[term]) != Bool(c2[term]);
        neqs.push_back(tmp);
    }

    if (neqs.size() > 1) {
        batch_or(neqs).assert_term();
    } else if (neqs.size() == 1) {
        neqs[0].assert_term();
    }
    return { c1, c2 };
}

template std::pair<Circuit<FFTerm>, Circuit<FFTerm>> unique_witness_ext(
    CircuitSchema& circuit_info,
    Solver* s,
    const std::vector<std::string>& equal = {},
    const std::vector<std::string>& not_equal = {},
    const std::vector<std::string>& equal_at_the_same_time = {},
    const std::vector<std::string>& not_eqaul_at_the_same_time = {});

template std::pair<Circuit<FFITerm>, Circuit<FFITerm>> unique_witness_ext(
    CircuitSchema& circuit_info,
    Solver* s,
    const std::vector<std::string>& equal = {},
    const std::vector<std::string>& not_equal = {},
    const std::vector<std::string>& equal_at_the_same_time = {},
    const std::vector<std::string>& not_eqaul_at_the_same_time = {});

/**
 * @brief Check your circuit for witness uniqueness
 *
 * @details Creates two Circuit objects that represent the same
 * circuit, however you can choose which variables should be equal in both cases,
 * other witness members will be marked as not equal at the same time
 * or basically they will have to differ by at least one element.
 *
 * @param circuit_info
 * @param s pointer to the global solver
 * @param equal The list of names of variables which should be equal in both circuits(each is equal)
 * @return std::pair<Circuit, Circuit>
 */
template <typename FF>
std::pair<Circuit<FF>, Circuit<FF>> unique_witness(CircuitSchema& circuit_info,
                                                   Solver* s,
                                                   const std::vector<std::string>& equal)
{
    Circuit<FF> c1(circuit_info, s, "circuit1");
    Circuit<FF> c2(circuit_info, s, "circuit2");

    for (const auto& term : equal) {
        c1[term] == c2[term];
    }

    std::vector<Bool> neqs;
    for (const auto& node : c1.symbolic_vars) {
        uint32_t i = node.first;
        if (std::find(equal.begin(), equal.end(), std::string(c1.variable_names[i])) != equal.end()) {
            continue;
        }
        Bool tmp = Bool(c1[i]) != Bool(c2[i]);
        neqs.push_back(tmp);
    }

    if (neqs.size() > 1) {
        batch_or(neqs).assert_term();
    } else if (neqs.size() == 1) {
        neqs[0].assert_term();
    }
    return { c1, c2 };
}

template std::pair<Circuit<FFTerm>, Circuit<FFTerm>> unique_witness(CircuitSchema& circuit_info,
                                                                    Solver* s,
                                                                    const std::vector<std::string>& equal = {});

template std::pair<Circuit<FFITerm>, Circuit<FFITerm>> unique_witness(CircuitSchema& circuit_info,
                                                                      Solver* s,
                                                                      const std::vector<std::string>& equal = {});
}; // namespace smt_circuit
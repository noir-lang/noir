#pragma once
#include <cvc5/cvc5.h>
#include <string>
#include <unordered_map>

namespace smt_solver {

/**
 * @brief Solver configuration
 * 
 * @param produce_model tells the solver to actually compute the values of the variables in SAT case.
 * @param timeout tells the solver to stop trying after `timeout` msecs.
 *
 * @todo TODO(alex): more cvc5 options.
 */
struct SolverConfiguration{
    bool produce_model;
    uint64_t timeout;
};

/**
 * @brief Class for the solver.
 *
 * @details Solver class that can be used to create
 * a solver, finite field terms and the circuit.
 * Check the satisfability of a system and get it's model.
 */
class Solver {
  public:
    cvc5::Solver s;
    cvc5::Sort fp;
    std::string modulus; // modulus in base 10
    bool res = false;
    bool checked = false;

    explicit Solver(const std::string& modulus, const SolverConfiguration& config = {false, 0}, uint32_t base = 16)
    {
        this->fp = s.mkFiniteFieldSort(modulus, base);
        this->modulus = fp.getFiniteFieldSize();
        if (config.produce_model) {
            s.setOption("produce-models", "true");
        }
        if (config.timeout > 0) {
            s.setOption("tlimit-per", std::to_string(config.timeout));
        }
    }

    Solver(const Solver& other) = delete;
    Solver(Solver&& other) = delete;
    Solver& operator=(const Solver& other) = delete;
    Solver& operator=(Solver&& other) = delete;

    bool check();

    [[nodiscard]] std::string getResult() const
    {
        if (!checked) {
            return "No result, yet";
        }
        return res ? "SAT" : "UNSAT";
    }

    std::unordered_map<std::string, std::string> model(std::unordered_map<std::string, cvc5::Term>& terms) const;
    ~Solver() = default;
};
}; // namespace smt_solver

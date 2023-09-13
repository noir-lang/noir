#pragma once
#include "barretenberg/smt_verification/solver/solver.hpp"

namespace smt_terms {
using namespace smt_solver;

/**
 * @brief Finite Field element class.
 *
 * @details Can be a finite field symbolic variable or a constant.
 * Both of them support basic arithmetic operations: +, -, *, /.
 * Check the satisfability of a system and get it's model.
 *
 */
class FFTerm {
  public:
    Solver* solver;
    cvc5::Term term;

    FFTerm()
        : solver(nullptr)
        , term(cvc5::Term()){};

    explicit FFTerm(const std::string& t, Solver* slv, bool isconst = false, uint32_t base = 16);
    FFTerm(cvc5::Term& term, Solver* s)
        : solver(s)
        , term(term){};
    FFTerm(const FFTerm& other) = default;
    FFTerm(FFTerm&& other) = default;

    static FFTerm Var(const std::string& name, Solver* slv);
    static FFTerm Const(const std::string& val, Solver* slv, uint32_t base = 16);

    FFTerm& operator=(const FFTerm& right) = default;
    FFTerm& operator=(FFTerm&& right) = default;

    FFTerm operator+(const FFTerm& other) const;
    void operator+=(const FFTerm& other);
    FFTerm operator-(const FFTerm& other) const;
    void operator-=(const FFTerm& other);
    FFTerm operator*(const FFTerm& other) const;
    void operator*=(const FFTerm& other);
    FFTerm operator/(const FFTerm& other) const;
    void operator/=(const FFTerm& other);

    void operator==(const FFTerm& other) const;
    void operator!=(const FFTerm& other) const;

    operator std::string() const { return term.isFiniteFieldValue() ? term.getFiniteFieldValue() : term.toString(); };
    operator cvc5::Term() const { return term; };
    
    ~FFTerm() = default;
    friend std::ostream& operator<<(std::ostream& out, const FFTerm& k) { return out << k.term; }

    friend FFTerm batch_add(const std::vector<FFTerm>& children)
    {
        Solver* slv = children[0].solver;
        std::vector<cvc5::Term> terms(children.begin(), children.end());
        cvc5::Term res = slv->s.mkTerm(cvc5::Kind::FINITE_FIELD_ADD, terms);
        return { res, slv };
    }

    friend FFTerm batch_mul(const std::vector<FFTerm>& children)
    {
        Solver* slv = children[0].solver;
        std::vector<cvc5::Term> terms(children.begin(), children.end());
        cvc5::Term res = slv->s.mkTerm(cvc5::Kind::FINITE_FIELD_MULT, terms);
        return { res, slv };
    }
};

} // namespace smt_terms
#pragma once
#include "ffiterm.hpp"
#include "ffterm.hpp"

namespace smt_terms {
using namespace smt_solver;

/**
 * @brief Bool element class.
 *
 * @details Can be used to create non trivial constraints.
 * Supports basic boolean arithmetic: &, |.
 *
 */
class Bool {
  public:
    cvc5::Solver* solver;
    cvc5::Term term;
    bool asserted = false;

    explicit Bool(const cvc5::Term& t, Solver& slv)
        : solver(&slv.s)
        , term(t){};
    explicit Bool(const FFTerm& t)
        : solver(&t.solver->s)
        , term(t.term){};

    explicit Bool(const FFITerm& t)
        : solver(&t.solver->s)
        , term(t.term){};

    explicit Bool(bool t, Solver& slv)
        : solver(&slv.s)
    {
        term = solver->mkBoolean(t);
    }
    Bool(const cvc5::Term& term, cvc5::Solver* s)
        : solver(s)
        , term(term){};
    Bool(const Bool& other) = default;
    Bool(Bool&& other) = default;

    Bool& operator=(const Bool& right) = default;
    Bool& operator=(Bool&& right) = default;

    void assert_term()
    {
        if (!asserted) {
            solver->assertFormula(term);
            asserted = true;
        }
    }

    Bool operator|(const Bool& other) const;
    void operator|=(const Bool& other);
    Bool operator|(const bool& other) const;
    void operator|=(const bool& other) const;

    Bool operator&(const Bool& other) const;
    void operator&=(const Bool& other);
    Bool operator&(const bool& other) const;
    void operator&=(const bool& other);

    Bool operator==(const Bool& other) const;
    Bool operator!=(const Bool& other) const;

    operator std::string() const { return term.toString(); };
    operator cvc5::Term() const { return term; };

    friend Bool batch_or(const std::vector<Bool>& children)
    {
        cvc5::Solver* s = children[0].solver;
        std::vector<cvc5::Term> terms(children.begin(), children.end());
        cvc5::Term res = s->mkTerm(cvc5::Kind::OR, terms);
        return { res, s };
    }

    friend Bool batch_and(const std::vector<Bool>& children)
    {
        cvc5::Solver* s = children[0].solver;
        std::vector<cvc5::Term> terms(children.begin(), children.end());
        cvc5::Term res = s->mkTerm(cvc5::Kind::AND, terms);
        return { res, s };
    }

    ~Bool() = default;
};
}; // namespace smt_terms

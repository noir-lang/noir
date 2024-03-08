#include "ffterm.hpp"

namespace smt_terms {

/**
 * Create a finite field symbolic variable.
 *
 * @param name Name of the variable. Should be unique per variable.
 * @param slv  Pointer to the global solver.
 * @return Finite field symbolic variable.
 * */
FFTerm FFTerm::Var(const std::string& name, Solver* slv)
{
    return FFTerm(name, slv);
};

/**
 * Create a finite field numeric member.
 *
 * @param val  String representation of the value.
 * @param slv  Pointer to the global solver.
 * @param base Base of the string representation. 16 by default.
 * @return Finite field constant.
 * */
FFTerm FFTerm::Const(const std::string& val, Solver* slv, uint32_t base)
{
    return FFTerm(val, slv, true, base);
};

FFTerm::FFTerm(const std::string& t, Solver* slv, bool isconst, uint32_t base)
    : solver(slv)
{
    if (!isconst) {
        this->term = slv->s.mkConst(slv->fp, t);
    } else {
        this->term = slv->s.mkFiniteFieldElem(t, slv->fp, base);
    }
}

FFTerm FFTerm::operator+(const FFTerm& other) const
{
    cvc5::Term res = this->solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_ADD, { this->term, other.term });
    return { res, this->solver };
}

void FFTerm::operator+=(const FFTerm& other)
{
    this->term = this->solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_ADD, { this->term, other.term });
}

FFTerm FFTerm::operator-(const FFTerm& other) const
{
    cvc5::Term res = this->solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_NEG, { other.term });
    res = solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_ADD, { this->term, res });
    return { res, this->solver };
}

FFTerm FFTerm::operator-() const
{
    cvc5::Term res = this->solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_NEG, { this->term });
    return { res, this->solver };
}

void FFTerm::operator-=(const FFTerm& other)
{
    cvc5::Term tmp_term = this->solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_NEG, { other.term });
    this->term = this->solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_ADD, { this->term, tmp_term });
}

FFTerm FFTerm::operator*(const FFTerm& other) const
{
    cvc5::Term res = solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_MULT, { this->term, other.term });
    return { res, this->solver };
}

void FFTerm::operator*=(const FFTerm& other)
{
    this->term = this->solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_MULT, { this->term, other.term });
}

/**
 * @brief Division operation
 *
 * @details Returns a result of the division by
 * creating a new symbolic variable and adding a new constraint
 * to the solver.
 *
 * @param other
 * @return FFTerm
 */
FFTerm FFTerm::operator/(const FFTerm& other) const
{
    other != bb::fr(0);
    FFTerm res = Var("df8b586e3fa7a1224ec95a886e17a7da_div_" + static_cast<std::string>(*this) + "_" +
                         static_cast<std::string>(other),
                     this->solver);
    res* other == *this;
    return res;
}

void FFTerm::operator/=(const FFTerm& other)
{
    other != bb::fr(0);
    FFTerm res = Var("df8b586e3fa7a1224ec95a886e17a7da_div_" + static_cast<std::string>(*this) + "_" +
                         static_cast<std::string>(other),
                     this->solver);
    res* other == *this;
    this->term = res.term;
}

/**
 * Create an equality constraint between two finite field elements.
 *
 */
void FFTerm::operator==(const FFTerm& other) const
{
    cvc5::Term eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { this->term, other.term });
    this->solver->s.assertFormula(eq);
}

/**
 * Create an inequality constraint between two finite field elements.
 *
 */
void FFTerm::operator!=(const FFTerm& other) const
{
    cvc5::Term eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { this->term, other.term });
    eq = this->solver->s.mkTerm(cvc5::Kind::NOT, { eq });
    this->solver->s.assertFormula(eq);
}

FFTerm operator+(const bb::fr& lhs, const FFTerm& rhs)
{
    return rhs + lhs;
}

FFTerm operator-(const bb::fr& lhs, const FFTerm& rhs)
{
    return (-rhs) + lhs;
}

FFTerm operator*(const bb::fr& lhs, const FFTerm& rhs)
{
    return rhs * lhs;
}

FFTerm operator^(__attribute__((unused)) const bb::fr& lhs, __attribute__((unused)) const FFTerm& rhs)
{
    info("Not compatible with Finite Field");
    return {};
}

FFTerm operator/(const bb::fr& lhs, const FFTerm& rhs)
{
    return FFTerm(lhs, rhs.solver) / rhs;
}

void operator==(const bb::fr& lhs, const FFTerm& rhs)
{
    rhs == lhs;
}

void operator!=(const bb::fr& lhs, const FFTerm& rhs)
{
    rhs != lhs;
}
} // namespace smt_terms
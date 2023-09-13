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
    cvc5::Term nz = this->solver->s.mkTerm(cvc5::Kind::EQUAL,
                                           { other.term, this->solver->s.mkFiniteFieldElem("0", this->solver->fp) });
    nz = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { nz, this->solver->s.mkBoolean(false) });
    this->solver->s.assertFormula(nz);

    cvc5::Term res = this->solver->s.mkConst(this->solver->fp,
                                             "fe0f65a52067384116dc1137d798e0ca00a7ed46950e4eab7db51e08481535f2_div_" +
                                                 std::string(*this) + "_" + std::string(other));
    cvc5::Term div = this->solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_MULT, { res, other.term });
    cvc5::Term eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { this->term, div });
    this->solver->s.assertFormula(eq);
    return { res, this->solver };
}

void FFTerm::operator/=(const FFTerm& other)
{
    cvc5::Term nz = this->solver->s.mkTerm(cvc5::Kind::EQUAL,
                                           { other.term, this->solver->s.mkFiniteFieldElem("0", this->solver->fp) });
    nz = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { nz, this->solver->s.mkBoolean(false) });
    this->solver->s.assertFormula(nz);

    cvc5::Term res = this->solver->s.mkConst(this->solver->fp,
                                             "fe0f65a52067384116dc1137d798e0ca00a7ed46950e4eab7db51e08481535f2_div_" +
                                                 std::string(*this) + "__" + std::string(other));
    cvc5::Term div = this->solver->s.mkTerm(cvc5::Kind::FINITE_FIELD_MULT, { res, other.term });
    cvc5::Term eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { this->term, div });
    this->solver->s.assertFormula(eq);
    this->term = res;
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
    eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { eq, this->solver->s.mkBoolean(false) });
    this->solver->s.assertFormula(eq);
}
} // namespace smt_terms
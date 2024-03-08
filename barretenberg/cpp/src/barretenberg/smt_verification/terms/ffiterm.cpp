#include "ffiterm.hpp"

namespace smt_terms {

/**
 * Create an integer mod symbolic variable.
 *
 * @param name Name of the variable. Should be unique per variable.
 * @param slv  Pointer to the global solver.
 * @return Finite field symbolic variable.
 * */
FFITerm FFITerm::Var(const std::string& name, Solver* slv)
{
    return FFITerm(name, slv);
};

/**
 * Create an integer mod numeric member.
 *
 * @param val  String representation of the value.
 * @param slv  Pointer to the global solver.
 * @param base Base of the string representation. 16 by default.
 * @return Finite field constant.
 * */
FFITerm FFITerm::Const(const std::string& val, Solver* slv, uint32_t base)
{
    return FFITerm(val, slv, true, base);
};

FFITerm::FFITerm(const std::string& t, Solver* slv, bool isconst, uint32_t base)
    : solver(slv)
    , modulus(slv->s.mkInteger(slv->modulus))
{
    if (!isconst) {
        this->term = slv->s.mkConst(slv->s.getIntegerSort(), t);
        cvc5::Term ge = slv->s.mkTerm(cvc5::Kind::GEQ, { this->term, slv->s.mkInteger(0) });
        cvc5::Term lt = slv->s.mkTerm(cvc5::Kind::LT, { this->term, this->modulus });
        slv->s.assertFormula(ge);
        slv->s.assertFormula(lt);
    } else {
        // TODO(alex): CVC5 doesn't provide integer initialization from hex. Yet.
        std::string strvalue = slv->s.mkFiniteFieldElem(t, slv->fp, base).getFiniteFieldValue();
        this->term = slv->s.mkInteger(strvalue);
        this->mod();
    }
}

void FFITerm::mod()
{
    this->term = this->solver->s.mkTerm(cvc5::Kind::INTS_MODULUS, { this->term, this->modulus });
}

FFITerm FFITerm::operator+(const FFITerm& other) const
{
    cvc5::Term res = this->solver->s.mkTerm(cvc5::Kind::ADD, { this->term, other.term });
    return { res, this->solver };
}

void FFITerm::operator+=(const FFITerm& other)
{
    this->term = this->solver->s.mkTerm(cvc5::Kind::ADD, { this->term, other.term });
}

FFITerm FFITerm::operator-(const FFITerm& other) const
{
    cvc5::Term res = this->solver->s.mkTerm(cvc5::Kind::SUB, { this->term, other.term });
    return { res, this->solver };
}

void FFITerm::operator-=(const FFITerm& other)
{
    this->term = this->solver->s.mkTerm(cvc5::Kind::SUB, { this->term, other.term });
}

FFITerm FFITerm::operator-() const
{
    cvc5::Term res = this->solver->s.mkTerm(cvc5::Kind::NEG, { this->term });
    return { res, this->solver };
}

FFITerm FFITerm::operator*(const FFITerm& other) const
{
    cvc5::Term res = solver->s.mkTerm(cvc5::Kind::MULT, { this->term, other.term });
    return { res, this->solver };
}

void FFITerm::operator*=(const FFITerm& other)
{
    this->term = this->solver->s.mkTerm(cvc5::Kind::MULT, { this->term, other.term });
}

/**
 * @brief Division operation
 *
 * @details Returns a result of the division by
 * creating a new symbolic variable and adding a new constraint
 * to the solver.
 *
 * @param other
 * @return FFITerm
 */
FFITerm FFITerm::operator/(const FFITerm& other) const
{
    other != bb::fr(0);
    FFITerm res = Var("df8b586e3fa7a1224ec95a886e17a7da_div_" + static_cast<std::string>(*this) + "_" +
                          static_cast<std::string>(other),
                      this->solver);
    res* other == *this;
    return res;
}

void FFITerm::operator/=(const FFITerm& other)
{
    other != bb::fr(0);
    FFITerm res = Var("df8b586e3fa7a1224ec95a886e17a7da_div_" + static_cast<std::string>(*this) + "_" +
                          static_cast<std::string>(other),
                      this->solver);
    res* other == *this;
    this->term = res.term;
}

/**
 * Create an equality constraint between two integer mod elements.
 *
 */
void FFITerm::operator==(const FFITerm& other) const
{
    FFITerm tmp1 = *this;
    if (tmp1.term.getNumChildren() > 1) {
        tmp1.mod();
    }
    FFITerm tmp2 = other;
    if (tmp2.term.getNumChildren() > 1) {
        tmp2.mod();
    }
    cvc5::Term eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { tmp1.term, tmp2.term });
    this->solver->s.assertFormula(eq);
}

/**
 * Create an inequality constraint between two integer mod elements.
 *
 */
void FFITerm::operator!=(const FFITerm& other) const
{
    FFITerm tmp1 = *this;
    if (tmp1.term.getNumChildren() > 1) {
        tmp1.mod();
    }
    FFITerm tmp2 = other;
    if (tmp2.term.getNumChildren() > 1) {
        tmp2.mod();
    }
    cvc5::Term eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { tmp1.term, tmp2.term });
    eq = this->solver->s.mkTerm(cvc5::Kind::NOT, { eq });
    this->solver->s.assertFormula(eq);
}

FFITerm operator+(const bb::fr& lhs, const FFITerm& rhs)
{
    return rhs + lhs;
}

FFITerm operator-(const bb::fr& lhs, const FFITerm& rhs)
{
    return (-rhs) + lhs;
}

FFITerm operator*(const bb::fr& lhs, const FFITerm& rhs)
{
    return rhs * lhs;
}

FFITerm operator/(const bb::fr& lhs, const FFITerm& rhs)
{
    return FFITerm(lhs, rhs.solver) / rhs;
}

FFITerm operator^(__attribute__((unused)) const bb::fr& lhs, __attribute__((unused)) const FFITerm& rhs)
{
    info("Not compatible with Integers");
    return {};
}
void operator==(const bb::fr& lhs, const FFITerm& rhs)
{
    rhs == lhs;
}

void operator!=(const bb::fr& lhs, const FFITerm& rhs)
{
    rhs != lhs;
}

void FFITerm::operator<(const bb::fr& other) const
{
    cvc5::Term lt = this->solver->s.mkTerm(cvc5::Kind::LT, { this->term, FFITerm(other, this->solver) });
    this->solver->s.assertFormula(lt);
}
void FFITerm::operator<=(const bb::fr& other) const
{
    cvc5::Term le = this->solver->s.mkTerm(cvc5::Kind::LEQ, { this->term, FFITerm(other, this->solver) });
    this->solver->s.assertFormula(le);
}
void FFITerm::operator>(const bb::fr& other) const
{
    cvc5::Term gt = this->solver->s.mkTerm(cvc5::Kind::GT, { this->term, FFITerm(other, this->solver) });
    this->solver->s.assertFormula(gt);
}
void FFITerm::operator>=(const bb::fr& other) const
{
    cvc5::Term ge = this->solver->s.mkTerm(cvc5::Kind::GEQ, { this->term, FFITerm(other, this->solver) });
    this->solver->s.assertFormula(ge);
}

} // namespace smt_terms

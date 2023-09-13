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
        std::string tmp = slv->s.mkFiniteFieldElem(t, slv->fp, base).getFiniteFieldValue(); // dumb but works
        if (tmp[0] == '-') {
            this->term = slv->s.mkTerm(cvc5::Kind::ADD, { slv->s.mkInteger(tmp), this->modulus });
        } else {
            this->term = slv->s.mkInteger(tmp);
        }
        // this->term = slv->s.mkInteger(tmp); won't work for now since the assertion will definitely fail
    }
}

FFITerm FFITerm::operator+(const FFITerm& other) const
{
    cvc5::Term res = this->solver->s.mkTerm(cvc5::Kind::ADD, { this->term, other.term });
    res = this->solver->s.mkTerm(cvc5::Kind::INTS_MODULUS, { res, this->modulus });
    return { res, this->solver };
}

void FFITerm::operator+=(const FFITerm& other)
{
    this->term = this->solver->s.mkTerm(cvc5::Kind::ADD, { this->term, other.term });
    this->term =
        this->solver->s.mkTerm(cvc5::Kind::INTS_MODULUS, { this->term, this->modulus }); // TODO(alex): is it faster?
}

FFITerm FFITerm::operator-(const FFITerm& other) const
{
    cvc5::Term res = this->solver->s.mkTerm(cvc5::Kind::SUB, { this->term, other.term });
    res = this->solver->s.mkTerm(cvc5::Kind::INTS_MODULUS, { res, this->modulus });
    return { res, this->solver };
}

void FFITerm::operator-=(const FFITerm& other)
{
    this->term = this->solver->s.mkTerm(cvc5::Kind::SUB, { this->term, other.term });
    this->term =
        this->solver->s.mkTerm(cvc5::Kind::INTS_MODULUS, { this->term, this->modulus }); // TODO(alex): is it faster?
}

FFITerm FFITerm::operator*(const FFITerm& other) const
{
    cvc5::Term res = solver->s.mkTerm(cvc5::Kind::MULT, { this->term, other.term });
    res = this->solver->s.mkTerm(cvc5::Kind::INTS_MODULUS, { res, this->modulus });
    return { res, this->solver };
}

void FFITerm::operator*=(const FFITerm& other)
{
    this->term = this->solver->s.mkTerm(cvc5::Kind::MULT, { this->term, other.term });
    this->term =
        this->solver->s.mkTerm(cvc5::Kind::INTS_MODULUS, { this->term, this->modulus }); // TODO(alex): is it faster?
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
    cvc5::Term nz = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { other.term, this->solver->s.mkInteger("0") });
    nz = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { nz, this->solver->s.mkBoolean(false) });
    this->solver->s.assertFormula(nz);

    cvc5::Term res = this->solver->s.mkConst(this->solver->s.getIntegerSort(),
                                             "fe0f65a52067384116dc1137d798e0ca00a7ed46950e4eab7db51e08481535f2_div_" +
                                                 std::string(*this) + "_" + std::string(other));
    cvc5::Term div = this->solver->s.mkTerm(cvc5::Kind::MULT, { res, other.term });
    div = this->solver->s.mkTerm(cvc5::Kind::INTS_MODULUS, { div, this->modulus });
    cvc5::Term eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { this->term, div });
    this->solver->s.assertFormula(eq);
    return { res, this->solver };
}

void FFITerm::operator/=(const FFITerm& other)
{
    cvc5::Term nz = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { other.term, this->solver->s.mkInteger("0") });
    nz = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { nz, this->solver->s.mkBoolean(false) });
    this->solver->s.assertFormula(nz);

    cvc5::Term res = this->solver->s.mkConst(this->solver->fp,
                                             "fe0f65a52067384116dc1137d798e0ca00a7ed46950e4eab7db51e08481535f2_div_" +
                                                 std::string(*this) + "__" + std::string(other));
    cvc5::Term div = this->solver->s.mkTerm(cvc5::Kind::MULT, { res, other.term });
    div = this->solver->s.mkTerm(cvc5::Kind::INTS_MODULUS, { div, this->modulus });
    cvc5::Term eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { this->term, div });
    this->solver->s.assertFormula(eq);
    this->term = res;
}

/**
 * Create an equality constraint between two integer mod elements.
 *
 */
void FFITerm::operator==(const FFITerm& other) const
{
    cvc5::Term eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { this->term, other.term });
    this->solver->s.assertFormula(eq);
}

/**
 * Create an inequality constraint between two integer mod elements.
 *
 */
void FFITerm::operator!=(const FFITerm& other) const
{
    cvc5::Term eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { this->term, other.term });
    eq = this->solver->s.mkTerm(cvc5::Kind::EQUAL, { eq, this->solver->s.mkBoolean(false) });
    this->solver->s.assertFormula(eq);
}
} // namespace smt_terms

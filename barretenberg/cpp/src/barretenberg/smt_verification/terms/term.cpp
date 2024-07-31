#include "barretenberg/smt_verification/terms/term.hpp"

namespace smt_terms {

/**
 * Create a symbolic variable.
 *
 * @param name Name of the variable. Should be unique per variable
 * @param slv  Pointer to the global solver
 * @param type FFTerm, FFITerm or BVTerm
 * @return symbolic variable
 * */
STerm STerm::Var(const std::string& name, Solver* slv, TermType type)
{
    return STerm(name, slv, false, 16, type);
};

/**
 * Create constant symbolic variable.
 * i.e. term with predefined constant value
 *
 * @param val  String representation of the value.
 * @param slv  Pointer to the global solver.
 * @param base Base of the string representation. 16 by default.
 * @param type FFTerm, FFITerm or BVTerm
 * @return numeric constant
 * */
STerm STerm::Const(const std::string& val, Solver* slv, uint32_t base, TermType type)
{
    return STerm(val, slv, true, base, type);
};

STerm::STerm(const std::string& t, Solver* slv, bool isconst, uint32_t base, TermType type)
    : solver(slv)
    , isFiniteField(type == TermType::FFTerm)
    , isInteger(type == TermType::FFITerm)
    , isBitVector(type == TermType::BVTerm)
    , type(type)
    , operations(typed_operations.at(type))
{
    if (!isconst) {
        cvc5::Term ge;
        cvc5::Term lt;
        cvc5::Term modulus;
        switch (type) {
        case TermType::FFTerm:
            this->term = slv->term_manager.mkConst(slv->ff_sort, t);
            break;
        case TermType::FFITerm:
            this->term = slv->term_manager.mkConst(slv->term_manager.getIntegerSort(), t);
            ge = slv->term_manager.mkTerm(cvc5::Kind::GEQ, { this->term, slv->term_manager.mkInteger(0) });
            modulus = slv->term_manager.mkInteger(slv->modulus);
            lt = slv->term_manager.mkTerm(cvc5::Kind::LT, { this->term, modulus });
            slv->assertFormula(ge);
            slv->assertFormula(lt);
            break;
        case TermType::ITerm:
            this->term = slv->term_manager.mkConst(slv->term_manager.getIntegerSort(), t);
            break;
        case TermType::BVTerm:
            this->term = slv->term_manager.mkConst(slv->bv_sort, t);
            break;
        }
    } else {
        std::string strvalue;
        switch (type) {
        case TermType::FFTerm:
            this->term = slv->term_manager.mkFiniteFieldElem(t, slv->ff_sort, base);
            break;
        case TermType::FFITerm:
            // TODO(alex): CVC5 doesn't provide integer initialization from hex. Yet.
            strvalue = slv->term_manager.mkFiniteFieldElem(t, slv->ff_sort, base).getFiniteFieldValue();
            this->term = slv->term_manager.mkInteger(strvalue);
            this->term = this->mod().term;
            break;
        case TermType::ITerm:
            strvalue = slv->term_manager.mkFiniteFieldElem(t, slv->ff_sort, base).getFiniteFieldValue();
            this->term = slv->term_manager.mkInteger(strvalue);
            break;
        case TermType::BVTerm:
            // it's better to have (-p/2, p/2) representation due to overflows
            strvalue = slv->term_manager.mkFiniteFieldElem(t, slv->ff_sort, base).getFiniteFieldValue();
            this->term = slv->term_manager.mkBitVector(slv->bv_sort.getBitVectorSize(), strvalue, 10);
            break;
        }
    }
}

STerm STerm::mod() const
{
    if (!this->operations.contains(OpType::MOD)) {
        info("Taking a remainder is not compatible with ", this->type);
        return *this;
    }
    cvc5::Term modulus = this->solver->term_manager.mkInteger(solver->modulus);
    cvc5::Term res_s = this->solver->term_manager.mkTerm(this->operations.at(OpType::MOD), { this->term, modulus });
    return { res_s, this->solver, this->type };
}

STerm STerm::operator+(const STerm& other) const
{
    cvc5::Term res = this->solver->term_manager.mkTerm(this->operations.at(OpType::ADD), { this->term, other.term });
    return { res, this->solver, this->type };
}

void STerm::operator+=(const STerm& other)
{
    this->term = this->solver->term_manager.mkTerm(this->operations.at(OpType::ADD), { this->term, other.term });
}

STerm STerm::operator-(const STerm& other) const
{
    cvc5::Term res = this->solver->term_manager.mkTerm(this->operations.at(OpType::NEG), { other.term });
    res = solver->term_manager.mkTerm(this->operations.at(OpType::ADD), { this->term, res });
    return { res, this->solver, this->type };
}

void STerm::operator-=(const STerm& other)
{
    cvc5::Term tmp_term = this->solver->term_manager.mkTerm(this->operations.at(OpType::NEG), { other.term });
    this->term = this->solver->term_manager.mkTerm(cvc5::Kind::FINITE_FIELD_ADD, { this->term, tmp_term });
}

STerm STerm::operator-() const
{
    cvc5::Term res = this->solver->term_manager.mkTerm(this->operations.at(OpType::NEG), { this->term });
    return { res, this->solver, this->type };
}

STerm STerm::operator*(const STerm& other) const
{
    cvc5::Term res = solver->term_manager.mkTerm(this->operations.at(OpType::MUL), { this->term, other.term });
    return { res, this->solver, this->type };
}

void STerm::operator*=(const STerm& other)
{
    this->term = this->solver->term_manager.mkTerm(this->operations.at(OpType::MUL), { this->term, other.term });
}

/**
 * @brief Division operation
 *
 * @details Returns a result of the division by
 * creating a new symbolic variable and adding a new constraint
 * to the solver.
 *
 * @param other
 * @return STerm
 */
STerm STerm::operator/(const STerm& other) const
{
    if (!this->operations.contains(OpType::DIV)) {
        info("Division is not compatible with ", this->type);
        return *this;
    }
    if (this->type == TermType::FFTerm || this->type == TermType::FFITerm) {
        other != bb::fr(0);
        // Random value added to the name to prevent collisions. This value is MD5('Aztec')
        STerm res = Var("df8b586e3fa7a1224ec95a886e17a7da_div_" + static_cast<std::string>(*this) + "_" +
                            static_cast<std::string>(other),
                        this->solver,
                        this->type);
        res* other == *this;
        return res;
    }
    other != bb::fr(0);
    cvc5::Term res_s = this->solver->term_manager.mkTerm(this->operations.at(OpType::DIV), { this->term, other.term });
    return { res_s, this->solver, this->type };
}

void STerm::operator/=(const STerm& other)
{
    if (!this->operations.contains(OpType::DIV)) {
        info("Division is not compatible with ", this->type);
        return;
    }
    if (this->type == TermType::FFTerm || this->type == TermType::FFITerm) {
        other != bb::fr(0);
        // Random value added to the name to prevent collisions. This value is MD5('Aztec')
        STerm res = Var("df8b586e3fa7a1224ec95a886e17a7da_div_" + static_cast<std::string>(*this) + "_" +
                            static_cast<std::string>(other),
                        this->solver,
                        this->type);
        res* other == *this;
        this->term = res.term;
    }
    other != bb::fr(0);
    this->term = this->solver->term_manager.mkTerm(this->operations.at(OpType::DIV), { this->term, other.term });
}

/**
 * Create an equality constraint between two symbolic variables of the same type
 *
 */
void STerm::operator==(const STerm& other) const
{
    STerm left = *this;
    STerm right = other;
    left = this->type == TermType::FFITerm && left.term.getNumChildren() > 1 ? left.mod() : left;
    right = this->type == TermType::FFITerm && right.term.getNumChildren() > 1 ? right.mod() : right;

    cvc5::Term eq = this->solver->term_manager.mkTerm(cvc5::Kind::EQUAL, { left.term, right.term });
    this->solver->assertFormula(eq);
}

/**
 * Create an inequality constraint between two symbolic variables of the same type
 *
 */
void STerm::operator!=(const STerm& other) const
{
    STerm left = *this;
    STerm right = other;
    left = this->type == TermType::FFITerm && left.term.getNumChildren() > 1 ? left.mod() : left;
    right = this->type == TermType::FFITerm && right.term.getNumChildren() > 1 ? right.mod() : right;

    cvc5::Term eq = this->solver->term_manager.mkTerm(cvc5::Kind::EQUAL, { left.term, right.term });
    eq = this->solver->term_manager.mkTerm(cvc5::Kind::NOT, { eq });
    this->solver->assertFormula(eq);
}

void STerm::operator<(const bb::fr& other) const
{
    if (!this->operations.contains(OpType::LT)) {
        info("LT is not compatible with ", this->type);
        return;
    }

    cvc5::Term lt = this->solver->term_manager.mkTerm(this->operations.at(OpType::LT),
                                                      { this->term, STerm(other, this->solver, this->type) });
    this->solver->assertFormula(lt);
}

void STerm::operator<=(const bb::fr& other) const
{
    if (!this->operations.contains(OpType::LE)) {
        info("LE is not compatible with ", this->type);
        return;
    }
    cvc5::Term le = this->solver->term_manager.mkTerm(this->operations.at(OpType::LE),
                                                      { this->term, STerm(other, this->solver, this->type) });
    this->solver->assertFormula(le);
}

void STerm::operator>(const bb::fr& other) const
{
    if (!this->operations.contains(OpType::GT)) {
        info("GT is not compatible with ", this->type);
        return;
    }
    cvc5::Term gt = this->solver->term_manager.mkTerm(this->operations.at(OpType::GT),
                                                      { this->term, STerm(other, this->solver, this->type) });
    this->solver->assertFormula(gt);
}

void STerm::operator>=(const bb::fr& other) const
{
    if (!this->operations.contains(OpType::GE)) {
        info("GE is not compatible with ", this->type);
        return;
    }
    cvc5::Term ge = this->solver->term_manager.mkTerm(this->operations.at(OpType::GE),
                                                      { this->term, STerm(other, this->solver, this->type) });
    this->solver->assertFormula(ge);
}

STerm STerm::operator^(const STerm& other) const
{
    if (!this->operations.contains(OpType::XOR)) {
        info("XOR is not compatible with ", this->type);
        return *this;
    }
    cvc5::Term res = solver->term_manager.mkTerm(this->operations.at(OpType::XOR), { this->term, other.term });
    return { res, this->solver, this->type };
}

void STerm::operator^=(const STerm& other)
{
    if (!this->operations.contains(OpType::XOR)) {
        info("XOR is not compatible with ", this->type);
        return;
    }
    this->term = solver->term_manager.mkTerm(this->operations.at(OpType::XOR), { this->term, other.term });
}

STerm STerm::operator&(const STerm& other) const
{
    if (!this->operations.contains(OpType::AND)) {
        info("AND is not compatible with ", this->type);
        return *this;
    }
    cvc5::Term res = solver->term_manager.mkTerm(this->operations.at(OpType::AND), { this->term, other.term });
    return { res, this->solver, this->type };
}

void STerm::operator&=(const STerm& other)
{
    if (!this->operations.contains(OpType::AND)) {
        info("AND is not compatible with ", this->type);
        return;
    }
    this->term = solver->term_manager.mkTerm(this->operations.at(OpType::AND), { this->term, other.term });
}

STerm STerm::operator|(const STerm& other) const
{
    if (!this->operations.contains(OpType::OR)) {
        info("OR is not compatible with ", this->type);
        return *this;
    }
    cvc5::Term res = solver->term_manager.mkTerm(this->operations.at(OpType::OR), { this->term, other.term });
    return { res, this->solver, this->type };
}

void STerm::operator|=(const STerm& other)
{
    if (!this->operations.contains(OpType::OR)) {
        info("OR is not compatible with ", this->type);
        return;
    }
    this->term = solver->term_manager.mkTerm(this->operations.at(OpType::OR), { this->term, other.term });
}

STerm STerm::operator<<(const uint32_t& n) const
{
    if (!this->operations.contains(OpType::LSH)) {
        info("SHIFT LEFT is not compatible with ", this->type);
        return *this;
    }
    cvc5::Term shift = this->solver->term_manager.mkBitVector(this->solver->bv_sort.getBitVectorSize(), n);
    cvc5::Term res = this->solver->term_manager.mkTerm(this->operations.at(OpType::LSH), { this->term, shift });
    return { res, this->solver, this->type };
}

void STerm::operator<<=(const uint32_t& n)
{
    if (!this->operations.contains(OpType::LSH)) {
        info("SHIFT LEFT is not compatible with ", this->type);
        return;
    }
    cvc5::Term shift = solver->term_manager.mkBitVector(this->solver->bv_sort.getBitVectorSize(), n);
    this->term = this->solver->term_manager.mkTerm(this->operations.at(OpType::LSH), { this->term, shift });
}

STerm STerm::operator>>(const uint32_t& n) const
{
    if (!this->operations.contains(OpType::RSH)) {
        info("SHIFT RIGHT is not compatible with ", this->type);
        return *this;
    }
    cvc5::Term shift = this->solver->term_manager.mkBitVector(this->solver->bv_sort.getBitVectorSize(), n);
    cvc5::Term res = this->solver->term_manager.mkTerm(this->operations.at(OpType::RSH), { this->term, shift });
    return { res, this->solver, this->type };
}

void STerm::operator>>=(const uint32_t& n)
{
    if (!this->operations.contains(OpType::RSH)) {
        info("SHIFT RIGHT is not compatible with ", this->type);
        return;
    }
    cvc5::Term shift = this->solver->term_manager.mkBitVector(this->solver->bv_sort.getBitVectorSize(), n);
    this->term = this->solver->term_manager.mkTerm(this->operations.at(OpType::RSH), { this->term, shift });
}

STerm STerm::rotr(const uint32_t& n) const
{
    if (!this->operations.contains(OpType::ROTR)) {
        info("ROTR is not compatible with ", this->type);
        return *this;
    }
    cvc5::Op rotr = solver->term_manager.mkOp(this->operations.at(OpType::ROTR), { n });
    cvc5::Term res = solver->term_manager.mkTerm(rotr, { this->term });
    return { res, this->solver, this->type };
}

STerm STerm::rotl(const uint32_t& n) const
{
    if (!this->operations.contains(OpType::ROTL)) {
        info("ROTL is not compatible with ", this->type);
        return *this;
    }
    cvc5::Op rotl = solver->term_manager.mkOp(this->operations.at(OpType::ROTL), { n });
    cvc5::Term res = solver->term_manager.mkTerm(rotl, { this->term });
    return { res, this->solver, this->type };
}

/**
 * @brief Create an inclusion constraint
 *
 * @param entry entry to be checked
 * @param table table that consists of elements, ranges mostly
 */
void STerm::in(const cvc5::Term& table) const
{
    STerm left = *this;
    left = this->type == TermType::FFITerm && left.term.getNumChildren() > 1 ? left.mod() : left;

    Solver* slv = this->solver;
    cvc5::Term inc = slv->term_manager.mkTerm(cvc5::Kind::SET_MEMBER, { left.term, table });
    slv->assertFormula(inc);
}

STerm operator+(const bb::fr& lhs, const STerm& rhs)
{
    return rhs + lhs;
}

STerm operator-(const bb::fr& lhs, const STerm& rhs)
{
    return (-rhs) + lhs;
}

STerm operator*(const bb::fr& lhs, const STerm& rhs)
{
    return rhs * lhs;
}

STerm operator^(const bb::fr& lhs, const STerm& rhs)
{
    return rhs ^ lhs;
}

STerm operator&(const bb::fr& lhs, const STerm& rhs)
{
    return rhs & lhs;
}

STerm operator|(const bb::fr& lhs, const STerm& rhs)
{
    return rhs | lhs;
}

STerm operator/(const bb::fr& lhs, const STerm& rhs)
{
    return STerm(lhs, rhs.solver, rhs.type) / rhs;
}

void operator==(const bb::fr& lhs, const STerm& rhs)
{
    rhs == lhs;
}

void operator!=(const bb::fr& lhs, const STerm& rhs)
{
    rhs != lhs;
}

std::ostream& operator<<(std::ostream& os, const TermType type)
{
    switch (type) {
    case TermType::FFTerm:
        os << "FFTerm";
        break;
    case TermType::FFITerm:
        os << "FFITerm";
        break;
    case TermType::BVTerm:
        os << "BVTerm";
        break;
    case TermType::ITerm:
        os << "ITerm";
        break;
    };
    return os;
}

// Parametrized calls to STerm constructor
// so you won't need to use TermType each time to define a new variable.

STerm FFVar(const std::string& name, Solver* slv)
{
    return STerm::Var(name, slv, TermType::FFTerm);
}

STerm FFConst(const std::string& val, Solver* slv, uint32_t base)
{
    return STerm::Const(val, slv, base, TermType::FFTerm);
}

STerm FFIVar(const std::string& name, Solver* slv)
{
    return STerm::Var(name, slv, TermType::FFITerm);
}

STerm FFIConst(const std::string& val, Solver* slv, uint32_t base)
{
    return STerm::Const(val, slv, base, TermType::FFITerm);
}

STerm IVar(const std::string& name, Solver* slv)
{
    return STerm::Var(name, slv, TermType::ITerm);
}

STerm IConst(const std::string& val, Solver* slv, uint32_t base)
{
    return STerm::Const(val, slv, base, TermType::ITerm);
}

STerm BVVar(const std::string& name, Solver* slv)
{
    return STerm::Var(name, slv, TermType::BVTerm);
}

STerm BVConst(const std::string& val, Solver* slv, uint32_t base)
{
    return STerm::Const(val, slv, base, TermType::BVTerm);
}

} // namespace smt_terms

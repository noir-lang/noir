#pragma once
#include "barretenberg/smt_verification/solver/solver.hpp"

namespace smt_terms {
using namespace smt_solver;

/**
 * @brief Allows to define three types of symbolic terms
 * STerm - Symbolic Variables acting like a Finte Field elements
 * FFITerm - Symbolic Variables acting like integers modulo prime
 * ITerm - Symbolic Variables acting like integers
 * BVTerm - Symbolic Variables acting like bitvectors modulo prime
 *
 */
enum class TermType { FFTerm, FFITerm, BVTerm, ITerm };
std::ostream& operator<<(std::ostream& os, TermType type);

enum class OpType : int32_t { ADD, SUB, MUL, DIV, NEG, XOR, AND, OR, GT, GE, LT, LE, MOD, RSH, LSH, ROTR, ROTL };

/**
 * @brief precomputed map that contains allowed
 * operations for each of three symbolic types
 *
 */
const std::unordered_map<TermType, std::unordered_map<OpType, cvc5::Kind>> typed_operations = {
    { TermType::FFTerm,
      { { OpType::ADD, cvc5::Kind::FINITE_FIELD_ADD },
        { OpType::MUL, cvc5::Kind::FINITE_FIELD_MULT },
        { OpType::NEG, cvc5::Kind::FINITE_FIELD_NEG },
        // Just a placeholder that marks it supports division
        { OpType::DIV, cvc5::Kind::FINITE_FIELD_MULT } } },
    { TermType::FFITerm,
      {

          { OpType::ADD, cvc5::Kind::ADD },
          { OpType::SUB, cvc5::Kind::SUB },
          { OpType::MUL, cvc5::Kind::MULT },
          { OpType::NEG, cvc5::Kind::NEG },
          { OpType::GT, cvc5::Kind::GT },
          { OpType::GE, cvc5::Kind::GEQ },
          { OpType::LT, cvc5::Kind::LT },
          { OpType::LE, cvc5::Kind::LEQ },
          { OpType::MOD, cvc5::Kind::INTS_MODULUS },
          // Just a placeholder that marks it supports division
          { OpType::DIV, cvc5::Kind::MULT } } },
    { TermType::ITerm,
      { { OpType::ADD, cvc5::Kind::ADD },
        { OpType::SUB, cvc5::Kind::SUB },
        { OpType::MUL, cvc5::Kind::MULT },
        { OpType::NEG, cvc5::Kind::NEG },
        { OpType::GT, cvc5::Kind::GT },
        { OpType::GE, cvc5::Kind::GEQ },
        { OpType::LT, cvc5::Kind::LT },
        { OpType::LE, cvc5::Kind::LEQ },
        { OpType::MOD, cvc5::Kind::INTS_MODULUS },
        { OpType::DIV, cvc5::Kind::INTS_DIVISION } } },
    { TermType::BVTerm,
      {

          { OpType::ADD, cvc5::Kind::BITVECTOR_ADD },
          { OpType::SUB, cvc5::Kind::BITVECTOR_SUB },
          { OpType::MUL, cvc5::Kind::BITVECTOR_MULT },
          { OpType::NEG, cvc5::Kind::BITVECTOR_NEG },
          { OpType::GT, cvc5::Kind::BITVECTOR_UGT },
          { OpType::GE, cvc5::Kind::BITVECTOR_UGE },
          { OpType::LT, cvc5::Kind::BITVECTOR_ULT },
          { OpType::LE, cvc5::Kind::BITVECTOR_ULE },
          { OpType::XOR, cvc5::Kind::BITVECTOR_XOR },
          { OpType::AND, cvc5::Kind::BITVECTOR_AND },
          { OpType::OR, cvc5::Kind::BITVECTOR_OR },
          { OpType::RSH, cvc5::Kind::BITVECTOR_LSHR },
          { OpType::LSH, cvc5::Kind::BITVECTOR_SHL },
          { OpType::ROTL, cvc5::Kind::BITVECTOR_ROTATE_LEFT },
          { OpType::ROTR, cvc5::Kind::BITVECTOR_ROTATE_RIGHT },
          { OpType::MOD, cvc5::Kind::BITVECTOR_UREM },
          { OpType::DIV, cvc5::Kind::BITVECTOR_UDIV } } }
};

/**
 * @brief Symbolic term element class.
 *
 * @details Can be a Finite Field/Integer Mod/BitVector symbolic variable or a constant.
 * Supports basic arithmetic operations: +, -, *, /.
 * Additionally
 * FFITerm supports inequalities: <, <=, >, >=
 * BVTerm supports inequalities and bitwise operations: ^, &, |, >>, <<, ror, rol
 *
 */
class STerm {
  private:
    STerm(cvc5::Term& term, Solver* s, TermType type = TermType::FFTerm)
        : solver(s)
        , term(term)
        , isFiniteField(type == TermType::FFTerm)
        , isInteger(type == TermType::FFITerm)
        , isBitVector(type == TermType::BVTerm)
        , type(type)
        , operations(typed_operations.at(type)){};

  public:
    Solver* solver;
    cvc5::Term term;

    bool isFiniteField;
    bool isInteger;
    bool isBitVector;

    TermType type;
    std::unordered_map<OpType, cvc5::Kind> operations;

    STerm()
        : solver(nullptr)
        , term(cvc5::Term())
        , isFiniteField(false)
        , isInteger(false)
        , isBitVector(false)
        , type(TermType::FFTerm){};

    explicit STerm(
        const std::string& t, Solver* slv, bool isconst = false, uint32_t base = 16, TermType type = TermType::FFTerm);

    STerm(const STerm& other) = default;
    STerm(STerm&& other) = default;

    static STerm Var(const std::string& name, Solver* slv, TermType type = TermType::FFTerm);
    static STerm Const(const std::string& val, Solver* slv, uint32_t base = 16, TermType type = TermType::FFTerm);

    STerm(bb::fr value, Solver* s, TermType type = TermType::FFTerm)
    {
        std::stringstream buf; // TODO(#893)
        buf << value;
        std::string tmp = buf.str();
        tmp[1] = '0'; // avoiding `x` in 0x prefix

        *this = Const(tmp, s, 16, type);
    }

    STerm mod() const;

    STerm& operator=(const STerm& right) = default;
    STerm& operator=(STerm&& right) = default;

    STerm operator+(const STerm& other) const;
    void operator+=(const STerm& other);
    STerm operator-(const STerm& other) const;
    void operator-=(const STerm& other);
    STerm operator-() const;

    STerm operator*(const STerm& other) const;
    void operator*=(const STerm& other);
    STerm operator/(const STerm& other) const;
    void operator/=(const STerm& other);

    void operator==(const STerm& other) const;
    void operator!=(const STerm& other) const;

    STerm operator^(const STerm& other) const;
    void operator^=(const STerm& other);
    STerm operator&(const STerm& other) const;
    void operator&=(const STerm& other);
    STerm operator|(const STerm& other) const;
    void operator|=(const STerm& other);
    STerm operator<<(const uint32_t& n) const;
    void operator<<=(const uint32_t& n);
    STerm operator>>(const uint32_t& n) const;
    void operator>>=(const uint32_t& n);

    STerm rotr(const uint32_t& n) const;
    STerm rotl(const uint32_t& n) const;

    void in(const cvc5::Term& table) const;

    operator std::string() const { return this->solver->stringify_term(term); };
    operator cvc5::Term() const { return term; };

    ~STerm() = default;

    friend std::ostream& operator<<(std::ostream& out, const STerm& term)
    {
        return out << static_cast<std::string>(term);
    };

    static STerm batch_add(const std::vector<STerm>& children)
    {
        Solver* slv = children[0].solver;
        std::vector<cvc5::Term> terms(children.begin(), children.end());
        cvc5::Term res = slv->term_manager.mkTerm(children[0].operations.at(OpType::ADD), terms);
        return { res, slv };
    }

    static STerm batch_mul(const std::vector<STerm>& children)
    {
        Solver* slv = children[0].solver;
        std::vector<cvc5::Term> terms(children.begin(), children.end());
        cvc5::Term res = slv->term_manager.mkTerm(children[0].operations.at(OpType::MUL), terms);
        return { res, slv };
    }

    /**
     * @brief Create an inclusion constraint
     *
     * @param entry the tuple entry to be checked
     * @param table lookup table that consists of tuples of lenght 3
     */
    static void in_table(std::vector<STerm>& entry, cvc5::Term& table)
    {
        STerm entry0 = entry[0];
        STerm entry1 = entry[1];
        STerm entry2 = entry[2];

        entry0 = entry0.type == TermType::FFITerm && entry0.term.getNumChildren() > 1 ? entry0.mod() : entry0;
        entry1 = entry1.type == TermType::FFITerm && entry1.term.getNumChildren() > 1 ? entry1.mod() : entry1;
        entry2 = entry2.type == TermType::FFITerm && entry2.term.getNumChildren() > 1 ? entry2.mod() : entry2;

        Solver* slv = entry[0].solver;
        cvc5::Term sym_entry = slv->term_manager.mkTuple({ entry0, entry1, entry2 });
        cvc5::Term inc = slv->term_manager.mkTerm(cvc5::Kind::SET_MEMBER, { sym_entry, table });
        slv->assertFormula(inc);
    }

    // arithmetic compatibility with Fr

    STerm operator+(const bb::fr& other) const { return *this + STerm(other, this->solver, this->type); }
    void operator+=(const bb::fr& other) { *this += STerm(other, this->solver, this->type); }
    STerm operator-(const bb::fr& other) const { return *this - STerm(other, this->solver, this->type); }
    void operator-=(const bb::fr& other) { *this -= STerm(other, this->solver, this->type); }
    STerm operator*(const bb::fr& other) const { return *this * STerm(other, this->solver, this->type); }
    void operator*=(const bb::fr& other) { *this *= STerm(other, this->solver, this->type); }
    STerm operator/(const bb::fr& other) const { return *this * STerm(other.invert(), this->solver, this->type); }
    void operator/=(const bb::fr& other) { *this *= STerm(other.invert(), this->solver, this->type); }

    void operator==(const bb::fr& other) const { *this == STerm(other, this->solver, this->type); }
    void operator!=(const bb::fr& other) const { *this != STerm(other, this->solver, this->type); }

    STerm operator^(const bb::fr& other) const { return *this ^ STerm(other, this->solver, this->type); };
    void operator^=(const bb::fr& other) { *this ^= STerm(other, this->solver, this->type); };
    STerm operator&(const bb::fr& other) const { return *this & STerm(other, this->solver, this->type); };
    void operator&=(const bb::fr& other) { *this &= STerm(other, this->solver, this->type); };
    STerm operator|(const bb::fr& other) const { return *this | STerm(other, this->solver, this->type); };
    void operator|=(const bb::fr& other) { *this |= STerm(other, this->solver, this->type); };

    void operator<(const bb::fr& other) const;
    void operator<=(const bb::fr& other) const;
    void operator>(const bb::fr& other) const;
    void operator>=(const bb::fr& other) const;
};

STerm operator+(const bb::fr& lhs, const STerm& rhs);
STerm operator-(const bb::fr& lhs, const STerm& rhs);
STerm operator*(const bb::fr& lhs, const STerm& rhs);
STerm operator/(const bb::fr& lhs, const STerm& rhs);
void operator==(const bb::fr& lhs, const STerm& rhs);
void operator!=(const bb::fr& lhs, const STerm& rhs);
STerm operator^(const bb::fr& lhs, const STerm& rhs);
STerm operator&(const bb::fr& lhs, const STerm& rhs);
STerm operator|(const bb::fr& lhs, const STerm& rhs);

STerm FFVar(const std::string& name, Solver* slv);
STerm FFConst(const std::string& val, Solver* slv, uint32_t base = 16);
STerm FFIVar(const std::string& name, Solver* slv);
STerm FFIConst(const std::string& val, Solver* slv, uint32_t base = 16);
STerm IVar(const std::string& name, Solver* slv);
STerm IConst(const std::string& val, Solver* slv, uint32_t base = 16);
STerm BVVar(const std::string& name, Solver* slv);
STerm BVConst(const std::string& val, Solver* slv, uint32_t base = 16);

} // namespace smt_terms
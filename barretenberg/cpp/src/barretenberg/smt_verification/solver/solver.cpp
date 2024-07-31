#include "solver.hpp"
#include "barretenberg/common/log.hpp"

namespace smt_solver {

/**
 * Check if the system is solvable.
 *
 * @return true if the system is solvable.
 * */
bool Solver::check()
{
    cvc5::Result result = this->solver.checkSat();
    this->checked = true;
    this->cvc_result = result;

    if (result.isUnknown()) {
        info("Unknown Result");
    }
    this->res = result.isSat();
    return this->res;
}

/**
 * If the system is solvable, extract the values for the given symbolic variables.
 * Specify the map to retrieve the values you need using the keys that are convenient for you.
 *
 * e.g. {"a": a}, where a is a symbolic term with the name "var78".
 * The return map will be {"a", value_of_a}
 *
 * @param terms A map containing pairs (name, symbolic term).
 * @return A map containing pairs (name, value).
 * */
std::unordered_map<std::string, std::string> Solver::model(std::unordered_map<std::string, cvc5::Term>& terms) const
{
    if (!this->checked) {
        throw std::length_error("Haven't checked yet");
    }
    if (!this->res) {
        throw std::length_error("There's no solution");
    }
    std::unordered_map<std::string, std::string> resulting_model;
    for (auto& term : terms) {
        cvc5::Term val = this->solver.getValue(term.second);
        std::string str_val;
        if (val.isIntegerValue()) {
            str_val = val.getIntegerValue();
        } else if (val.isFiniteFieldValue()) {
            str_val = val.getFiniteFieldValue();
        } else if (val.isBitVectorValue()) {
            str_val = val.getBitVectorValue();
        } else {
            throw std::invalid_argument("Expected Integer or FiniteField sorts. Got: " + val.getSort().toString());
        }
        resulting_model.insert({ term.first, str_val });
    }
    return resulting_model;
}

/**
 * If the system is solvable, extract the values for the given symbolic variables.
 * The return map will contain the resulting values, which are available by the
 * names of the corresponding symbolic variable.
 *
 * e.g. if the input vector is {a} and a is a term with name var78,
 * it will return {"var78": value_of_var78}
 *
 * @param terms A vector containing symbolic terms.
 * @return A map containing pairs (variable name, value).
 * */
std::unordered_map<std::string, std::string> Solver::model(std::vector<cvc5::Term>& terms) const
{
    if (!this->checked) {
        throw std::length_error("Haven't checked yet");
    }
    if (!this->res) {
        throw std::length_error("There's no solution");
    }
    std::unordered_map<std::string, std::string> resulting_model;
    for (auto& term : terms) {
        cvc5::Term val = this->solver.getValue(term);
        std::string str_val;
        if (val.isIntegerValue()) {
            str_val = val.getIntegerValue();
        } else if (val.isFiniteFieldValue()) {
            str_val = val.getFiniteFieldValue();
        } else if (val.isBitVectorValue()) {
            str_val = val.getBitVectorValue();
        } else {
            throw std::invalid_argument("Expected Integer or FiniteField sorts. Got: " + val.getSort().toString());
        }
        resulting_model.insert({ term.toString(), str_val });
    }
    return resulting_model;
}

/**
 * A simple recursive function that converts native smt language
 * to somewhat readable by humans.
 *
 * e.g. converts
 * (or (= a0 #b0000000000) (= a0 #b0000000001)) to ((a0 == 0) || (a0 == 1))
 * (= (* (+ a b) c) 10) to ((a + b) * c) == 10
 *
 * @param term cvc5 term.
 * @return Parsed term.
 * */
std::string Solver::stringify_term(const cvc5::Term& term, bool parenthesis)
{
    if (term.getKind() == cvc5::Kind::CONSTANT) {
        return term.toString();
    }
    if (term.getKind() == cvc5::Kind::CONST_FINITE_FIELD) {
        return term.getFiniteFieldValue();
    }
    if (term.getKind() == cvc5::Kind::CONST_INTEGER) {
        return term.getIntegerValue();
    }
    if (term.getKind() == cvc5::Kind::CONST_BITVECTOR) {
        return term.getBitVectorValue();
    }
    if (term.getKind() == cvc5::Kind::CONST_BOOLEAN) {
        std::vector<std::string> bool_res = { "false", "true" };
        return bool_res[static_cast<size_t>(term.getBooleanValue())];
    }
    // handling tuples
    if (term.getKind() == cvc5::Kind::APPLY_CONSTRUCTOR) {
        std::string res = "(";
        for (const auto& t : term) {
            res += stringify_term(t) + ", ";
        }
        return res + ")";
    }
    if (term.getKind() == cvc5::Kind::INTERNAL_KIND) {
        return "";
    }
    if (term.getKind() == cvc5::Kind::SET_INSERT) {
        return "set_" + std::to_string(this->tables[term]);
    }
    if (term.getKind() == cvc5::Kind::SET_EMPTY) {
        return "{}";
    }

    std::string res;
    std::string op;
    bool child_parenthesis = true;
    bool back = false;
    switch (term.getKind()) {
    case cvc5::Kind::ADD:
    case cvc5::Kind::FINITE_FIELD_ADD:
    case cvc5::Kind::BITVECTOR_ADD:
        op = " + ";
        child_parenthesis = false;
        break;
    case cvc5::Kind::SUB:
    case cvc5::Kind::BITVECTOR_SUB:
        op = " - ";
        child_parenthesis = false;
        break;
    case cvc5::Kind::NEG:
    case cvc5::Kind::FINITE_FIELD_NEG:
    case cvc5::Kind::BITVECTOR_NEG:
        res = "-";
        break;
    case cvc5::Kind::MULT:
    case cvc5::Kind::FINITE_FIELD_MULT:
    case cvc5::Kind::BITVECTOR_MULT:
        op = " * ";
        break;
    case cvc5::Kind::EQUAL:
        op = " == ";
        child_parenthesis = false;
        break;
    case cvc5::Kind::LT:
    case cvc5::Kind::BITVECTOR_ULT:
        op = " < ";
        break;
    case cvc5::Kind::GT:
    case cvc5::Kind::BITVECTOR_UGT:
        op = " > ";
        break;
    case cvc5::Kind::LEQ:
    case cvc5::Kind::BITVECTOR_ULE:
        op = " <= ";
        break;
    case cvc5::Kind::GEQ:
    case cvc5::Kind::BITVECTOR_UGE:
        op = " >= ";
        break;
    case cvc5::Kind::XOR:
    case cvc5::Kind::BITVECTOR_XOR:
        op = " ^ ";
        break;
    case cvc5::Kind::BITVECTOR_OR:
        op = " | ";
        break;
    case cvc5::Kind::OR:
        op = " || ";
        break;
    case cvc5::Kind::BITVECTOR_AND:
        op = " & ";
        break;
    case cvc5::Kind::BITVECTOR_SHL:
        op = " << ";
        break;
    case cvc5::Kind::BITVECTOR_LSHR:
        op = " >> ";
        break;
    case cvc5::Kind::BITVECTOR_ROTATE_LEFT:
        back = true;
        op = " ><< " + term.getOp()[0].toString();
        break;
    case cvc5::Kind::BITVECTOR_ROTATE_RIGHT:
        back = true;
        op = " >>< " + term.getOp()[0].toString();
        break;
    case cvc5::Kind::AND:
        op = " && ";
        break;
    case cvc5::Kind::NOT:
        res = "!";
        break;
    case cvc5::Kind::INTS_MODULUS:
        op = " % ";
        parenthesis = true;
        break;
    case cvc5::Kind::SET_MEMBER:
        op = " in ";
        parenthesis = true;
        break;
    default:
        info("Invalid operand :", term.getKind());
        info(term);
        break;
    }

    size_t i = 0;
    cvc5::Term child;
    for (const auto& t : term) {
        if (i == term.getNumChildren() - 1) {
            child = t;
            break;
        }
        res += stringify_term(t, child_parenthesis) + op;
        i += 1;
    }

    res = res + stringify_term(child, child_parenthesis);
    if (back) {
        res += op;
    }
    if (parenthesis) {
        return "(" + res + ")";
    }
    return res;
}

/**
 * Output assertions in human readable format.
 *
 * */
void Solver::print_assertions()
{
    for (const auto& t : this->solver.getAssertions()) {
        info(this->stringify_term(t));
    }
}

cvc5::Term Solver::create_lookup_table(std::vector<std::vector<cvc5::Term>>& table)
{
    cvc5::Term tmp = table[0][0];
    cvc5::Sort tuple_sort = this->term_manager.mkTupleSort({ tmp.getSort(), tmp.getSort(), tmp.getSort() });
    cvc5::Sort relation = this->term_manager.mkSetSort(tuple_sort);
    cvc5::Term resulting_table = this->term_manager.mkEmptySet(relation);

    std::vector<cvc5::Term> children;
    children.reserve(table.size() + 1);
    for (auto& table_entry : table) {
        cvc5::Term entry = this->term_manager.mkTuple(table_entry);
        children.push_back(entry);
    }
    children.push_back(resulting_table);
    cvc5::Term res = this->term_manager.mkTerm(cvc5::Kind::SET_INSERT, children);
    size_t cursize = this->tables.size();
    info("Creating table for op: ", children.size(), ", № ", cursize);
    this->tables.insert({ res, cursize });
    return res;
}

cvc5::Term Solver::create_table(std::vector<cvc5::Term>& table)
{
    cvc5::Term tmp = table[0];
    cvc5::Sort relation = this->term_manager.mkSetSort(tmp.getSort());
    cvc5::Term resulting_table = this->term_manager.mkEmptySet(relation);

    std::vector<cvc5::Term> children;
    children.reserve(table.size() + 1);
    for (auto& table_entry : table) {
        children.push_back(table_entry);
    }
    children.push_back(resulting_table);
    cvc5::Term res = this->term_manager.mkTerm(cvc5::Kind::SET_INSERT, children);
    size_t cursize = this->tables.size();
    info("Creating table for range: ", children.size(), ", № ", cursize);

    this->tables.insert({ res, cursize });
    return res;
}
}; // namespace smt_solver

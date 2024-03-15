#include "bool.hpp"

namespace smt_terms {

Bool Bool::operator|(const Bool& other) const
{
    cvc5::Term res = solver->term_manager.mkTerm(cvc5::Kind::OR, { this->term, other.term });
    ;
    return { res, this->solver };
}

void Bool::operator|=(const Bool& other)
{
    this->term = this->solver->term_manager.mkTerm(cvc5::Kind::OR, { this->term, other.term });
}

Bool Bool::operator&(const Bool& other) const
{
    cvc5::Term res = solver->term_manager.mkTerm(cvc5::Kind::AND, { this->term, other.term });
    return { res, this->solver };
}

void Bool::operator&=(const Bool& other)
{
    this->term = this->solver->term_manager.mkTerm(cvc5::Kind::AND, { this->term, other.term });
}

Bool Bool::operator==(const Bool& other) const
{
    cvc5::Term res = solver->term_manager.mkTerm(cvc5::Kind::EQUAL, { this->term, other.term });
    return { res, this->solver };
}

Bool Bool::operator!=(const Bool& other) const
{
    cvc5::Term res = solver->term_manager.mkTerm(cvc5::Kind::EQUAL, { this->term, other.term });
    res = solver->term_manager.mkTerm(cvc5::Kind::NOT, { res });
    return { res, this->solver };
}

Bool Bool::operator!() const
{
    cvc5::Term res = solver->term_manager.mkTerm(cvc5::Kind::NOT, { this->term });
    return { res, this->solver };
}
}; // namespace smt_terms
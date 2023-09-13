#include "bool.hpp"

namespace smt_terms {

Bool Bool::operator|(const Bool& other) const
{
    cvc5::Term res = solver->mkTerm(cvc5::Kind::OR, { this->term, other.term });
    ;
    return { res, this->solver };
}

void Bool::operator|=(const Bool& other)
{
    this->term = this->solver->mkTerm(cvc5::Kind::OR, { this->term, other.term });
}

Bool Bool::operator&(const Bool& other) const
{
    cvc5::Term res = solver->mkTerm(cvc5::Kind::AND, { this->term, other.term });
    return { res, this->solver };
}

void Bool::operator&=(const Bool& other)
{
    this->term = this->solver->mkTerm(cvc5::Kind::AND, { this->term, other.term });
}

Bool Bool::operator==(const Bool& other) const
{
    cvc5::Term res = solver->mkTerm(cvc5::Kind::EQUAL, { this->term, other.term });
    return { res, this->solver };
}

Bool Bool::operator!=(const Bool& other) const
{
    cvc5::Term res = solver->mkTerm(cvc5::Kind::EQUAL, { this->term, other.term });
    res = solver->mkTerm(cvc5::Kind::EQUAL, { res, this->solver->mkBoolean(false) });
    return { res, this->solver };
}
}; // namespace smt_terms
#include "config.hpp"
#include "statement_def.hpp"

namespace noir {
namespace parser {
BOOST_SPIRIT_INSTANTIATE(statement_type, iterator_type, context_type);
BOOST_SPIRIT_INSTANTIATE(function_statement_type, iterator_type, context_type);
} // namespace parser
} // namespace noir

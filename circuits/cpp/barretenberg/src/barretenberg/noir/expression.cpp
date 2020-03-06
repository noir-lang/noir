#include "config.hpp"
#include "expression_def.hpp"
#include <boost/fusion/include/io.hpp>
#include <boost/spirit/home/x3/support/ast/position_tagged.hpp>

namespace noir {
namespace parser {
BOOST_SPIRIT_INSTANTIATE(expression_type, iterator_type, context_type);
}
} // namespace noir

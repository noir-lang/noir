#pragma once
#include "error_handler.hpp"
#include "skipper.hpp"
#include <boost/spirit/home/x3.hpp>

namespace noir {
namespace parser {

typedef std::string::const_iterator iterator_type;
typedef x3::phrase_parse_context<decltype(space_comment)>::type phrase_context_type;
typedef error_handler<iterator_type> error_handler_type;
typedef x3::context<error_handler_tag, std::reference_wrapper<error_handler_type>, phrase_context_type> context_type;

} // namespace parser
} // namespace noir

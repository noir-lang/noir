#include "../common/log.hpp"
#include "parse.hpp"
#include "config.hpp"
#include "expression.hpp"
#include "skipper.hpp"
#include <iostream>

namespace noir {
namespace parser {

template <typename T, typename AST> AST parse(iterator_type begin, iterator_type end, T const& parser)
{
    AST ast;

    using boost::spirit::x3::with;
    using parser::error_handler_type;
    error_handler_type error_handler(begin, end, std::cerr);

    // we pass our error handler to the parser so we can access it later on in our on_error and on_sucess handlers.
    auto const eparser = with<error_handler_tag>(std::ref(error_handler))[parser];

    bool success = phrase_parse(begin, end, eparser, space_comment, ast);

    if (!success || begin != end) {
        std::ostringstream os;
        os << "Parser failed at: " << std::string(begin, begin + 10);
        abort(os.str());
    }

    return ast;
}

ast::statement_list parse(iterator_type begin, iterator_type end)
{
    return parse<statement_type, ast::statement_list>(begin, end, statement());
}

ast::statement_list parse(std::string const& source)
{
    return parse(source.begin(), source.end());
}

ast::function_statement_list parse_function_statements(std::string const& source)
{
    return parse<function_statement_type, ast::function_statement_list>(
        source.begin(), source.end(), function_statement());
}

} // namespace parser
} // namespace noir
#pragma once
#include "../parser/ast.hpp"
#include <map>
#include <vector>

namespace noir {
namespace printer {

struct printer {
    typedef void result_type;

    void operator()(ast::nil) const { BOOST_ASSERT(0); }
    void operator()(unsigned int x) const;
    void operator()(bool x) const;
    void operator()(ast::variable const& x) const;
    void operator()(ast::operation const& x) const;
    void operator()(ast::unary const& x) const;
    void operator()(ast::expression const& x) const;
    void operator()(ast::assignment const& x) const;
    void operator()(ast::function_declaration const& x) const;
    void operator()(ast::function_call const& x) const;
    void operator()(ast::variable_declaration const& x) const;
    void operator()(ast::statement_list const& x) const;
    void operator()(ast::statement const& x) const;
    void operator()(ast::constant const& x) const;
    void operator()(ast::array const& x) const;
    void operator()(ast::for_statement const& x) const;
    void operator()(ast::return_expr const& x) const;

    void start(ast::statement_list const& x) const;
};

} // namespace printer
} // namespace noir

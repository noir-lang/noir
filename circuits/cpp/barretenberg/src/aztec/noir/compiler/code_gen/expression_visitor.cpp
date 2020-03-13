#include "expression_visitor.hpp"
#include "function_call.hpp"
#include "function_statement_visitor.hpp"
#include "../common/log.hpp"
#include "operators.hpp"
#include "type_info_from.hpp"
#include <boost/format.hpp>
#include <iostream>

namespace noir {
namespace code_gen {

ExpressionVisitor::ExpressionVisitor(CompilerContext& ctx, type_info const& target_type)
    : ctx_(ctx)
    , target_type_(target_type)
{}

var_t ExpressionVisitor::operator()(unsigned int x)
{
    debug("uint_nt constant (target type %1%): %2%", target_type_, x);
    auto it = boost::get<int_type>(&target_type_.type);
    if (!it) {
        abort(format("Cannot create type %s from constant %d.", target_type_.type_name(), x));
    }
    return var_t(uint_nt(it->width, &ctx_.composer, x), target_type_);
}

var_t ExpressionVisitor::operator()(bool x)
{
    debug("bool %1%", x);
    if (!boost::get<bool_type>(&target_type_.type)) {
        abort(format("Cannot create type %s from constant %d.", target_type_.type_name(), x));
    }
    return var_t(bool_ct(&ctx_.composer, x), target_type_);
}

var_t ExpressionVisitor::operator()(ast::array const& x)
{
    debug("defining array of size %1%", x.size());
    auto arr = boost::get<array_type>(&target_type_.type);
    if (!arr) {
        abort(format("Cannot create type %s from array.", target_type_.type_name()));
    }
    std::vector<var_t> result;
    std::transform(x.begin(), x.end(), std::back_inserter(result), [this, arr](ast::expression const& e) {
        return ExpressionVisitor(ctx_, arr->element_type)(e);
    });

    return var_t(result, target_type_);
}

var_t ExpressionVisitor::operator()(var_t vlhs, ast::operation const& x)
{
    if (x.operator_ == ast::op_index) {
        debug("op_index");

        auto rhs = boost::apply_visitor(ExpressionVisitor(ctx_, type_uint32), x.operand_);

        // Evaluate index.
        uint_nt* iptr = boost::get<uint_nt>(&rhs.value());
        if (!iptr) {
            abort("Index must be an integer.");
        }
        uint32_t i = static_cast<uint32_t>((*iptr).get_value());

        return boost::apply_visitor(IndexVisitor(i), vlhs.value());
    }

    var_t vrhs = boost::apply_visitor(*this, x.operand_);
    var_t::value_t& lhs = vlhs.value();
    var_t::value_t& rhs = vrhs.value();

    switch (x.operator_) {
    case ast::op_plus: {
        auto t = boost::apply_visitor(AdditionVisitor(), lhs, rhs);
        debug("op_add %1% + %2% = %3%", vlhs, vrhs, t);
        return t;
    }
    case ast::op_minus:
        debug("op_sub");
        return boost::apply_visitor(SubtractionVisitor(), lhs, rhs);
    case ast::op_times:
        debug("op_times");
        return boost::apply_visitor(MultiplyVisitor(), lhs, rhs);
    case ast::op_divide:
        debug("op_divide");
        return boost::apply_visitor(DivideVisitor(), lhs, rhs);
    case ast::op_mod:
        debug("op_mod");
        return boost::apply_visitor(ModVisitor(), lhs, rhs);

    case ast::op_equal:
        debug("op_equal");
        return boost::apply_visitor(EqualityVisitor(), lhs, rhs);
    case ast::op_not_equal:
        debug("op_ne");
        break;
    case ast::op_less:
        debug("op_lt");
        break;
    case ast::op_less_equal:
        debug("op_lte");
        break;
    case ast::op_greater:
        debug("op_gt");
        break;
    case ast::op_greater_equal:
        debug("op_gte");
        break;

    case ast::op_and:
        debug("op_and");
        break;
    case ast::op_or:
        debug("op_or");
        break;

    case ast::op_bitwise_and:
        debug("op_bitwise_and");
        return boost::apply_visitor(BitwiseAndVisitor(), lhs, rhs);
    case ast::op_bitwise_or:
        debug("op_bitwise_or");
        return boost::apply_visitor(BitwiseOrVisitor(), lhs, rhs);
    case ast::op_bitwise_xor:
        debug("op_bitwise_xor");
        return boost::apply_visitor(BitwiseXorVisitor(), lhs, rhs);
    case ast::op_bitwise_ror:
        debug("op_bitwise_ror");
        return boost::apply_visitor(BitwiseRorVisitor(), lhs, rhs);
    case ast::op_bitwise_rol:
        debug("op_bitwise_rol");
        return boost::apply_visitor(BitwiseRolVisitor(), lhs, rhs);
    case ast::op_bitwise_shl:
        debug("op_bitwise_shl");
        return boost::apply_visitor(BitwiseShlVisitor(), lhs, rhs);
    case ast::op_bitwise_shr:
        debug("op_bitwise_shr");
        return boost::apply_visitor(BitwiseShrVisitor(), lhs, rhs);

    default:
        BOOST_ASSERT(0);
    }

    return vlhs;
}

var_t ExpressionVisitor::operator()(ast::unary const& x)
{
    var_t var = (*this)(x.operand_);

    switch (x.operator_) {
    case ast::op_negative:
        debug("op_neg");
        return boost::apply_visitor(NegVis(), var.value());
    case ast::op_not: {
        debug("op_not ");
        auto v = boost::apply_visitor(NotVis(), var.value());
        return v;
    }
    case ast::op_bitwise_not: {
        debug("op_bitwise_not");
        auto v = boost::apply_visitor(BitwiseNotVisitor(), var.value());
        return v;
    }
    case ast::op_positive:
        return var;
    default:
        abort("Unknown operator.");
    }
}

var_t ExpressionVisitor::operator()(ast::expression const& x)
{
    var_t var = boost::apply_visitor(*this, x.first);
    for (ast::operation const& oper : x.rest) {
        var = (*this)(var, oper);
    }
    return var;
}

var_t ExpressionVisitor::operator()(ast::variable const& x)
{
    auto v = ctx_.symbol_table[x.name];
    debug("variable %1%: %2%", x.name, v);
    return v;
}

struct IndexedAssignVisitor : boost::static_visitor<var_t> {
    IndexedAssignVisitor(CompilerContext& ctx, size_t i, ast::expression const& rhs_expr, type_info const& ti)
        : ctx(ctx)
        , i(i)
        , rhs_expr(rhs_expr)
        , ti(ti)
    {}

    template <typename T> var_t operator()(std::vector<T>& lhs) const
    {
        // Evaluate rhs of assignment, should resolve to lhs element type.
        auto arr = boost::get<array_type>(ti.type);
        var_t rhs = ExpressionVisitor(ctx, arr.element_type)(rhs_expr);
        debug("indexed assign %1% %2%->%3%", i, lhs[i], rhs);
        return lhs[i] = rhs;
    }

    var_t operator()(uint_nt& lhs) const
    {
        // Evaluate rhs of assignment, should resolve to bool.
        var_t rhs = ExpressionVisitor(ctx, type_bool)(rhs_expr);
        bool_ct bit = boost::get<bool_ct>(rhs.value());
        std::vector<bool_ct> wires(lhs.width());
        size_t flipped = lhs.width() - i - 1;
        for (size_t j = 0; j < lhs.width(); ++j) {
            wires[j] = j == flipped ? bit : lhs.at(j);
        }
        lhs = uint_nt(&ctx.composer, wires);
        debug("indexed assign bit %1% to %2% = lhs(%3%)", i, bit, lhs);
        return bit;
    }

    template <typename T> var_t operator()(T const& t) const
    {
        abort(format("Unsupported type in indexed assign: %s", typeid(t).name()));
    }

    CompilerContext& ctx;
    size_t i;
    ast::expression const& rhs_expr;
    type_info const& ti;
};

var_t ExpressionVisitor::operator()(ast::assignment const& x)
{
    debug("get symbol ref for assign %1%", x.lhs.name);
    var_t lhs = ctx_.symbol_table[x.lhs.name];
    var_t* lhs_ptr = &lhs;

    // If our lhs has indexes, we need to get the ref to indexed element.
    if (x.lhs.indexes.size()) {
        for (size_t j = 0; j < x.lhs.indexes.size() - 1; ++j) {
            // Evaluate index.
            auto ivar = ExpressionVisitor(ctx_, type_uint32)(x.lhs.indexes[0]);
            auto iv = boost::get<uint_nt>(ivar.value());
            if (!iv.is_constant()) {
                abort("Index must be constant.");
            }
            size_t i = static_cast<size_t>(iv.get_value());
            auto arr = boost::get<array_type>(lhs_ptr->type.type);
            if (i >= arr.size) {
                abort("Index out of bounds.");
            }
            lhs_ptr = &boost::get<std::vector<var_t>>(lhs_ptr->value())[i];
            debug("indexed to new lhs: %1%", *lhs_ptr);
        }

        // Evaluate final index.
        auto ivar = ExpressionVisitor(ctx_, type_uint32)(x.lhs.indexes.back());
        uint_nt iv = boost::get<uint_nt>(ivar.value());
        if (!iv.is_constant()) {
            abort("Index must be constant.");
        }
        size_t i = static_cast<size_t>(iv.get_value());

        return boost::apply_visitor(IndexedAssignVisitor(ctx_, i, x.rhs, lhs_ptr->type), lhs_ptr->value());
    } else {
        var_t rhs = ExpressionVisitor(ctx_, lhs.type)(x.rhs);
        debug("op_store %1% = %2%", x.lhs.name, rhs);
        lhs.value() = rhs.value();
        // ctx_.symbol_table.set(rhs, x.lhs.name);
        return rhs;
    }
}

var_t ExpressionVisitor::operator()(ast::function_call const& x)
{
    debug("function call %1%", x.name);

    auto builtin = builtin_lookup(ctx_, x.name);
    if (builtin) {
        std::vector<var_t> args;
        for (size_t i = 0; i < x.args.size(); ++i) {
            // We need differing types here, but for now just trying to get length() working.
            var_t arg = ExpressionVisitor(ctx_, type_uint32)(x.args[i]);
            args.push_back(arg);
        }
        return builtin(args);
    }

    auto func = function_lookup(ctx_, x.name, x.args.size());

    std::vector<var_t> args;
    for (size_t i = 0; i < x.args.size(); ++i) {
        auto ti = type_info_from_type_id(func.args[i].type);
        var_t arg = ExpressionVisitor(ctx_, ti)(x.args[i]);
        args.push_back(arg);
    }

    return function_call(ctx_, func, args);
}

var_t ExpressionVisitor::operator()(ast::constant const& x)
{
    return boost::apply_visitor(*this, x);
}

} // namespace code_gen
} // namespace noir
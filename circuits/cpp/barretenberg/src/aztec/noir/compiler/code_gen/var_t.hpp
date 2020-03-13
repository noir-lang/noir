#pragma once
#include "../parser/ast.hpp"
#include <stdlib/types/turbo.hpp>
#include "../common/format.hpp"
#include "lambda_visitor.hpp"
#include "type_info.hpp"
#include "uint_nt.hpp"
#include <sstream>

namespace noir {
namespace code_gen {

using namespace plonk::stdlib::types::turbo;

struct var_t;

struct var_t_ref {
    var_t_ref(var_t& v)
        : v(&v)
    {}

    var_t_ref(var_t_ref const& other)
        : v(other.v)
    {}

    var_t_ref& operator=(var_t_ref const& rhs)
    {
        v = rhs.v;
        return *this;
    }

    var_t* operator->() const { return v; }

    var_t* v;
};

struct var_t {
    typedef boost::variant<bool_ct, uint_nt, boost::recursive_wrapper<std::vector<var_t>>> value_t;
    typedef boost::variant<value_t, var_t_ref> internal_value_t;

    var_t(value_t const& value, type_info const& type)
        : type(type)
        , value_(value){};

    var_t(uint_nt value)
        : type(int_type(value.width(), false))
        , value_(value)
    {}

    var_t(std::string const& str, Composer& composer)
        : type(array_type(type_uint8.type, str.length()))
    {
        std::vector<var_t> values;
        std::transform(str.begin(), str.end(), std::back_inserter(values), [&](char c) {
            return uint_nt(8, witness_ct(&composer, c));
        });
        value_ = values;
    }

    var_t(std::vector<uint8_t> const& input, Composer& composer)
        : type(array_type(type_uint8.type, input.size()))
    {
        std::vector<var_t> values;
        std::transform(input.begin(), input.end(), std::back_inserter(values), [&](uint8_t c) {
            return uint_nt(8, witness_ct(&composer, c));
        });
        value_ = values;
    }

    var_t(std::vector<uint32_t> const& input, Composer& composer)
        : type(array_type(type_uint32.type, input.size()))
    {
        std::vector<var_t> values;
        std::transform(input.begin(), input.end(), std::back_inserter(values), [&](uint32_t c) {
            return uint_nt(32, witness_ct(&composer, c));
        });
        value_ = values;
    }

    var_t(char value)
        : type(type_uint8)
        , value_(uint_nt(8, (uint8_t)value))
    {}

    var_t(bool_ct value)
        : type(type_bool)
        , value_(value)
    {}

    var_t(var_t_ref value)
        : type(value.v->type)
        , value_(value)
    {}

    template <typename T>
    var_t(std::vector<T> value)
        : type(array_type{ .size = value.size(), .element_type = value[0].type.type })
        , value_(value)
    {}

    var_t(var_t const& rhs)
        : type(rhs.type)
        , value_(rhs.value_)
    {}

    var_t(var_t&& rhs)
        : type(std::move(rhs.type))
        , value_(std::move(rhs.value_))
    {}

    var_t& operator=(var_t const& rhs)
    {
        value_ = rhs.value_;
        type = rhs.type;
        return *this;
    }

    std::string const to_string() const;

    value_t& value() { return boost::apply_visitor(*this, value_); }

    value_t const& value() const { return boost::apply_visitor(*this, value_); }

    value_t& operator()(value_t& v) { return v; }

    value_t& operator()(var_t_ref& v) { return v->value(); }

    value_t const& operator()(value_t const& v) const { return v; }

    value_t const& operator()(var_t_ref const& v) const { return v->value(); }

    type_info type;

  private:
    internal_value_t value_;
};

struct var_t_printer : boost::static_visitor<std::ostream&> {
    var_t_printer(std::ostream& os)
        : os(os)
    {}
    result_type operator()(uint_nt const& v) const { return os << v; }
    result_type operator()(bool_ct const& v) const { return os << v; }

    result_type operator()(std::vector<var_t> const& v) const
    {
        os << "[";
        for (auto it = v.begin(); it != v.end(); ++it) {
            it->value().apply_visitor(*this);
            if (it != --v.end()) {
                os << ", ";
            }
        }
        return os << "]";
    }

    result_type operator()(var_t_ref const& v) const { return v->value().apply_visitor(*this); }

    std::ostream& os;
};

inline std::string const var_t::to_string() const
{
    std::ostringstream os;
    boost::apply_visitor(var_t_printer(os), value());
    return os.str();
}

struct VarTFactoryVisitor : boost::static_visitor<var_t> {
    VarTFactoryVisitor(type_info const& type, Composer& composer)
        : composer(composer)
        , type(type){};
    result_type operator()(bool_type const&) const { return var_t(bool_ct(&composer), type); }
    result_type operator()(int_type const& t) const { return var_t(uint_nt(t.width, &composer), type); }
    // result_type operator()(int_type const& t) const { return var_t(uint(t.width, &composer), type); }
    result_type operator()(array_type const& arr) const
    {
        var_t defaultElement = boost::apply_visitor(VarTFactoryVisitor(arr.element_type, composer), arr.element_type);
        return var_t(std::vector<var_t>(arr.size, defaultElement), type);
    }
    Composer& composer;
    type_info const& type;
};

inline var_t var_t_factory(type_info const& type, Composer& composer)
{
    return boost::apply_visitor(VarTFactoryVisitor(type, composer), type.type);
}

inline std::ostream& operator<<(std::ostream& os, var_t const& v)
{
    os << "(" << v.type << ")";
    return boost::apply_visitor(var_t_printer(os), v.value());
}

} // namespace code_gen
} // namespace noir
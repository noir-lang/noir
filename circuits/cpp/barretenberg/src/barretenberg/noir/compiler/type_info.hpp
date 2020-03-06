#pragma once
#include "../ast.hpp"
#include <boost/algorithm/string/join.hpp>
#include <unordered_map>

namespace noir {
namespace code_gen {

struct int_type {
    int_type(size_t w, bool s)
        : width(w)
        , signed_(s)
    {}
    bool operator==(int_type const& other) const { return width == other.width && signed_ == other.signed_; }
    size_t width;
    bool signed_;
};

struct bool_type {
    bool operator==(bool_type const&) const { return true; }
};
struct array_type;
struct tuple_type;
struct struct_type;

typedef boost::variant<bool_type, int_type> intrinsic_type;

typedef boost::variant<bool_type,
                       int_type,
                       boost::recursive_wrapper<array_type>/*,
                       boost::recursive_wrapper<tuple_type>,
                       boost::recursive_wrapper<struct_type>*/>
    noir_type;

struct array_type {
    array_type(noir_type et, size_t s)
        : element_type(et)
        , size(s)
    {}

    bool operator==(array_type const& other) const
    {
        return element_type == other.element_type && (size == other.size || size == 0 || other.size == 0);
    }

    noir_type element_type;
    size_t size;
};

struct tuple_type {
    std::vector<noir_type> types;
};

struct struct_type {
    std::unordered_map<std::string, noir_type> fields;
};

struct IntrinsicTypeInfoVisitor : boost::static_visitor<intrinsic_type> {
    result_type operator()(ast::bool_type const&) const { return bool_type{}; }
    result_type operator()(ast::int_type const& v) const { return int_type(v.size, v.type == "int"); }
};

struct TypeIdNameVisitor : boost::static_visitor<std::string const> {
    result_type operator()(bool_type const&) const { return "bool"; }
    result_type operator()(int_type const& v) const { return format("uint%d", v.width); }
    result_type operator()(array_type const& v) const { return format("%s[%d]", (*this)(v.element_type), v.size); }
    result_type operator()(tuple_type const& v) const
    {
        std::vector<std::string> result;
        std::transform(
            v.types.begin(), v.types.end(), result.begin(), [this](noir_type const& t) { return (*this)(t); });
        return format("(%s)", boost::algorithm::join(result, ","));
    }
    result_type operator()(noir_type const& v) const { return v.apply_visitor(*this); }
};

struct type_info {
    type_info(noir_type const& t, size_t array_size, bool m = false)
        : type(t)
        , mutable_(m)
    {
        type = array_type(type, array_size);
    }

    type_info(noir_type const& t, bool m = false)
        : type(t)
        , mutable_(m)
    {}

    type_info(type_info const& other)
        : type(other.type)
        , mutable_(other.mutable_)
    {}

    type_info& operator=(const type_info& other) = default;

    std::string const type_name() const { return boost::apply_visitor(TypeIdNameVisitor(), type); }

    bool operator==(type_info const& other) const { return type == other.type; }

    bool operator!=(type_info const& other) const { return !operator==(other); }

    noir_type type;
    bool mutable_;
};

inline std::ostream& operator<<(std::ostream& os, type_info const& t)
{
    return os << t.type_name();
}

const auto type_uint32 = type_info(int_type(32, false));
const auto type_uint8 = type_info(int_type(8, false));
const auto type_bool = type_info(bool_type());

} // namespace code_gen
} // namespace noir
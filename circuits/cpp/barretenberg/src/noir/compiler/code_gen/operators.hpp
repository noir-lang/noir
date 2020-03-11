#pragma once
#include "../common/log.hpp"

namespace noir {
namespace code_gen {

struct AdditionVisitor : boost::static_visitor<var_t> {
    var_t operator()(uint_nt const& lhs, uint_nt const& rhs) const { return lhs + rhs; }
    template <typename T, typename U> var_t operator()(T const&, U const&) const { abort("Cannot perform add."); }
};

struct SubtractionVisitor : boost::static_visitor<var_t> {
    var_t operator()(uint_nt& lhs, uint_nt const& rhs) const { return lhs - rhs; }
    template <typename T, typename U> var_t operator()(T const&, U const&) const
    {
        abort("Cannot perform subtraction.");
    }
};

struct MultiplyVisitor : boost::static_visitor<var_t> {
    var_t operator()(uint_nt& lhs, uint_nt const& rhs) const { return lhs * rhs; }
    template <typename T, typename U> var_t operator()(T const& lhs, U const& rhs) const
    {
        std::cout << typeid(lhs).name() << std::endl;
        std::cout << typeid(rhs).name() << std::endl;
        abort("Cannot perform multiplication.");
    }
};

struct DivideVisitor : boost::static_visitor<var_t> {
    var_t operator()(uint_nt& lhs, uint_nt const& rhs) const { return lhs / rhs; }
    template <typename T, typename U> var_t operator()(T const&, U const&) const { abort("Cannot perform division."); }
};

struct ModVisitor : boost::static_visitor<var_t> {
    var_t operator()(uint_nt& lhs, uint_nt const& rhs) const
    {
        if (!lhs.is_constant() || !rhs.is_constant()) {
            abort("Can only modulo constants.");
        }
        return uint_nt(lhs.width(), lhs.get_value() % rhs.get_value());
    }
    template <typename T, typename U> var_t operator()(T const&, U const&) const { abort("Cannot perform modulo."); }
};

struct EqualityVisitor : boost::static_visitor<var_t> {
    template <typename T> var_t operator()(std::vector<T> const&, std::vector<T> const&) const
    {
        abort("No array equality.");
    }
    template <typename T> var_t operator()(T const& lhs, T const& rhs) const { return lhs == rhs; }

    template <typename T, typename U> var_t operator()(T const&, U const&) const
    {
        abort("Cannot compare differing types.");
    }
};

struct BitwiseOrVisitor : boost::static_visitor<var_t> {
    template <typename T> var_t operator()(std::vector<T>&, std::vector<T> const&) const { abort("No array support."); }
    template <typename T> var_t operator()(T& lhs, T const& rhs) const { return lhs | rhs; }

    template <typename T, typename U> var_t operator()(T const&, U const&) const
    {
        abort("Cannot OR differing types.");
    }
};

struct BitwiseAndVisitor : boost::static_visitor<var_t> {
    template <typename T> var_t operator()(std::vector<T>&, std::vector<T> const&) const { abort("No array support."); }
    template <typename T> var_t operator()(T& lhs, T const& rhs) const { return lhs & rhs; }

    template <typename T, typename U> var_t operator()(T const&, U const&) const
    {
        abort("Cannot AND differing types.");
    }
};

struct BitwiseXorVisitor : boost::static_visitor<var_t> {
    template <typename T> var_t operator()(std::vector<T>&, std::vector<T> const&) const { abort("No array support."); }
    template <typename T> var_t operator()(T& lhs, T const& rhs) const { return lhs ^ rhs; }

    template <typename T, typename U> var_t operator()(T const&, U const&) const
    {
        abort("Cannot XOR differing types.");
    }
};

struct BitwiseRorVisitor : boost::static_visitor<var_t> {
    var_t operator()(uint_nt& lhs, uint_nt const& rhs) const
    {
        if (!rhs.is_constant()) {
            abort("Can only perform bitwise rotation by constants.");
        }
        return lhs.ror(static_cast<size_t>(rhs.get_value()));
    }
    template <typename T, typename U> var_t operator()(T const&, U const&) const { abort("Cannot rotate right."); }
};

struct BitwiseRolVisitor : boost::static_visitor<var_t> {
    var_t operator()(uint_nt& lhs, uint_nt const& rhs) const
    {
        if (!rhs.is_constant()) {
            abort("Can only perform bitwise rotation by constants.");
        }
        return lhs.rol(static_cast<size_t>(rhs.get_value()));
    }
    template <typename T, typename U> var_t operator()(T const&, U const&) const { abort("Cannot rotate left."); }
};

struct BitwiseShlVisitor : boost::static_visitor<var_t> {
    var_t operator()(uint_nt& lhs, uint_nt const& rhs) const
    {
        if (!rhs.is_constant()) {
            abort("Can only perform bitwise shift by constants.");
        }
        return lhs << static_cast<size_t>(rhs.get_value());
    }
    template <typename T, typename U> var_t operator()(T const&, U const&) const { abort("Cannot shift left."); }
};

struct BitwiseShrVisitor : boost::static_visitor<var_t> {
    var_t operator()(uint_nt& lhs, uint_nt const& rhs) const
    {
        if (!rhs.is_constant()) {
            abort("Can only perform bitwise shift by constants.");
        }
        return lhs >> static_cast<size_t>(rhs.get_value());
    }
    template <typename T, typename U> var_t operator()(T const&, U const&) const { abort("Cannot shift right."); }
};

struct NegVis : boost::static_visitor<var_t> {
    template <typename T> var_t operator()(std::vector<T> const&) const { abort("No array support."); }
    var_t operator()(bool_ct const&) const { abort("Cannot neg bool."); }
    var_t operator()(uint_nt const&) const { abort("Cannot neg uint_nt."); }
};

struct NotVis : boost::static_visitor<var_t> {
    template <typename T> var_t operator()(std::vector<T> const&) const { abort("No array support."); }
    var_t operator()(bool_ct const& var) const { return !var; }
    var_t operator()(uint_nt const&) const { abort("Cannot NOT a uint_nt."); }
};

struct BitwiseNotVisitor : boost::static_visitor<var_t> {
    template <typename T> var_t operator()(std::vector<T> const&) const { abort("No array support."); }
    var_t operator()(bool_ct& var) const { return ~var; }
    var_t operator()(uint_nt& var) const { return ~var; }
};

struct IndexVisitor : boost::static_visitor<var_t> {
    IndexVisitor(size_t i)
        : i(i)
    {}

    var_t operator()(std::vector<var_t>& lhs) const
    {
        debug("indexing %1%: %2%", i, lhs[i]);
        return var_t_ref(lhs[i]);
    }
    var_t operator()(noir::code_gen::uint_nt& lhs) const
    {
        bool_ct bit = lhs.at(lhs.width() - i - 1);
        debug("indexing uint_nt for bit %1%: %2%", i, bit);
        return bit;
    }
    template <typename T> var_t operator()(T& t) const
    {
        abort(format("Cannot index given type: %s", typeid(t).name()));
    }

    size_t i;
};

} // namespace code_gen
} // namespace noir
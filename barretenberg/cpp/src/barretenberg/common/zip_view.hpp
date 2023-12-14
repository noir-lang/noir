#pragma once
/* ********************************* FILE ************************************/
/** \file    mzip.hpp
 *
 * \brief    This header contains the zip iterator class.
 *
 * WARNING this is a zip view, not a zip copy!
 *
 * \remark
 * - c++17
 * - no dependencies
 * - header only
 * - tested by test_zip_iterator.cpp
 * - not thread safe
 * - view !
 * - extends lifetime of rvalue inputs untill the end of the for loop
 *
 * \todo
 * - add algorithm tests, probably does not work at all...
 *
 *
 * \example
 * std::vector<int> as{1,2},bs{1,2,3};
 * for(auto [index, a,b]: zip(as,bs)){
 *  a++;
 * }
 * cout<<as<<endl; // shows (2, 3)
 * works for any number
 *
 * zip returns tuples of references to the contents
 *
 *
 *
 *
 *
 *
 *
 *
 *
 * does not copy the containers
 * returns tuple of references to the containers content
 * iterates untill the first iterator hits end.
 * extends ownership to the end of the for loop, or untill zip goes out of scope.
 *
 * possibly risky behavior on clang, gcc for fun(const zip& z) when called as fun(zip(a,b))
 *
 *
 * Depends on the following behavior for for loops:
 *
 *   // in for(x:zip)
 *   // equiv:
 *  { // c++ 11+
 *      auto && __range = range_expression ;
 *          for (auto __begin = begin_expr, __end = end_expr; __begin != __end; ++__begin) {
 *          range_declaration = *__begin;
 *          loop_statement
 *      }
 *  }
 *
 *   { // in c++ 17
 *      auto && __range = range_expression ;
 *      auto __begin = begin_expr ;
 *      auto __end = end_expr ;
 *      for ( ; __begin != __end; ++__begin) {
 *          range_declaration = *__begin;
 *          loop_statement
 *      }
 *  }
 *
 *
 * \author   Mikael Persson
 * \date     2019-09-01
 ******************************************************************************/

static_assert(__cplusplus >= 201703L,
              " must be c++17 or greater"); // could be rewritten in c++11, but the features you must use will be buggy
                                            // in an older compiler anyways.
#include "barretenberg/common/assert.hpp"
#include <cassert>
#include <functional>
#include <iostream>
#include <sstream>
#include <tuple>
#include <type_traits>
#include <vector>

template <class T>
/**
 * @brief The zip_iterator class
 *
 * Provides a zip iterator which is at end when any is at end
 */
class zip_iterator {
  public:
    // speeds up compilation a little bit...
    using tuple_indexes = std::make_index_sequence<std::tuple_size_v<std::remove_reference_t<T>>>;

    zip_iterator(T iter, T iter_end)
        : iter(iter)
        , iter_end(iter_end)
    {}
    // prefix, inc first, then return
    zip_iterator& operator++()
    {
        for_each_in_tuple([](auto&& x) { return x++; }, iter);
        // then if any hit end, update all to point to end.
        auto end = apply2([](auto x, auto y) { return x == y; }, iter, iter_end);
        if (if_any_in(end)) {
            apply2([](auto& x, auto y) { return x = y; }, iter, iter_end);
        }
        index++;
        return *this;
    }
    // sufficient because ++ keeps track and sets all to end when any is
    bool operator!=(const zip_iterator& other) const { return other.iter != iter; }
    auto operator*() const
    {
        return std::forward<decltype(get_refs(iter, tuple_indexes{}))>(get_refs(iter, tuple_indexes{}));
    }

  private:
    T iter, iter_end;
    std::size_t index = 0;

    template <std::size_t... I> auto get_refs(T t, std::index_sequence<I...>) const
    {
        return std::make_tuple(std::ref(*std::get<I>(t))...);
    }

    template <class F, class A, std::size_t... I> auto apply2_impl(F&& f, A&& a, A&& b, std::index_sequence<I...>)
    {
        return std::make_tuple(f(std::get<I>(a), std::get<I>(b))...);
    }
    template <class F, class A> auto apply2(F&& f, A&& a, A&& b)
    {
        return apply2_impl(std::forward<F>(f), std::forward<A>(a), std::forward<A>(b), tuple_indexes{});
    }
    template <class A, std::size_t... I> bool if_any_impl(const A& t, std::index_sequence<I...>) const
    {
        return (... || std::get<I>(t)); // c++17
    }

    // in general context we must enforce that these are tuples
    template <class A> bool if_any_in(A&& t) const { return if_any_impl(std::forward<A>(t), tuple_indexes{}); }

    template <class F, class Tuple, std::size_t... I>
    auto for_each_in_impl(F&& f, Tuple&& t, std::index_sequence<I...>) const
    {
        return std::make_tuple(f(std::get<I>(t))...);
    }

    template <class F, class A> void for_each_in_tuple(F&& f, A&& t) const
    {
        for_each_in_impl(std::forward<F>(f), std::forward<A>(t), tuple_indexes{});
    }
};

enum class ZipAllowDifferentSizes { FLAG };
template <class... S> class zip_view {
    using arg_indexes = std::make_index_sequence<sizeof...(S)>;

  public:
    zip_view(S... args)
        : args(std::forward<S>(args)...)
    {
        // min size matches max size
        ASSERT(size() == max_size_impl(arg_indexes{}));
    }
    zip_view(ZipAllowDifferentSizes /*unused*/, S... args)
        : args(std::forward<S>(args)...)
    {
        // Same in a release build, in a debug build doesn't error with different container sizes
    }
    auto begin() const { return get_begins(arg_indexes{}); }
    auto end() const { return get_ends(arg_indexes{}); }
    [[nodiscard]] std::size_t size() const { return size_impl(arg_indexes{}); }

  private:
    std::tuple<S...> args;
    template <std::size_t... I> auto get_begins(std::index_sequence<I...>) const
    {
        return zip_iterator(std::make_tuple(std::get<I>(args).begin()...), std::make_tuple(std::get<I>(args).end()...));
    }
    template <std::size_t... I> auto get_ends(std::index_sequence<I...>) const
    {
        return zip_iterator(std::make_tuple(std::get<I>(args).end()...), std::make_tuple(std::get<I>(args).end()...));
    }
    template <std::size_t... I> auto size_impl(std::index_sequence<I...>) const
    {
        return std::min({ std::size_t(std::get<I>(args).size())... });
    }
    template <std::size_t... I> auto max_size_impl(std::index_sequence<I...>) const
    {
        return std::max({ std::size_t(std::get<I>(args).size())... });
    }

    template <class A, std::size_t... I> bool if_any_impl(const A& t, std::index_sequence<I...>) const
    {
        return (... || std::get<I>(t)); // c++17
    }
};

// deduction guide,
template <class... S> zip_view(S&&...) -> zip_view<S...>;

// deduction guide,
template <class... S> zip_view(ZipAllowDifferentSizes, S&&...) -> zip_view<S...>;

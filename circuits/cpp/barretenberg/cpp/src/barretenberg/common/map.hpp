#pragma once
#include <algorithm>
#include <array>
#include <type_traits>

/*
 * Generic map function for mapping a containers element to another type.
 */
template <template <typename, typename...> typename Cont,
          typename InElem,
          typename... Args,
          typename F,
          typename OutElem = typename std::invoke_result<F, InElem const&>::type>
Cont<OutElem> map(Cont<InElem, Args...> const& in, F&& op)
{
    Cont<OutElem> result;
    std::transform(in.begin(), in.end(), std::back_inserter(result), op);
    return result;
}

/*
 * Generic map function for mapping a std::array's elements to another type.
 * TODO: this has only been added because I (Mike) couldn't get the above to work
 * with an array.
 */
template <std::size_t SIZE,
          typename InElem,
          typename F,
          typename OutElem = typename std::invoke_result<F, InElem const&>::type>
std::array<OutElem, SIZE> map(std::array<InElem, SIZE> const& in, F&& op)
{
    std::array<OutElem, SIZE> result;
    std::transform(in.begin(), in.end(), result.begin(), op);
    return result;
}

/*
 * Generic map function for mapping a containers element to another type.
 * This version passes the element index as a second argument to the operator function.
 */
template <template <typename, typename...> typename Cont,
          typename InElem,
          typename... Args,
          typename F,
          typename OutElem = typename std::invoke_result<F, InElem const&, size_t>::type>
Cont<OutElem> mapi(Cont<InElem, Args...> const& in, F op)
{
    Cont<OutElem> result;
    for (size_t i = 0; i < in.size(); ++i) {
        result.push_back(op(in[i], i));
    }
    return result;
}

/*
 * Generic filter function for containers.
 */
template <typename Cont, typename F> Cont filter(Cont const& in, F op)
{
    Cont copy(in);
    std::remove_if(copy.begin(), copy.end(), op);
    return copy;
}

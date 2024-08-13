#pragma once

#include <cstddef>
#include <tuple>
#include <utility>

namespace bb {

/**
 * @brief Implements a loop using a compile-time iterator. Requires c++20.
 * Implementation (and description) from https://artificial-mind.net/blog/2020/10/31/constexpr-for
 *
 * @tparam Start the loop start value
 * @tparam End the loop end value
 * @tparam Inc how much the iterator increases by per iteration
 * @tparam F a Lambda function that is executed once per loop
 *
 * @param f An rvalue reference to the lambda
 * @details Implements a `for` loop where the iterator is a constexpr variable.
 * Use this when you need to evaluate `if constexpr` statements on the iterator (or apply other constexpr expressions)
 * Outside of this use-case avoid using this fn as it gives negligible performance increases vs regular loops.
 *
 * N.B. A side-effect of this method is that all loops will be unrolled
 * (each loop iteration uses different iterator template parameters => unique constexpr_for implementation per
 * iteration)
 * Do not use this for large (~100+) loops!
 *
 * ##############################
 * EXAMPLE USE OF `constexpr_for`
 * ##############################
 *
 * constexpr_for<0, 10, 1>([&]<size_t i>(){
 *  if constexpr (i & 1 == 0)
 *  {
 *      foo[i] = even_container[i >> 1];
 *  }
 *  else
 *  {
 *      foo[i] = odd_container[i >> 1];
 *  }
 * });
 *
 * In the above example we are iterating from i = 0 to i < 10.
 * The provided lambda function has captured everything in its surrounding scope (via `[&]`),
 * which is where `foo`, `even_container` and `odd_container` have come from.
 *
 * We do not need to explicitly define the `class F` parameter as the compiler derives it from our provided input
 * argument `F&& f` (i.e. the lambda function)
 *
 * In the loop itself we're evaluating a constexpr if statement that defines which code path is taken.
 *
 * The above example benefits from `constexpr_for` because a run-time `if` statement has been reduced to a compile-time
 * `if` statement. N.B. this would only give measurable improvements if the `constexpr_for` statement is itself in a hot
 * loop that's iterated over many (>thousands) times
 */
template <size_t Start, size_t End, size_t Inc, class F> constexpr void constexpr_for(F&& f)
{
    // Call function `f<Start>()` iff Start < End
    if constexpr (Start < End) {
        // F must be a template lambda with a single **typed** template parameter that represents the iterator
        // (e.g. [&]<size_t i>(){ ... } is good)
        // (and [&]<typename i>(){ ... } won't compile!)

        /**
         * Explaining f.template operator()<Start>()
         *
         * The following line must explicitly tell the compiler that <Start> is a template parameter by using the
         * `template` keyword.
         * (if we wrote f<Start>(), the compiler could legitimately interpret `<` as a less than symbol)
         *
         * The fragment `f.template` tells the compiler that we're calling a *templated* member of `f`.
         * The "member" being called is the function operator, `operator()`, which must be explicitly provided
         * (for any function X, `X(args)` is an alias for `X.operator()(args)`)
         * The compiler has no alias `X.template <tparam>(args)` for `X.template operator()<tparam>(args)` so we must
         * write it explicitly here
         *
         * To summarize what the next line tells the compiler...
         * 1. I want to call a member of `f` that expects one or more template parameters
         * 2. The member of `f` that I want to call is the function operator
         * 3. The template parameter is `Start`
         * 4. The function operator itself contains no arguments
         */
        f.template operator()<Start>();

        // Once we have executed `f`, we recursively call the `constexpr_for` function, increasing the value of `Start`
        // by `Inc`
        constexpr_for<Start + Inc, End, Inc>(f);
    }
}

}; // namespace bb

#pragma once
#include "../relation_parameters.hpp"
#include "../relation_types.hpp"

namespace proof_system::Fib_vm {

template <typename FF> struct Row {
    FF Fibonacci_LAST{};
    FF Fibonacci_FIRST{};
    FF Fibonacci_x{};
    FF Fibonacci_y{};
    FF Fibonacci_x_shift{};
    FF Fibonacci_y_shift{};
};

#define DECLARE_VIEWS(index)                                                                                           \
    using View = typename std::tuple_element<index, ContainerOverSubrelations>::type;                                  \
    [[maybe_unused]] auto Fibonacci_LAST = View(new_term.Fibonacci_LAST);                                              \
    [[maybe_unused]] auto Fibonacci_FIRST = View(new_term.Fibonacci_FIRST);                                            \
    [[maybe_unused]] auto Fibonacci_x = View(new_term.Fibonacci_x);                                                    \
    [[maybe_unused]] auto Fibonacci_y = View(new_term.Fibonacci_y);                                                    \
    [[maybe_unused]] auto Fibonacci_x_shift = View(new_term.Fibonacci_x_shift);                                        \
    [[maybe_unused]] auto Fibonacci_y_shift = View(new_term.Fibonacci_y_shift);

template <typename FF_> class FibImpl {
  public:
    using FF = FF_;

    static constexpr std::array<size_t, 2> SUBRELATION_PARTIAL_LENGTHS{
        4,
        4,
    };

    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(ContainerOverSubrelations& evals,
                           const AllEntities& new_term,
                           [[maybe_unused]] const RelationParameters<FF>&,
                           [[maybe_unused]] const FF& scaling_factor)
    {

        // Contribution 0
        {
            DECLARE_VIEWS(0);

            auto tmp = (((-Fibonacci_FIRST + FF(1)) * (-Fibonacci_LAST + FF(1))) * (Fibonacci_x_shift - Fibonacci_y));
            tmp *= scaling_factor;
            std::get<0>(evals) += tmp;
        }
        // Contribution 1
        {
            DECLARE_VIEWS(1);

            auto tmp = (((-Fibonacci_FIRST + FF(1)) * (-Fibonacci_LAST + FF(1))) *
                        (Fibonacci_y_shift - (Fibonacci_x + Fibonacci_y)));
            tmp *= scaling_factor;
            std::get<1>(evals) += tmp;
        }
    }
};

template <typename FF> using Fib = Relation<FibImpl<FF>>;

} // namespace proof_system::Fib_vm
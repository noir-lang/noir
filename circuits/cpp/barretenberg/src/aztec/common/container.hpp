#pragma once
#include <stdint.h>

template <typename C> C slice(C const& container, size_t start, size_t end)
{
    auto b = container.begin();
    auto e = container.begin();
    std::advance(b, start);
    std::advance(e, end);
    return C(b, e);
}

template <typename C> C join(std::initializer_list<C> to_join)
{
    C result;
    for (auto& e : to_join) {
        result.insert(result.end(), e.begin(), e.end());
    }
    return result;
}

template <template <typename, typename...> typename Cont, typename InnerCont, typename... Args>
InnerCont flatten(Cont<InnerCont, Args...> const& in)
{
    InnerCont result;
    for (auto& e : in) {
        result.insert(result.end(), e.begin(), e.end());
    }
    return result;
}
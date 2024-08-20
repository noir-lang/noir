#pragma once
#include <algorithm>
#include <cstddef>
#include <string>
#include <vector>

namespace bb {

template <typename C> C slice(C const& container, size_t start)
{
    auto b = container.begin();
    auto e = container.end();
    std::advance(b, start);
    return C(b, e);
}

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

inline std::string join(std::vector<std::string> const& to_join, std::string const& with = ",")
{
    auto it = to_join.begin();
    std::string result(*it++);
    for (; it != to_join.end(); ++it) {
        result += with;
        result += *it;
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

// Return the first index at which a given item can be found in the vector.
// Only safe for vectors with length less than the size_t overflow size.
template <typename T> int64_t index_of(std::vector<T> const& vec, T const& item)
{
    auto const& begin = vec.begin();
    auto const& end = vec.end();

    auto const& itr = std::find(begin, end, item);

    return itr == end ? -1 : std::distance(begin, itr);
}

// A simple sum meant for small containers (i.e. doesn't use threading)
template <template <typename, typename...> typename Cont, typename Inner, typename... Args>
Inner sum(Cont<Inner, Args...> const& in)
{
    Inner result{};
    for (auto& e : in) {
        result += e;
    }
    return result;
}

// A simple sum meant for small containers (i.e. doesn't use threading)
template <template <typename, typename...> typename Cont, typename Left, typename Right, typename... Args>
std::pair<Left, Right> sum_pairs(Cont<std::pair<Left, Right>, Args...> const& in)
{
    std::pair<Left, Right> result{ {}, {} };
    for (auto& e : in) {
        result.first += e.first;
        result.second += e.second;
    }
    return result;
}

} // namespace bb
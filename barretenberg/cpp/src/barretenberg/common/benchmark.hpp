#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/env/hardware_concurrency.hpp"
#include <cstdlib>
#include <ctime>
#include <fcntl.h>
#include <iostream>
#include <sstream>
#include <string>
#include <unistd.h>

#pragma GCC diagnostic ignored "-Wunused-result" // GCC13 hits this

namespace {
/**
 * If user provides the env var BENCHMARK_FD write benchmarks to this fd, otherwise default to -1 (disable).
 * e.g:
 *   BENCHMARK_FD=3 bb 3> benchmarks.jsonl
 */
auto bfd = []() {
    try {
        static auto bfd_str = std::getenv("BENCHMARK_FD");
        int bfd = bfd_str ? (int)std::stoul(bfd_str) : -1;
        if (bfd >= 0 && (fcntl(bfd, F_GETFD) == -1 || errno == EBADF)) {
            throw_or_abort("fd is not open. Did you redirect in your shell?");
        }
        return bfd;
    } catch (std::exception const& e) {
        std::string inner_msg = e.what();
        throw_or_abort("Invalid BENCHMARK_FD: " + inner_msg);
    }
}();
} // namespace

template <typename T, typename Enable = void> struct TypeTraits;

template <typename T> struct TypeTraits<T, typename std::enable_if<std::is_arithmetic<T>::value>::type> {
    static const char* type;
};

template <typename T>
const char* TypeTraits<T, typename std::enable_if<std::is_arithmetic<T>::value>::type>::type = "number";

template <> struct TypeTraits<std::string> {
    static const char* type;
};

const char* TypeTraits<std::string>::type = "string";

template <> struct TypeTraits<double> {
    static const char* type;
};

const char* TypeTraits<double>::type = "number";

template <> struct TypeTraits<bool> {
    static const char* type;
};

const char* TypeTraits<bool>::type = "bool";

// Helper function to get the current timestamp in the desired format
std::string getCurrentTimestamp()
{
    std::time_t now = std::time(nullptr);
    std::tm* now_tm = std::gmtime(&now);
    char buf[21] = { 0 };
    strftime(buf, sizeof(buf), "%Y-%m-%dT%H:%M:%SZ", now_tm);
    return std::string(buf);
}

template <typename T> std::string toString(const T& value)
{
    std::ostringstream oss;
    oss << value;
    return oss.str();
}

void appendToStream(std::ostringstream&)
{
    // base case: do nothing
}

template <typename K, typename V, typename... Args>
void appendToStream(std::ostringstream& oss, const K& key, const V& value, Args... args)
{
    oss << ", \"" << key << "\": \"" << toString(value) << "\"";
    appendToStream(oss, args...); // recursively process the remaining arguments
}

template <typename T, typename... Args> void write_benchmark(const std::string& name, const T& value, Args... args)
{
    if (bfd == -1) {
        return;
    }
    std::ostringstream oss;
    oss << "{\"timestamp\": \"" << getCurrentTimestamp() << "\", "
        << "\"name\": \"" << name << "\", "
        << "\"type\": \"" << TypeTraits<T>::type << "\", "
        << "\"value\": " << value << ", "
        << "\"threads\": " << env_hardware_concurrency();

    appendToStream(oss, args...); // unpack and append the key-value pairs

    oss << "}" << std::endl;
    const std::string& tmp = oss.str();
    write((int)bfd, tmp.c_str(), tmp.size());
}
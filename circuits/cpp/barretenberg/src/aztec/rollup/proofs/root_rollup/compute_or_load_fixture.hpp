#pragma once
#include <fstream>
#include <functional>
#include <sys/stat.h>
#include <common/serialize.hpp>
#include <common/log.hpp>

namespace rollup {
namespace proofs {
namespace root_rollup {

inline bool exists(std::string const& path)
{
    struct stat st;
    return (stat(path.c_str(), &st) != -1);
}

inline std::vector<uint8_t> compute_or_load_fixture(std::string const& path,
                                                    std::string const& name,
                                                    std::function<std::vector<uint8_t>()> const& f)
{
    // Tests are being run from build directory.
    auto filename = path + "/" + name;
    if (exists(filename)) {
        auto stream = std::ifstream(filename);
        std::vector<uint8_t> data;
        read(stream, data);
        error("Loaded fixture: ", filename);
        return data;
    } else {
        error("Computing fixture: ", name, "...");
        auto data = f();
        mkdir(path.c_str(), 0700);
        auto stream = std::ofstream(filename);
        write(stream, data);
        return data;
    }
}

} // namespace root_rollup
} // namespace proofs
} // namespace rollup

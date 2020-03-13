#pragma once
#include <map>
#include <string>
#include <polynomials/polynomial.hpp>

namespace waffle {

struct program_witness {
    std::map<std::string, barretenberg::polynomial> wires;
};

} // namespace waffle
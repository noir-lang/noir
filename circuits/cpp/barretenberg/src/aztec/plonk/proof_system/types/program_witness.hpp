#pragma once
#include <map>
#include <polynomials/polynomial.hpp>
#include <string>

namespace waffle {

struct program_witness {
    std::map<std::string, barretenberg::polynomial> wires;
};

} // namespace waffle
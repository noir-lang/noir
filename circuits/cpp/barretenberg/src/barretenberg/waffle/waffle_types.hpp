#pragma once

#include <map>
#include <string>

namespace barretenberg
{
    class polynomial;
}

namespace waffle
{

struct program_witness
{
    std::map<std::string, barretenberg::polynomial> wires;
};


struct plonk_proof
{
    std::vector<uint8_t> proof_data;
};

}
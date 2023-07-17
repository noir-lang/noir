#pragma once
#include "exec_pipe.hpp"

/**
 * We can assume for now we're running on a unix like system and use the following to extract the bytecode.
 * Maybe we should consider bytecode being output into its own independent file alongside the JSON?
 */
inline std::vector<uint8_t> get_witness_data(const std::string& path)
{
    std::string command = "cat " + path + " | gunzip";
    return exec_pipe(command);
}
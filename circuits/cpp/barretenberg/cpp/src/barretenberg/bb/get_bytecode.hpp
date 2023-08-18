#pragma once
#include "exec_pipe.hpp"

/**
 * We can assume for now we're running on a unix like system and use the following to extract the bytecode.
 */
inline std::vector<uint8_t> get_bytecode(const std::string& bytecodePath)
{
    std::string command = "base64 -d " + bytecodePath + " | gunzip";
    return exec_pipe(command);
}
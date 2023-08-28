#pragma once
#include "exec_pipe.hpp"

/**
 * We can assume for now we're running on a unix like system and use the following to extract the bytecode.
 */
inline std::vector<uint8_t> get_bytecode(const std::string& bytecodePath)
{
// base64 on mac is different from linux
#ifdef __APPLE__
    std::string command = "base64 -D -i " + bytecodePath + " | gunzip";
#else
    std::string command = "base64 -d " + bytecodePath + " | gunzip";
#endif

    return exec_pipe(command);
}
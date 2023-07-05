#pragma once
#include "exec_pipe.hpp"

/**
 * We can assume for now we're running on a unix like system and use the following to extract the bytecode.
 * Maybe we should consider bytecode being output into its own independent file alongside the JSON?
 */
inline std::vector<uint8_t> get_bytecode(const std::string& jsonPath)
{
    std::string command =
        "awk -F'\"bytecode\":' '{print $2}' " + jsonPath + " | awk -F'\"' '{print $2}' | base64 -d | gunzip";
    return exec_pipe(command);
}
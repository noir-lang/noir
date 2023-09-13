#pragma once
#include <cstdio>
#include <iostream>
#include <vector>

inline std::vector<uint8_t> exec_pipe(std::string const& command)
{
    FILE* pipe = popen(command.c_str(), "r");
    if (!pipe) {
        throw std::runtime_error("popen() failed!");
    }

    std::vector<uint8_t> result;
    while (!feof(pipe)) {
        uint8_t buffer[128];
        size_t count = fread(buffer, 1, sizeof(buffer), pipe);
        result.insert(result.end(), buffer, buffer + count);
    }

    pclose(pipe);
    return result;
}
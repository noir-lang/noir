#include "data_store.hpp"
#include <cstring>
#include <fstream>
#include <ios>
#include <map>
#include <string>
#include <vector>
// #include <barretenberg/common/log.hpp>

namespace {
std::map<std::string, std::vector<uint8_t>> store;
}

extern "C" {

void set_data(char const* key, uint8_t const* addr, size_t length)
{
    std::string k = key;
    store[k] = std::vector<uint8_t>(addr, addr + length);
    // info("set data: ", key, " length: ", length, " hash: ", crypto::sha256(store[k]));
    // std::ofstream file("/mnt/user-data/charlie/debugging/x86_" + k, std::ios::binary);
    // file.write(reinterpret_cast<const char*>(addr), (std::streamsize)length);
}

void get_data(char const* key, uint8_t* out_buf)
{
    std::string k = key;
    if (store.contains(key)) {
        // info("get data hit: ", key, " length: ", *length_out, " ptr ", (void*)ptr);
        std::memcpy(out_buf, store[k].data(), store[k].size());
    }
    // info("get data miss: ", key);
    // return nullptr;
}
}

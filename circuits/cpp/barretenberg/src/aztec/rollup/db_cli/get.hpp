#pragma once
#include <common/serialize.hpp>

struct GetRequest {
    uint128_t index;
};

struct GetResponse {
    std::array<uint8_t, 64> value;
};

void read(std::istream& s, GetRequest& r) {
    ::read(s, r.index);
}

void write(std::ostream& s, GetResponse const& r)
{
    write(s, r.value);
}


#pragma once
#include <common/serialize.hpp>
#include <ecc/curves/bn254/fr.hpp>

struct PutRequest {
    uint128_t index;
    std::array<uint8_t, 64> value;
};

struct PutResponse {
    barretenberg::fr root;
};

void read(std::istream& s, PutRequest& r) {
    ::read(s, r.index);
    read(s, r.value);
}

void write(std::ostream& s, PutResponse const& r)
{
    write(s, r.root);
}


#pragma once
#include <common/serialize.hpp>
#include <ecc/curves/bn254/fr.hpp>

struct GetRequest {
    uint8_t tree_id;
    uint256_t index;
};

struct GetResponse {
    barretenberg::fr value;
};

void read(std::istream& s, GetRequest& r)
{
    read(s, r.tree_id);
    read(s, r.index);
}

void write(std::ostream& s, GetResponse const& r)
{
    write(s, r.value);
}

std::ostream& operator<<(std::ostream& os, GetRequest const& get_request)
{
    return os << "GET (tree:" << (int)get_request.tree_id << " index:" << get_request.index << ")";
}

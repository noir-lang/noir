#pragma once
#include <common/serialize.hpp>
#include <common/streams.hpp>
#include <ecc/curves/bn254/fr.hpp>

struct PutRequest {
    uint8_t tree_id;
    uint256_t index;
    std::vector<uint8_t> value;
};

struct PutResponse {
    barretenberg::fr root;
};

void read(std::istream& s, PutRequest& r)
{
    read(s, r.tree_id);
    read(s, r.index);
    read(s, r.value);
}

void write(std::ostream& s, PutResponse const& r)
{
    write(s, r.root);
}

std::ostream& operator<<(std::ostream& os, PutRequest const& put_request)
{
    return os << "PUT (tree:" << (int)put_request.tree_id << " index:" << put_request.index
              << " value:" << put_request.value << ")";
}

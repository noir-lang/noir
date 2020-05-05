#pragma once
#include <algorithm>
#include <arpa/inet.h>
#include <sstream>
#include <common/serialize.hpp>

namespace rollup {
namespace tx {

struct rollup_tx {
    uint32_t rollup_id;
    uint32_t num_txs;
    uint32_t proof_lengths;
    std::vector<std::vector<uint8_t>> txs;
};

void read(uint8_t const*& buf, rollup_tx& tx)
{
    ::read(buf, tx.rollup_id);
    ::read(buf, tx.num_txs);
    ::read(buf, tx.proof_lengths);
    tx.txs.resize(tx.num_txs, std::vector<uint8_t>(tx.proof_lengths));
    read(buf, tx.txs);
}

template <typename B> void write(B& buf, rollup_tx const& tx)
{
    ::write(buf, tx.rollup_id);
    ::write(buf, tx.num_txs);
    ::write(buf, tx.proof_lengths);
    ::write(buf, tx.txs);
}

} // namespace tx
} // namespace rollup
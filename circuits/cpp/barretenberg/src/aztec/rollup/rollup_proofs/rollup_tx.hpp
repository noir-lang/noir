#pragma once
#include <algorithm>
#include <arpa/inet.h>
#include <sstream>
#include <common/serialize.hpp>
#include <common/streams.hpp>

namespace rollup {
namespace rollup_proofs {

struct rollup_tx {
    uint32_t rollup_id;
    uint32_t num_txs;
    uint32_t proof_lengths;
    std::vector<std::vector<uint8_t>> txs;
};

template <typename B> void read(B& buf, rollup_tx& tx)
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
    write(buf, tx.txs);
}

bool operator==(rollup_tx const& lhs, rollup_tx const& rhs)
{
    return lhs.rollup_id == rhs.rollup_id && lhs.num_txs == rhs.num_txs && lhs.proof_lengths == rhs.proof_lengths &&
           lhs.txs == rhs.txs;
}

std::ostream& operator<<(std::ostream& os, rollup_tx const& tx)
{
    os << "rollup_id: " << tx.rollup_id << "\nnum_txs: " << tx.num_txs << "\nproof_lengths: " << tx.proof_lengths
       << "\nproof_data:\n";
    for (auto p : tx.txs) {
        os << p << "\n";
    }
    return os;
}

} // namespace rollup_proofs
} // namespace rollup
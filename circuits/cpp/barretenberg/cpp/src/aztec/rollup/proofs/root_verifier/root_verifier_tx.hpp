#pragma once
#include <common/serialize.hpp>

namespace rollup {
namespace proofs {
namespace root_verifier {

struct root_verifier_tx {
    std::vector<uint8_t> broadcast_data;
    std::vector<uint8_t> proof_data;

    bool operator==(root_verifier_tx const&) const = default;
};

template <typename B> inline void read(B& buf, root_verifier_tx& tx)
{
    using serialize::read;
    read(buf, tx.broadcast_data);
    read(buf, tx.proof_data);
}

template <typename B> inline void write(B& buf, root_verifier_tx const& tx)
{
    using serialize::write;
    write(buf, tx.broadcast_data);
    write(buf, tx.proof_data);
}

inline std::ostream& operator<<(std::ostream& os, root_verifier_tx const& tx)
{
    os << "broadcast_data:\n";
    for (auto p : tx.broadcast_data) {
        os << p << "\n";
    }
    os << "proof_data:\n";
    for (auto p : tx.proof_data) {
        os << p << "\n";
    }
    return os;
}

} // namespace root_verifier
} // namespace proofs
} // namespace rollup

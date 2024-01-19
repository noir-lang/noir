#pragma once
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>
#include <iomanip>
#include <ostream>
#include <vector>

namespace bb::plonk {

struct proof {
    std::vector<uint8_t> proof_data;
    // For serialization, serialize as a buffer alias
    void msgpack_pack(auto& packer) const { packer.pack(proof_data); }
    void msgpack_unpack(auto object) { proof_data = (std::vector<uint8_t>)object; }
    void msgpack_schema(auto& packer) const { packer.pack_alias("Proof", "bin32"); }
    bool operator==(proof const& other) const = default;
};

inline void read(uint8_t const*& it, proof& data)
{
    using serialize::read;
    read(it, data.proof_data);
};

template <typename B> inline void write(B& buf, proof const& data)
{
    using serialize::write;
    write(buf, data.proof_data);
}

inline std::ostream& operator<<(std::ostream& os, proof const& data)
{
    // REFACTOR: This is copied from barretenberg/common/streams.hpp,
    // which means we could just cout proof_data directly, but that breaks the build in the CI with
    // a redefined operator<< error in barretenberg/stdlib/hash/keccak/keccak.test.cpp,
    // which is something we really don't want to deal with right now.
    std::ios_base::fmtflags f(os.flags());
    os << "[" << std::hex << std::setfill('0');
    for (auto byte : data.proof_data) {
        os << ' ' << std::setw(2) << +(unsigned char)byte;
    }
    os << " ]";
    os.flags(f);
    return os;
}

} // namespace bb::plonk

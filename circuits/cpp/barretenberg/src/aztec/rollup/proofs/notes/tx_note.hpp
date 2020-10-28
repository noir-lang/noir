#pragma once
#include <common/serialize.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include "./note_types.hpp"
#include "./note_generator_indices.hpp"

namespace rollup {
namespace proofs {
namespace notes {

using namespace barretenberg;

struct tx_note {
    grumpkin::g1::affine_element owner;
    uint256_t value;
    barretenberg::fr secret;
    uint32_t asset_id;
    grumpkin::g1::affine_element encrypt_note() const;
    uint128_t compute_nullifier(const uint32_t tree_index, const bool is_real_note) const;
};

struct tx_account_note {
    grumpkin::g1::affine_element owner_key;
    grumpkin::g1::affine_element signing_key;

    uint128_t compute_nullifier() {
        std::vector<barretenberg::fr> hash_elements {
            owner_key.x,
            signing_key.x,
        };
        const auto result = crypto::pedersen::compress_native(hash_elements, rollup::proofs::notes::ACCOUNT_NULLIFIER_INDEX);
        return uint128_t(uint256_t(result));
    }
};

bool decrypt_note(grumpkin::g1::affine_element const& encrypted_note,
                  grumpkin::fr const& private_key,
                  fr const& viewing_key,
                  uint32_t const asset_id,
                  uint256_t& r);

inline bool operator==(tx_note const& lhs, tx_note const& rhs)
{
    return lhs.owner == rhs.owner && lhs.value == rhs.value && lhs.secret == rhs.secret;
}

inline std::ostream& operator<<(std::ostream& os, tx_note const& note)
{
    os << "{ owner_x: " << note.owner.x << ", owner_y: " << note.owner.y << ", view_key: " << note.secret
       << ", value: " << note.value << ", asset_id: " << note.asset_id << " }";
    return os;
}

inline void read(uint8_t const*& it, tx_note& note)
{
    using serialize::read;
    read(it, note.owner);
    read(it, note.value);
    read(it, note.secret);
    read(it, note.asset_id);
}

inline void write(std::vector<uint8_t>& buf, tx_note const& note)
{
    using serialize::write;
    write(buf, note.owner);
    write(buf, note.value);
    write(buf, note.secret);
    write(buf, note.asset_id);
}

} // namespace notes
} // namespace proofs
} // namespace rollup
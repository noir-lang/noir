#pragma once
#include "tx_note.hpp"
#include "user_context.hpp"
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace tx {

struct join_split_tx {
    grumpkin::g1::affine_element owner_pub_key;
    uint32_t public_input;
    uint32_t public_output;
    uint32_t num_input_notes;
    uint32_t input_note_index[2];
    tx_note input_note[2];
    tx_note output_note[2];
    crypto::schnorr::signature signature;
};

inline std::ostream& operator<<(std::ostream& os, join_split_tx const& tx)
{
    return os << "public_input: " << tx.public_input << "\n"
              << "public_output: " << tx.public_output << "\n"
              << "in_value1: " << tx.input_note[0].value << "\n"
              << "in_value2: " << tx.input_note[1].value << "\n"
              << "out_value1: " << tx.output_note[0].value << "\n"
              << "out_value2: " << tx.output_note[1].value << "\n"
              << "num_input_notes: " << tx.num_input_notes << "\n"
              << "owner: " << tx.owner_pub_key.x << " " << tx.owner_pub_key.y << "\n";
}

join_split_tx create_join_split_tx(std::vector<std::string> const& args, user_context const& user);

join_split_tx hton(join_split_tx const& tx);
join_split_tx ntoh(join_split_tx const& tx);
std::ostream& write(std::ostream& os, join_split_tx const& tx);
std::istream& read(std::istream& is, join_split_tx& tx);

} // namespace tx
} // namespace rollup
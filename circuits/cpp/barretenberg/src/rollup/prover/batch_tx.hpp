#pragma once
#include "join_split_tx.hpp"
#include <sstream>

namespace rollup {

struct batch_tx {
  uint32_t batch_num;
  std::vector<join_split_tx> txs;
};

batch_tx hton(batch_tx const& txs) {
    batch_tx be_txs;
    be_txs.batch_num = htonl(txs.batch_num);
    be_txs.txs.resize(txs.txs.size());
    std::transform(txs.txs.begin(), txs.txs.end(), be_txs.txs.begin(), [](auto& tx){ return hton(tx); });
    return be_txs;
}

batch_tx ntoh(batch_tx const& be_txs) {
    batch_tx txs;
    txs.batch_num = ntohl(be_txs.batch_num);
    txs.txs.resize(be_txs.txs.size());
    std::transform(be_txs.txs.begin(), be_txs.txs.end(), txs.txs.begin(), [](auto& tx){ return ntoh(tx); });
    return txs;
}

std::ostream& write(std::ostream& os, batch_tx const& be_txs) {
    uint32_t size = static_cast<uint32_t>(be_txs.txs.size());
    uint32_t nsize = htonl(size);
    os.write(reinterpret_cast<char const*>(&be_txs.batch_num), sizeof(be_txs.batch_num));
    os.write(reinterpret_cast<char*>(&nsize), sizeof(nsize));
    for (auto tx : be_txs.txs) {
        write(os, tx);
    }
    return os;
}

std::istream& read(std::istream& is, batch_tx& txs) {
    batch_tx be_txs;
    uint32_t size;
    is.read(reinterpret_cast<char*>(&be_txs.batch_num), sizeof(be_txs.batch_num));
    is.read(reinterpret_cast<char*>(&size), sizeof(size));
    size = ntohl(size);
    be_txs.txs.resize(size);
    for (size_t i=0; i<size; ++i) {
        read(is, be_txs.txs[i]);
    }
    txs = ntoh(be_txs);
    return is;
}

std::ostream& write_json(std::ostream& os, crypto::pedersen_note::private_note const& tx, size_t indent=0) {
    std::string i(indent, ' ');
    os << i << "{\n"
       << i << "  \"owner\": {\n"
       << i << "    \"x\": \"" << tx.owner.x << "\",\n"
       << i << "    \"y\": \"" << tx.owner.y << "\"\n"
       << i << "  },\n"
       << i << "  \"value\": " << tx.value << ",\n"
       << i << "  \"viewing_key\": \"" << tx.secret << "\"\n"
       << i << "}";
    return os;
}

std::string arr_to_hex_string(std::array<uint8_t, 32> const& arr)
{
    std::ostringstream os;
    os << "0x" << std::hex << std::setfill('0');
    for (auto byte : arr) {
        os << std::setw(2) << +(unsigned char)byte;
    }
    return os.str();
}

template<typename T>
void delim_writer(std::ostream& os, T const& v, size_t indent=0) {
    auto actual_delim = ",\n";
    auto delim = "";
    for (auto note : v) {
        os << delim;
        write_json(os, note, indent);
        delim = actual_delim;
    }
}

std::ostream& write_json(std::ostream& os, join_split_tx const& tx, size_t indent=0) {
    std::string i(indent, ' ');
    os << i << "{\n"
       << i << "  \"owner\": {\n"
       << i << "    \"x\": \"" << tx.owner_pub_key.x << "\",\n"
       << i << "    \"y\": \"" << tx.owner_pub_key.y << "\"\n"
       << i << "  },\n"
       << i << "  \"public_input\": " << tx.public_input << ",\n"
       << i << "  \"public_output\": " << tx.public_output << ",\n"
       << i << "  \"num_input_notes\": " << tx.num_input_notes << ",\n"
       << i << "  \"input_note_index\": [" << tx.input_note_index[0] << ", " << tx.input_note_index[1] << "],\n"
       << i << "  \"input_notes\": [\n";
    delim_writer(os, tx.input_note, indent + 4);
    os << "\n"
       << i << "  ],\n"
       << i << "  \"output_notes\": [\n";
    delim_writer(os, tx.output_note, indent + 4);
    os << "\n"
       << i << "  ],\n"
       << i << "  \"signature\": {\n"
       << i << "    \"s\": \"" << arr_to_hex_string(tx.signature.s) << "\",\n"
       << i << "    \"e\": \"" << arr_to_hex_string(tx.signature.e) << "\"\n"
       << i << "  }\n"
       << i << "}";
    return os;
}

std::ostream& write_json(std::ostream& os, batch_tx const& txs) {
    os << "{\n"
       << "  \"batch_num\": " << txs.batch_num << ",\n"
       << "  \"txs\": [\n";
    delim_writer(os, txs.txs, 4);
    os << "\n"
       << "  ]\n"
       << "}\n"
       << std::flush;
    return os;
}

}
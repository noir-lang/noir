#include "join_split_tx.hpp"
#include "io.hpp"

namespace rollup {

using namespace barretenberg;

typedef crypto::pedersen_note::private_note  tx_note;

fr generate_random_secret()
{
    fr secret = fr::random_element().from_montgomery_form();
    secret.data[3] = secret.data[3] & (~0b1111110000000000000000000000000000000000000000000000000000000000ULL);
    return secret.to_montgomery_form();
};

tx_note create_note(user_context const& user, uint32_t value)
{
    return { { user.public_key.x, user.public_key.y }, value, user.note_secret };
}

tx_note create_gibberish_note(user_context const& user, uint32_t value)
{
    return { { user.public_key.x, user.public_key.y }, value, generate_random_secret() };
}

crypto::schnorr::signature sign_notes(std::array<tx_note, 4> const& notes, user_context const& user)
{
    std::array<grumpkin::fq, 8> to_compress;
    for (size_t i = 0; i < 4; ++i) {
        auto encrypted = crypto::pedersen_note::encrypt_note(notes[i]);
        to_compress[i * 2] = encrypted.x;
        to_compress[i * 2 + 1] = encrypted.y;
    }
    fr compressed = crypto::pedersen::compress_eight_native(to_compress);
    std::vector<uint8_t> message(sizeof(fr));
    fr::serialize_to_buffer(compressed, &message[0]);
    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            std::string(message.begin(), message.end()), { user.private_key, user.public_key });
    return signature;
}

join_split_tx create_join_split_tx(std::vector<std::string> const& args, user_context const& user)
{
    uint32_t index1 = (uint32_t)atoi(args[0].c_str());
    uint32_t index2 = (uint32_t)atoi(args[1].c_str());
    uint32_t in_value1 = (uint32_t)atoi(args[2].c_str());
    uint32_t in_value2 = (uint32_t)atoi(args[3].c_str());
    uint32_t out_value1 = (uint32_t)atoi(args[4].c_str());
    uint32_t out_value2 = (uint32_t)atoi(args[5].c_str());
    uint32_t public_input = args.size() > 6 ? (uint32_t)atoi(args[6].c_str()) : 0;
    uint32_t public_output = args.size() > 7 ? (uint32_t)atoi(args[7].c_str()) : 0;
    uint32_t num_input_notes = (uint32_t)(args[2][0] != '-') + (uint32_t)(args[3][0] != '-');

    tx_note in_note1 = num_input_notes < 1 ? create_gibberish_note(user, in_value1) : create_note(user, in_value1);
    tx_note in_note2 = num_input_notes < 2 ? create_gibberish_note(user, in_value2) : create_note(user, in_value2);
    tx_note out_note1 = create_note(user, out_value1);
    tx_note out_note2 = create_note(user, out_value2);

    auto signature = sign_notes({ in_note1, in_note2, out_note1, out_note2 }, user);

    return {
        user.public_key,
        public_input,
        public_output,
        num_input_notes,
        {
            index1,
            index2,
        },
        {
            in_note1,
            in_note2,
        },
        {
            out_note1,
            out_note2,
        },
        signature,
    };
}

join_split_tx hton(join_split_tx const& tx) {
    join_split_tx be_tx;
    be_tx.owner_pub_key = hton(tx.owner_pub_key);
    be_tx.public_input = htonl(tx.public_input);
    be_tx.public_output = htonl(tx.public_output);
    be_tx.num_input_notes = htonl(tx.num_input_notes);
    be_tx.input_note_index[0] = htonl(tx.input_note_index[0]);
    be_tx.input_note_index[1] = htonl(tx.input_note_index[1]);
    be_tx.input_note[0] = hton(tx.input_note[0]);
    be_tx.input_note[1] = hton(tx.input_note[1]);
    be_tx.output_note[0] = hton(tx.output_note[0]);
    be_tx.output_note[1] = hton(tx.output_note[1]);
    be_tx.signature = tx.signature;
    return be_tx;
}

join_split_tx ntoh(join_split_tx const& be_tx) {
    join_split_tx tx;
    tx.owner_pub_key = ntoh(be_tx.owner_pub_key);
    tx.public_input = ntohl(be_tx.public_input);
    tx.public_output = ntohl(be_tx.public_output);
    tx.num_input_notes = ntohl(be_tx.num_input_notes);
    tx.input_note_index[0] = ntohl(be_tx.input_note_index[0]);
    tx.input_note_index[1] = ntohl(be_tx.input_note_index[1]);
    tx.input_note[0] = ntoh(be_tx.input_note[0]);
    tx.input_note[1] = ntoh(be_tx.input_note[1]);
    tx.output_note[0] = ntoh(be_tx.output_note[0]);
    tx.output_note[1] = ntoh(be_tx.output_note[1]);
    tx.signature = be_tx.signature;
    return tx;
}

std::ostream& write(std::ostream& os, join_split_tx const& be_tx) {
    os.write(reinterpret_cast<char const*>(&be_tx.owner_pub_key), sizeof(be_tx.owner_pub_key));
    os.write(reinterpret_cast<char const*>(&be_tx.public_input), sizeof(be_tx.public_input));
    os.write(reinterpret_cast<char const*>(&be_tx.public_output), sizeof(be_tx.public_output));
    os.write(reinterpret_cast<char const*>(&be_tx.num_input_notes), sizeof(be_tx.num_input_notes));
    os.write(reinterpret_cast<char const*>(&be_tx.input_note_index), sizeof(be_tx.input_note_index));
    for (auto n : {be_tx.input_note[0], be_tx.input_note[1], be_tx.output_note[0], be_tx.output_note[1]}) {
        os.write(reinterpret_cast<char*>(&n.owner.x), sizeof(n.owner.x));
        os.write(reinterpret_cast<char*>(&n.owner.y), sizeof(n.owner.y));
        os.write(reinterpret_cast<char*>(&n.value), sizeof(n.value));
        os.write(reinterpret_cast<char*>(&n.secret), sizeof(n.secret));
    }
    os.write(reinterpret_cast<char const*>(&be_tx.signature.s), sizeof(be_tx.signature.s));
    os.write(reinterpret_cast<char const*>(&be_tx.signature.e), sizeof(be_tx.signature.e));
    return os;
}

std::istream& read(std::istream& is, join_split_tx& be_tx) {
    is.read(reinterpret_cast<char*>(&be_tx.owner_pub_key), sizeof(be_tx.owner_pub_key));
    is.read(reinterpret_cast<char*>(&be_tx.public_input), sizeof(be_tx.public_input));
    is.read(reinterpret_cast<char*>(&be_tx.public_output), sizeof(be_tx.public_output));
    is.read(reinterpret_cast<char*>(&be_tx.num_input_notes), sizeof(be_tx.num_input_notes));
    is.read(reinterpret_cast<char*>(&be_tx.input_note_index), sizeof(be_tx.input_note_index));
    for (size_t i=0; i<2; ++i) {
        auto& n = be_tx.input_note[i];
        is.read(reinterpret_cast<char*>(&n.owner.x), sizeof(n.owner.x));
        is.read(reinterpret_cast<char*>(&n.owner.y), sizeof(n.owner.y));
        is.read(reinterpret_cast<char*>(&n.value), sizeof(n.value));
        is.read(reinterpret_cast<char*>(&n.secret), sizeof(n.secret));
    }
    for (size_t i=0; i<2; ++i) {
        auto& n = be_tx.output_note[i];
        is.read(reinterpret_cast<char*>(&n.owner.x), sizeof(n.owner.x));
        is.read(reinterpret_cast<char*>(&n.owner.y), sizeof(n.owner.y));
        is.read(reinterpret_cast<char*>(&n.value), sizeof(n.value));
        is.read(reinterpret_cast<char*>(&n.secret), sizeof(n.secret));
    }
    is.read(reinterpret_cast<char*>(&be_tx.signature.s), sizeof(be_tx.signature.s));
    is.read(reinterpret_cast<char*>(&be_tx.signature.e), sizeof(be_tx.signature.e));
    return is;
}

} // namespace rollup

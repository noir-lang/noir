#include "ecdsa.hpp"
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <ecc/curves/secp256r1/secp256r1.hpp>
#include <gtest/gtest.h>

using namespace barretenberg;

TEST(ecdsa, verify_signature_grumpkin_sha256)
{
    std::string message = "The quick brown dog jumped over the lazy fox.";

    crypto::ecdsa::key_pair<grumpkin::fr, grumpkin::g1> account;
    account.private_key = grumpkin::fr::random_element();
    account.public_key = grumpkin::g1::one * account.private_key;

    crypto::ecdsa::signature signature =
        crypto::ecdsa::construct_signature<Sha256Hasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(message, account);

    bool result = crypto::ecdsa::verify_signature<Sha256Hasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        message, account.public_key, signature);

    EXPECT_EQ(result, true);
}

TEST(ecdsa, verify_signature_secp256r1_sha256)
{
    std::string message = "The quick brown dog jumped over the lazy fox.";

    crypto::ecdsa::key_pair<secp256r1::fr, secp256r1::g1> account;
    account.private_key = secp256r1::fr::random_element();
    account.public_key = secp256r1::g1::one * account.private_key;

    crypto::ecdsa::signature signature =
        crypto::ecdsa::construct_signature<Sha256Hasher, secp256r1::fq, secp256r1::fr, secp256r1::g1>(message, account);

    bool result = crypto::ecdsa::verify_signature<Sha256Hasher, secp256r1::fq, secp256r1::fr, secp256r1::g1>(
        message, account.public_key, signature);

    EXPECT_EQ(result, true);
}

std::vector<uint8_t> HexToBytes(const std::string& hex)
{
    std::vector<uint8_t> bytes;

    for (unsigned int i = 0; i < hex.length(); i += 2) {
        std::string byteString = hex.substr(i, 2);
        uint8_t byte = (uint8_t)strtol(byteString.c_str(), NULL, 16);
        bytes.push_back(byte);
    }

    return bytes;
}

TEST(ecdsa, verify_signature_secp256r1_sha256_NIST_1)
{
    /*
    Msg =
    5905238877c77421f73e43ee3da6f2d9e2ccad5fc942dcec0cbd25482935faaf416983fe165b1a045ee2bcd2e6dca3bdf46c4310a7461f9a37960ca672d3feb5473e253605fb1ddfd28065b53cb5858a8ad28175bf9bd386a5e471ea7a65c17cc934a9d791e91491eb3754d03799790fe2d308d16146d5c9b0d0debd97d79ce8
    d = 519b423d715f8b581f4fa8ee59f4771a5b44c8130b4e3eacca54a56dda72b464
    Qx = 1ccbe91c075fc7f4f033bfa248db8fccd3565de94bbfb12f3c59ff46c271bf83
    Qy = ce4014c68811f9a21a1fdb2c0e6113e06db7ca93b7404e78dc7ccd5ca89a4ca9
    k = 94a1bbb14b906a61a280f245f9e93c7f3b4a6247824f5d33b9670787642a68de
    R = f3ac8061b514795b8843e3d6629527ed2afd6b1f6a555a7acabb5e6f79c8c2ac
    S = 8bf77819ca05a6b2786c76262bf7371cef97b218e96f175a3ccdda2acc058903
    */

    secp256r1::fq P_x = secp256r1::fq(0x3c59ff46c271bf83, 0xd3565de94bbfb12f, 0xf033bfa248db8fcc, 0x1ccbe91c075fc7f4)
                            .to_montgomery_form();
    secp256r1::fq P_y = secp256r1::fq(0xdc7ccd5ca89a4ca9, 0x6db7ca93b7404e78, 0x1a1fdb2c0e6113e0, 0xce4014c68811f9a2)
                            .to_montgomery_form();

    secp256r1::g1::affine_element public_key(P_x, P_y);
    std::array<uint8_t, 32> r{
        0xf3, 0xac, 0x80, 0x61, 0xb5, 0x14, 0x79, 0x5b, 0x88, 0x43, 0xe3, 0xd6, 0x62, 0x95, 0x27, 0xed,
        0x2a, 0xfd, 0x6b, 0x1f, 0x6a, 0x55, 0x5a, 0x7a, 0xca, 0xbb, 0x5e, 0x6f, 0x79, 0xc8, 0xc2, 0xac,
    };

    std::array<uint8_t, 32> s{
        0x8b, 0xf7, 0x78, 0x19, 0xca, 0x05, 0xa6, 0xb2, 0x78, 0x6c, 0x76, 0x26, 0x2b, 0xf7, 0x37, 0x1c,
        0xef, 0x97, 0xb2, 0x18, 0xe9, 0x6f, 0x17, 0x5a, 0x3c, 0xcd, 0xda, 0x2a, 0xcc, 0x05, 0x89, 0x03,
    };

    crypto::ecdsa::signature sig{ r, s };
    std::vector<uint8_t> message_vec =
        HexToBytes("5905238877c77421f73e43ee3da6f2d9e2ccad5fc942dcec0cbd25482935faaf416983fe165b1a045ee2bcd2e6dca3bdf46"
                   "c4310a7461f9a37960ca672d3feb5473e253605fb1ddfd28065b53cb5858a8ad28175bf9bd386a5e471ea7a65c17cc934a9"
                   "d791e91491eb3754d03799790fe2d308d16146d5c9b0d0debd97d79ce8");
    std::string message(message_vec.begin(), message_vec.end());

    bool result = crypto::ecdsa::verify_signature<Sha256Hasher, secp256r1::fq, secp256r1::fr, secp256r1::g1>(
        message, public_key, sig);
    EXPECT_EQ(result, true);
}
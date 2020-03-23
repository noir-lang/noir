#include "schnorr.hpp"
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <stdlib/hash/blake2s/blake2s.hpp>

namespace plonk {
namespace stdlib {
namespace schnorr {

using namespace plonk::stdlib::types::turbo;

signature_bits convert_signature(Composer* context, const crypto::schnorr::signature& signature)
{
    signature_bits sig{ bit_array_ct(context, 256), bit_array_ct(context, 256) };

    for (size_t i = 0; i < 32; ++i) {
        for (size_t j = 7; j < 8; --j) {
            uint8_t s_shift = static_cast<uint8_t>(signature.s[i] >> j);
            uint8_t e_shift = static_cast<uint8_t>(signature.e[i] >> j);
            bool s_bit = (s_shift & 1U) == 1U;
            bool e_bit = (e_shift & 1U) == 1U;
            sig.s[i * 8 + (7 - j)] = witness_t(context, s_bit);
            sig.e[i * 8 + (7 - j)] = witness_t(context, e_bit);
        }
    }
    return sig;
}

bit_array_ct convert_message(Composer* context, const std::string& message_string)
{
    bit_array_ct message(context, message_string.size() * 8);
    for (size_t i = 0; i < message_string.size(); ++i) {
        uint8_t msg_byte = static_cast<uint8_t>(message_string[i]);
        for (size_t j = 7; j < 8; --j) {
            uint8_t msg_shift = static_cast<uint8_t>(msg_byte >> j);
            bool msg_bit = (msg_shift & 1U) == 1U;
            message[i * 8 + (7 - j)] = witness_t(context, msg_bit);
        }
    }
    return message;
}

point variable_base_mul(const point& pub_key, const bit_array_ct& scalar)
{
    point accumulator{ pub_key.x, pub_key.y };
    bool_ct initialized(pub_key.x.context, false);
    field_ct one(pub_key.x.context, barretenberg::fr::one());
    field_ct two(pub_key.x.context, barretenberg::fr{ 2, 0, 0, 0 }.to_montgomery_form());
    field_ct three(pub_key.x.context, barretenberg::fr{ 3, 0, 0, 0 }.to_montgomery_form());
    for (size_t i = 0; i < 256; ++i) {
        field_ct dbl_lambda = (accumulator.x * accumulator.x * three) / (accumulator.y * two);
        field_ct x_dbl = (dbl_lambda * dbl_lambda) - (accumulator.x * two);
        field_ct y_dbl = dbl_lambda * (accumulator.x - x_dbl) - accumulator.y;

        accumulator.x = accumulator.x + ((x_dbl - accumulator.x) * field_ct(initialized));
        accumulator.y = accumulator.y + ((y_dbl - accumulator.y) * field_ct(initialized));
        bool_ct was_initialized = initialized;
        initialized = initialized | scalar[i];

        field_ct add_lambda = (accumulator.y - pub_key.y) / (accumulator.x - pub_key.x);
        field_ct x_add = (add_lambda * add_lambda) - (accumulator.x + pub_key.x);
        field_ct y_add = add_lambda * (pub_key.x - x_add) - pub_key.y;

        bool_ct add_predicate = scalar[i] & was_initialized;
        accumulator.x = accumulator.x + ((x_add - accumulator.x) * field_ct(add_predicate));
        accumulator.y = accumulator.y + ((y_add - accumulator.y) * field_ct(add_predicate));
    }
    accumulator.x = accumulator.x.normalize();
    accumulator.y = accumulator.y.normalize();
    return accumulator;
}

bool verify_signature(const bit_array_ct& message, const point& pub_key, const signature_bits& sig)
{
    Composer* context = pub_key.x.context;

    point generator{ field_ct(context, grumpkin::g1::affine_one.x), field_ct(context, grumpkin::g1::affine_one.y) };

    point R_1 = variable_base_mul(generator, sig.s);
    point R_2 = variable_base_mul(pub_key, sig.e);

    field_ct lambda = (R_1.y - R_2.y) / (R_1.x - R_2.x);
    field_ct x_3 = lambda * lambda - (R_1.x + R_2.x);
    x_3 = x_3.normalize();

    bit_array_ct hash_input(context, 256 + message.size());

    barretenberg::fr r_x = x_3.get_value();
    r_x = r_x.from_montgomery_form();

    field_ct sum(context, barretenberg::fr::one());
    field_ct accumulator(context, barretenberg::fr::zero());
    size_t input_length = 256 + message.size();

    for (size_t i = 0; i < 256; ++i) {
        bool_t temp = witness_t(context, r_x.get_bit(i));
        accumulator = accumulator + (sum * field_ct(temp));
        sum = sum + sum;
        temp = temp.normalize();
        hash_input[input_length - 1 - (255 - i)] = temp;
    }
    accumulator = accumulator.normalize();
    context->assert_equal(accumulator.witness_index, x_3.witness_index);

    for (size_t i = 0; i < message.size(); ++i) {
        hash_input[input_length - 1 - (256 + i)] = message[i];
    }

    bit_array_ct output = blake2s(byte_array_ct(hash_input));

    bool valid = true;
    for (size_t i = 0; i < 256; ++i) {
        valid = valid && (output[255 - i].get_value() == sig.e[i].get_value());

        context->assert_equal(output[255 - i].witness_index, sig.e[i].witness_index);
    }
    return valid;
}

} // namespace schnorr
} // namespace stdlib
} // namespace plonk
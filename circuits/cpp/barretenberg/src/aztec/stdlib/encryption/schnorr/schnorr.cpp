#include "schnorr.hpp"
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <stdlib/hash/blake2s/blake2s.hpp>

#include "../../primitives/composers/composers.hpp"

namespace plonk {
namespace stdlib {
namespace schnorr {

template <typename C> signature_bits<C> convert_signature(C* context, const crypto::schnorr::signature& signature)
{
    signature_bits<C> sig{ bit_array<C>(context, 256), bit_array<C>(context, 256) };

    for (size_t i = 0; i < 32; ++i) {
        for (size_t j = 7; j < 8; --j) {
            uint8_t s_shift = static_cast<uint8_t>(signature.s[i] >> j);
            uint8_t e_shift = static_cast<uint8_t>(signature.e[i] >> j);
            bool s_bit = (s_shift & 1U) == 1U;
            bool e_bit = (e_shift & 1U) == 1U;
            sig.s[i * 8 + (7 - j)] = witness_t<C>(context, s_bit);
            sig.e[i * 8 + (7 - j)] = witness_t<C>(context, e_bit);
        }
    }
    return sig;
}

template <typename C> bit_array<C> convert_message(C* context, const std::string& message_string)
{
    bit_array<C> message(context, message_string.size() * 8);
    for (size_t i = 0; i < message_string.size(); ++i) {
        uint8_t msg_byte = static_cast<uint8_t>(message_string[i]);
        for (size_t j = 7; j < 8; --j) {
            uint8_t msg_shift = static_cast<uint8_t>(msg_byte >> j);
            bool msg_bit = (msg_shift & 1U) == 1U;
            message[(message_string.size() - i - 1) * 8 + j] = witness_t<C>(context, msg_bit);
        }
    }
    return message;
}

template <typename C> point<C> variable_base_mul(const point<C>& pub_key, const bit_array<C>& scalar)
{
    point<C> accumulator{ pub_key.x, pub_key.y };
    bool_t<C> initialized(pub_key.x.context, false);
    field_t<C> one(pub_key.x.context, barretenberg::fr::one());
    field_t<C> two(pub_key.x.context, barretenberg::fr{ 2, 0, 0, 0 }.to_montgomery_form());
    field_t<C> three(pub_key.x.context, barretenberg::fr{ 3, 0, 0, 0 }.to_montgomery_form());
    for (size_t i = 0; i < 256; ++i) {
        field_t<C> dbl_lambda = (accumulator.x * accumulator.x * three) / (accumulator.y * two);
        field_t<C> x_dbl = (dbl_lambda * dbl_lambda) - (accumulator.x * two);
        field_t<C> y_dbl = dbl_lambda * (accumulator.x - x_dbl) - accumulator.y;

        accumulator.x = accumulator.x + ((x_dbl - accumulator.x) * field_t<C>(initialized));
        accumulator.y = accumulator.y + ((y_dbl - accumulator.y) * field_t<C>(initialized));
        bool_t<C> was_initialized = initialized;
        initialized = initialized | scalar[i];

        field_t<C> add_lambda = (accumulator.y - pub_key.y) / (accumulator.x - pub_key.x);
        field_t<C> x_add = (add_lambda * add_lambda) - (accumulator.x + pub_key.x);
        field_t<C> y_add = add_lambda * (pub_key.x - x_add) - pub_key.y;

        bool_t<C> add_predicate = scalar[i] & was_initialized;
        accumulator.x = accumulator.x + ((x_add - accumulator.x) * field_t<C>(add_predicate));
        accumulator.y = accumulator.y + ((y_add - accumulator.y) * field_t<C>(add_predicate));
    }
    accumulator.x = accumulator.x.normalize();
    accumulator.y = accumulator.y.normalize();
    return accumulator;
}

template <typename C>
bool verify_signature(const bit_array<C>& message, const point<C>& pub_key, const signature_bits<C>& sig)
{
    C* context = pub_key.x.context;

    point<C> generator{ field_t<C>(context, grumpkin::g1::affine_one.x),
                        field_t<C>(context, grumpkin::g1::affine_one.y) };

    point<C> R_1 = variable_base_mul(generator, sig.s);
    point<C> R_2 = variable_base_mul(pub_key, sig.e);

    field_t<C> lambda = (R_1.y - R_2.y) / (R_1.x - R_2.x);
    field_t<C> x_3 = lambda * lambda - (R_1.x + R_2.x);
    x_3 = x_3.normalize();

    bit_array<C> hash_input(context, 256 + message.size());

    barretenberg::fr r_x = x_3.get_value();
    r_x = r_x.from_montgomery_form();

    field_t<C> sum(context, barretenberg::fr::one());
    field_t<C> accumulator(context, barretenberg::fr::zero());

    for (size_t i = 0; i < 256; ++i) {
        bool_t<C> temp = witness_t<C>(context, r_x.get_bit(i));
        accumulator = accumulator + (sum * field_t<C>(temp));
        sum = sum + sum;
        temp = temp.normalize();
        hash_input[message.size() + i] = temp;
    }
    accumulator = accumulator.normalize();
    context->assert_equal(accumulator.witness_index, x_3.witness_index);

    for (size_t i = 0; i < message.size(); ++i) {
        hash_input[i] = message[i];
    }

    bit_array<C> output = blake2s(byte_array<C>(hash_input));

    bool valid = true;
    for (size_t i = 0; i < 256; ++i) {
        valid = valid && (output[255 - i].get_value() == sig.e[i].get_value());

        context->assert_equal(output[255 - i].witness_index, sig.e[i].witness_index, "bad signature");
    }
    return valid;
}

template <typename C>
bool verify_signature(const byte_array<C>& message, const point<C>& pub_key, const signature_bits<C>& sig)
{
    return verify_signature(bit_array<C>(message), pub_key, sig);
}

template point<waffle::TurboComposer> variable_base_mul<waffle::TurboComposer>(const point<waffle::TurboComposer>&,
                                                                               const bit_array<waffle::TurboComposer>&);
template point<waffle::PLookupComposer> variable_base_mul<waffle::PLookupComposer>(
    const point<waffle::PLookupComposer>&, const bit_array<waffle::PLookupComposer>&);

template bool verify_signature<waffle::TurboComposer>(const bit_array<waffle::TurboComposer>&,
                                                      const point<waffle::TurboComposer>&,
                                                      const signature_bits<waffle::TurboComposer>&);
template bool verify_signature<waffle::PLookupComposer>(const bit_array<waffle::PLookupComposer>&,
                                                        const point<waffle::PLookupComposer>&,
                                                        const signature_bits<waffle::PLookupComposer>&);

template bool verify_signature<waffle::TurboComposer>(const byte_array<waffle::TurboComposer>&,
                                                      const point<waffle::TurboComposer>&,
                                                      const signature_bits<waffle::TurboComposer>&);
template bool verify_signature<waffle::PLookupComposer>(const byte_array<waffle::PLookupComposer>&,
                                                        const point<waffle::PLookupComposer>&,
                                                        const signature_bits<waffle::PLookupComposer>&);

template signature_bits<waffle::TurboComposer> convert_signature<waffle::TurboComposer>(
    waffle::TurboComposer*, const crypto::schnorr::signature&);
template signature_bits<waffle::PLookupComposer> convert_signature<waffle::PLookupComposer>(
    waffle::PLookupComposer*, const crypto::schnorr::signature&);

template bit_array<waffle::TurboComposer> convert_message<waffle::TurboComposer>(waffle::TurboComposer*,
                                                                                 const std::string&);
template bit_array<waffle::PLookupComposer> convert_message<waffle::PLookupComposer>(waffle::PLookupComposer*,
                                                                                     const std::string&);
} // namespace schnorr
} // namespace stdlib
} // namespace plonk
#pragma once

namespace crypto {
namespace schnorr {
template <typename Hash, typename Fq, typename Fr, typename G1>
signature construct_signature(const std::string& message,
                              const key_pair<Fr, G1>& account,
                              numeric::random::Engine* engine)
{
    signature sig;
    Fr k = Fr::random_element(engine); // TODO replace with HMAC
    typename G1::affine_element R(G1::one * k);

    std::vector<uint8_t> r(sizeof(Fq));
    Fq::serialize_to_buffer(R.x, &r[0]);

    std::vector<uint8_t> message_buffer;
    // message_buffer.resize(r.size() + message.size());
    std::copy(r.begin(), r.end(), std::back_inserter(message_buffer));
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));

    auto ev = Hash::hash(message_buffer);
    std::copy(ev.begin(), ev.end(), sig.e.begin());

    Fr e = Fr::serialize_from_buffer(&sig.e[0]);

    Fr s = k - (account.private_key * e);

    Fr::serialize_to_buffer(s, &sig.s[0]);
    return sig;
}

template <typename Hash, typename Fq, typename Fr, typename G1>
signature_b construct_signature_b(const std::string& message, const key_pair<Fr, G1>& account)
{
    signature_b sig;
    Fr k = Fr::random_element(); // TODO replace with HMAC
    typename G1::affine_element R(G1::one * k);
    Fq::serialize_to_buffer(R.x, &sig.r[0]);

    Fq yy = R.x.sqr() * R.x + G1::element::curve_b;
    Fq y_candidate = yy.sqrt();

    // if the signer / verifier sqrt algorithm is consistent, this *should* work...
    bool flip_sign = R.y != y_candidate;

    sig.r[0] = sig.r[0] | static_cast<uint8_t>(flip_sign ? 128U : 0U);
    std::vector<uint8_t> message_buffer;
    std::copy(sig.r.begin(), sig.r.end(), std::back_inserter(message_buffer));
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));
    auto e_vec = Hash::hash(message_buffer);

    Fr e = Fr::serialize_from_buffer(&e_vec[0]);
    Fr s = account.private_key - (k * e);

    Fr::serialize_to_buffer(s, &sig.s[0]);
    return sig;
}

template <typename Hash, typename Fq, typename Fr, typename G1>
typename G1::affine_element ecrecover(const std::string& message, const signature_b& sig)
{
    std::vector<uint8_t> message_buffer;
    std::copy(sig.r.begin(), sig.r.end(), std::back_inserter(message_buffer));
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));
    auto e_vec = Hash::hash(message_buffer);
    Fr target_e = Fr::serialize_from_buffer(&e_vec[0]);

    std::vector<uint8_t> r;
    std::copy(sig.r.begin(), sig.r.end(), std::back_inserter(r));

    bool flip_sign = (r[0] & 128U) == 128U;
    r[0] = r[0] & 127U;
    Fq r_x = Fq::serialize_from_buffer(&r[0]);
    Fq r_yy = r_x.sqr() * r_x + G1::element::curve_b;
    Fq r_y = r_yy.sqrt();

    if ((flip_sign)) {
        r_y.self_neg();
    }
    typename G1::affine_element R{ r_x, r_y };
    Fr s = Fr::serialize_from_buffer(&sig.s[0]);
    typename G1::affine_element R1(G1::one * s);
    typename G1::affine_element R2(typename G1::element(R) * target_e);
    typename G1::element R2_jac{ R2.x, R2.y, Fq::one() };
    typename G1::element key_jac;
    key_jac = R2_jac + R1;
    key_jac = key_jac.normalize();
    typename G1::affine_element key{ key_jac.x, key_jac.y };
    return key;
}

template <typename Hash, typename Fq, typename Fr, typename G1>
bool verify_signature(const std::string& message, const typename G1::affine_element& public_key, const signature& sig)
{
    // r = g^s . pub^e
    // e = H(r, m)
    Fr s = Fr::serialize_from_buffer(&sig.s[0]);
    Fr source_e = Fr::serialize_from_buffer(&sig.e[0]);

    typename G1::affine_element R1(G1::one * s);
    typename G1::affine_element R2(typename G1::element(public_key) * source_e);

    typename G1::element R2_ele{ R2.x, R2.y, Fq::one() };

    typename G1::element R;
    R = R2_ele + R1;
    R = R.normalize();

    std::vector<uint8_t> r(sizeof(Fq));
    Fq::serialize_to_buffer(R.x, &r[0]);

    std::vector<uint8_t> message_buffer;
    std::copy(r.begin(), r.end(), std::back_inserter(message_buffer));
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));
    auto e_vec = Hash::hash(message_buffer);
    Fr target_e = Fr::serialize_from_buffer(&e_vec[0]);

    return source_e == target_e;
}
} // namespace schnorr
} // namespace crypto
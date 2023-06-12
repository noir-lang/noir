#pragma once

#include "circuit_types.hpp"
#include "native_types.hpp"

#include <barretenberg/barretenberg.hpp>

#include <array>

namespace aztec3::utils::types {

using plonk::stdlib::witness_t;

namespace {

template <typename Composer> using CT = aztec3::utils::types::CircuitTypes<Composer>;
using NT = aztec3::utils::types::NativeTypes;

}  // namespace

/// TODO: Lots of identical functions here (but for their in/out types). Can we use templates? I couldn't figure out how
/// to keep the NT:: or CT:: prefixes with templates.
template <typename Composer> typename CT<Composer>::boolean to_ct(Composer& composer, typename NT::boolean const& e)
{
    return typename CT<Composer>::boolean(witness_t<Composer>(&composer, e));
};

template <typename Composer> typename CT<Composer>::fr to_ct(Composer& composer, typename NT::fr const& e)
{
    return typename CT<Composer>::fr(witness_t<Composer>(&composer, e));
};

template <typename Composer> typename CT<Composer>::fq to_ct(Composer& composer, typename NT::fq const& e)
{
    return typename CT<Composer>::fq(witness_t<Composer>(&composer, e));
};

template <typename Composer> typename CT<Composer>::address to_ct(Composer& composer, typename NT::address const& e)
{
    return typename CT<Composer>::address(witness_t<Composer>(&composer, e));
};

template <typename Composer> typename CT<Composer>::uint32 to_ct(Composer& composer, typename NT::uint32 const& e)
{
    return typename CT<Composer>::uint32(witness_t<Composer>(&composer, e));
};

template <typename Composer>
typename CT<Composer>::grumpkin_point to_ct(Composer& composer, typename NT::grumpkin_point const& e)
{
    return plonk::stdlib::create_point_witness<Composer, typename NT::grumpkin_point>(composer, e, true);
};

template <typename Composer>
typename CT<Composer>::bn254_point to_ct(Composer& composer, typename NT::bn254_point const& e)
{
    return CT<Composer>::bn254_point::from_witness(&composer, e);
};

template <typename Composer>
typename CT<Composer>::ecdsa_signature to_ct(Composer& composer, typename NT::ecdsa_signature const& e)
{
    return CT<Composer>::ecdsa_signature::template from_witness<Composer>(&composer, e);
};

template <typename Composer>
std::optional<typename CT<Composer>::boolean> to_ct(Composer& composer, std::optional<typename NT::boolean> const& e)
{
    return e ? std::make_optional<typename CT<Composer>::boolean>(to_ct(composer, *e)) : std::nullopt;
};

template <typename Composer>
std::optional<typename CT<Composer>::fr> to_ct(Composer& composer, std::optional<typename NT::fr> const& e)
{
    return e ? std::make_optional<typename CT<Composer>::fr>(to_ct(composer, *e)) : std::nullopt;
};

template <typename Composer>
std::optional<typename CT<Composer>::address> to_ct(Composer& composer, std::optional<typename NT::address> const& e)
{
    return e ? std::make_optional<typename CT<Composer>::address>(to_ct(composer, *e)) : std::nullopt;
};

template <typename Composer>
std::optional<typename CT<Composer>::grumpkin_point> to_ct(Composer& composer,
                                                           std::optional<typename NT::grumpkin_point> const& e)
{
    return e ? std::make_optional<typename CT<Composer>::grumpkin_point>(to_ct(composer, *e)) : std::nullopt;
};

template <typename Composer>
std::optional<typename CT<Composer>::ecdsa_signature> to_ct(Composer& composer,
                                                            std::optional<typename NT::ecdsa_signature> const& e)
{
    return e ? std::make_optional<typename CT<Composer>::ecdsa_signature>(to_ct(&composer, e)) : std::nullopt;
};

template <typename Composer>
std::vector<typename CT<Composer>::fr> to_ct(Composer& composer, std::vector<typename NT::fr> const& vec)
{
    auto ref_to_ct = [&](typename NT::fr const& e) { return to_ct(composer, e); };

    return map(vec, ref_to_ct);
};

template <typename Composer>
std::optional<std::vector<typename CT<Composer>::fr>> to_ct(Composer& composer,
                                                            std::optional<std::vector<typename NT::fr>> const& vec)
{
    auto ref_to_ct = [&](typename NT::fr const& e) { return to_ct(composer, e); };

    return vec ? std::make_optional<std::vector<typename CT<Composer>::fr>>(map(*vec, ref_to_ct)) : std::nullopt;
};

template <typename Composer, std::size_t SIZE>
std::array<typename CT<Composer>::fr, SIZE> to_ct(Composer& composer, std::array<typename NT::fr, SIZE> const& arr)
{
    auto ref_to_ct = [&](typename NT::fr const& e) { return to_ct(composer, e); };

    return map(arr, ref_to_ct);
};


template <typename Composer, std::size_t SIZE> std::array<std::optional<typename CT<Composer>::fr>, SIZE> to_ct(
    Composer& composer, std::array<std::optional<typename NT::fr>, SIZE> const& arr)
{
    auto ref_to_ct = [&](std::optional<typename NT::fr> const& e) { return to_ct(composer, e); };

    return map(arr, ref_to_ct);
};

/**
 * @brief Convert from an array of any native types (NT_TYPE) to array of circuit types (CT_TYPE)
 */
template <typename Composer, typename CT_TYPE, typename NT_TYPE, std::size_t SIZE>
std::array<CT_TYPE, SIZE> to_ct(Composer& composer, std::array<NT_TYPE, SIZE> const& arr)
{
    auto ref_to_ct = [&](NT_TYPE const& e) { return e.to_circuit_type(composer); };

    return map(arr, ref_to_ct);
};

/**
 * @brief Convert from an array of any native types (NT_TYPE) to array of circuit types (CT_TYPE).
 * Allow array entries to be optional.
 */
template <typename Composer, typename CT_TYPE, typename NT_TYPE, std::size_t SIZE>
std::array<std::optional<CT_TYPE>, SIZE> to_ct(Composer& composer, std::array<std::optional<NT_TYPE>, SIZE> const& arr)
{
    auto ref_to_ct = [&](std::optional<NT_TYPE> const& e) { return e.to_circuit_type(composer); };

    return map(arr, ref_to_ct);
};

// to_nt() below ********************************

template <typename Composer> typename NT::boolean to_nt(typename CT<Composer>::boolean const& e)
{
    return e.get_value();
};

template <typename Composer> typename NT::fr to_nt(typename CT<Composer>::fr const& e)
{
    return e.get_value();
};

template <typename Composer> typename NT::fq to_nt(typename CT<Composer>::fq const& e)
{
    return e.get_value();
};

template <typename Composer> typename NT::address to_nt(typename CT<Composer>::address const& e)
{
    return NT::address(e.address_.get_value());  // TODO: add get_value() method to address types.
};

template <typename Composer> typename NT::uint32 to_nt(typename CT<Composer>::uint32 const& e)
{
    NT::uint256 const e_256 = e.get_value();
    NT::uint64 const e_64 = e_256.data[0];  // TODO: check that this endianness is correct!
    auto const e_32 = static_cast<NT::uint32>(e_64);
    return e_32;
};

template <typename Composer> typename NT::grumpkin_point to_nt(typename CT<Composer>::grumpkin_point const& e)
{
    return NT::grumpkin_point{ e.x.get_value(), e.y.get_value() };
};

template <typename Composer> typename NT::bn254_point to_nt(typename CT<Composer>::bn254_point const& e)
{
    return e.get_value();
};

template <typename Composer> typename NT::ecdsa_signature to_nt(typename CT<Composer>::ecdsa_signature const& e)
{
    std::vector<uint8_t> r_bytes = e.r.get_value();
    std::vector<uint8_t> s_bytes = e.s.get_value();

    std::array<uint8_t, 32> r_array;
    std::array<uint8_t, 32> s_array;
    std::copy(r_bytes.begin(), r_bytes.end(), r_array.begin());
    std::copy(s_bytes.begin(), s_bytes.end(), s_array.begin());

    return NT::ecdsa_signature{ r_array, s_array, e.v };
};

template <typename Composer>
std::optional<typename NT::boolean> to_nt(std::optional<typename CT<Composer>::boolean> const& e)
{
    return e ? std::make_optional<typename NT::boolean>(to_nt<Composer>(*e)) : std::nullopt;
};

template <typename Composer> std::optional<typename NT::fr> to_nt(std::optional<typename CT<Composer>::fr> const& e)
{
    return e ? std::make_optional<typename NT::fr>(to_nt<Composer>(*e)) : std::nullopt;
};

template <typename Composer>
std::optional<typename NT::address> to_nt(std::optional<typename CT<Composer>::address> const& e)
{
    return e ? std::make_optional<typename NT::address>(to_nt<Composer>(*e)) : std::nullopt;
};

template <typename Composer>
std::optional<typename NT::grumpkin_point> to_nt(std::optional<typename CT<Composer>::grumpkin_point> const& e)
{
    return e ? std::make_optional<typename NT::grumpkin_point>(to_nt<Composer>(*e)) : std::nullopt;
};

template <typename Composer>
std::optional<typename NT::ecdsa_signature> to_nt(std::optional<typename CT<Composer>::ecdsa_signature> const& e)
{
    return e ? std::make_optional<typename NT::ecdsa_signature>(to_nt<Composer>(*e)) : std::nullopt;
};

template <typename Composer> std::vector<typename NT::fr> to_nt(std::vector<typename CT<Composer>::fr> const& vec)
{
    auto ref_to_nt = [&](typename CT<Composer>::fr const& e) { return to_nt<Composer>(e); };

    return map(vec, ref_to_nt);
};

template <typename Composer>
std::optional<std::vector<typename NT::fr>> to_nt(std::optional<std::vector<typename CT<Composer>::fr>> const& vec)
{
    auto ref_to_nt = [&](typename CT<Composer>::fr const& e) { return to_nt<Composer>(e); };

    return vec ? std::make_optional<std::vector<typename NT::fr>>(map(*vec, ref_to_nt)) : std::nullopt;
};

template <typename Composer, std::size_t SIZE>
std::array<typename NT::fr, SIZE> to_nt(std::array<typename CT<Composer>::fr, SIZE> const& arr)
{
    auto ref_to_nt = [&](typename CT<Composer>::fr const& e) { return to_nt<Composer>(e); };

    return map(arr, ref_to_nt);
};

// template <typename Composer, std::size_t SIZE>
// std::optional<std::array<typename NT::fr, SIZE>> to_nt(
//     std::optional<std::array<typename CT<Composer>::fr, SIZE>> const& arr)
// {
//     auto ref_to_nt = [&](typename CT<Composer>::fr const& e) { return to_nt(e); };

//     return arr ? std::make_optional<std::array<typename NT::fr, SIZE>>(map(arr, ref_to_nt)) : std::nullopt;
// };

template <typename Composer, std::size_t SIZE> std::array<std::optional<typename NT::fr>, SIZE> to_nt(
    std::array<std::optional<typename CT<Composer>::fr>, SIZE> const& arr)
{
    auto ref_to_nt = [&](std::optional<typename CT<Composer>::fr> const& e) { return to_nt<Composer>(e); };

    return map(arr, ref_to_nt);
};

}  // namespace aztec3::utils::types
